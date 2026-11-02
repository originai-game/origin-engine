//! og.motion's presentation: the motion model on the renderer's GPU, and
//! each agent's generated pose drawn in place of og.anim's joint palette.
//!
//! Before each frame is drawn ([`present`], from `engine::tick`'s render):
//!
//! 1. **The weights.** The first frame an og.MotionAgent exists (or
//!    og.motion.capabilities asks), `cooked/motion/model.ogb` is read
//!    through the host (natively at once; on the web a fetch that lands at
//!    a later frame), unpacked, and the two networks are put on the
//!    renderer's device ([`RenderBackend::compute`]). Until then, or when
//!    that fails, the backend is none and og.anim's pose is drawn.
//! 2. **Goals for the model.** An agent whose generated frames run short
//!    of its `lookahead` gets the next model goal: where the simulation's
//!    goals will have it about [`HORIZON`] seconds on (the mover run ahead
//!    with [`super::advance`]; without drive, its Transform's velocity),
//!    in the pose it will be in there (walking, running, or the goal's).
//! 3. **One batch for every agent** runs at a time, never blocking the
//!    frame: its readback is polled each frame and lands a frame or two
//!    after the GPU finishes. A native host blocks only when an agent has
//!    nothing left to show (`og run` stepping faster than real time draws
//!    every frame that way, so its frames do not depend on timing).
//! 4. **The pose.** Each agent plays its frames at 30 Hz on the frame's
//!    clock, blending the two around it: re-rooted onto og.Transform (the
//!    hips over the entity, facing its heading), retargeted to its rig
//!    (`og_anim::retarget`), its feet planted in the world from the
//!    model's contacts (`og_motion::soma::Planter`), as the joint palette
//!    of its og.Skeleton's skin.
//!
//! Presentation only: nothing here writes the world, and nothing the tick
//! reads depends on it.

use super::{
    AGENT, Goals, Kind, RUN_SPEED, Settings, advance, heading_of, lock, rotation_of, shared,
    transform_of,
};
use crate::RenderBackend;
use crate::engine::{Engine, hooks};
use crate::extract::rows;
use glam::{Mat4, Vec3};
use og_anim::{HumanoidMap, NONE, Pose as AnimPose, RetargetOptions, SkeletonAsset, Trs};
use og_core::IdMap;
use og_motion::contract::{LEFT_LEG, Pose, RIGHT_LEG, heading};
use og_motion::gpu::{Gpu, GpuError};
use og_motion::intent::{Batch, Frame, Goal, Motion};
use og_motion::library::{Library, POSES};
use og_motion::soma::{Leg, Planter, Skeleton};
use og_motion::{Bundle, Generation};
use og_render::wgpu;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// The model's weights in the cooked bundle.
pub(crate) const MODEL_FILE: &str = "motion/model.ogb";
/// The weights' file inside that pack.
pub(crate) const MODEL_ENTRY: &str = "motion/model.safetensors";
/// How far ahead of the simulation the model's next goal is placed.
const HORIZON: f32 = 1.2;
/// The model's frame rate.
const RATE: f32 = 30.0;

type Running = Pin<Box<dyn Future<Output = (Gpu, Result<Generation, GpuError>)>>>;

/// A batch on the GPU.
struct Flight {
    future: Running,
    batch: Batch,
    started: f64,
}

struct Ready {
    /// None while a batch has it.
    gpu: Option<Gpu>,
    device: wgpu::Device,
    flight: Option<Flight>,
    motion: Motion,
    library: Library,
    soma: SkeletonAsset,
    soma_skeleton: Skeleton,
    soma_map: HumanoidMap,
}

#[derive(Default)]
enum Model {
    #[default]
    Unloaded,
    /// Waiting: for the file, or for a renderer to run on.
    Waiting(String),
    Failed(String),
    Ready(Box<Ready>),
}

