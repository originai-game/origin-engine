//! og.motion's presentation: the motion model on the renderer's GPU, and
//! each agent's generated pose drawn in place of og.anim's joint palette.
//!
//! Before each frame is drawn ([`present`], from `engine::tick`'s render):
//!
//! 1. **The weights.** As soon as the open bundle declares the model (its
//!    manifest's `motion` entry, a game that uses og.motion) and a
//!    renderer is attached, `cooked/motion/model.ogb` is read, unpacked
//!    and its two networks put on the renderer's device
//!    ([`RenderBackend::compute`]): natively on a thread of its own, on
//!    the web from a fetch that lands at a later frame. Until then, or
//!    when that fails, the backend is none and og.anim's pose is drawn.
//! 2. **Goals for the model.** An agent whose generated frames run short
//!    of its `lookahead` (or of two batches' measured latency) gets the
//!    next model goal: where the simulation's goals will have it at the
//!    first change of its pose, at most [`HORIZON`] seconds on (the mover
//!    run ahead with [`super::advance`]; without drive, its Transform's
//!    velocity), in the pose it will be in there.
//! 3. **One batch for every agent** runs at a time, never blocking the
//!    frame: it is submitted in slices of about [`FRAME_BUDGET_MS`] of GPU
//!    time, one a frame, its readback polled each frame, and its
//!    characters decoded into frames a few a frame ([`DECODE_BUDGET_MS`]).
//!    Until an agent's first frames land, og.anim's pose is drawn; after
//!    that a native host blocks when the agent has nothing left to show
//!    (`og run` stepping faster than real time draws every frame that way,
//!    so its frames after the first batch do not depend on timing).
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
use og_motion::Bundle;
use og_motion::contract::{
    BOUNDARY_FRAMES, FRAMES, MAX_TOKENS, MIN_TOKENS, MOTION_FEATURES, POSES_PER_TOKEN,
};
use og_motion::contract::{LEFT_LEG, Pose, RIGHT_LEG, heading};
use og_motion::glue::{Statics, StudentOutput, decoder_inputs, finish_character, student_inputs};
use og_motion::gpu::{Gpu, GpuError};
use og_motion::intent::{Batch, Frame, Goal, Motion};
use og_motion::library::{Library, POSES};
use og_motion::soma::{Leg, Planter, Skeleton};
use og_render::wgpu;
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
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
/// The GPU time a batch may add to one frame, ms: a batch runs in slices of
/// about this much, one a frame (`Gpu::set_frame_budget`), so a frame of
/// 16.7 ms keeps room for the scene.
const FRAME_BUDGET_MS: f64 = 3.0;
/// The CPU time a landed batch's decoding may take in one frame, ms: its
/// characters are decoded in turn until this is spent (one at least), the
/// rest in the next frames (16 agents took 16.5 ms in one frame).
const DECODE_BUDGET_MS: f64 = 3.0;
/// The model's frame rate.
const RATE: f32 = 30.0;
/// The most an agent's body leans from upright in the frames of a jump
/// goal, rad (30°): the pelvis to the chest (the root's pitch) and the
/// hips to the head (the spine's), each.
const JUMP_LEAN: f32 = 0.524;
/// The same for a crouch goal (45°): the knees and hips bend, the torso
/// inclines, the body does not pitch down flat.
const CROUCH_LEAN: f32 = 0.785;
/// How fast the drawn body may pitch further forward, rad/s (150°/s): a
/// jump into a crouch cannot fold flat in a few frames across the goals'
/// boundary.
const LEAN_RATE: f32 = 2.62;
/// Tokens either side of a model goal's own duration it may take
/// ([`token_counts`]).
const TOKEN_SLACK: usize = 2;
/// The library's jump pose.
const JUMP: usize = 3;
/// The library's crouch pose.
const CROUCH: usize = 4;
/// SOMA's chest joint.
const CHEST: usize = 3;
/// SOMA's first spine joint (the hips' child toward the head).
const SPINE: usize = 1;
/// How far behind the frame clock a drawn frame may fall before frames are
/// skipped to catch up, s.
const MAX_LAG: f64 = 0.25;
/// SOMA's head joint.
const HEAD: usize = 6;

type Running = Pin<Box<dyn Future<Output = (Gpu, Result<(StudentOutput, Vec<f32>), GpuError>)>>>;

/// A batch on the GPU.
struct Flight {
    future: Running,
    batch: Batch,
    started: f64,
}

