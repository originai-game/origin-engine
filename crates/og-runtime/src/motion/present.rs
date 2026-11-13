//! og.motion's presentation: the motion model, on the renderer's GPU by
//! default or on the CPU across the lanes as an opt-in
//! (`og.motion.setBackend`), and each agent's generated pose drawn in place
//! of og.anim's joint palette.
//!
//! Once a tick ([`step`], from the probe's tick gate before the next tick,
//! before a frame is drawn and before a capture or snapshot; at most once
//! per tick), with a renderer attached:
//!
//! 1. **The weights.** As soon as the open bundle declares the model (its
//!    manifest's `motion` entry, a game that uses og.motion) and a
//!    renderer is attached, `cooked/motion/model.ogb` is read and
//!    unpacked, and on the GPU its two networks put on the renderer's
//!    device ([`RenderBackend::compute`]): natively on a thread of its
//!    own, on the web from a fetch that lands at a later frame. Once an
//!    agent needs it the Presenter waits for the load at that tick (behind
//!    the world, as for a batch, step 3), so the tick the model starts at
//!    is the agent's. Until then, or when that fails, og.anim's pose is
//!    drawn.
//! 2. **Goals for the model.** An agent whose generated frames run short
//!    of its `lookahead` (or of two batches' [`BATCH_SECONDS`]) gets the
//!    next model goal: where the simulation's goals will have it at the
//!    first change of its pose, at most [`HORIZON`] seconds on (the mover
//!    run ahead with [`super::advance`]; without drive, its Transform's
//!    velocity), in the pose it will be in there.
//! 3. **Batches.** On the GPU, the default, one batch for every agent
//!    asking runs at a time: it is submitted in slices of about
//!    [`FRAME_BUDGET_MS`] of GPU time, one a tick, and its frames are taken
//!    [`BATCH_SECONDS`] after it started, at a fixed tick (waited for
//!    then), all its characters decoded at once; an agent with nothing
//!    left to show waits for it too. No tick waits for it on any host:
//!    the step stops there ([`Presenter`]'s `held`), the Presenter falls
//!    behind the world, keeping each later tick's [`Input`], and when the
//!    readback lands it steps those ticks with it, as if it had been on
//!    time; a frame meanwhile draws the last tick stepped. A capture, a
//!    presentation hash and a fork check [`settle`] first (natively they
//!    wait for the readback; the web asks again), so their frames
//!    reproduce run to run on one host and GPU.
//!    On the CPU, the opt-in, each agent asking gets a
//!    batch of its own, due [`BATCH_SECONDS`] after the tick it starts:
//!    its units (half encoder layers, decoder stages) are spread evenly
//!    over the ticks to its due tick and run across the lanes
//!    (`og_lanes::Site::Motion`), the rest at that tick, so no tick waits
//!    for it. What a tick steps never depends on the wall clock or the
//!    machine's speed (only what a frame shows while the Presenter is
//!    behind does): captures and hashes, which settle, are the same run to
//!    run, and a presentation checkpoint ([`Presenter::checkpoint`]) keeps
//!    the agents, the queued frames, the ticks behind and the outstanding
//!    batches, run again on a restore.
//! 4. **The pose.** Each agent plays its frames on the tick's clock,
//!    blending the two around it, a walk or run at the rate that makes its
//!    stride cover the ground the Transform does (30 Hz otherwise):
//!    re-rooted onto og.Transform (the hips over the entity, facing its
//!    heading, and following the generated hips' own travel within
//!    [`MAX_OFFSET`]), retargeted to its rig (`og_anim::retarget`), its
//!    feet planted in the world where the pose shows them on the ground
//!    (never the model's contact channel) and stepped when left behind
//!    (`og_anim::motion::soma::Planter`), as the joint palette of its
//!    og.Skeleton's skin.
//!
//! Presentation only: nothing here writes the world, and nothing the tick
//! reads depends on it.

use super::{
    AGENT, Goals, JumpStage, Kind, RUN_SPEED, Settings, advance, heading_of, lock, rotation_of,
    shared, transform_of,
};
use crate::RenderBackend;
use crate::engine::{Engine, hooks};
use crate::extract::rows;
use crate::wire::{FromJson, record};
use glam::{Mat4, Vec3};
use og_anim::motion::contract::{BOUNDARY_FRAMES, MIN_TOKENS, POSES_PER_TOKEN};
use og_anim::motion::contract::{LEFT_LEG, Pose, RIGHT_LEG, heading};
use og_anim::motion::flight;
use og_anim::motion::footing::*;
use og_anim::motion::intent::{Batch, Frame, Goal, Motion};
use og_anim::motion::library::POSES;
use og_anim::motion::runner::*;
use og_anim::motion::soma::{Leg, Planter, Skeleton};
use og_anim::{HumanoidMap, NONE, Pose as AnimPose, RetargetOptions, SkeletonAsset, Trs};
use og_core::IdMap;
use og_render::wgpu;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

/// The model's weights in the cooked bundle.
pub(crate) const MODEL_FILE: &str = "motion/model.ogb";
/// The weights' file inside that pack.
pub(crate) const MODEL_ENTRY: &str = "motion/model.safetensors";
/// How far ahead of the simulation the model's next goal is placed.
const HORIZON: f32 = 1.2;
/// The shortest model goal, s: [`MIN_TOKENS`] tokens less the frames at
/// their boundaries ([`token_counts`]).
const MIN_SEGMENT: f32 = (MIN_TOKENS * POSES_PER_TOKEN - 2 * BOUNDARY_FRAMES - 1) as f32 / RATE;
/// The library's jump pose.
const JUMP: usize = 3;
/// A jump clip's drawn frame catches up with its retimed one (an early
/// landing, frames that arrived late) at most this many times the clip's rate.
const CATCH_UP: f32 = 3.0;
/// SOMA's head joint.
const HEAD: usize = 6;
/// The bounds of a walk's or run's playback rate (stride warping by
/// time): the generated gait is played so its hips cover the ground the
/// simulation's Transform does, between these multiples of its rate.
const MIN_WARP: f32 = 0.5;
const MAX_WARP: f32 = 2.5;
/// The half-life over which the playback rate follows the speed ratio, s.
const WARP_HALF_LIFE: f32 = 0.08;
/// The drawn hips follow the generated hips' own travel and are drawn
/// back to the Transform with this half-life, s, never farther from it
/// than [`MAX_OFFSET`] m (times the character's scale): within a stride
/// the hips speed up and slow down, as a walker's do, over feet that stay
/// put, while the Transform moves at the simulation's even speed.
const OFFSET_HALF_LIFE: f32 = 0.35;
const MAX_OFFSET: f32 = 0.15;
/// How fast the drawn hips may move to keep the planted feet still, m/s.
const FOLLOW_SPEED: f32 = 1.5;

/// What an agent is doing, as [`Footing`] counts it: a walk or a run (by
/// the speed it asks for), a turn, a jump's takeoff, flight or landing,
/// or holding a pose.
fn segment_of(agent: Option<&super::Agent>, drive: bool) -> &'static str {
    let Some(c) = agent.and_then(|g| g.current) else {
        return "hold";
    };
    match c.goal.kind {
        Kind::WalkTo if c.goal.speed >= RUN_SPEED => "run",
        Kind::WalkTo => "walk",
        Kind::Face | Kind::Place if drive => "turn",
        Kind::Jump => match c.jump.map(|j| j.stage) {
            Some(JumpStage::Air) => "air",
            Some(JumpStage::Landed) => "landing",
            _ => "takeoff",
        },
        _ => "hold",
    }
}

#[derive(Default)]
enum Model {
    #[default]
    Unloaded,
    /// Waiting: for the bundle, or for a renderer to run on.
    Waiting(String),
    /// Being read and uploaded (natively on its thread, on the web from the
    /// host's fetch): what a tick with an agent waits for.
    Loading(String),
    Failed(String),
    Ready(Box<Ready>),
}

/// An agent as drawn.
#[derive(Clone)]
struct Shown {
    character: usize,
    prev: Option<Frame>,
    next: Option<Frame>,
    /// Where between `prev` and `next` the frame is, 0 to 1.
    phase: f32,
    /// The last simulation intent examined by presentation.
    serial: u32,
    requested: Option<Goal>,
    planter: Planter,
    skeleton: u64,
    rig: u64,
    target: Option<(Arc<SkeletonAsset>, HumanoidMap)>,
    failure: Option<String>,
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
    /// The rate the generated frames play at (1 is the model's own): a
    /// walk or run is played at the simulation's speed over the ground.
    warp: f32,
    /// The feet's contacts as the drawn pose shows them.
    feet: [Foot; 2],
    /// The drawn root's offset from the Transform in the world, and the
    /// generated hips (model world, over the ground) and heading it last
    /// followed: presentation only, never read by the simulation.
    offset: Vec3,
    followed: Option<(Vec3, f32)>,
    /// The drawn feet measured without any contact signal (`dump_motion`'s
    /// `footing`).
    footing: Footing,
    /// How far the drawn ankles are ahead of the hips along the facing, m
    /// (their mean): behind in a stride's push-off, under or ahead of the
    /// hips coming down to a landing.
    feet_ahead: Option<f32>,
    /// The model goals asked for, oldest first, until their frames are
    /// shown: what each frame's tag names.
    issued: VecDeque<Issued>,
    next_id: u32,
    /// While the simulation's current goal is a jump: the generated frames
    /// from the jump's start on, drawn retimed onto the simulation's flight.
    clip: Option<Clip>,
    /// Frames a finished jump clip had generated past the one drawn,
    /// played before the model's next.
    pending: VecDeque<Frame>,
}

/// A jump's generated frames and the one drawn (fractional).
#[derive(Clone)]
struct Clip {
    frames: Vec<Frame>,
    drawn: f32,
    /// The clip's own flight, once it has generated one.
    flight: Option<flight::ClipFlight>,
}

/// A model goal asked for: its id (the frames' tag), its library pose and
/// the frame clock its first frame is for (now plus the frames queued
/// before it).
#[derive(Clone)]
struct Issued {
    id: u32,
    pose: usize,
    start: f64,
}

impl Shown {
    fn reconcile_request(&self, motion: &mut Motion, goal: &Goal) {
        // The context remains owned by Motion until this generation lands.
        // A simulation transition alone does not change its conditioning.
        // Heading passes through Transform's quaternion between predictions;
        // allow its f32 roundoff (10 micrometres for target positions).
        let same = self.requested.as_ref().is_some_and(|old| {
            old.token_counts == goal.token_counts
                && old.targets.len() == goal.targets.len()
                && old.targets.iter().zip(&goal.targets).all(|(a, b)| {
                    a.positions
                        .iter()
                        .flatten()
                        .zip(b.positions.iter().flatten())
                        .chain(
                            a.rotations
                                .iter()
                                .flatten()
                                .flatten()
                                .zip(b.rotations.iter().flatten().flatten()),
                        )
                        .all(|(a, b)| (a - b).abs() <= 1e-5)
                })
        });
        if !same {
            motion.clear_goals(self.character);
        }
    }