/// An agent as drawn.
struct Shown {
    character: usize,
    prev: Option<Frame>,
    next: Option<Frame>,
    /// Where between `prev` and `next` the frame is, 0 to 1.
    phase: f32,
    /// The last goal serial seen (a change clears the model's queued goal).
    serial: u32,
    planter: Planter,
    skeleton: u64,
    rig: u64,
    target: Option<(Arc<SkeletonAsset>, HumanoidMap)>,
    last_position: Vec3,
    velocity: Vec3,
    shown: u64,
    starved: u64,
    /// The drawn pose's heading in the character's own space (its thighs'
    /// across direction): about 0 when it faces where its Transform does.
    facing: f32,
    /// Each leg's ankle in the world at the last frame it was planted (the
    /// model's heel contact), and how far planted ankles moved between
    /// planted frames: the summed metres and the frames (`dump_motion`'s
    /// `plantedSlide`, the foot plant's own measure).
    planted: [Option<Vec3>; 2],
    slide: (f32, u32, f32),
}

/// og.motion's presentation state (in `RenderState`, never the world).
#[derive(Default)]
pub(crate) struct Presenter {
    model: Model,
    agents: BTreeMap<u64, Shown>,
    /// The palette each agent is drawn with this frame, by entity bits
    /// (`fx::posed_model` takes it in place of og-anim's).
    pub palettes: IdMap<u64, Vec<Mat4>>,
    /// og.motion.capabilities asked: load even without an agent.
    wanted: bool,
    /// The frame clock of the last frame presented.
    clock: Option<f64>,
    model_bytes: u64,
    batches: u64,
    last_batch: (usize, f64),
    errors: Vec<String>,
}

impl Presenter {
    /// The world was restored or reloaded: every agent starts again.
    pub(crate) fn forget(&mut self) {
        self.agents.clear();
        self.palettes.clear();
        self.clock = None;
    }
}

fn now_ms() -> f64 {
    hooks().map_or(0.0, |h| h.now_ms())
}

/// The cooked bundle's directory.
fn cooked_dir(eng: &Engine) -> Option<String> {
    let a = eng.assets.borrow();
    let i = a.default_bundle?;
    match &a.entries[i].payload {
        crate::assets::Payload::Bundle { dir, .. } => Some(dir.clone()),
        _ => None,
    }
}

/// Reads and puts the model on the renderer's device; answers the state.
fn load(eng: &Engine, backend: Option<&dyn RenderBackend>, bytes_out: &mut u64) -> Model {
    let Some((device, queue)) = backend.and_then(|b| b.compute()) else {
        return Model::Waiting(
            "no renderer is attached: the motion model runs on the renderer's GPU".into(),
        );
    };
    let Some(dir) = cooked_dir(eng) else {
        return Model::Waiting("the cooked bundle is not open yet".into());
    };
    let path = format!("{dir}/{MODEL_FILE}");
    let bytes = match hooks().map(|h| h.read_asset(&path)) {
        Some(crate::assets::Fetch::Ready(b)) => b,
        Some(crate::assets::Fetch::Pending) => return Model::Waiting(format!("loading {path}")),
        Some(crate::assets::Fetch::Failed(e)) => {
            return Model::Failed(format!(
                "{path}: {e} (og cook writes it for a project that uses og.motion)"
            ));
        }
        None => return Model::Failed("no host is installed to read assets".into()),
    };
    *bytes_out = bytes.len() as u64;
    let r = (|| {
        let files = og_bundle::runtime::unpack(&bytes).map_err(|e| e.to_string())?;
        let (_, weights) = files
            .iter()
            .find(|(n, _)| n == MODEL_ENTRY)
            .ok_or_else(|| format!("{path} holds no {MODEL_ENTRY}"))?;
        let bundle = Bundle::from_safetensors(weights).map_err(|e| e.to_string())?;
        let library = Library::from_bundle(&bundle).map_err(|e| e.to_string())?;
        let soma_skeleton = Skeleton::from_bundle(&bundle).map_err(|e| e.to_string())?;
        let soma = soma_skeleton.asset();
        let soma_map = HumanoidMap::detect(&soma);
        let mut gpu = Gpu::new(device.clone(), queue, &bundle).map_err(|e| e.to_string())?;
        gpu.set_blocking(false);
        Ok::<_, String>(Ready {
            gpu: Some(gpu),
            device,
            flight: None,
            motion: Motion::default(),
            library,
            soma,
            soma_skeleton,
            soma_map,
        })
    })();
    match r {
        Ok(ready) => Model::Ready(Box::new(ready)),
        Err(e) => Model::Failed(format!("{path}: {e}")),
    }
}