/// A batch read back, its characters decoded a few a frame
/// ([`DECODE_BUDGET_MS`]) into their agents' frames.
struct Landing {
    batch: Batch,
    student: StudentOutput,
    local: Vec<f32>,
    /// The next character to decode.
    next: usize,
    /// One character's motion, reused.
    motion: Vec<f32>,
    started: f64,
    /// Waited for (a starved agent on a native host): its latency is not
    /// the sliced one.
    forced: bool,
}

struct Ready {
    /// None while a batch has it.
    gpu: Option<Gpu>,
    /// Polled to wait for a batch (native only).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    device: wgpu::Device,
    flight: Option<Flight>,
    landing: Option<Landing>,
    statics: Arc<Statics>,
    motion: Motion,
    library: Library,
    soma: SkeletonAsset,
    soma_skeleton: Skeleton,
    soma_map: HumanoidMap,
    /// The device's rate as the last batch measured it, GFLOP/s.
    rate: f64,
    /// A sliced batch's latency, start to decoded, as the last one not
    /// waited for measured (ms; 500 before one has).
    latency: f64,
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
    warmup: u64,
    /// The drawn pose's heading in the character's own space (its thighs'
    /// across direction): about 0 when it faces where its Transform does.
    facing: f32,
    /// Each leg's ankle in the world at the last frame it was planted (the
    /// model's heel contact), and how far planted ankles moved between
    /// planted frames: the summed metres and the frames (`dump_motion`'s
    /// `plantedSlide`, the foot plant's own measure).
    planted: [Option<Vec3>; 2],
    slide: (f32, u32, f32),
    /// The lean (hips to head) last drawn, rad.
    lean: Option<f32>,
    drawn_lean: Option<f32>,
    /// The model goals asked for, oldest first, until their frames are
    /// shown: what each frame's tag names.
    issued: VecDeque<Issued>,
    next_id: u32,
}

/// A model goal asked for: its id (the frames' tag), its library pose and
/// the frame clock its first frame is for (now plus the frames queued
/// before it).
struct Issued {
    id: u32,
    pose: usize,
    start: f64,
}

impl Shown {
    fn issued(&self, f: &Frame) -> Option<&Issued> {
        self.issued.iter().find(|g| g.id == f.tag)
    }

