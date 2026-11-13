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
    AGENT, Agent, Goals, JumpStage, Kind, RUN_SPEED, Settings, advance, facing, lock, performance,
    shared, transform_of,
};
use crate::RenderBackend;
use crate::engine::{Engine, hooks};
use crate::host::wire::{FromJson, record};
use crate::present::extract::rows;
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

mod jump;
mod load;
mod pose;
mod predict;
mod report;
mod shown;
mod tick;

pub(crate) use jump::*;
pub(crate) use load::*;
pub(crate) use pose::*;
pub(crate) use predict::*;
pub(crate) use report::*;
pub(crate) use shown::*;
pub(crate) use tick::*;

#[derive(Default)]
pub(crate) enum Model {
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
    /// rotation slerped, as og-anim's `crate::present::interpolate::pose`) when that
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
                .map(|(&m0, &m1)| crate::present::extract::blend(m0, m1, a, f32::INFINITY))
                .collect(),
            None => now.clone(),
        })
    }

    /// The state a presentation checkpoint keeps (`crate::present::presentation`):
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
        og_core::hash::hash_bytes(text.as_bytes())
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

#[cfg(test)]
mod tests;