/// One og.MotionAgent as this frame reads it.
struct Seen {
    entity: u64,
    settings: Settings,
    position: Vec3,
    rotation: glam::Quat,
    scale: f32,
    skeleton: u64,
    skin: u32,
}

fn seen(eng: &Engine) -> Vec<Seen> {
    let Ok(world) = eng.world() else {
        return Vec::new();
    };
    let st = eng.state.borrow();
    let (Some(agent), Some(transform), Some(skeleton)) = (
        st.layouts.get(AGENT),
        st.layouts.get("og.Transform"),
        st.layouts.get("og.Skeleton"),
    ) else {
        return Vec::new();
    };
    let (Ok(tid), Ok(sid)) = (
        world.id_by_name("og.Transform"),
        world.id_by_name("og.Skeleton"),
    ) else {
        return Vec::new();
    };
    rows(world, agent)
        .into_iter()
        .filter_map(|(bits, a)| {
            let e = og_ecs::Entity::from_bits(bits);
            let t = transform.decode(&world.get_bytes(e, tid).ok()?);
            let s = skeleton.decode(&world.get_bytes(e, sid).ok()?);
            let (position, rotation) = transform_of(&t);
            Some(Seen {
                entity: bits,
                settings: Settings::of(&a),
                position,
                rotation,
                scale: t["scale"]["x"].as_f64().unwrap_or(1.0) as f32,
                skeleton: s["asset"].as_f64().map_or(0, |x| x as u64),
                skin: s["skin"].as_u64().unwrap_or(0) as u32,
            })
        })
        .collect()
}

/// The pose a moving character is in at `speed`.
fn gait(speed: f32) -> usize {
    if speed >= RUN_SPEED { 2 } else { 1 }
}

/// Where the simulation will have an agent [`HORIZON`] seconds on, its
/// heading there and the library pose it will be in.
fn predict(goals: &Goals, a: &Seen, shown: &Shown) -> (Vec3, f32, usize) {
    let h0 = heading_of(a.rotation);
    match goals.agents.get(&a.entity) {
        Some(agent) if a.settings.drive => {
            let mut agent = agent.clone();
            let (mut p, mut h) = (a.position, h0);
            let step = 1.0 / RATE;
            let mut speed = 0.0;
            for _ in 0..(HORIZON * RATE) as usize {
                let before = p;
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
                speed = (p - before).length() / step;
            }
            // A place or pose goal asks for its own pose even while the mover
            // carries the character there; only a walk is walked or run.
            let pose = match agent.current {
                Some(c) if matches!(c.goal.kind, Kind::Place | Kind::Pose) => c.goal.pose as usize,
                _ if speed > 0.3 => gait(speed),
                Some(c) if c.goal.kind == Kind::WalkTo => c.goal.pose as usize,
                _ => agent.hold as usize,
            };
            (p, h, pose)
        }
        agent => {
            let v = Vec3::new(shown.velocity.x, 0.0, shown.velocity.z);
            let speed = v.length();
            if speed > 0.3 {
                (
                    a.position + v * HORIZON,
                    og_core::math::atan2(v.x, v.z),
                    gait(speed),
                )
            } else {
                let pose = agent.map_or(0, |g| match g.current {
                    Some(c) if c.goal.kind == Kind::Pose => c.goal.pose as usize,
                    _ => g.hold as usize,
                });
                (a.position, h0, pose)
            }
        }
    }
}