    fn count_wait(&mut self) {
        if self.prev.is_none() {
            self.warmup += 1;
        } else if self.next.is_none() || self.phase >= 1.0 {
            self.starved += 1;
        }
    }
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
    #[cfg(not(target_arch = "wasm32"))]
    loading: Option<Loading>,
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

/// The model file the open bundle declares (its manifest's `motion` entry,
/// which og cook writes for a project whose code uses og.motion): `None`
/// while no bundle is open, else the path or why there is none.
fn declared(eng: &Engine) -> Option<Result<String, String>> {
    let a = eng.assets.borrow();
    let i = a.default_bundle?;
    let crate::assets::Payload::Bundle { bundle, dir } = &a.entries[i].payload else {
        return None;
    };
    Some(match &bundle.manifest().motion {
        Some(m) => Ok(format!("{dir}/{}", m.file)),
        None => Err(format!(
            "{dir} declares no og.motion model: og cook writes {MODEL_FILE} for a project whose src/ uses og.motion and finds the engine's weights"
        )),
    })
}

/// Whether the open bundle declares og.motion's model (the game uses it).
pub(crate) fn declares(eng: &Engine) -> bool {
    matches!(declared(eng), Some(Ok(_)))
}

/// What [`build`] makes off the engine thread: everything of [`Ready`] but
/// the batch state.
struct Parts {
    gpu: Gpu,
    statics: Statics,
    device: wgpu::Device,
    library: Library,
    soma: SkeletonAsset,
    soma_skeleton: Skeleton,
    soma_map: HumanoidMap,
}

/// The model from its pack's bytes, on `device`.
fn build(bytes: &[u8], device: wgpu::Device, queue: wgpu::Queue) -> Result<Parts, String> {
    let files = og_bundle::runtime::unpack(bytes).map_err(|e| e.to_string())?;
    let (_, weights) = files
        .iter()
        .find(|(n, _)| n == MODEL_ENTRY)
        .ok_or_else(|| format!("it holds no {MODEL_ENTRY}"))?;
    let bundle = Bundle::from_safetensors(weights).map_err(|e| e.to_string())?;
    let library = Library::from_bundle(&bundle).map_err(|e| e.to_string())?;
    let soma_skeleton = Skeleton::from_bundle(&bundle).map_err(|e| e.to_string())?;
    let soma = soma_skeleton.asset();
    let soma_map = HumanoidMap::detect(&soma);
    let statics = Statics::new(&bundle).map_err(|e| e.to_string())?;
    let mut gpu = Gpu::new(device.clone(), queue, &bundle).map_err(|e| e.to_string())?;
    gpu.set_blocking(false);
    gpu.set_frame_budget(Some(FRAME_BUDGET_MS));
    Ok(Parts {
        gpu,
        statics,
        device,
        library,
        soma,
        soma_skeleton,
        soma_map,
    })
}

fn ready(parts: Parts) -> Model {
    Model::Ready(Box::new(Ready {
        rate: parts.gpu.rate(),
        gpu: Some(parts.gpu),
        device: parts.device,
        flight: None,
        landing: None,
        latency: 500.0,
        statics: Arc::new(parts.statics),
        motion: Motion::default(),
        library: parts.library,
        soma: parts.soma,
        soma_skeleton: parts.soma_skeleton,
        soma_map: parts.soma_map,
    }))
}

/// A load on a native host's background thread: the pack's size, then the
/// model or why it failed.
#[cfg(not(target_arch = "wasm32"))]
type Loading = std::sync::mpsc::Receiver<(u64, Result<Parts, String>)>;

/// Starts or continues reading the model and putting it on the renderer's
/// device; answers the state. Natively the file is read, unpacked and
/// uploaded on a thread of its own (wgpu's device takes buffers and
/// pipelines from any thread), so no frame waits for it; on the web the
/// host's fetch lands at a later frame, which then unpacks and uploads it.
fn load(p: &mut Presenter, eng: &Engine, backend: Option<&dyn RenderBackend>) -> Model {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rx) = &p.loading {
        return match rx.try_recv() {
            Ok((bytes, r)) => {
                p.loading = None;
                p.model_bytes = p.model_bytes.max(bytes);
                r.map_or_else(Model::Failed, ready)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                Model::Waiting("loading: the weights are being read and uploaded".into())
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                p.loading = None;
                Model::Failed("the model's loading thread ended without an answer".into())
            }
        };
    }
    let Some((device, queue)) = backend.and_then(|b| b.compute()) else {
        return Model::Waiting(
            "no renderer is attached: the motion model runs on the renderer's GPU".into(),
        );
    };
    let path = match declared(eng) {
        None => return Model::Waiting("the cooked bundle is not open yet".into()),
        Some(Err(why)) => return Model::Failed(why),
        Some(Ok(path)) => path,
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("og-motion load".into())
            .spawn(move || {
                let answer = match std::fs::read(&path) {
                    Ok(bytes) => (
                        bytes.len() as u64,
                        build(&bytes, device, queue).map_err(|e| format!("{path}: {e}")),
                    ),
                    Err(e) => (0, Err(format!("{path}: {e}"))),
                };
                let _ = tx.send(answer);
            });
        if let Err(e) = spawned {
            return Model::Failed(format!("the model's loading thread: {e}"));
        }
        p.loading = Some(rx);
        Model::Waiting("loading: the weights are being read and uploaded".into())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = match hooks().map(|h| h.read_asset(&path)) {
            Some(crate::assets::Fetch::Ready(b)) => b,
            Some(crate::assets::Fetch::Pending) => {
                return Model::Waiting(format!("loading {path}"));
            }
            Some(crate::assets::Fetch::Failed(e)) => return Model::Failed(format!("{path}: {e}")),
            None => return Model::Failed("no host is installed to read assets".into()),
        };
        p.model_bytes = p.model_bytes.max(bytes.len() as u64);
        build(&bytes, device, queue).map_or_else(|e| Model::Failed(format!("{path}: {e}")), ready)
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

/// The library pose a driven agent is in, moving at `speed`: a place or
/// pose goal asks for its own pose even while the mover carries the
/// character there; only a walk is walked or run.
fn pose_of(agent: &super::Agent, speed: f32) -> usize {
    match agent.current {
        Some(c) if matches!(c.goal.kind, Kind::Place | Kind::Pose) => c.goal.pose as usize,
        _ if speed > 0.3 => gait(speed),
        Some(c) if c.goal.kind == Kind::WalkTo => c.goal.pose as usize,
        _ => agent.hold as usize,
    }
}

/// The next model goal of an agent whose generated frames run `queued`
/// seconds past now: where and facing which way the simulation will have
/// it, the library pose it will be in, and how many seconds after those
/// frames that is. A driven agent's mover is run past the queued frames,
/// then on to where its pose first changes, at most [`HORIZON`]:
/// the model reaches each pose of the course in
/// turn (a run, then the jump at its landing) when the simulation does,
/// instead of skipping a short goal for the one after it.
fn predict(goals: &Goals, a: &Seen, shown: &Shown, queued: f32) -> (Vec3, f32, usize, f32) {
    let h0 = heading_of(a.rotation);
    match goals.agents.get(&a.entity) {
        Some(agent) if a.settings.drive => {
            let mut agent = agent.clone();
            let (mut p, mut h) = (a.position, h0);
            let step = 1.0 / RATE;
            for _ in 0..(queued * RATE) as usize {
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
            }
            let mut last = None;
            for k in 0..(HORIZON * RATE) as usize {
                let (p0, h0) = (p, h);
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
                let pose = pose_of(&agent, (p - p0).length() / step);
                if let Some(prev) = last
                    && pose != prev
                {
                    return (p0, h0, prev, k as f32 * step);
                }
                last = Some(pose);
            }
            (p, h, last.unwrap_or(agent.hold as usize), HORIZON)
        }
        agent => {
            let v = Vec3::new(shown.velocity.x, 0.0, shown.velocity.z);
            let speed = v.length();
            if speed > 0.3 {
                (
                    a.position + v * (queued + HORIZON),
                    og_core::math::atan2(v.x, v.z),
                    gait(speed),
                    HORIZON,
                )
            } else {
                let pose = agent.map_or(0, |g| match g.current {
                    Some(c) if c.goal.kind == Kind::Pose => c.goal.pose as usize,
                    _ => g.hold as usize,
                });
                (a.position, h0, pose, HORIZON)
            }
        }
    }
}

/// The token counts a model goal `seconds` long may take: about as many
/// as make its motion frames last that long (the model reaches the goal's
/// pose at its last frame, so the drawn motion keeps the simulation's
/// time), with a little room either side for the model's own timing.
fn token_counts(seconds: f32) -> Vec<usize> {
    let frames = seconds * RATE + (2 * BOUNDARY_FRAMES + 1) as f32;
    let n = (frames / POSES_PER_TOKEN as f32).round() as usize;
    let lo = n.saturating_sub(TOKEN_SLACK).clamp(MIN_TOKENS, MAX_TOKENS);
    (lo..=(n + TOKEN_SLACK).clamp(MIN_TOKENS, MAX_TOKENS)).collect()
}

/// Polls the batch in flight once; `block` waits for it (native only).
fn poll_flight(r: &mut Ready, block: bool, errors: &mut Vec<String>) -> bool {
    let Some(flight) = r.flight.as_mut() else {
        return false;
    };
    let mut cx = Context::from_waker(Waker::noop());
    // Each poll submits the batch's next slice or polls its readback;
    // blocking (native) runs every slice left and waits for the GPU.
    let (gpu, result) = loop {
        match flight.future.as_mut().poll(&mut cx) {
            Poll::Ready(done) => break done,
            Poll::Pending if block && cfg!(not(target_arch = "wasm32")) => {
                #[cfg(not(target_arch = "wasm32"))]
                let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
            }
            Poll::Pending => return false,
        }
    };
    let flight = r.flight.take().expect("polled above");
    r.rate = gpu.rate();
    r.gpu = Some(gpu);
    match result {
        Ok((student, local)) => {
            r.landing = Some(Landing {
                batch: flight.batch,
                student,
                local,
                next: 0,
                motion: vec![0.0; FRAMES * MOTION_FEATURES],
                started: flight.started,
                forced: block,
            });
        }
        Err(e) => {
            if errors.len() < 8 {
                errors.push(e.to_string());
            }
        }
    }
    true
}

/// Decodes the landed batch's characters into their frames for about
/// `budget` ms (`None`: all of them); answers whether none are left.
fn decode(r: &mut Ready, budget: Option<f64>, stats: &mut (u64, (usize, f64))) -> bool {
    let Some(l) = r.landing.as_mut() else {
        return true;
    };
    let t0 = now_ms();
    let n = l.batch.characters.len();
    while l.next < n {
        let i = l.next;
        finish_character(
            &r.statics,
            &l.batch.inputs,
            &l.student,
            &l.local,
            i,
            &mut l.motion,
        );
        r.motion
            .accept_character(&l.batch, i, l.student.counts[i], &l.motion);
        l.next += 1;
        if budget.is_some_and(|b| now_ms() - t0 >= b) {
            break;
        }
    }
    if l.next < n {
        return false;
    }
    let latency = now_ms() - l.started;
    if !(l.forced || budget.is_none()) {
        r.latency = latency;
    }
    stats.0 += 1;
    stats.1 = (n, latency);
    r.landing = None;
    true
}

fn start_flight(r: &mut Ready) {
    if r.flight.is_some() || r.landing.is_some() {
        return;
    }
    let Some(batch) = r.motion.batch(usize::MAX) else {
        return;
    };
    let Some(mut gpu) = r.gpu.take() else {
        return;
    };
    let inputs = batch.inputs.clone();
    let statics = r.statics.clone();
    r.flight = Some(Flight {
        future: Box::pin(async move {
            let result = async {
                let student = gpu.student(&student_inputs(&statics, &inputs)).await?;
                let local = gpu
                    .decoder(&decoder_inputs(&statics, &inputs, &student))
                    .await?;
                Ok((student, local))
            }
            .await;
            (gpu, result)
        }),
        batch,
        started: now_ms(),
    });
}

/// `pose` re-rooted: the hips over the origin at their height, facing +Z.
/// How far `v` leans from straight up, rad.
fn lean_of(v: Vec3) -> f32 {
    og_core::math::acos(v.normalize_or_zero().y.clamp(-1.0, 1.0))
}

/// The rotation that brings `v` to lean `max` from up (identity when it
/// leans no more).
fn unlean(v: Vec3, max: f32) -> glam::Quat {
    let axis = v.cross(Vec3::Y);
    let lean = lean_of(v);
    if lean <= max || axis.length_squared() < 1e-12 {
        return glam::Quat::IDENTITY;
    }
    og_core::math::quat_from_axis_angle(axis.normalize(), lean - max)
}

/// `pose`'s local transforms re-rooted: the hips over the origin at their
/// height, facing +Z. Its lean from upright is bounded by `max`: first, off
/// the ground, the root's pitch (pelvis to chest), turning the whole body
/// about the hips,
/// then the spine's (hips to head), turning the torso at the spine so the
/// legs keep their bend. Answers the transforms and the lean (hips to
/// head) they have.
fn rerooted(
    skeleton: &Skeleton,
    asset: &SkeletonAsset,
    pose: &Pose,
    max: f32,
    grounded: bool,
) -> (Vec<Trs>, f32) {
    let h = heading(pose.positions[RIGHT_LEG], pose.positions[LEFT_LEG]);
    let mut local = skeleton.local(pose);
    let undo = rotation_of(-h);
    local[0].translation = Vec3::new(0.0, local[0].translation.y, 0.0);
    local[0].rotation = (undo * local[0].rotation).normalize();
    // Measured on the transforms as drawn (forward kinematics), not on the
    // model's positions.
    let from_hips = |local: &[Trs], j: usize| {
        let m = asset.model_space(local);
        m[j].translation - m[0].translation
    };
    // On the ground the legs stay where the model put them (turning the
    // root would swing a planted foot): only the torso is turned there.
    if !grounded {
        let root = unlean(from_hips(&local, CHEST), max);
        local[0].rotation = (root * local[0].rotation).normalize();
    }
    if skeleton.parents[SPINE] == 0 {
        let spine = unlean(from_hips(&local, HEAD), max);
        let r0 = local[0].rotation;
        local[SPINE].rotation = (r0.inverse() * spine * r0 * local[SPINE].rotation).normalize();
    }
    let lean = lean_of(from_hips(&local, HEAD));
    (local, lean)
}

/// The lean a frame of library pose `pose` may have, rad.
fn lean_bound(pose: Option<usize>) -> f32 {
    match pose {
        Some(JUMP) => JUMP_LEAN,
        Some(CROUCH) => CROUCH_LEAN,
        _ => std::f32::consts::PI,
    }
}

fn palette(
    r: &Ready,
    a: &Seen,
    s: &mut Shown,
    elapsed: f32,
    goals: &Goals,
    sys: &og_anim::AnimationSystem,
) -> Option<Vec<Mat4>> {
    let (target, map) = s.target.as_ref()?;
    let prev = s.prev?;
    let next = s.next.unwrap_or(prev);
    // A jump's and a crouch's frames keep the body near upright (the model
    // can pitch it close to horizontal, which reads as a dive), and no
    // frame pitches forward faster than LEAN_RATE from the last drawn.
    let rate = s.lean.map_or(std::f32::consts::PI, |l| {
        l + LEAN_RATE * elapsed.max(1.0 / 120.0)
    });
    let grounded = |f: &Frame| f.contacts.iter().any(|c| *c > og_motion::soma::CONTACT);
    let bound = |f: &Frame| lean_bound(s.issued(f).map(|g| g.pose)).min(rate);
    let (a0, l0) = rerooted(
        &r.soma_skeleton,
        &r.soma,
        &prev.pose,
        bound(&prev),
        grounded(&prev),
    );
    let (a1, l1) = rerooted(
        &r.soma_skeleton,
        &r.soma,
        &next.pose,
        bound(&next),
        grounded(&next),
    );
    s.lean = Some(l0 + (l1 - l0) * s.phase.clamp(0.0, 1.0));
    let frames = [a0, a1];
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
    if let Some(agent) = goals.agents.get(&a.entity) {
        agent
            .performance
            .apply(sys, target, map, &root, &mut pose.nodes);
    }
    let model = target.model_space(&pose.nodes);
    s.drawn_lean = map.node("hips").zip(map.node("head")).map(|(hips, head)| {
        lean_of(model[head as usize].translation - model[hips as usize].translation)
    });
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

/// Compose authored performance on the animation pose during model startup.
fn fallback(
    eng: &Engine,
    p: &mut Presenter,
    agents: &[Seen],
    goals: &Goals,
    sys: Option<&og_anim::AnimationSystem>,
) {
    let Some(sys) = sys else {
        return;
    };
    for a in agents {
        if p.palettes.contains_key(&a.entity) {
            continue;
        }
        let Some(agent) = goals.agents.get(&a.entity) else {
            continue;
        };
        if !agent.performance.looking() && !agent.performance.reacting() {
            continue;
        }
        let Ok((asset, map)) =
            crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent")
        else {
            continue;
        };
        let Some(mut nodes) = sys.pose(og_ecs::Entity::from_bits(a.entity)) else {
            continue;
        };
        let root = Trs::new(a.position, a.rotation, Vec3::splat(a.scale));
        agent
            .performance
            .apply(sys, &asset, &map, &root, &mut nodes);
        let mut palette = Vec::new();
        asset.palette(a.skin, &nodes, &mut palette);
        p.palettes.insert(a.entity, palette);
    }
}

/// Before a frame is drawn: generated motion, then authored performance.
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
    // The model loads as soon as the bundle declares it (a game that uses
    // og.motion), before any agent needs it.
    if agents.is_empty() && !p.wanted && !declares(eng) {
        p.agents.clear();
        return;
    }
    if matches!(p.model, Model::Unloaded | Model::Waiting(_)) {
        p.model = load(p, eng, rs.backend.as_deref());
    }
    if agents.is_empty() {
        p.agents.clear();
        return;
    }
    let elapsed = p.clock.map_or(0.0, |c| (now - c).clamp(0.0, 0.25)) as f32;
    p.clock = Some(now);
    let Model::Ready(r) = &mut p.model else {
        fallback(eng, p, &agents, &goals, sys.as_ref());
        return;
    };
    let mut stats = (p.batches, p.last_batch);
    poll_flight(r, false, &mut p.errors);
    decode(r, Some(DECODE_BUDGET_MS), &mut stats);
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
                warmup: 0,
                facing: 0.0,
                planted: [None; 2],
                slide: (0.0, 0, 0.0),
                lean: None,
                drawn_lean: None,
                issued: VecDeque::new(),
                next_id: 0,
            }
        });
        if s.skeleton != a.skeleton || s.rig != a.settings.rig {
            s.skeleton = a.skeleton;
            s.rig = a.settings.rig;
            s.target = sys.as_ref().and_then(|sys| {
                match crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent") {
                    Ok(target) => Some(target),
                    Err(error) => {
                        p.errors.push(format!("agent {}: {error}", a.entity));
                        None
                    }
                }
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
        // At least the agent's lookahead, and enough to cover two batches'
        // latency (sliced across frames, 0.5 to 0.6 s for 16 agents): one
        // batch runs at a time, so a goal asked for while another is on its
        // way waits for it, then for its own.
        let ahead = a.settings.lookahead.max(2.0 * r.latency as f32 / 1000.0);
        let lookahead = (ahead * RATE) as usize;
        if r.motion.goals(s.character) == 0 && r.motion.queued(s.character) < lookahead.max(1) {
            let queued = r.motion.queued(s.character) as f32 / RATE;
            let (at, h, pose, seconds) = predict(&goals, a, s, queued);
            let mut goal = Goal::placed(r.library.frames(pose), [at.x, at.z], h);
            goal.tag = s.next_id;
            // A walk or run keeps time by its distance; forced short, the
            // model crawled into a run. A pose (jump, crouch, wave) has no
            // distance to keep time by: its length is the segment's.
            if pose != gait(RUN_SPEED) && pose != gait(0.0) {
                goal.token_counts = token_counts(seconds);
            }
            s.issued.push_back(Issued {
                id: s.next_id,
                pose,
                start: now + f64::from(queued),
            });
            s.next_id += 1;
            r.motion.push(s.character, goal);
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
            // Until its first frames land an agent is drawn as og.anim
            // poses it: the fallback covers the model's start, as its load.
            let pending = r.flight.is_some() || r.landing.is_some();
            if !(starved && cfg!(not(target_arch = "wasm32")) && pending && s.prev.is_some()) {
                break;
            }
            poll_flight(r, true, &mut p.errors);
            decode(r, None, &mut stats);
            start_flight(r);
        }
        // Behind the clock (the first batch's latency, a goal the model
        // took longer over): frames are skipped until the drawn one is
        // within MAX_LAG of the time it was generated for.
        while let Some(next) = s.next
            && let Some(at) = s
                .issued(&next)
                .map(|g| g.start + f64::from(next.index) / f64::from(RATE))
            && now - at > MAX_LAG
            && let Some(f) = r.motion.pop(s.character)
        {
            s.prev = s.next;
            s.next = Some(f);
        }
        if let Some(prev) = s.prev {
            // Goals before the drawn frame's are done with (ids ascend).
            while s.issued.front().is_some_and(|g| g.id < prev.tag) {
                s.issued.pop_front();
            }
        }
        s.count_wait();
        if s.next.is_none() || s.phase >= 1.0 {
            s.phase = s.phase.min(1.0);
        }
        if let Some(sys) = sys.as_ref()
            && let Some(m) = palette(r, a, s, elapsed, &goals, sys)
        {
            s.shown += 1;
            p.palettes.insert(a.entity, m);
        }
    }
    start_flight(r);
    (p.batches, p.last_batch) = stats;
    fallback(eng, p, &agents, &goals, sys.as_ref());
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
                "framesWarmup": s.warmup,
                "facing": s.facing,
                "plantedSlide": {
                    "frames": s.slide.1,
                    "meanMm": if s.slide.1 > 0 { 1000.0 * s.slide.0 / s.slide.1 as f32 } else { 0.0 },
                    "maxMm": 1000.0 * s.slide.2,
                },
                "rig": s.target.as_ref().map(|(_, m)| m.scheme.clone()),
                "rigMap": s.rig as f64,
                "bones": s.target.as_ref().map(|(asset, m)| m.bones(asset)),
                // The drawn body's lean from upright (hips to head).
                "leanDeg": s.drawn_lean.map(|l| l.to_degrees()),
                "sourceLeanDeg": s.lean.map(|l| l.to_degrees()),
                // The drawn frame: the pose its model goal asked for, and
                // how far behind the frame clock it is (s).
                "drawnPose": s.next.and_then(|f| s.issued(&f)).map(|g| POSES[g.pose]),
                "lag": s.next.and_then(|f| s.issued(&f).map(|g| p.clock.unwrap_or(0.0) - (g.start + f64::from(f.index) / f64::from(RATE)))),
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
        "frameBudgetMs": FRAME_BUDGET_MS,
        "gpuGflops": ready.map_or(0.0, |r| r.rate),
        "errors": p.errors,
        "agents": agents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown() -> Shown {
        Shown {
            character: 0,
            prev: None,
            next: None,
            phase: 0.0,
            serial: 0,
            planter: Planter::new(Vec::new()),
            skeleton: 1,
            rig: 42,
            target: None,
            last_position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            shown: 0,
            starved: 0,
            warmup: 0,
            facing: 0.0,
            planted: [None; 2],
            slide: (0.0, 0, 0.0),
            lean: None,
            drawn_lean: None,
            issued: VecDeque::new(),
            next_id: 0,
        }
    }

    #[test]
    fn prediction_keeps_a_short_remaining_pose_before_the_next_goal() {
        let goal = |pose, duration| super::super::Goal {
            kind: Kind::Pose,
            pose,
            duration,
            target: Vec3::ZERO,
            heading: 0.0,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
        };
        let a = Seen {
            entity: 1,
            settings: Settings::of(&json!({"drive":true})),
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            skeleton: 1,
            skin: 0,
        };
        for pose in [JUMP, 5] {
            let mut goals = Goals::default();
            let mut agent = super::super::Agent::default();
            agent.queue.push_back(goal(pose as u32, 0.2));
            agent.queue.push_back(goal(CROUCH as u32, 1.2));
            goals.agents.insert(1, agent);
            let (_, _, predicted, seconds) = predict(&goals, &a, &shown(), 0.0);
            assert_eq!(predicted, pose, "the imminent pose was skipped");
            assert!(seconds <= 0.3);
        }
    }

    #[test]
    fn warmup_is_not_playback_starvation() {
        let mut s = shown();
        for _ in 0..27 {
            s.count_wait();
        }
        assert_eq!((s.warmup, s.starved), (27, 0));
        let frame = Frame {
            pose: Pose {
                positions: [[0.0; 3]; og_motion::contract::JOINTS],
                rotations: [[[0.0; 3]; 3]; og_motion::contract::JOINTS],
            },
            contacts: [0.0; 4],
            tag: 0,
            index: 0,
        };
        s.prev = Some(frame);
        s.count_wait();
        assert_eq!((s.warmup, s.starved), (27, 1));
        s.next = Some(frame);
        s.phase = 0.99;
        s.count_wait();
        assert_eq!(s.starved, 1);
        s.phase = 1.0;
        s.count_wait();
        assert_eq!(s.starved, 2);
    }

    #[test]
    fn dump_distinguishes_the_rig_map_from_its_naming_scheme() {
        use crate::{og_engine_create, og_game_game, og_host_begin_bundle};
        use og_anim::{RigAsset, SkeletonNode};

        og_host_begin_bundle();
        let game = og_game_game(Some(r#"{"seed":1}"#.into()));
        let created: Value = serde_json::from_str(&og_engine_create(game, "{}".into())).unwrap();
        let eng = crate::engine::engine(created["value"].as_u64().unwrap() as u32).unwrap();
        let asset = SkeletonAsset {
            nodes: ["Hips", "Spine02", "Spine01", "Spine", "neck", "Head"]
                .iter()
                .enumerate()
                .map(|(i, name)| SkeletonNode {
                    name: (*name).into(),
                    parent: if i == 0 { NONE } else { i as u32 - 1 },
                    rest: Trs::IDENTITY,
                    mesh: NONE,
                })
                .collect(),
            ..Default::default()
        };
        let rig = RigAsset::parse("type rig\nversion 1\n---\nscheme mixamo\nspine \"Spine02\"\nchest \"Spine01\"\nupperChest \"Spine\"\nneck \"neck\"\n").unwrap();
        let map = rig.map(&asset).unwrap();
        let mut s = shown();
        s.target = Some((Arc::new(asset), map));
        s.lean = Some(15.0_f32.to_radians());
        s.drawn_lean = Some(7.0_f32.to_radians());
        eng.render.borrow_mut().motion.agents.insert(1, s);
        let result = dump(&eng);
        let a = &result["agents"][0];
        assert_eq!(a["rig"], "mixamo");
        assert_eq!(a["rigMap"], 42.0);
        let bones = a["bones"].as_array().unwrap();
        for pair in [
            ["spine", "Spine02"],
            ["chest", "Spine01"],
            ["upperChest", "Spine"],
            ["neck", "neck"],
        ] {
            assert!(bones.contains(&json!(pair)), "{a}");
        }
        assert!((a["leanDeg"].as_f64().unwrap() - 7.0).abs() < 1e-4);
        assert!((a["sourceLeanDeg"].as_f64().unwrap() - 15.0).abs() < 1e-4);
    }
}
