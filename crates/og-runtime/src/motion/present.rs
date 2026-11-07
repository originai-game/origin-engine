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
//! 4. **The pose.** Each agent plays its frames on the frame's clock,
//!    blending the two around it, a walk or run at the rate that makes its
//!    stride cover the ground the Transform does (30 Hz otherwise):
//!    re-rooted onto og.Transform (the hips over the entity, facing its
//!    heading, and following the generated hips' own travel within
//!    [`MAX_OFFSET`]), retargeted to its rig (`og_anim::retarget`), its
//!    feet planted in the world where the pose shows them on the ground
//!    (never the model's contact channel) and stepped when left behind
//!    (`og_motion::soma::Planter`), as the joint palette of its
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
use glam::{Mat4, Vec3};
use og_anim::{HumanoidMap, NONE, Pose as AnimPose, RetargetOptions, SkeletonAsset, Trs};
use og_core::IdMap;
use og_motion::Bundle;
use og_motion::contract::{
    BOUNDARY_FRAMES, FRAMES, MAX_TOKENS, MIN_TOKENS, MOTION_FEATURES, POSES_PER_TOKEN,
};
use og_motion::contract::{LEFT_LEG, Pose, RIGHT_LEG, heading};
use og_motion::flight;
use og_motion::glue::{Statics, StudentOutput, decoder_inputs, finish_character, student_inputs};
use og_motion::gpu::{Gpu, GpuError};
use og_motion::intent::{Batch, Frame, Goal, Motion};
use og_motion::library::{Library, POSES};
use og_motion::soma::{Leg, Planter, Skeleton};
use og_render::wgpu;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
/// The shortest model goal, s: [`MIN_TOKENS`] tokens less the frames at
/// their boundaries ([`token_counts`]).
const MIN_SEGMENT: f32 = (MIN_TOKENS * POSES_PER_TOKEN - 2 * BOUNDARY_FRAMES - 1) as f32 / RATE;
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
/// Tokens either side of a model goal's own duration it may take
/// ([`token_counts`]).
const TOKEN_SLACK: usize = 2;
/// The library's jump pose.
const JUMP: usize = 3;
/// A jump clip's drawn frame catches up with its retimed one (an early
/// landing, frames that arrived late) at most this many times the clip's rate.
const CATCH_UP: f32 = 3.0;
/// SOMA's ankles, the joints a jump's local height measures to the ground.
const LEFT_ANKLE: usize = 24;
const RIGHT_ANKLE: usize = 28;
/// How far behind the frame clock a drawn frame may fall before frames are
/// skipped to catch up, s.
const MAX_LAG: f64 = 0.25;
/// SOMA's head joint.
const HEAD: usize = 6;
/// The bounds of a walk's or run's playback rate (stride warping by
/// time): the generated gait is played so its hips cover the ground the
/// simulation's Transform does, between these multiples of its rate.
const MIN_WARP: f32 = 0.5;
const MAX_WARP: f32 = 2.5;
/// The half-life over which the playback rate follows the speed ratio, s.
const WARP_HALF_LIFE: f32 = 0.08;
/// A foot is on the ground, as the pose shows it (in the model's own
/// world, never from its contact channel), when its ankle comes within
/// [`DOWN_HEIGHT`] m of its rest height over the ground moving slower than
/// [`DOWN_SPEED`] m/s, or rests within [`HELD_HEIGHT`] of it, slower than
/// [`HELD_SPEED`], for [`HELD_FRAMES`] drawn frames (a held pose's foot,
/// put down onto the ground); it lifts when it rises [`LIFT_HEIGHT`] over
/// where it touched down or moves faster than [`LIFT_SPEED`].
const DOWN_HEIGHT: f32 = 0.05;
const DOWN_SPEED: f32 = 0.6;
const HELD_HEIGHT: f32 = 0.2;
/// A character holding a pose in place keeps a foot down up to this high,
/// m: a held pose's shifting and fidgeting feet stand on the ground (the
/// planter steps them when they are left behind).
const HOLD_HEIGHT: f32 = 0.35;
const HELD_SPEED: f32 = 0.06;
const HELD_FRAMES: u32 = 6;
const LIFT_HEIGHT: f32 = 0.04;
const LIFT_SPEED: f32 = 1.0;
/// The drawn hips follow the generated hips' own travel and are drawn
/// back to the Transform with this half-life, s, never farther from it
/// than [`MAX_OFFSET`] m (times the character's scale): within a stride
/// the hips speed up and slow down, as a walker's do, over feet that stay
/// put, while the Transform moves at the simulation's even speed.
const OFFSET_HALF_LIFE: f32 = 0.35;
const MAX_OFFSET: f32 = 0.15;
/// How fast the drawn hips may move to keep the planted feet still, m/s.
const FOLLOW_SPEED: f32 = 1.5;