/// Polls the batch in flight once; `block` waits for it (native only).
fn poll_flight(
    r: &mut Ready,
    block: bool,
    stats: &mut (u64, (usize, f64)),
    errors: &mut Vec<String>,
) -> bool {
    let Some(flight) = r.flight.as_mut() else {
        return false;
    };
    #[cfg(not(target_arch = "wasm32"))]
    if block {
        let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
    }
    #[cfg(target_arch = "wasm32")]
    let _ = block;
    let mut cx = Context::from_waker(Waker::noop());
    let Poll::Ready((gpu, result)) = flight.future.as_mut().poll(&mut cx) else {
        return false;
    };
    let flight = r.flight.take().expect("polled above");
    r.gpu = Some(gpu);
    match result {
        Ok(g) => {
            r.motion.accept(&flight.batch, &g);
            stats.0 += 1;
            stats.1 = (flight.batch.characters.len(), now_ms() - flight.started);
        }
        Err(e) => {
            if errors.len() < 8 {
                errors.push(e.to_string());
            }
        }
    }
    true
}

fn start_flight(r: &mut Ready) {
    if r.flight.is_some() {
        return;
    }
    let Some(batch) = r.motion.batch(usize::MAX) else {
        return;
    };
    let Some(mut gpu) = r.gpu.take() else {
        return;
    };
    let inputs = batch.inputs.clone();
    r.flight = Some(Flight {
        future: Box::pin(async move {
            let result = gpu.generate(&inputs).await;
            (gpu, result)
        }),
        batch,
        started: now_ms(),
    });
}

/// `pose` re-rooted: the hips over the origin at their height, facing +Z.
fn rerooted(skeleton: &Skeleton, pose: &Pose) -> Vec<Trs> {
    let h = heading(pose.positions[RIGHT_LEG], pose.positions[LEFT_LEG]);
    let mut local = skeleton.local(pose);
    let undo = rotation_of(-h);
    local[0].translation = Vec3::new(0.0, local[0].translation.y, 0.0);
    local[0].rotation = (undo * local[0].rotation).normalize();
    local
}

/// The palette of `a` for the frame between `prev` and `next` at `phase`.
fn palette(r: &Ready, a: &Seen, s: &mut Shown) -> Option<Vec<Mat4>> {
    let (target, map) = s.target.as_ref()?;
    let prev = s.prev?;
    let next = s.next.unwrap_or(prev);
    let frames = [
        rerooted(&r.soma_skeleton, &prev.pose),
        rerooted(&r.soma_skeleton, &next.pose),
    ];
    let clip = og_motion::soma::clip("motion", &r.soma, &frames);
    let out = og_anim::retarget(
        &r.soma,
        &r.soma_map,
        target,
        map,
        &clip,
        &RetargetOptions::default(),
    )
    .ok()?;
    let mut pose = AnimPose::rest(target);
    let binding = og_anim::bind(target, &out.clip);
    og_anim::sample(target, &out.clip, &binding, s.phase / RATE, NONE, &mut pose);
    let contacts = if s.phase < 0.5 {
        prev.contacts
    } else {
        next.contacts
    };
    let root = Trs::new(a.position, a.rotation, Vec3::splat(a.scale));
    s.planter.step(target, &mut pose.nodes, &root, &contacts);
    let model = target.model_space(&pose.nodes);
    for (k, (ankle, contact)) in [(map.nodes[16], 0), (map.nodes[20], 2)]
        .into_iter()
        .enumerate()
    {
        let down = contacts[contact] > og_motion::soma::CONTACT;
        if ankle == NONE || !down {
            s.planted[k] = None;
            continue;
        }
        let at = root.transform_point(model[ankle as usize].translation);
        if let Some(was) = s.planted[k] {
            let d = (at - was).length();
            s.slide = (s.slide.0 + d, s.slide.1 + 1, s.slide.2.max(d));
        }
        s.planted[k] = Some(at);
    }
    let (l, r) = (map.nodes[14], map.nodes[18]);
    if l != NONE && r != NONE {
        let at = |n: u32| model[n as usize].translation.to_array();
        s.facing = heading(at(r), at(l));
    }
    let mut m = Vec::new();
    target.palette(a.skin, &pose.nodes, &mut m);
    (!m.is_empty()).then_some(m)
}