    fn issued(&self, f: &Frame) -> Option<&Issued> {
        self.issued.iter().find(|g| g.id == f.tag)
    }

    /// Frames generated past the drawn one: the model's queue, a jump clip's
    /// frames after the drawn one and a finished clip's.
    fn ahead(&self, motion: &Motion) -> usize {
        let clip = self.clip.as_ref().map_or(0, |c| {
            c.frames.len().saturating_sub(c.drawn.floor() as usize + 1)
        });
        motion.queued(self.character) + clip + self.pending.len()
    }

    /// The playback rate after `elapsed` s: in a walk or run, the
    /// simulation's speed over the ground over the generated hips' (so a
    /// planted foot stays where it is in the world while the Transform
    /// carries the hips), and fast while the simulation has stopped and
    /// the model is still walking there; 1 in anything else.
    fn warp_for(&mut self, elapsed: f32) -> f32 {
        let gait = self.clip.is_none()
            && self
                .prev
                .and_then(|f| self.issued(&f))
                .is_some_and(|g| is_gait(g.pose));
        let to = match (self.prev, self.next) {
            (Some(a), Some(b)) if gait => {
                // The generated stride's speed: how fast the hips pass over
                // the feet on the ground (what a planted foot sees, whether
                // or not the model holds it still), else the hips' own.
                let flat = |f: &Frame, j: usize| {
                    let [x, _, z] = f.pose.positions[j];
                    Vec3::new(x, 0.0, z)
                };
                let hips = flat(&b, 0) - flat(&a, 0);
                let down: Vec<Vec3> = [LEFT_ANKLE, RIGHT_ANKLE]
                    .into_iter()
                    .zip(self.feet)
                    .filter(|(_, f)| f.touched.is_some())
                    .map(|(j, _)| hips - (flat(&b, j) - flat(&a, j)))
                    .collect();
                let stride = if down.is_empty() {
                    hips.length()
                } else {
                    down.iter().map(|d| d.length()).sum::<f32>() / down.len() as f32
                } * RATE;
                let sim = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
                if sim < 0.1 || stride < 0.05 {
                    MAX_WARP
                } else {
                    (sim / stride).clamp(MIN_WARP, MAX_WARP)
                }
            }
            _ => 1.0,
        };
        let k = 1.0 - og_core::math::exp(-elapsed * std::f32::consts::LN_2 / WARP_HALF_LIFE);
        self.warp += (to - self.warp) * k;
        self.warp
    }

    fn count_wait(&mut self) {
        if self.prev.is_none() {
            self.warmup += 1;
        } else if self.next.is_none() || self.phase >= 1.0 {
            self.starved += 1;
        }
    }
}

/// A tick's step stopped for a batch that has not landed: at agent `agent`
/// of the tick's agents, its batch waited for `tries` times before, with
/// the tick's `elapsed` seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Held {
    agent: usize,
    tries: usize,
    elapsed: f32,
}

/// What a tick's step reads of the world, taken as the tick ends: the
/// Presenter steps through these in order, behind the world while a batch
/// it needs has not landed, so no tick ever waits for it.
#[derive(Clone)]
pub(crate) struct Input {
    tick: u64,
    dt: f32,
    agents: Vec<Seen>,
    goals: Goals,
    /// og.anim's pose of each agent at the tick (what is drawn while the
    /// model has nothing for it); og.anim's live state is a later tick's
    /// by the time a step behind reads it.
    poses: BTreeMap<u64, Vec<Trs>>,
}

/// og.motion's presentation state (in `RenderState`, never the world).
#[derive(Default)]
pub(crate) struct Presenter {
    model: Model,
    agents: BTreeMap<u64, Shown>,
    /// The palette each agent is drawn with this frame, by entity bits
    /// (`fx::posed_model` takes it in place of og-anim's).
    pub palettes: IdMap<u64, Vec<Mat4>>,
    /// The palettes of the tick stepped before [`Presenter::stepped`], with
    /// that tick: what a display frame between the two blends from
    /// ([`Presenter::drawn`]). In the checkpoint (a display frame after a
    /// restore blends as the run's did), outside the hash (display only).
    previous: Option<(u64, IdMap<u64, Vec<Mat4>>)>,
    /// Only palettes made from generated frames this render, never fallback.
    generated: BTreeSet<u64>,
    /// og.motion.capabilities asked: load even without an agent.
    wanted: bool,
    /// The CPU opt-in: the model runs on the CPU across the lanes, the same
    /// bits on every host (the default is the renderer's GPU, the same bits
    /// on one host and GPU).
    cpu: bool,
    #[cfg(not(target_arch = "wasm32"))]
    loading: Option<Loading>,
    /// The clock (tick × dt) of the last tick stepped.
    clock: Option<f64>,
    /// The last tick stepped ([`step`] steps each tick once, in order).
    stepped: Option<u64>,
    /// The ticks the world has run that are not stepped yet, oldest first:
    /// the front one stopped for a batch that has not landed.
    behind: VecDeque<Input>,
    /// The last tick whose [`Input`] was taken.
    taken_tick: Option<u64>,
    /// Where the front of [`Presenter::behind`]'s step stopped for a batch
    /// that has not landed; it goes on from here once it has.
    held: Option<Held>,
    /// Ticks the world ran while the Presenter was behind it, and the most
    /// it was behind (`dump_motion`'s `late`): how often a result came
    /// after its tick. A tick never waits for one; only a capture does.
    late: (u64, usize),
    /// A system read the presentation on the GPU backend ([`warn_read`]).
    #[cfg(not(feature = "publish"))]
    warned_read: bool,
    model_bytes: u64,
    batches: u64,
    last_batch: (usize, f64),
    /// The last [`TAKEN`] batches decoded: the tick each was taken at, and
    /// the tick it was due at (`dump_motion`'s `taken`).
    taken: VecDeque<(u64, u64)>,
    errors: Vec<String>,
}

impl Presenter {
    /// The world was restored or reloaded: every agent starts again.
    pub(crate) fn forget(&mut self) {
        self.agents.clear();
        self.palettes.clear();
        self.previous = None;
        self.generated.clear();
        self.clock = None;
        self.stepped = None;
        self.held = None;
        self.behind.clear();
        self.taken_tick = None;
    }

    /// The palette `entity` is drawn with on a frame at `alpha` (the
    /// hierarchy's blend for it, `None` at the tick): this tick's, per joint
    /// blended from the tick before's (translation and scale lerped,
    /// rotation slerped, as og-anim's `crate::interpolate::pose`) when that
    /// tick drew the agent with as many joints. Captures, the probe and a
    /// frame on a tick boundary draw the tick; nothing here is stepped.
    pub(crate) fn drawn(&self, entity: u64, alpha: Option<f32>) -> Option<Vec<Mat4>> {
        let now = match self.palettes.get(&entity) {
            Some(now) => now,
            // A step stopped for a batch has not posed this agent yet.
            None if self.held.is_some() => return self.previous.as_ref()?.1.get(&entity).cloned(),
            None => return None,
        };
        let before = alpha
            .zip(self.previous.as_ref())
            .and_then(|(a, (tick, p))| {
                let m = p.get(&entity).filter(|m| m.len() == now.len())?;
                (Some(tick + 1) == self.stepped).then_some((a, m))
            });
        Some(match before {
            Some((a, m0)) => m0
                .iter()
                .zip(now)
                .map(|(&m0, &m1)| crate::extract::blend(m0, m1, a, f32::INFINITY))
                .collect(),
            None => now.clone(),
        })
    }

    /// The state a presentation checkpoint keeps (`crate::presentation`):
    /// every agent as drawn, the model's characters with their queued
    /// frames, and the batch outstanding (its inputs and tick, not its GPU
    /// work, which [`Presenter::restore`] starts again).
    pub(crate) fn checkpoint(&self) -> MotionCheckpoint {
        let ready = match &self.model {
            Model::Ready(r) => Some(r),
            _ => None,
        };
        MotionCheckpoint {
            agents: self.agents.clone(),
            palettes: self.palettes.clone(),
            generated: self.generated.clone(),
            clock: self.clock,
            stepped: self.stepped,
            held: self.held,
            behind: self.behind.clone(),
            taken_tick: self.taken_tick,
            previous: self.previous.clone(),
            motion: ready.map(|r| r.motion.clone()),
            outstanding: ready.map(|r| outstanding(r)).unwrap_or_default(),
        }
    }