/// A drawn foot is on the ground, for [`Footing`], when its ankle is
/// within this of its rest height over the Transform, m (times the scale).
const ON_GROUND: f32 = 0.03;
/// The slide speeds [`Footing`] counts samples above, m/s.
const SLIDE: [f32; 2] = [0.1, 0.3];
/// How long a held foot stays up and still before [`Footing`] counts it
/// floating, s: longer than a step's rise.
const FLOATING: f32 = 0.1;

/// The drawn feet as they move in the world, measured on every shown frame
/// from where they are drawn and nothing else (no contact channel, the
/// model's or the pose's): a foot on the ground in two consecutive frames
/// is a sample of its speed over the ground, counted by what the character
/// is doing; while it holds a pose, how high its feet are over the ground.
#[derive(Clone, Debug, Default, PartialEq)]
struct Footing {
    last: [Option<(Vec3, bool)>; 2],
    segments: BTreeMap<&'static str, Slides>,
    /// Frames of a held pose, those with a foot held over [`ON_GROUND`]
    /// without moving across the ground (floating, not stepping), and the
    /// highest a foot was over its rest height, m.
    held: (u32, u32, f32),
    floating: bool,
    /// How long each foot has been up and still, s.
    up: [f32; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Slides {
    frames: u32,
    samples: u32,
    over: [u32; 2],
    max: f32,
}

impl Footing {
    /// Records foot `k` drawn `at` over `rest` (its rest ankle height in
    /// the world) in `segment`, `dt` s after the last frame.
    /// A jump's clip (`clip`) is no held pose, whatever the goal.
    fn record(
        &mut self,
        k: usize,
        at: Vec3,
        rest: f32,
        (segment, air, clip): (&'static str, bool, bool),
        dt: f32,
    ) {
        let height = at.y - rest;
        let down = !air && height < ON_GROUND;
        let across = match self.last[k] {
            Some((was, _)) if dt > 0.0 => Vec3::new(at.x - was.x, 0.0, at.z - was.z).length() / dt,
            _ => 0.0,
        };
        let seg = self.segments.entry(segment).or_default();
        if k == 0 {
            seg.frames += 1;
        }
        if let Some((_, true)) = self.last[k]
            && down
            && dt > 0.0
        {
            let speed = across;
            seg.samples += 1;
            for (n, limit) in seg.over.iter_mut().zip(SLIDE) {
                *n += (speed > limit) as u32;
            }
            seg.max = seg.max.max(speed);
        }
        self.last[k] = Some((at, down));
        if segment == "hold" && !air && !clip {
            // A frame counts once, however many of its feet float.
            if k == 0 {
                self.held.0 += 1;
                self.floating = false;
            }
            // Up and not moving across for [`FLOATING`] s: floating (a
            // step's foot rises before it moves across, and a foot moving
            // across is stepping).
            self.up[k] = if height > ON_GROUND && across < SLIDE[0] {
                self.up[k] + dt
            } else {
                0.0
            };
            if self.up[k] >= FLOATING && !self.floating {
                self.floating = true;
                self.held.1 += 1;
            }
            self.held.2 = self.held.2.max(height);
        }
    }

    fn json(&self) -> Value {
        let segments: serde_json::Map<String, Value> = self
            .segments
            .iter()
            .map(|(k, s)| {
                (
                    (*k).to_string(),
                    json!({
                        "frames": s.frames,
                        "samples": s.samples,
                        "over0_1": s.over[0],
                        "over0_3": s.over[1],
                        "maxMps": s.max,
                    }),
                )
            })
            .collect();
        json!({
            "onGroundM": ON_GROUND,
            "segments": segments,
            "held": { "frames": self.held.0, "floating": self.held.1, "maxM": self.held.2 },
        })
    }
}

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

/// One foot's contact as the pose shows it, with hysteresis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Foot {
    /// The ankle's height over its rest height where it touched down.
    touched: Option<f32>,
    /// Drawn frames it has been held still.
    still: u32,
}

impl Foot {
    /// Whether the foot is down at `height` m over its rest height, moving
    /// at `speed` m/s. A character `holding` a pose in place keeps every foot
    /// below [`HOLD_HEIGHT`] that is not swung fast on the ground.
    fn step(&mut self, height: f32, speed: f32, holding: bool) -> bool {
        self.still = if speed < HELD_SPEED {
            self.still + 1
        } else {
            0
        };
        let held = holding && height < HOLD_HEIGHT && speed < LIFT_SPEED;
        match self.touched {
            Some(_) if held => {}
            Some(t) if height > t.max(0.0) + LIFT_HEIGHT || speed > LIFT_SPEED => {
                self.touched = None;
            }
            Some(_) => {}
            None if held
                || (height < DOWN_HEIGHT && speed < DOWN_SPEED)
                || (height < HELD_HEIGHT && self.still >= HELD_FRAMES) =>
            {
                self.touched = Some(height);
            }
            None => {}
        }
        self.touched.is_some()
    }
}

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
    /// SOMA's ankle height standing on the ground (the library idle's
    /// lower ankle), the clearance of a jump's local height.
    clearance: f32,
    /// The device's rate as the last batch measured it, GFLOP/s.
    rate: f64,
    /// A sliced batch's latency, start to decoded, as the last one not
    /// waited for measured (ms; 500 before one has).
    latency: f64,
    failure: Option<String>,
    startup: Startup,
}

/// Wall-time diagnostics only; none of these clocks enter the world.
#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Startup {
    compile_ms: Option<f64>,
    upload_submit_ms: Option<f64>,
    first_dispatch_delay_ms: Option<f64>,
    first_inference_wall_ms: Option<f64>,
    first_inference_poll_ms: f64,
    first_palette_delay_ms: Option<f64>,
    #[serde(skip)]
    resident_at: f64,
    #[serde(skip)]
    inference_finished_at: Option<f64>,
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
struct Clip {
    frames: Vec<Frame>,
    drawn: f32,
    /// The clip's own flight, once it has generated one.
    flight: Option<flight::ClipFlight>,
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

/// og.motion's presentation state (in `RenderState`, never the world).
#[derive(Default)]
pub(crate) struct Presenter {
    model: Model,
    agents: BTreeMap<u64, Shown>,
    /// The palette each agent is drawn with this frame, by entity bits
    /// (`fx::posed_model` takes it in place of og-anim's).
    pub palettes: IdMap<u64, Vec<Mat4>>,
    /// Only palettes made from generated frames this render, never fallback.
    generated: BTreeSet<u64>,
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
        self.generated.clear();
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
    let startup = Startup {
        #[cfg(not(target_arch = "wasm32"))]
        compile_ms: Some(parts.gpu.startup_ms.0),
        #[cfg(not(target_arch = "wasm32"))]
        upload_submit_ms: Some(parts.gpu.startup_ms.1),
        resident_at: now_ms(),
        ..Default::default()
    };
    Model::Ready(Box::new(Ready {
        rate: parts.gpu.rate(),
        gpu: Some(parts.gpu),
        device: parts.device,
        flight: None,
        landing: None,
        latency: 500.0,
        failure: None,
        startup,
        statics: Arc::new(parts.statics),
        motion: Motion::default(),
        clearance: parts.library.frames(0).first().map_or(0.0, |p| {
            p.positions[LEFT_ANKLE][1].min(p.positions[RIGHT_ANKLE][1])
        }),
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
        let start = now_ms();
        let polled = flight.future.as_mut().poll(&mut cx);
        if r.startup.first_inference_wall_ms.is_none() {
            r.startup.first_inference_poll_ms += now_ms() - start;
        }
        match polled {
            Poll::Ready(done) => break done,
            Poll::Pending if block && cfg!(not(target_arch = "wasm32")) => {
                #[cfg(not(target_arch = "wasm32"))]
                let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
            }
            Poll::Pending => return false,
        }
    };
    let flight = r.flight.take().expect("polled above");
    if r.startup.first_inference_wall_ms.is_none() {
        let finished = now_ms();
        r.startup.first_inference_wall_ms = Some(finished - flight.started);
        r.startup.inference_finished_at = Some(finished);
    }
    r.rate = gpu.rate();
    r.gpu = Some(gpu);
    match result {
        Ok((student, local)) => {
            r.failure = None;
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
            r.failure = Some(e.to_string());
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
    let started = now_ms();
    r.startup
        .first_dispatch_delay_ms
        .get_or_insert(started - r.startup.resident_at);
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
        started,
    });
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
fn jump_clip(
    motion: &mut Motion,
    s: &mut Shown,
    agent: Option<&super::Agent>,
    elapsed: f32,
    keep: usize,
) {
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
    // the clip keeps `keep` frames past the drawn one and the model starts
    // again from them, so the next request is the jump.
    let asked = s
        .issued
        .iter()
        .any(|g| g.pose == JUMP && g.id >= clip.frames[0].tag);
    if started && !asked {
        let k = (clip.drawn.floor() as usize + 1 + keep).min(clip.frames.len() - 1);
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
    p.generated.clear();
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
            s.target = sys.as_ref().and_then(|sys| {
                match crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent") {
                    Ok(target) => Some(target),
                    Err(error) => {
                        s.failure = Some(error.clone());
                        p.errors.push(format!("agent {}: {error}", a.entity));
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
        let ahead = a.settings.lookahead.max(2.0 * r.latency as f32 / 1000.0);
        let lookahead = (ahead * RATE) as usize;
        if serial != s.serial
            || (r.motion.goals(s.character) == 0 && s.ahead(&r.motion) < lookahead.max(1))
        {
            let queued = s.ahead(&r.motion) as f32 / (RATE * s.warp);
            let (at, h, pose, seconds) = predict(&goals, a, s, queued);
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
    start_flight(r);
    for a in &agents {
        let s = p.agents.get_mut(&a.entity).expect("added above");
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
        for _ in 0..3 {
            while s.clip.is_none() && (s.next.is_none() || s.phase >= 1.0) {
                let Some(f) = s.pending.pop_front().or_else(|| r.motion.pop(s.character)) else {
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
        while s.clip.is_none()
            && let Some(next) = s.next
            && let Some(at) = s
                .issued(&next)
                .map(|g| g.start + f64::from(next.index) / f64::from(RATE))
            && now - at > MAX_LAG
            && let Some(f) = s.pending.pop_front().or_else(|| r.motion.pop(s.character))
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
        // Frames kept past the drawn one when a jump restarts the model: what
        // covers a batch's latency (a native host waits for it instead).
        let keep = if cfg!(target_arch = "wasm32") {
            (r.latency / 1000.0 * f64::from(RATE)).ceil() as usize
        } else {
            1
        };
        jump_clip(&mut r.motion, s, goals.agents.get(&a.entity), elapsed, keep);
        s.count_wait();
        if s.next.is_none() || s.phase >= 1.0 {
            s.phase = s.phase.min(1.0);
        }
        if let Some(sys) = sys.as_ref()
            && let Some(m) = palette(r, a, s, &goals, sys, elapsed)
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
    start_flight(r);
    (p.batches, p.last_batch) = stats;
    fallback(eng, p, &agents, &goals, sys.as_ref());
}

impl Presenter {
    fn presentation(&self, entity: u64) -> Value {
        let (state, reason) = match &self.model {
            Model::Unloaded => ("loading", "the model is read at the next frame"),
            Model::Waiting(why) => ("loading", why.as_str()),
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
        "inFlight": ready.is_some_and(|r| r.flight.is_some()),
        "batches": p.batches,
        "lastBatch": { "characters": p.last_batch.0, "ms": p.last_batch.1 },
        "frameBudgetMs": FRAME_BUDGET_MS,
        "startup": ready.map(|r| &r.startup),
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
        let spawned: Value = serde_json::from_str(&og_world_spawn(world, None)).unwrap();
        let entity = spawned["value"].as_f64().unwrap();
        let inserted: Value =
            serde_json::from_str(&og_world_insert(world, entity, AGENT.into(), None)).unwrap();
        assert_eq!(inserted["ok"], true);
        let eng = crate::engine::engine(world).unwrap();
        eng.render.borrow_mut().motion.model = Model::Failed("weights could not be read".into());
        let expected = json!({"state":"failed", "reason":"weights could not be read"});
        let answer: Value = serde_json::from_str(&og_motion_presentation(world, entity)).unwrap();
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
            positions: [[0.0; 3]; og_motion::contract::JOINTS],
            rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
                og_motion::contract::JOINTS],
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
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0, 1);
        assert_eq!(drawn(&s), 10.0);
        // Falling past the takeoff height: held before the clip's
        // touchdown until the simulation lands.
        at(&mut agent, Some(jump(JumpStage::Air, -2.0 * v0, 0.0)));
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0, 1);
        assert_eq!(drawn(&s), 14.0);
        // Landed: the touchdown frame, reached at no more than CATCH_UP
        // times the clip's rate.
        at(&mut agent, Some(jump(JumpStage::Landed, 0.0, 0.0)));
        jump_clip(
            &mut motion,
            &mut s,
            Some(&agent),
            0.25 / (CATCH_UP * RATE),
            1,
        );
        assert_eq!(drawn(&s), 14.25);
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0, 1);
        assert_eq!(drawn(&s), 15.0);
        // The goal over: the frames after the drawn one play on.
        at(&mut agent, None);
        jump_clip(&mut motion, &mut s, Some(&agent), 1.0, 1);
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
        jump_clip(&mut motion, &mut s, Some(&agent), 0.0, 1);
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
            settings: Settings::of(&json!({"drive":true})),
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
            positions: [[0.0; 3]; og_motion::contract::JOINTS],
            rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
                og_motion::contract::JOINTS],
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
            settings: Settings::of(&json!({"drive":true})),
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
            skeleton: 1,
            skin: 0,
        };
        let crouch = og_motion::library::index("crouch").unwrap();
        let wave = og_motion::library::index("wave").unwrap();
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
            settings: Settings::of(&json!({"drive":true})),
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
        let mut rest: [Trs; og_motion::contract::JOINTS] =
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
        sys.add_clip(1, og_motion::soma::clip("crouch", &asset, &crouch))
            .unwrap();
        sys.add_clip(2, og_motion::soma::clip("step", &asset, &step))
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