/// Before a frame is drawn: the model loaded, goals given, batches run and
/// each agent's palette made (`alpha`: the frame between the last two
/// ticks, as `super::super::interpolate`).
pub(crate) fn present(eng: &Engine, alpha: Option<f32>) {
    let agents = seen(eng);
    let (tick, dt) = match eng.world() {
        Ok(w) => (w.time().tick, w.time().dt),
        Err(_) => return,
    };
    let now = (tick as f64 + f64::from(alpha.unwrap_or(0.0))) * f64::from(dt);
    let sys = eng.state.borrow().anim.clone();
    let goals = lock(&shared(eng.id)).clone();
    let mut rs = eng.render.borrow_mut();
    let rs = &mut *rs;
    let p = &mut rs.motion;
    p.palettes.clear();
    if agents.is_empty() && !p.wanted {
        p.agents.clear();
        return;
    }
    if matches!(p.model, Model::Unloaded | Model::Waiting(_)) {
        let mut bytes = 0;
        p.model = load(eng, rs.backend.as_deref(), &mut bytes);
        p.model_bytes = p.model_bytes.max(bytes);
    }
    let elapsed = p.clock.map_or(0.0, |c| (now - c).clamp(0.0, 0.25)) as f32;
    p.clock = Some(now);
    let Model::Ready(r) = &mut p.model else {
        return;
    };
    let mut stats = (p.batches, p.last_batch);
    poll_flight(r, false, &mut stats, &mut p.errors);
    p.agents
        .retain(|e, _| agents.iter().any(|a| a.entity == *e));
    for a in &agents {
        let s = p.agents.entry(a.entity).or_insert_with(|| {
            let idle = r.library.frames(0);
            let placed = Goal::placed(idle, [a.position.x, a.position.z], heading_of(a.rotation));
            let mut recent = [placed.targets[0]; 4];
            for (k, t) in placed.targets.iter().take(4).enumerate() {
                recent[k] = *t;
            }
            Shown {
                character: r.motion.add(recent),
                prev: None,
                next: None,
                phase: 0.0,
                serial: u32::MAX,
                planter: Planter::new(Vec::new()),
                skeleton: 0,
                rig: u64::MAX,
                target: None,
                last_position: a.position,
                velocity: Vec3::ZERO,
                shown: 0,
                starved: 0,
                facing: 0.0,
                planted: [None; 2],
                slide: (0.0, 0, 0.0),
            }
        });
        if s.skeleton != a.skeleton || s.rig != a.settings.rig {
            s.skeleton = a.skeleton;
            s.rig = a.settings.rig;
            s.target = sys.as_ref().and_then(|sys| {
                crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent").ok()
            });
            let legs = s.target.as_ref().map_or(Vec::new(), |(_, m)| {
                [(14usize, 0usize), (18, 2)]
                    .iter()
                    .filter(|(b, _)| (0..3).all(|k| m.nodes[b + k] != NONE))
                    .map(|&(b, contact)| Leg {
                        chain: [m.nodes[b], m.nodes[b + 1], m.nodes[b + 2]],
                        contact,
                    })
                    .collect()
            });
            s.planter = Planter::new(legs);
        }
        if elapsed > 0.0 {
            s.velocity = (a.position - s.last_position) / elapsed;
        }
        s.last_position = a.position;
        let serial = goals.agents.get(&a.entity).map_or(0, |g| g.serial);
        if serial != s.serial {
            s.serial = serial;
            r.motion.clear_goals(s.character);
        }
        let lookahead = (a.settings.lookahead * RATE) as usize;
        if r.motion.goals(s.character) == 0 && r.motion.queued(s.character) < lookahead.max(1) {
            let (at, h, pose) = predict(&goals, a, s);
            r.motion.push(
                s.character,
                Goal::placed(r.library.frames(pose), [at.x, at.z], h),
            );
        }
    }
    start_flight(r);
    for a in &agents {
        let s = p.agents.get_mut(&a.entity).expect("added above");
        s.phase += elapsed * RATE;
        if s.prev.is_none() {
            s.phase = 0.0;
        }
        for _ in 0..3 {
            while s.next.is_none() || s.phase >= 1.0 {
                let Some(f) = r.motion.pop(s.character) else {
                    break;
                };
                if s.prev.is_none() {
                    s.prev = Some(f);
                } else if s.next.is_none() {
                    s.next = Some(f);
                } else {
                    s.prev = s.next;
                    s.next = Some(f);
                    s.phase -= 1.0;
                }
            }
            let starved = s.prev.is_none() || (s.next.is_none() || s.phase >= 1.0);
            // A native host draws the generated motion, never a gap: it
            // waits for the batch in flight when the agent ran out.
            if !(starved && cfg!(not(target_arch = "wasm32")) && r.flight.is_some()) {
                break;
            }
            poll_flight(r, true, &mut stats, &mut p.errors);
            start_flight(r);
        }
        if s.next.is_none() || s.phase >= 1.0 {
            s.phase = s.phase.min(1.0);
            s.starved += 1;
        }
        if let Some(m) = palette(r, a, s) {
            s.shown += 1;
            p.palettes.insert(a.entity, m);
        }
    }
    start_flight(r);
    (p.batches, p.last_batch) = stats;
}