    /// Brings back [`Presenter::checkpoint`]'s state: the frames after it
    /// are the ones the saved run drew. The batch on the GPU now is not the
    /// checkpoint's: natively it is waited for and dropped, on the web
    /// dropped when it lands; the checkpoint's batch runs again in its
    /// place, due at its own tick.
    pub(crate) fn restore(&mut self, c: &MotionCheckpoint) {
        let Model::Ready(r) = &mut self.model else {
            self.forget();
            return;
        };
        let Some(motion) = &c.motion else {
            self.forget();
            return;
        };
        // A CPU batch is dropped; a GPU one, natively, waited for and dropped.
        if matches!(r.compute, Compute::Cpu(_)) {
            r.flights.clear();
        }
        for f in &mut r.flights {
            f.stale = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        run_flights(r, 0, |_| true, true, &mut self.errors);
        r.landings.clear();
        r.motion = motion.clone();
        r.queued = c.outstanding.clone();
        self.agents = c.agents.clone();
        self.palettes = c.palettes.clone();
        self.previous = c.previous.clone();
        self.generated = c.generated.clone();
        self.clock = c.clock;
        self.stepped = c.stepped;
        self.held = c.held;
        self.behind = c.behind.clone();
        self.taken_tick = c.taken_tick;
    }

    /// FNV-1a over what [`Presenter::checkpoint`] keeps (part of the
    /// presentation hash): equal on one host and backend means the same
    /// poses drawn from the same world.
    pub(crate) fn hash(&self) -> u64 {
        use std::fmt::Write;
        let mut text = String::new();
        let _ = write!(text, "{:?}|{:?}|{:?}|", self.clock, self.stepped, self.held);
        for (e, s) in &self.agents {
            let _ = write!(
                text,
                "{e}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|",
                (s.character, s.prev, s.next, s.phase, s.serial, &s.requested),
                (
                    &s.planter,
                    s.last_position,
                    s.velocity,
                    s.shown,
                    s.starved,
                    s.warmup
                ),
                (s.facing, s.planted, s.slide, s.lean, s.drawn_lean, s.warp),
                (s.feet, s.offset, s.followed, s.feet_ahead),
                s.footing,
                s.issued
                    .iter()
                    .map(|g| (g.id, g.pose, g.start))
                    .collect::<Vec<_>>(),
                s.next_id,
                s.clip.as_ref().map(|c| (&c.frames, c.drawn, c.flight)),
                s.pending,
                self.palettes.get(e),
                self.generated.contains(e),
            );
        }
        if let Model::Ready(r) = &self.model {
            let _ = write!(
                text,
                "{:?}|{:?}",
                r.motion,
                outstanding(r)
                    .into_iter()
                    .map(|(b, due)| (b.characters, b.inputs, due))
                    .collect::<Vec<_>>()
            );
        }
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for b in text.bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }
}

/// og.motion's part of a presentation checkpoint ([`Presenter::checkpoint`]).
#[derive(Clone, Default)]
pub(crate) struct MotionCheckpoint {
    agents: BTreeMap<u64, Shown>,
    palettes: IdMap<u64, Vec<Mat4>>,
    generated: BTreeSet<u64>,
    clock: Option<f64>,
    stepped: Option<u64>,
    held: Option<Held>,
    behind: VecDeque<Input>,
    taken_tick: Option<u64>,
    previous: Option<(u64, IdMap<u64, Vec<Mat4>>)>,
    motion: Option<Motion>,
    outstanding: Vec<(Batch, u64)>,
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

/// The model from its pack's bytes: on the CPU, or with `gpu` on that device.
fn build(bytes: &[u8], gpu: Option<(wgpu::Device, wgpu::Queue)>) -> Result<Parts, String> {
    let files = og_bundle::runtime::unpack(bytes).map_err(|e| e.to_string())?;
    let (_, weights) = files
        .iter()
        .find(|(n, _)| n == MODEL_ENTRY)
        .ok_or_else(|| format!("it holds no {MODEL_ENTRY}"))?;
    parts(weights, gpu)
}

fn ready(parts: Parts) -> Model {
    Model::Ready(Box::new(resident(parts, now_ms)))
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
/// `wait`: natively, wait for the thread's answer (an agent needs the model
/// this tick); the web cannot wait.
fn load(p: &mut Presenter, eng: &Engine, backend: Option<&dyn RenderBackend>, wait: bool) -> Model {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rx) = &p.loading {
        let answer = if wait {
            rx.recv()
                .map_err(|_| std::sync::mpsc::TryRecvError::Disconnected)
        } else {
            rx.try_recv()
        };
        return match answer {
            Ok((bytes, r)) => {
                p.loading = None;
                p.model_bytes = p.model_bytes.max(bytes);
                r.map_or_else(Model::Failed, ready)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                Model::Loading("loading: the weights are being read and uploaded".into())
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                p.loading = None;
                Model::Failed("the model's loading thread ended without an answer".into())
            }
        };
    }
    let gpu = if !p.cpu {
        let Some(compute) = backend.and_then(|b| b.compute()) else {
            return Model::Waiting(
                "no renderer is attached: the model runs on the renderer's GPU (og.motion.setBackend cpu runs it without one)"
                    .into(),
            );
        };
        Some(compute)
    } else {
        None
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
                        build(&bytes, gpu).map_err(|e| format!("{path}: {e}")),
                    ),
                    Err(e) => (0, Err(format!("{path}: {e}"))),
                };
                let _ = tx.send(answer);
            });
        if let Err(e) = spawned {
            return Model::Failed(format!("the model's loading thread: {e}"));
        }
        p.loading = Some(rx);
        Model::Loading("loading: the weights are being read and uploaded".into())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = wait;
        let bytes = match hooks().map(|h| h.read_asset(&path)) {
            Some(crate::assets::Fetch::Ready(b)) => b,
            Some(crate::assets::Fetch::Pending) => {
                return Model::Loading(format!("loading {path}"));
            }
            Some(crate::assets::Fetch::Failed(e)) => return Model::Failed(format!("{path}: {e}")),
            None => return Model::Failed("no host is installed to read assets".into()),
        };
        p.model_bytes = p.model_bytes.max(bytes.len() as u64);
        build(&bytes, gpu).map_or_else(|e| Model::Failed(format!("{path}: {e}")), ready)
    }
}

/// One og.MotionAgent as this frame reads it.
#[derive(Clone)]
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
            let t: record::Transform = transform.read(&world.get_bytes(e, tid).ok()?).ok()?;
            let s: record::Skeleton = skeleton.read(&world.get_bytes(e, sid).ok()?).ok()?;
            let (position, rotation) = transform_of(&t);
            Some(Seen {
                entity: bits,
                settings: Settings::of(&record::MotionAgent::from_json(&a).ok()?),
                position,
                rotation,
                scale: t.scale.x,
                skeleton: s.asset,
                skin: s.skin,
            })
        })
        .collect()
}