/// og.motion.capabilities.
pub(crate) fn capabilities(eng: &Engine) -> Value {
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    p.wanted = true;
    let (backend, reason) = match &p.model {
        Model::Ready(_) => ("gpu", String::new()),
        Model::Unloaded => (
            "none",
            "loading: the model is read at the next frame".into(),
        ),
        Model::Waiting(why) => ("none", why.clone()),
        Model::Failed(why) => ("none", why.clone()),
    };
    json!({ "backend": backend, "reason": reason, "modelBytes": p.model_bytes as f64 })
}

/// The presenter's side of `dump_motion`.
pub(crate) fn dump(eng: &Engine) -> Value {
    let rs = eng.render.borrow();
    let p = &rs.motion;
    let (state, reason) = match &p.model {
        Model::Ready(_) => ("gpu", String::new()),
        Model::Unloaded => ("unloaded", String::new()),
        Model::Waiting(w) => ("waiting", w.clone()),
        Model::Failed(f) => ("failed", f.clone()),
    };
    let ready = match &p.model {
        Model::Ready(r) => Some(r),
        _ => None,
    };
    let agents: Vec<Value> = p
        .agents
        .iter()
        .map(|(e, s)| {
            json!({
                "entity": *e as f64,
                "ahead": ready.map_or(0, |r| r.motion.queued(s.character)),
                "modelGoals": ready.map_or(0, |r| r.motion.goals(s.character)),
                "framesShown": s.shown,
                "framesStarved": s.starved,
                "facing": s.facing,
                "plantedSlide": {
                    "frames": s.slide.1,
                    "meanMm": if s.slide.1 > 0 { 1000.0 * s.slide.0 / s.slide.1 as f32 } else { 0.0 },
                    "maxMm": 1000.0 * s.slide.2,
                },
                "rig": s.target.as_ref().map(|(_, m)| m.scheme.clone()),
                "drawn": p.palettes.contains_key(e),
            })
        })
        .collect();
    json!({
        "state": state,
        "reason": reason,
        "modelBytes": p.model_bytes as f64,
        "poses": POSES,
        "inFlight": ready.is_some_and(|r| r.flight.is_some()),
        "batches": p.batches,
        "lastBatch": { "characters": p.last_batch.0, "ms": p.last_batch.1 },
        "errors": p.errors,
        "agents": agents,
    })
}