/// Whether library pose `pose` is one [`gait`] gives.
fn is_gait(pose: usize) -> bool {
    pose == gait(0.0) || pose == gait(RUN_SPEED)
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
        // A jump asks for the library's jump through the preparation and
        // the rise, so the model reaches it at the apex, then for the
        // jump's pose from the fall on.
        Some(c) if c.goal.kind == Kind::Jump => match c.jump {
            Some(j)
                if j.stage == JumpStage::Landed
                    || (j.stage == JumpStage::Air && j.velocity <= 0.0) =>
            {
                c.goal.pose as usize
            }
            _ => JUMP,
        },
        Some(c) if matches!(c.goal.kind, Kind::Place | Kind::Pose) => c.goal.pose as usize,
        // A walk is walked or run at the speed it asks for, through its
        // start and its braking: a run that brakes into a jump is a run.
        Some(c) if c.goal.kind == Kind::WalkTo && speed > 0.3 => gait(c.goal.speed),
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
            // The rest of the pose the last goal asked for (the walk or run
            // the model is in, or a held pose) merges into the next when
            // less than the shortest generation of it is left: asked for on
            // its own, the model spends that generation on it and falls
            // behind the simulation, and the goal after asks it to lunge. A
            // new pose keeps its goal however short, a change of gait (a
            // run between two walks) included, and a jump's rise keeps its
            // own: the model reaches the jump's pose at the apex.
            let last_asked = shown.issued.back().map(|i| i.pose);
            let carries_on = |pose: usize| pose != JUMP && last_asked == Some(pose);
            let mut last = None;
            for k in 0..(HORIZON * RATE) as usize {
                let (p0, h0) = (p, h);
                advance(&mut agent, &a.settings, &mut p, &mut h, step);
                let pose = pose_of(&agent, (p - p0).length() / step);
                if let Some(prev) = last
                    && pose != prev
                    && (k as f32 * step >= MIN_SEGMENT || !carries_on(prev))
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

/// How far `v` leans from straight up, rad.
fn lean_of(v: Vec3) -> f32 {
    og_core::math::acos(v.normalize_or_zero().y.clamp(-1.0, 1.0))
}

/// A jump's shortest flight, as a share of the simulation's: a clip's
/// shorter runs with no foot down are strides, not the jump.
const MIN_FLIGHT: f32 = 0.4;

/// While the simulation's current goal is a jump (`docs/design.md` §2.7,
/// "Jumps"): the generated frames from the jump's start on are held in the
/// agent's clip, and the frame drawn is the one [`flight::retime`] puts at
/// the simulation's moment of the jump, reached at most [`CATCH_UP`] times
/// the clip's rate. Until the clip has jumped it plays at its own rate. When
/// the goal ends, the frames past the drawn one play on from `pending`.
fn jump_clip(motion: &mut Motion, s: &mut Shown, agent: Option<&super::Agent>, elapsed: f32) {
    let Some(c) = agent
        .and_then(|a| a.current)
        .filter(|c| c.goal.kind == Kind::Jump)
    else {
        if let Some(clip) = s.clip.take() {
            let k = clip.drawn.floor() as usize;
            s.prev = clip.frames.get(k).copied().or(s.prev);
            s.next = clip.frames.get(k + 1).copied();
            s.phase = clip.drawn - k as f32;
            let rest: Vec<Frame> = clip.frames.into_iter().skip(k + 2).collect();
            for f in rest.into_iter().rev() {
                s.pending.push_front(f);
            }
        }
        return;
    };
    let started = s.clip.is_none();
    if started {
        let Some(prev) = s.prev else { return };
        s.clip = Some(Clip {
            frames: std::iter::once(prev).chain(s.next).collect(),
            drawn: s.phase.min(if s.next.is_some() { 1.0 } else { 0.0 }),
            flight: None,
        });
    }
    let clip = s.clip.as_mut().expect("set above");
    clip.frames.extend(s.pending.drain(..));
    while let Some(f) = motion.pop(s.character) {
        clip.frames.push(f);
    }
    // Frames generated toward other goals past the jump's start (a walk
    // the model took longer over) never jump: unless a jump was asked for,
    // the clip keeps a frame past the drawn one and the model starts
    // again from them, so the next request is the jump.
    let asked = s
        .issued
        .iter()
        .any(|g| g.pose == JUMP && g.id >= clip.frames[0].tag);
    if started && !asked {
        let k = (clip.drawn.floor() as usize + 2).min(clip.frames.len() - 1);
        clip.frames.truncate(k + 1);
        let recent = std::array::from_fn(|i| clip.frames[(k + i).saturating_sub(3)].pose);
        motion.restart(s.character, recent);
        s.requested = None;
    }
    let natural = clip.drawn + elapsed * RATE;
    let flight_frames = c.jump.map(|j| 2.0 * j.v0 / j.gravity.max(1e-3) * RATE);
    let min = ((MIN_FLIGHT * flight_frames.unwrap_or(0.0)) as usize).max(2);
    // A clip that lands and takes off again (the model hopping on: a
    // second jump nobody asked for) is cut at that takeoff, and the model
    // starts again from the landed frames before it: whatever follows the
    // jump, in the clip or after it, stands on the ground the simulation
    // landed on instead of carrying the hop's elevation.
    let contacts: Vec<[f32; 4]> = clip.frames.iter().map(|f| f.contacts).collect();
    if c.jump.is_some()
        && let Some(flight::ClipFlight {
            touchdown: Some(touchdown),
            ..
        }) = flight::clip_flight(&contacts, 0, min)
        && let Some(again) = flight::clip_flight(&contacts, touchdown, min)
    {
        // The landing's absorption ends where the hips are lowest between
        // touchdown and the hop: the frames after it push off for the hop,
        // and the model restarts from those before.
        let lowest = (touchdown..again.takeoff)
            .min_by(|&i, &j| {
                let y = |k: usize| clip.frames[k].pose.positions[0][1];
                y(i).total_cmp(&y(j))
            })
            .unwrap_or(again.takeoff);
        let cut = (lowest + 1).max(clip.drawn.floor() as usize + 2);
        if cut < clip.frames.len() {
            clip.frames.truncate(cut);
            let recent = std::array::from_fn(|i| clip.frames[(cut + i).saturating_sub(4)].pose);
            motion.restart(s.character, recent);
            s.requested = None;
        }
    }
    let target = c.jump.and_then(|j| {
        let contacts: Vec<[f32; 4]> = clip.frames.iter().map(|f| f.contacts).collect();
        let fl = flight::clip_flight(&contacts, 0, min)?;
        clip.flight = Some(fl);
        let time = match j.stage {
            JumpStage::Prepare => flight::JumpTime::Prepare {
                elapsed: c.elapsed,
                prepare: c.goal.prepare,
            },
            // Through the flight the tick predicts: to its landing, on a
            // ledge or below a drop, not back at the takeoff height.
            JumpStage::Air => flight::JumpTime::Air {
                phase: match j.eta {
                    Some(eta) if j.air + eta > 0.0 => j.air / (j.air + eta),
                    _ => flight::air_phase(j.velocity, j.v0),
                },
            },
            JumpStage::Landed => flight::JumpTime::Landed { since: j.landed },
        };
        Some(flight::retime(fl, 0, clip.frames.len(), RATE, time))
    });
    let step = CATCH_UP * RATE * elapsed;
    let to = target.unwrap_or(natural);
    let last = clip.frames.len().saturating_sub(1) as f32;
    clip.drawn = (clip.drawn + (to - clip.drawn).clamp(-step, step)).clamp(0.0, last);
    let k = clip.drawn.floor() as usize;
    s.prev = clip.frames.get(k).copied();
    s.next = clip.frames.get(k + 1).copied();
    s.phase = clip.drawn - k as f32;
}

/// Moves the drawn root's offset on by `dt` s: the generated hips' travel
/// since the last frame (turned from the model's heading to the
/// Transform's) less the Transform's, eased back to none and bounded. A
/// jump clip (the Transform carries it), a cut in the generated motion or
/// the first frame draws the hips on the Transform.
fn follow_hips(
    (offset, followed): (&mut Vec3, &mut Option<(Vec3, f32)>),
    a: &Seen,
    (prev, next, t): (&Frame, &Frame, f32),
    velocity: Vec3,
    cut: bool,
    dt: f32,
) {
    let at = |j: usize| {
        let (u, v) = (prev.pose.positions[j], next.pose.positions[j]);
        Vec3::new(u[0] + (v[0] - u[0]) * t, 0.0, u[2] + (v[2] - u[2]) * t)
    };
    let hips = at(0);
    let h = heading(at(RIGHT_LEG).to_array(), at(LEFT_LEG).to_array());
    match *followed {
        Some((last, last_h)) if !cut && hips.distance(last) < 0.3 => {
            let moved = rotation_of(heading_of(a.rotation) - last_h) * (hips - last);
            let sim = Vec3::new(velocity.x, 0.0, velocity.z) * dt;
            *offset += moved * a.scale - sim;
            let k = og_core::math::exp(-dt * std::f32::consts::LN_2 / OFFSET_HALF_LIFE);
            *offset = (*offset * k).clamp_length_max(MAX_OFFSET * a.scale);
        }
        _ => *offset = Vec3::ZERO,
    }
    *followed = Some((hips, h));
}

/// `pose`'s local transforms re-rooted: the hips over the origin at their
/// height, facing +Z, and the lean (hips to head) they have.
fn rerooted(skeleton: &Skeleton, asset: &SkeletonAsset, pose: &Pose) -> (Vec<Trs>, f32) {
    let h = heading(pose.positions[RIGHT_LEG], pose.positions[LEFT_LEG]);
    let mut local = skeleton.local(pose);
    let undo = rotation_of(-h);
    local[0].translation = Vec3::new(0.0, local[0].translation.y, 0.0);
    local[0].rotation = (undo * local[0].rotation).normalize();
    // Measured on the transforms as drawn (forward kinematics), not on the
    // model's positions.
    let m = asset.model_space(&local);
    let lean = lean_of(m[HEAD].translation - m[0].translation);
    (local, lean)
}

fn palette(
    r: &Ready,
    a: &Seen,
    s: &mut Shown,
    goals: &Goals,
    sys: &og_anim::AnimationSystem,
    dt: f32,
) -> Option<Vec<Mat4>> {
    let (target, map) = s.target.as_ref()?;
    let prev = s.prev?;
    let next = s.next.unwrap_or(prev);
    let (a0, l0) = rerooted(&r.soma_skeleton, &r.soma, &prev.pose);
    let (a1, l1) = rerooted(&r.soma_skeleton, &r.soma, &next.pose);
    s.lean = Some(l0 + (l1 - l0) * s.phase.clamp(0.0, 1.0));
    let mut frames = [a0, a1];
    // In a jump the Transform carries the simulation's elevation: the hips
    // keep only their height over the lowest foot's clearance.
    if s.clip.is_some() {
        for (local, f) in frames.iter_mut().zip([&prev, &next]) {
            let foot = f.pose.positions[LEFT_ANKLE][1].min(f.pose.positions[RIGHT_ANKLE][1]);
            local[0].translation.y =
                flight::local_height(local[0].translation.y, foot, r.clearance);
        }
    }
    let clip = og_anim::motion::soma::clip("motion", &r.soma, &frames);
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
    // The feet's contacts from the generated pose itself: each ankle's
    // height over its rest height in the model's world and its speed
    // there (the model's own time), with hysteresis. Never in the air.
    let jump = goals
        .agents
        .get(&a.entity)
        .and_then(|g| g.current)
        .and_then(|c| c.jump)
        .map(|j| j.stage);
    let airborne = jump == Some(JumpStage::Air);
    // Holding a pose in place: no goal, or a pose goal, with the Transform
    // standing still.
    let holding = jump.is_none()
        && Vec3::new(s.velocity.x, 0.0, s.velocity.z).length() < 0.05
        && goals
            .agents
            .get(&a.entity)
            .is_none_or(|g| g.current.is_none_or(|c| c.goal.kind == Kind::Pose));
    let rest = r.soma.model_space(&r.soma.rest_pose())[LEFT_ANKLE]
        .translation
        .y;
    let t = s.phase.clamp(0.0, 1.0);
    let mut contacts = [0.0; 4];
    for (k, ankle) in [LEFT_ANKLE, RIGHT_ANKLE].into_iter().enumerate() {
        let (u, v) = (prev.pose.positions[ankle], next.pose.positions[ankle]);
        let height = u[1] + (v[1] - u[1]) * t - rest;
        let speed = Vec3::from_array(v).distance(Vec3::from_array(u)) * RATE;
        let down = s.feet[k].step(height, speed, holding) && !airborne;
        if airborne {
            s.feet[k] = Foot::default();
        }
        contacts[2 * k] = if down { 1.0 } else { 0.0 };
    }
    let root = Trs::new(a.position, a.rotation, Vec3::splat(a.scale));
    let cut = s.clip.is_some();
    let (t, velocity) = (s.phase.clamp(0.0, 1.0), s.velocity);
    follow_hips(
        (&mut s.offset, &mut s.followed),
        a,
        (&prev, &next, t),
        velocity,
        cut,
        dt,
    );
    // The body is carried over the feet it stands on: the hips move by
    // how far the generated ankles of the planted feet have drifted from
    // where those feet stand (at most FOLLOW_SPEED), so the drawn stride
    // keeps its feet still and the offset, bounded and eased back, takes
    // up the difference with the Transform.
    if !airborne {
        let model = target.model_space(&pose.nodes);
        let drift: Vec<Vec3> = s
            .planter
            .planted()
            .map(|(leg, at)| {
                let ankle = model[leg.chain[2] as usize].translation;
                let g = root.transform_point(ankle + a.rotation.inverse() * s.offset / a.scale);
                Vec3::new(at.x - g.x, 0.0, at.z - g.z)
            })
            .collect();
        if !drift.is_empty() {
            let mean = drift.iter().copied().sum::<Vec3>() / drift.len() as f32;
            s.offset += mean.clamp_length_max(FOLLOW_SPEED * a.scale * dt);
            s.offset = s.offset.clamp_length_max(MAX_OFFSET * a.scale);
        }
    }
    if let Some(hips) = map.node("hips")
        && s.offset != Vec3::ZERO
    {
        let local = a.rotation.inverse() * s.offset / a.scale;
        let parent = target.nodes[hips as usize].parent;
        let local = if parent == NONE {
            local
        } else {
            let p = target.model_space(&pose.nodes)[parent as usize];
            p.rotation.inverse() * local / p.scale
        };
        pose.nodes[hips as usize].translation += local;
    }
    perform_and_plant(
        &mut s.planter,
        goals.agents.get(&a.entity).map(|g| &g.performance),
        sys,
        (target, map),
        &root,
        &mut pose.nodes,
        (contacts, (!airborne).then_some(a.position.y), dt),
    );
    let model = target.model_space(&pose.nodes);
    s.drawn_lean = map.node("hips").zip(map.node("head")).map(|(hips, head)| {
        lean_of(model[head as usize].translation - model[hips as usize].translation)
    });
    for (k, (ankle, contact)) in [(map.nodes[16], 0), (map.nodes[20], 2)]
        .into_iter()
        .enumerate()
    {
        let down = contacts[contact] > og_anim::motion::soma::CONTACT;
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
    let rest = s.planter.rest();
    let segment = segment_of(goals.agents.get(&a.entity), a.settings.drive);
    for (k, ankle) in [map.nodes[16], map.nodes[20]].into_iter().enumerate() {
        if ankle == NONE {
            continue;
        }
        let at = root.transform_point(model[ankle as usize].translation);
        let clip = s.clip.is_some();
        s.footing.record(
            k,
            at,
            a.position.y + rest[k] * a.scale,
            (segment, airborne, clip),
            dt,
        );
    }
    let (l, r) = (map.nodes[14], map.nodes[18]);
    if l != NONE && r != NONE {
        let at = |n: u32| model[n as usize].translation.to_array();
        s.facing = heading(at(r), at(l));
        let forward = Vec3::new(
            og_core::math::sin(s.facing),
            0.0,
            og_core::math::cos(s.facing),
        );
        let (la, ra) = (map.nodes[16], map.nodes[20]);
        s.feet_ahead = map
            .node("hips")
            .filter(|_| la != NONE && ra != NONE)
            .map(|hips| {
                let mid = (model[la as usize].translation + model[ra as usize].translation) * 0.5;
                (mid - model[hips as usize].translation).dot(forward)
            });
    }
    let mut m = Vec::new();
    target.palette(a.skin, &pose.nodes, &mut m);
    (!m.is_empty()).then_some(m)
}

/// The generated pose's layers, in priority order (`docs/design.md` §2.7,
/// "Priorities"): the authored performance over the generated pose (the
/// reaction, the look, the hand contact on the head), then the feet planted
/// on the generated contacts, which outrank both. A foot the reaction's own
/// clip steps ([`Performance::stepping`]) is the clip's to move and is not
/// planted.
fn perform_and_plant(
    planter: &mut Planter,
    performance: Option<&super::performance::Performance>,
    sys: &og_anim::AnimationSystem,
    (target, map): (&SkeletonAsset, &HumanoidMap),
    root: &Trs,
    nodes: &mut [Trs],
    (mut contacts, ground, dt): ([f32; 4], Option<f32>, f32),
) {
    if let Some(performance) = performance {
        performance.apply(sys, target, map, root, nodes);
        let stepping = performance.stepping(sys, target, map, root, [map.nodes[16], map.nodes[20]]);
        for (k, step) in stepping.into_iter().enumerate() {
            if step {
                contacts[2 * k] = 0.0;
                contacts[2 * k + 1] = 0.0;
            }
        }
    }
    planter.step(target, nodes, root, &contacts, ground, dt);
}

/// Compose authored performance on the animation pose during model startup.
fn fallback(
    eng: &Engine,
    p: &mut Presenter,
    agents: &[Seen],
    goals: &Goals,
    sys: Option<&og_anim::AnimationSystem>,
    poses: &BTreeMap<u64, Vec<Trs>>,
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
        let Some(mut nodes) = poses.get(&a.entity).cloned() else {
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

/// og.motion at the world's tick: from the probe's tick gate (before the
/// next tick) and before a frame is drawn. Takes the tick's [`Input`] and
/// steps every tick behind it can (generated motion, then authored
/// performance), stopping where a batch it needs has not come back: the
/// world goes on and a later call goes on from there, the results applied
/// at the ticks they were due as soon as they arrive. True: no tick waits
/// for the model; a frame meanwhile draws the poses of the last tick
/// stepped.
pub(crate) fn step(eng: &Engine) -> bool {
    catch_up(eng, false);
    true
}

/// [`step`], then, natively, every batch the ticks behind need waited for,
/// so the Presenter is at the world's tick: before a capture, a
/// presentation hash or a fork check, whose frames then reproduce run to
/// run. On the web, which cannot wait for the GPU, false while it is still
/// behind: ask again.
pub(crate) fn settle(eng: &Engine) -> bool {
    catch_up(eng, true)
}

/// Takes the world's tick and steps the ticks behind in order, each until
/// it needs what has not come (with `wait`, natively, waited for).
fn catch_up(eng: &Engine, wait: bool) -> bool {
    let Ok((tick, dt)) = eng.world().map(|w| (w.time().tick, w.time().dt)) else {
        return true;
    };
    if eng.render.borrow().backend.is_none() {
        return true;
    }
    take(eng, tick, dt);
    loop {
        let Some(input) = eng.render.borrow_mut().motion.behind.pop_front() else {
            return true;
        };
        if !step_tick(eng, &input, wait) {
            eng.render.borrow_mut().motion.behind.push_front(input);
            return false;
        }
    }
}

/// The world's tick as [`Input`], once a tick.
fn take(eng: &Engine, tick: u64, dt: f32) {
    if eng.render.borrow().motion.taken_tick == Some(tick) {
        return;
    }
    let agents = seen(eng);
    let goals = lock(&shared(eng)).clone();
    let sys = eng.state.borrow().anim.clone();
    // og.anim's pose where `fallback` may draw it: an agent looking or reacting.
    let poses = sys.map_or_else(BTreeMap::new, |sys| {
        agents
            .iter()
            .filter(|a| {
                goals
                    .agents
                    .get(&a.entity)
                    .is_some_and(|g| g.performance.looking() || g.performance.reacting())
            })
            .filter_map(|a| Some((a.entity, sys.pose(og_ecs::Entity::from_bits(a.entity))?)))
            .collect()
    });
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    if !p.behind.is_empty() {
        p.late.0 += 1;
        p.late.1 = p.late.1.max(p.behind.len());
    }
    p.behind.push_back(Input {
        tick,
        dt,
        agents,
        goals,
        poses,
    });
    p.taken_tick = Some(tick);
}

/// One tick's step from its [`Input`]: false where it stops for a batch
/// that has not landed (or the weights), to go on from there.
fn step_tick(eng: &Engine, input: &Input, wait: bool) -> bool {
    let (tick, dt) = (input.tick, input.dt);
    let (agents, goals) = (&input.agents, &input.goals);
    let now = tick as f64 * f64::from(dt);
    let batch_ticks = if dt > 0.0 {
        (BATCH_SECONDS / dt).ceil() as u64
    } else {
        1
    };
    let due = tick + batch_ticks;
    let sys = eng.state.borrow().anim.clone();
    let mut rs = eng.render.borrow_mut();
    let rs = &mut *rs;
    let p = &mut rs.motion;
    // A held step goes on where it stopped; anything else is the tick's
    // first call.
    let resume = p.held.take().filter(|_| p.stepped == Some(tick));
    let elapsed = match resume {
        Some(h) => h.elapsed,
        None => {
            // What the tick waits for comes first, before anything of it
            // changes. An agent waits for the weights: the tick the model
            // starts at is then the agent's, never the load's wall time.
            let loadable =
                |m: &Model| matches!(m, Model::Unloaded | Model::Waiting(_) | Model::Loading(_));
            let loaded = !agents.is_empty() && loadable(&p.model);
            if loaded {
                p.model = load(p, eng, rs.backend.as_deref(), false);
                if matches!(p.model, Model::Loading(_)) {
                    if !wait || cfg!(target_arch = "wasm32") {
                        return false;
                    }
                    p.model = load(p, eng, rs.backend.as_deref(), true);
                }
            }
            // The batch runs a slice a tick; its frames are taken at its
            // tick, however soon it landed, and waited for then.
            if !agents.is_empty()
                && let Model::Ready(r) = &mut p.model
            {
                run_flights(r, tick, |_| false, false, &mut p.errors);
                if !settle_batches(r, tick, None, wait, &mut p.errors) {
                    return false;
                }
            }
            // The tick before's palettes, which display frames until the
            // next tick blend from.
            p.previous = p.stepped.map(|t| (t, std::mem::take(&mut p.palettes)));
            p.stepped = Some(tick);
            p.generated.clear();
            // The model loads as soon as the bundle declares it (a game
            // that uses og.motion), before any agent needs it.
            if agents.is_empty() && !p.wanted && !declares(eng) {
                p.agents.clear();
                return true;
            }
            if !loaded && loadable(&p.model) {
                p.model = load(p, eng, rs.backend.as_deref(), false);
            }
            if agents.is_empty() {
                p.agents.clear();
                return true;
            }
            let elapsed = p.clock.map_or(0.0, |c| (now - c).clamp(0.0, 0.25)) as f32;
            p.clock = Some(now);
            elapsed
        }
    };
    let Model::Ready(r) = &mut p.model else {
        fallback(eng, p, agents, goals, sys.as_ref(), &input.poses);
        return true;
    };
    let mut stats = (p.batches, p.last_batch, std::mem::take(&mut p.taken));
    if resume.is_none() {
        step_goals(
            eng,
            &mut p.agents,
            r,
            agents,
            goals,
            sys.as_ref(),
            (elapsed, now, tick, due),
            &mut stats,
            &mut p.errors,
        );
    }
    let first = resume.map_or(0, |h| h.agent);
    for (k, a) in agents.iter().enumerate().skip(first) {
        let s = p.agents.get_mut(&a.entity).expect("added above");
        // Resumed: this agent's clock has moved and its frames were taken;
        // the step stopped at its wait.
        let mut waiting = resume.filter(|h| h.agent == k).map(|h| h.tries);
        let mut tries = waiting.unwrap_or(0);
        if waiting.is_none() {
            // Frames played faster or slower than the model's rate are due
            // that much sooner or later: the goals' clocks move with them.
            let warp = s.warp_for(elapsed);
            for g in &mut s.issued {
                g.start += f64::from((1.0 - warp) * elapsed);
            }
            s.phase += elapsed * RATE * warp;
            if s.prev.is_none() {
                s.phase = 0.0;
            }
        }
        while tries < 3 {
            if waiting.take().is_none() {
                while s.clip.is_none() && (s.next.is_none() || s.phase >= 1.0) {
                    let Some(f) = s.pending.pop_front().or_else(|| r.motion.pop(s.character))
                    else {
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
                // The generated motion is drawn, never a gap: an agent that
                // ran out waits for the batch outstanding. Until its first
                // frames land it is drawn as og.anim poses it: the fallback
                // covers the model's start, as its load.
                if !(starved && r.awaits(s.character) && s.prev.is_some()) {
                    break;
                }
            }
            if !settle_batches(r, tick, Some(s.character), wait, &mut p.errors) {
                p.held = Some(Held {
                    agent: k,
                    tries,
                    elapsed,
                });
                (p.batches, p.last_batch, p.taken) = stats;
                return false;
            }
            decode(r, &mut stats, tick, Some(s.character));
            start_flights(r, due);
            tries += 1;
        }
        if let Some(prev) = s.prev {
            // Goals before the drawn frame's are done with (ids ascend).
            while s.issued.front().is_some_and(|g| g.id < prev.tag) {
                s.issued.pop_front();
            }
        }
        jump_clip(&mut r.motion, s, goals.agents.get(&a.entity), elapsed);
        s.count_wait();
        if s.next.is_none() || s.phase >= 1.0 {
            s.phase = s.phase.min(1.0);
        }
        if let Some(sys) = sys.as_ref()
            && let Some(m) = palette(r, a, s, goals, sys, elapsed)
        {
            s.shown += 1;
            p.palettes.insert(a.entity, m);
            p.generated.insert(a.entity);
            if r.startup.first_palette_delay_ms.is_none() {
                r.startup.first_palette_delay_ms =
                    r.startup.inference_finished_at.map(|at| now_ms() - at);
            }
        }
    }
    start_flights(r, due);
    (p.batches, p.last_batch, p.taken) = stats;
    fallback(eng, p, agents, goals, sys.as_ref(), &input.poses);
    true
}

/// A tick's first call ([`step`]): the batch due now decoded, every agent
/// added or kept, the model's goals asked for, the next batch started.
#[allow(clippy::too_many_arguments)]
fn step_goals(
    eng: &Engine,
    shown: &mut BTreeMap<u64, Shown>,
    r: &mut Ready,
    agents: &[Seen],
    goals: &Goals,
    sys: Option<&og_anim::AnimationSystem>,
    (elapsed, now, tick, due): (f32, f64, u64, u64),
    stats: &mut Stats,
    errors: &mut Vec<String>,
) {
    decode(r, stats, tick, None);
    shown.retain(|e, _| agents.iter().any(|a| a.entity == *e));
    for a in agents {
        let s = shown.entry(a.entity).or_insert_with(|| {
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
                requested: None,
                planter: Planter::new(Vec::new()),
                skeleton: 0,
                rig: u64::MAX,
                target: None,
                failure: None,
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
                warp: 1.0,
                feet: [Foot::default(); 2],
                offset: Vec3::ZERO,
                followed: None,
                footing: Footing::default(),
                feet_ahead: None,
                issued: VecDeque::new(),
                next_id: 0,
                clip: None,
                pending: VecDeque::new(),
            }
        });
        if s.skeleton != a.skeleton || s.rig != a.settings.rig {
            s.skeleton = a.skeleton;
            s.rig = a.settings.rig;
            s.failure = None;
            s.target = sys.and_then(|sys| {
                match crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent") {
                    Ok(target) => Some(target),
                    Err(error) => {
                        s.failure = Some(error.clone());
                        errors.push(format!("agent {}: {error}", a.entity));
                        None
                    }
                }
            });
            let legs = s.target.as_ref().map_or(Vec::new(), |(t, m)| {
                let rest = t.model_space(&t.rest_pose());
                [(14usize, 0usize), (18, 2)]
                    .iter()
                    .filter(|(b, _)| (0..3).all(|k| m.nodes[b + k] != NONE))
                    .map(|&(b, contact)| Leg {
                        chain: [m.nodes[b], m.nodes[b + 1], m.nodes[b + 2]],
                        contact,
                        rest: rest[m.nodes[b + 2] as usize].translation.y,
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
        // At least the agent's lookahead, and enough to cover two batches'
        // latency (sliced across frames, 0.5 to 0.6 s for 16 agents): one
        // batch runs at a time, so a goal asked for while another is on its
        // way waits for it, then for its own.
        let ahead = a.settings.lookahead.max(2.0 * BATCH_SECONDS);
        // On the CPU each agent's generation is its own batch: one playing
        // faster than the model's rate asks that much sooner.
        let lookahead = match r.compute {
            Compute::Cpu(_) => (ahead * RATE * s.warp.max(1.0)).ceil() as usize,
            Compute::Gpu { .. } => (ahead * RATE) as usize,
        };
        if serial != s.serial
            || (r.motion.goals(s.character) == 0 && s.ahead(&r.motion) < lookahead.max(1))
        {
            let queued = s.ahead(&r.motion) as f32 / (RATE * s.warp);
            let (at, h, pose, seconds) = predict(goals, a, s, queued);
            let mut goal = Goal::placed(r.library.frames(pose), [at.x, at.z], h);
            // A walk or run keeps time by its distance; forced short, the
            // model crawled into a run. A pose (jump, crouch, wave) has no
            // distance to keep time by: its length is the segment's.
            if !is_gait(pose) {
                goal.token_counts = token_counts(seconds);
            }
            if serial != s.serial {
                s.serial = serial;
                s.reconcile_request(&mut r.motion, &goal);
            }
            if r.motion.goals(s.character) != 0 || s.ahead(&r.motion) >= lookahead.max(1) {
                continue;
            }
            s.requested = Some(goal.clone());
            goal.tag = s.next_id;
            s.issued.push_back(Issued {
                id: s.next_id,
                pose,
                start: now + f64::from(queued),
            });
            s.next_id += 1;
            r.motion.push(s.character, goal);
        }
    }
    start_flights(r, due);
}

impl Presenter {
    fn presentation(&self, entity: u64) -> Value {
        let (state, reason) = match &self.model {
            Model::Unloaded => ("loading", "the model is read at the next frame"),
            Model::Waiting(why) | Model::Loading(why) => ("loading", why.as_str()),
            Model::Failed(why) => ("failed", why.as_str()),
            Model::Ready(r) => {
                let failure = self.agents.get(&entity).and_then(|s| s.failure.as_ref());
                if let Some(why) = failure.or(r.failure.as_ref()) {
                    ("failed", why.as_str())
                } else if self.generated.contains(&entity) {
                    ("ready", "")
                } else {
                    ("warming", "waiting for generated frames and their palette")
                }
            }
        };
        json!({ "state": state, "reason": reason })
    }
}

pub(crate) fn presentation(eng: &Engine, entity: u64) -> Value {
    eng.render.borrow().motion.presentation(entity)
}

/// og.motion.setBackend: `gpu` for the opt-in. A change drops the loaded
/// model, which loads again on the other backend, and every agent starts
/// again.
pub(crate) fn set_backend(eng: &Engine, gpu: bool) {
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    if p.cpu == !gpu {
        return;
    }
    p.cpu = !gpu;
    if matches!(p.model, Model::Ready(_) | Model::Failed(_)) {
        p.model = Model::Unloaded;
        p.agents.clear();
        p.palettes.clear();
        p.previous = None;
        p.generated.clear();
    }
}

/// The backend chosen: "gpu" or "cpu".
#[cfg(not(feature = "publish"))]
pub(crate) fn backend(eng: &Engine) -> &'static str {
    if eng.render.borrow().motion.cpu {
        "cpu"
    } else {
        "gpu"
    }
}

/// A system read og.motion's presentation on the GPU backend: one warning
/// in the engine's log, the first time.
#[cfg(not(feature = "publish"))]
pub(crate) fn warn_read(eng: &Engine) {
    let tick = eng.world().map_or(0, |w| w.time().tick);
    let first = {
        let mut rs = eng.render.borrow_mut();
        !std::mem::replace(&mut rs.motion.warned_read, true)
    };
    if first {
        eng.state.borrow_mut().log.push(json!({
            "tick": tick,
            "warning": true,
            "text": "og.motion.presentation was read inside a system on the gpu backend: game logic must not read og.motion's presentation or poses (they arrive when the GPU answers and differ run to run); a game that needs poses in its simulation sets og.motion.setBackend(world, \"cpu\")",
        }));
    }
}

/// og.motion.capabilities.
pub(crate) fn capabilities(eng: &Engine) -> Value {
    let mut rs = eng.render.borrow_mut();
    let p = &mut rs.motion;
    p.wanted = true;
    let (backend, reason) = match &p.model {
        Model::Ready(r) => (r.backend(), String::new()),
        Model::Unloaded => (
            "none",
            "loading: the model is read at the next frame".into(),
        ),
        Model::Waiting(why) | Model::Loading(why) => ("none", why.clone()),
        Model::Failed(why) => ("none", why.clone()),
    };
    json!({ "backend": backend, "reason": reason, "modelBytes": p.model_bytes as f64 })
}

/// The presenter's side of `dump_motion`.
pub(crate) fn dump(eng: &Engine) -> Value {
    let rs = eng.render.borrow();
    let p = &rs.motion;
    let (state, reason) = match &p.model {
        Model::Ready(r) => (r.backend(), String::new()),
        Model::Unloaded => ("unloaded", String::new()),
        Model::Waiting(w) | Model::Loading(w) => ("waiting", w.clone()),
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
                "presentation": p.presentation(*e),
                "ahead": ready.map_or(0, |r| s.ahead(&r.motion)),
                "modelGoals": ready.map_or(0, |r| r.motion.goals(s.character)),
                "framesShown": s.shown,
                "framesStarved": s.starved,
                "framesWarmup": s.warmup,
                "facing": s.facing,
                "footing": s.footing.json(),
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
                "feetAheadM": s.feet_ahead,
                // The drawn frame: the pose its model goal asked for, and
                // how far behind the frame clock it is (s).
                "drawnPose": s.next.and_then(|f| s.issued(&f)).map(|g| POSES[g.pose]),
                "lag": s.next.and_then(|f| s.issued(&f).map(|g| p.clock.unwrap_or(0.0) - (g.start + f64::from(f.index) / f64::from(RATE)))),
                "drawn": p.palettes.contains_key(e),
                // A jump's clip: its generated frames, the one drawn, its
                // own flight by its contacts.
                "jumpClip": s.clip.as_ref().map(|c| json!({
                    "frames": c.frames.len(),
                    "drawn": c.drawn,
                    "takeoff": c.flight.map(|f| f.takeoff),
                    "touchdown": c.flight.and_then(|f| f.touchdown),
                })),
                // The drawn frame's lower ankle over its standing height:
                // 0 on the ground, the generated flight's lift in the air.
                "footLiftM": ready.zip(s.prev).map(|(r, f)| {
                    f.pose.positions[LEFT_ANKLE][1].min(f.pose.positions[RIGHT_ANKLE][1]) - r.clearance
                }),
            })
        })
        .collect();
    json!({
        "state": state,
        "reason": reason,
        "modelBytes": p.model_bytes as f64,
        "poses": POSES,
        "inFlight": ready.map_or(0, |r| r.flights.len()),
        "batches": p.batches,
        // [taken, due] ticks of the last batches: their schedule, the
        // same however late their results came back.
        "taken": p.taken,
        // How far the Presenter is behind the world now, the ticks the
        // world ran while it was behind and the most it was: a result
        // that came back after its tick (no tick waits for one).
        "late": { "behind": p.behind.len(), "ticks": p.late.0, "most": p.late.1 },
        "lastBatch": { "characters": p.last_batch.0, "ms": p.last_batch.1 },
        "frameBudgetMs": FRAME_BUDGET_MS,
        "startup": ready.map(|r| &r.startup),
        "gpuGflops": ready.map_or(0.0, |r| r.rate),
        "cpuTicks": ready.map(|r| r.cpu_ticks.json()),
        "generations": ready.map_or(0, |r| r.generations),
        "errors": p.errors,
        "agents": agents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_anim::motion::contract::{FRAMES, MOTION_FEATURES};

    fn shown() -> Shown {
        Shown {
            character: 0,
            prev: None,
            next: None,
            phase: 0.0,
            serial: 0,
            requested: None,
            planter: Planter::new(Vec::new()),
            skeleton: 1,
            rig: 42,
            target: None,
            failure: None,
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
            warp: 1.0,
            feet: [Foot::default(); 2],
            offset: Vec3::ZERO,
            followed: None,
            footing: Footing::default(),
            feet_ahead: None,
            issued: VecDeque::new(),
            next_id: 0,
            clip: None,
            pending: VecDeque::new(),
        }
    }

    #[test]
    fn presentation_reports_load_failure_through_api_and_probe() {
        use crate::{
            og_engine_create, og_game_game, og_host_begin_bundle, og_motion_presentation,
            og_world_insert, og_world_spawn,
        };
        og_host_begin_bundle();
        let game = og_game_game(None);
        let created: Value = serde_json::from_str(&og_engine_create(game, "{}".into())).unwrap();
        let world = created["value"].as_u64().unwrap() as u32;
        let spawned: Value =
            serde_json::from_str(&crate::host_impl::answer(og_world_spawn(world, None))).unwrap();
        let entity = spawned["value"].as_f64().unwrap();
        let inserted: Value = serde_json::from_str(&crate::host_impl::answer(og_world_insert(
            world,
            entity,
            AGENT.into(),
            None,
        )))
        .unwrap();
        assert_eq!(inserted["ok"], true);
        let eng = crate::engine::engine(world).unwrap();
        eng.render.borrow_mut().motion.model = Model::Failed("weights could not be read".into());
        let expected = json!({"state":"failed", "reason":"weights could not be read"});
        let answer: Value = serde_json::from_str(&crate::host_impl::answer(
            og_motion_presentation(world, entity),
        ))
        .unwrap();
        assert_eq!(answer["ok"], true);
        assert_eq!(answer["value"], expected);
        assert_eq!(
            super::super::dump(world)["agents"][0]["presentation"],
            expected
        );
    }

    #[test]
    fn jump_clip_lands_on_the_simulations_landing() {
        // Five frames standing, ten in the air, five landed.
        let pose = Pose {
            positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
            rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
                og_anim::motion::contract::JOINTS],
        };
        let frame = |index: u32| Frame {
            pose,
            contacts: if (5..15).contains(&index) {
                [0.0; 4]
            } else {
                [1.0; 4]
            },
            tag: 0,
            index,
        };
        let mut motion = Motion::default();
        let mut s = shown();
        s.character = motion.add([pose; 4]);
        s.prev = Some(frame(0));
        s.next = Some(frame(1));
        s.pending.extend((2..20).map(frame));
        s.issued.push_back(Issued {
            id: 0,
            pose: JUMP,
            start: 0.0,
        });
        let (v0, gravity) = (3.13, 9.81);
        let goal = super::super::Goal {
            kind: Kind::Jump,
            pose: 0,
            duration: 2.0,
            target: Vec3::ZERO,
            heading: 0.0,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.5,
            prepare: 0.25,
        };
        let jump = |stage, velocity, landed| super::super::Jump {
            stage,
            v0,
            gravity,
            base: 0.0,
            air: 0.0,
            landed,
            velocity,
            left: true,
            rest: None,
            eta: None,
        };
        let mut agent = super::super::Agent::default();
        let at = |agent: &mut super::super::Agent, j: Option<super::super::Jump>| {
            agent.current = j.map(|j| super::super::Current {
                goal,
                elapsed: 0.5,
                start: Vec3::ZERO,
                start_heading: 0.0,
                jump: Some(j),
            });
        };
        let drawn = |s: &Shown| s.prev.unwrap().index as f32 + s.phase;
        // At the apex: halfway through the clip's flight.
        at(&mut agent, Some(jump(JumpStage::Air, 0.0, 0.0)));
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
        assert_eq!(drawn(&s), 10.0);
        // Falling past the takeoff height: held before the clip's
        // touchdown until the simulation lands.
        at(&mut agent, Some(jump(JumpStage::Air, -2.0 * v0, 0.0)));
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
        assert_eq!(drawn(&s), 14.0);
        // Landed: the touchdown frame, reached at no more than CATCH_UP
        // times the clip's rate.
        at(&mut agent, Some(jump(JumpStage::Landed, 0.0, 0.0)));
        jump_clip(&mut motion, &mut s, Some(&agent), 0.25 / (CATCH_UP * RATE));
        assert_eq!(drawn(&s), 14.25);
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
        assert_eq!(drawn(&s), 15.0);
        // The goal over: the frames after the drawn one play on.
        at(&mut agent, None);
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
        assert!(s.clip.is_none());
        assert_eq!((s.prev.unwrap().index, s.next.unwrap().index), (15, 16));
        assert_eq!(
            s.pending.iter().map(|f| f.index).collect::<Vec<_>>(),
            [17, 18, 19]
        );

        // A jump starting before frames generated toward a walk end: the
        // walk's frames past the one after next are dropped and the model
        // starts again from the kept ones, to be asked for the jump.
        let mut s = shown();
        s.character = motion.add([pose; 4]);
        s.issued.push_back(Issued {
            id: 0,
            pose: 1,
            start: 0.0,
        });
        s.prev = Some(frame(0));
        s.next = Some(frame(1));
        s.pending.extend((2..20).map(frame));
        at(&mut agent, Some(jump(JumpStage::Prepare, 0.0, 0.0)));
        jump_clip(&mut motion, &mut s, Some(&agent), 0.0);
        let clip = s.clip.as_ref().unwrap();
        assert_eq!(
            clip.frames.iter().map(|f| f.index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!((s.ahead(&motion), motion.goals(s.character)), (2, 0));
    }

    #[test]
    fn initial_face_to_idle_keeps_the_first_generated_window() {
        let a = Seen {
            entity: 1,
            settings: Settings::of(&record::MotionAgent {
                drive: true,
                ..Default::default()
            }),
            position: Vec3::new(2.0, 0.0, -3.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            skeleton: 1,
            skin: 0,
        };
        let face = super::super::Goal {
            kind: Kind::Face,
            pose: 0,
            duration: 0.1,
            target: Vec3::ZERO,
            heading: std::f32::consts::PI + 0.25,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        };
        let mut goals = Goals::default();
        let mut agent = super::super::Agent::default();
        agent.queue.push_back(face);
        agent.queue.push_back(super::super::Goal {
            kind: Kind::Pose,
            duration: 600.0,
            ..face
        });
        goals.agents.insert(a.entity, agent);
        let mut s = shown();
        let mut pose = Pose {
            positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
            rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
                og_anim::motion::contract::JOINTS],
        };
        pose.positions[LEFT_LEG][0] = -0.1;
        pose.positions[RIGHT_LEG][0] = 0.1;
        let request = |goals: &Goals, a: &Seen, s: &Shown| {
            let (at, h, index, seconds) = predict(goals, a, s, 0.0);
            assert_eq!(index, 0);
            let mut goal = Goal::placed(&[pose], [at.x, at.z], h);
            goal.token_counts = token_counts(seconds);
            goal
        };
        let first = request(&goals, &a, &s);
        let mut motion = Motion::default();
        s.character = motion.add([pose; 4]);
        s.requested = Some(first.clone());
        motion.push(s.character, first.clone());
        let batch = motion.batch(1).unwrap();
        let mut a = a;
        let mut h = 0.0;
        let agent = goals.agents.get_mut(&a.entity).unwrap();
        for _ in 0..60 {
            advance(agent, &a.settings, &mut a.position, &mut h, 1.0 / 60.0);
        }
        assert_ne!(agent.serial, s.serial);
        a.rotation = rotation_of(h);
        let next = request(&goals, &a, &s);
        s.reconcile_request(&mut motion, &next);
        let output = vec![0.0; FRAMES * MOTION_FEATURES];
        motion.accept_character(&batch, 0, 12, &output);
        assert!(
            motion.pop(s.character).is_some(),
            "initial window was discarded"
        );

        let mut changed_pose = first.clone();
        changed_pose.targets[0].positions[5][1] += 0.01;
        for changed in [
            Goal {
                targets: Goal::placed(&[pose], [2.001, -3.0], h).targets,
                ..first.clone()
            },
            changed_pose,
            Goal {
                token_counts: vec![6],
                ..first.clone()
            },
        ] {
            let mut motion = Motion::default();
            s.character = motion.add([pose; 4]);
            motion.push(s.character, first.clone());
            let batch = motion.batch(1).unwrap();
            s.reconcile_request(&mut motion, &changed);
            motion.accept_character(&batch, 0, 12, &output);
            assert!(
                motion.pop(s.character).is_none(),
                "changed request survived"
            );
        }
    }

    #[test]
    fn prediction_keeps_a_short_new_pose_and_merges_the_rest_of_an_asked_one() {
        let goal = |pose, duration| super::super::Goal {
            kind: Kind::Pose,
            pose,
            duration,
            target: Vec3::ZERO,
            heading: 0.0,
            speed: 0.0,
            arrival_heading: None,
            deceleration: 0.0,
            acceleration: 0.0,
            height: 0.0,
            prepare: 0.0,
        };
        let a = Seen {
            entity: 1,
            settings: Settings::of(&record::MotionAgent {
                drive: true,
                ..Default::default()
            }),
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            skeleton: 1,
            skin: 0,
        };
        let crouch = og_anim::motion::library::index("crouch").unwrap();
        let wave = og_anim::motion::library::index("wave").unwrap();
        for (pose, asked, kept) in [
            (JUMP, None, true),
            (wave, None, true),
            (wave, Some(0), true),
            // The last goal asked for the wave: its last 0.2 s belong to
            // the crouch's goal, so the model does not stand behind the
            // simulation finishing it.
            (wave, Some(wave), false),
            // A jump's rise always keeps its own goal.
            (JUMP, Some(JUMP), true),
        ] {
            let mut goals = Goals::default();
            let mut agent = super::super::Agent::default();
            agent.queue.push_back(goal(pose as u32, 0.2));
            agent.queue.push_back(goal(crouch as u32, 1.2));
            goals.agents.insert(1, agent);
            let mut s = shown();
            s.issued.extend(asked.map(|pose| Issued {
                id: 0,
                pose,
                start: 0.0,
            }));
            let (_, _, predicted, seconds) = predict(&goals, &a, &s, 0.0);
            if kept {
                assert_eq!(predicted, pose, "the imminent pose was skipped ({asked:?})");
                assert!(seconds <= 0.3);
            } else {
                assert_eq!(
                    predicted, crouch,
                    "the rest of the asked pose kept its own goal"
                );
            }
        }
    }

    #[test]
    fn prediction_never_merges_a_change_of_gait() {
        // The model is walking (the last goal asked for a walk) and the
        // simulation runs 1 m at 3.6 m/s, then walks on: the run is shorter
        // than the shortest generation, but a change of gait keeps its own
        // goal instead of being folded into the walk after it.
        let walk = |x: f32, speed: f32| super::super::Goal {
            kind: Kind::WalkTo,
            pose: 0,
            target: Vec3::new(x, 0.0, 0.0),
            heading: 0.0,
            duration: 0.0,
            speed,
            arrival_heading: None,
            deceleration: 3.0,
            acceleration: 1000.0,
            height: 0.0,
            prepare: 0.0,
        };
        let a = Seen {
            entity: 1,
            settings: Settings::of(&record::MotionAgent {
                drive: true,
                ..Default::default()
            }),
            position: Vec3::ZERO,
            rotation: rotation_of(std::f32::consts::FRAC_PI_2),
            scale: 1.0,
            skeleton: 1,
            skin: 0,
        };
        let mut goals = Goals::default();
        let mut agent = super::super::Agent::default();
        agent.queue.push_back(walk(1.0, 3.6));
        agent.queue.push_back(walk(4.0, 1.3));
        goals.agents.insert(1, agent);
        let mut s = shown();
        s.issued.push_back(Issued {
            id: 0,
            pose: gait(1.3),
            start: 0.0,
        });
        let (at, _, pose, seconds) = predict(&goals, &a, &s, 0.0);
        assert_eq!(pose, gait(3.6), "the run was merged into the walk");
        assert!(seconds < MIN_SEGMENT, "{seconds} s");
        assert!(
            (at.x - 1.0).abs() < 0.05,
            "the run's goal ends at x {}",
            at.x
        );
    }

    /// A SOMA-named skeleton standing upright: hips 1 m up, legs of two
    /// 0.45 m bones, feet forward.
    fn standing() -> Skeleton {
        let parent = |j: usize| match j {
            0 => NONE,
            1..=9 => j as u32 - 1,
            10..=21 => 3,
            22 | 26 => 0,
            _ => j as u32 - 1,
        };
        let offset = |j: usize| match j {
            22 => Vec3::new(0.1, -0.05, 0.0),
            26 => Vec3::new(-0.1, -0.05, 0.0),
            23 | 24 | 27 | 28 => Vec3::new(0.0, -0.45, 0.0),
            25 | 29 => Vec3::new(0.0, -0.05, 0.12),
            _ => Vec3::new(0.0, 0.12, 0.0),
        };
        let mut rest: [Trs; og_anim::motion::contract::JOINTS] =
            std::array::from_fn(|j| Trs::new(offset(j), glam::Quat::IDENTITY, Vec3::ONE));
        rest[0].translation = Vec3::new(0.0, 1.0, 0.0);
        Skeleton {
            parents: std::array::from_fn(parent),
            rest,
        }
    }

    /// Planted feet outrank an authored reaction: a reaction that lowers and
    /// pitches the hips leaves both planted ankles on their anchors, one
    /// whose clip steps the left foot moves that foot only, and one turned
    /// 90° from the facing steps both (planted, the legs would cross).
    #[test]
    fn a_planter_on_the_ground_puts_feet_down_and_steps_them_one_at_a_time() {
        // Ways this could fail: a foot planted where the pose holds it up
        // (a floating foot), a foot left behind sliding to its pose along
        // the ground instead of stepping, both feet stepping at once.
        let asset = standing().asset();
        let map = HumanoidMap::detect(&asset);
        let at_rest = asset.model_space(&asset.rest_pose());
        let legs: Vec<Leg> = [(14usize, 0usize), (18, 2)]
            .iter()
            .map(|&(b, contact)| Leg {
                chain: [map.nodes[b], map.nodes[b + 1], map.nodes[b + 2]],
                contact,
                rest: at_rest[map.nodes[b + 2] as usize].translation.y,
            })
            .collect();
        let rest_y = legs[0].rest;
        // Knees bent: the generated ankles are held 0.12 m over the ground.
        let mut pose = asset.rest_pose();
        for (hip, knee) in [(22, 23), (26, 27)] {
            pose[hip].rotation = glam::Quat::from_rotation_x(-0.6);
            pose[knee].rotation = glam::Quat::from_rotation_x(1.2);
        }
        let ankles = legs.iter().map(|l| l.chain[2] as usize).collect::<Vec<_>>();
        let lifted = asset.model_space(&pose)[ankles[0]].translation.y - rest_y;
        assert!(lifted > 0.05, "the generated ankle is {lifted} m up");
        let mut planter = Planter::new(legs);
        let dt = 1.0 / 60.0;
        let mut root = Trs::IDENTITY;
        let mut nodes = pose.clone();
        planter.step(&asset, &mut nodes, &root, &[1.0; 4], Some(0.0), dt);
        let m = asset.model_space(&nodes);
        for &a in &ankles {
            let up = m[a].translation.y - rest_y;
            assert!(up.abs() < 0.01, "a planted foot {up} m off the ground");
        }
        // The body carried on at 0.9 m/s over its planted feet.
        let mut last: Vec<Vec3> = ankles.iter().map(|&a| m[a].translation).collect();
        let mut steps = [0u32; 2];
        let mut highest = [0.0f32; 2];
        for frame in 1..90 {
            root.translation.x = 0.015 * frame as f32;
            let mut nodes = pose.clone();
            planter.step(&asset, &mut nodes, &root, &[1.0; 4], Some(0.0), dt);
            let m = asset.model_space(&nodes);
            let mut moving = 0;
            for (k, &a) in ankles.iter().enumerate() {
                let w = root.transform_point(m[a].translation);
                let across = Vec3::new(w.x - last[k].x, 0.0, w.z - last[k].z).length();
                // A step moves a foot centimetres a frame; a planted foot
                // at the edge of its reach eases by a fraction of one.
                if across > 2e-3 {
                    moving += 1;
                    steps[k] += 1;
                    // Across the ground only ever lifted, past the first
                    // and last few millimetres of the step.
                    highest[k] = highest[k].max(w.y - rest_y);
                }
                last[k] = w;
            }
            assert!(moving < 2, "frame {frame}: both feet stepping at once");
        }
        for k in 0..2 {
            assert!(steps[k] > 0, "foot {k} never stepped after the body");
            assert!(
                highest[k] > 0.03,
                "foot {k} stepped {} m high: slid along the ground",
                highest[k]
            );
        }
        // Each foot kept up with the body's pose.
        let m = asset.model_space(&pose);
        for (k, &a) in ankles.iter().enumerate() {
            let generated = root.transform_point(m[a].translation);
            assert!(
                (last[k].x - generated.x).abs() < 0.25,
                "foot {k} at {} behind {}",
                last[k].x,
                generated.x
            );
        }
    }

    #[test]
    fn planted_feet_outrank_a_reaction_except_where_it_steps() {
        use super::super::performance::Performance;
        let asset = standing().asset();
        let map = HumanoidMap::detect(&asset);
        let legs: Vec<Leg> = [(14usize, 0usize), (18, 2)]
            .iter()
            .map(|&(b, contact)| Leg {
                chain: [map.nodes[b], map.nodes[b + 1], map.nodes[b + 2]],
                contact,
                rest: 0.0,
            })
            .collect();
        assert!(legs.iter().all(|l| l.chain.iter().all(|&n| n != NONE)));
        let mut world = og_ecs::World::new(&og_ecs::WorldDesc {
            tick_rate: 60,
            seed: 1,
        })
        .unwrap();
        let sys = og_anim::install(&mut world).unwrap();
        // Knees bent 0.5 rad, within reach (straight legs are soft IK's).
        let mut rest = asset.rest_pose();
        for (hip, knee) in [(22, 23), (26, 27)] {
            rest[hip].rotation = glam::Quat::from_rotation_x(-0.25);
            rest[knee].rotation = glam::Quat::from_rotation_x(0.5);
        }
        let frames = |f: &dyn Fn(usize, &mut Vec<Trs>)| -> Vec<Vec<Trs>> {
            (0..31)
                .map(|k| {
                    let mut p = rest.clone();
                    f(k, &mut p);
                    p
                })
                .collect()
        };
        // Held: the hips 0.15 m lower and pitched 20° forward.
        let crouch = frames(&|_, p| {
            p[0].translation.y -= 0.15;
            p[0].rotation = glam::Quat::from_rotation_x(20f32.to_radians());
        });
        // The left thigh swinging forward at 0.6 rad/s: a step.
        let step = frames(&|k, p| {
            p[22].rotation = glam::Quat::from_rotation_x(-0.25 - 0.02 * k as f32);
        });
        sys.add_clip(1, og_anim::motion::soma::clip("crouch", &asset, &crouch))
            .unwrap();
        sys.add_clip(2, og_anim::motion::soma::clip("step", &asset, &step))
            .unwrap();
        let root = Trs::IDENTITY;
        let ankles = [map.nodes[16], map.nodes[20]].map(|n| n as usize);
        let down = [1.0; 4];
        for (clip, turned, moved) in [
            (1, false, [false, false]),
            (2, false, [true, false]),
            (1, true, [true, true]),
        ] {
            let mut planter = Planter::new(legs.clone());
            let mut nodes = rest.clone();
            perform_and_plant(
                &mut planter,
                None,
                &sys,
                (&asset, &map),
                &root,
                &mut nodes,
                (down, None, 1.0 / 30.0),
            );
            let anchors = ankles.map(|a| asset.model_space(&nodes)[a].translation);
            let performance = Performance::holding(
                clip,
                1.0,
                0.5,
                if turned { [1.0, 0.0, 0.0] } else { [0.0; 3] },
            );
            let mut nodes = rest.clone();
            perform_and_plant(
                &mut planter,
                Some(&performance),
                &sys,
                (&asset, &map),
                &root,
                &mut nodes,
                (down, None, 1.0 / 30.0),
            );
            let m = asset.model_space(&nodes);
            for k in 0..2 {
                let off = (m[ankles[k]].translation - anchors[k]).length();
                assert_eq!(
                    off > 0.05,
                    moved[k],
                    "clip {clip} ankle {k}: {off} m off its anchor"
                );
                if !moved[k] {
                    assert!(off < 0.005, "clip {clip} ankle {k}: {off} m off its anchor");
                }
            }
            // The reaction itself was drawn: the hips lowered, or the thigh swung.
            if clip == 1 && !turned {
                assert!((m[0].translation.y - 0.85).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn a_display_frame_between_two_ticks_draws_a_pose_between_theirs() {
        use glam::Quat;
        let joint = |x: f32, yaw: f32| {
            Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), Vec3::new(x, 0.0, 0.0))
        };
        let before = vec![joint(0.0, 0.0), joint(1.0, 0.2)];
        let now = vec![joint(1.0, 0.6), joint(3.0, -0.4)];
        let mut p = Presenter {
            previous: Some((4, IdMap::from_iter([(7, before.clone())]))),
            stepped: Some(5),
            ..Presenter::default()
        };
        p.palettes.insert(7, now.clone());
        // The tick itself (captures, the probe, a tick boundary).
        assert_eq!(p.drawn(7, None), Some(now.clone()));
        for a in [0.25f32, 0.5, 0.75] {
            let m = p.drawn(7, Some(a)).unwrap();
            for (k, (want0, want1)) in [((0.0, 0.0), (1.0, 0.6)), ((1.0, 0.2), (3.0, -0.4))]
                .into_iter()
                .enumerate()
            {
                let (s, r, t) = m[k].to_scale_rotation_translation();
                let (yaw, _, _) = r.to_euler(glam::EulerRot::YXZ);
                let x = want0.0 + (want1.0 - want0.0) * a;
                let y = want0.1 + (want1.1 - want0.1) * a;
                assert!(
                    (t.x - x).abs() < 1e-5,
                    "joint {k} at {a}: x {} not {x}",
                    t.x
                );
                assert!(
                    (yaw - y).abs() < 1e-5,
                    "joint {k} at {a}: yaw {yaw} not {y}"
                );
                assert!(
                    (s - Vec3::ONE).abs().max_element() < 1e-5,
                    "joint {k}: scale {s}"
                );
                assert!(m[k] != before[k] && m[k] != now[k]);
            }
        }
        // A tick skipped since, or another rig: the tick's pose.
        p.previous.as_mut().unwrap().0 = 3;
        assert_eq!(p.drawn(7, Some(0.5)), Some(now.clone()));
        p.previous = Some((4, IdMap::from_iter([(7, before[..1].to_vec())])));
        assert_eq!(p.drawn(7, Some(0.5)), Some(now));
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
                positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
                rotations: [[[0.0; 3]; 3]; og_anim::motion::contract::JOINTS],
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
