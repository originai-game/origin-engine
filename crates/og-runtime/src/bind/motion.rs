//! `og.motion` (docs/design.md §2.7, "Motion generation"): goals for
//! humanoids, kept and finished by the simulation, and the engine's motion
//! model drawing them ([`present`]).
//!
//! Two halves, one rule (§2.4): the simulation never reads the model.
//!
//! - **The goals** are world state: per og.MotionAgent entity its queue,
//!   its current goal and how long it has been current, in the `og.motion`
//!   state block (snapshots, restores and the world hash carry it). The
//!   `og.motion` system finishes goals by fixed rules and, with
//!   `MotionAgent.drive`, moves og.Transform to them with fixed math
//!   (walkTo at its speed in a straight line, place over its duration,
//!   turns at `turnSpeed`). og.motion.status answers from here alone.
//! - **The pose** is presentation ([`present`]): before each frame is
//!   drawn, the model (on the renderer's device) is asked for motion that
//!   reaches where the goals will have the character, a batch for every
//!   agent at once, and the frame's generated pose is re-rooted onto
//!   og.Transform, retargeted to the agent's rig, its feet planted, and
//!   drawn in place of og.anim's joint palette. Nothing writes the world.
//!
//! og.MotionAgent is registered lazily (`crate::engine::lazy`): a world that never
//! names it has neither the system nor the state block, so its saves and
//! hash are what they were before og.motion existed.

mod performance;
pub(crate) mod present;
pub use performance::{og_motion_look_at, og_motion_react};

use crate::engine::{Engine, Mount, engine};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{
    self, FaceGoalDesc, FromJson, JumpGoalDesc, MotionBackend, MotionCapabilities, MotionGoalKind,
    MotionStatus, PlaceGoalDesc, PoseGoalDesc, ToJson, WalkGoalDesc,
};
use crate::parse;
use glam::{Quat, Vec3};
use og_anim::motion::library::{POSES, index as pose_index};
use og_core::facing;
use og_core::math::{atan2, sin_cos};
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

mod api;
mod mover;
mod probe;
mod state;
mod system;

pub use api::*;
pub(crate) use mover::*;
pub(crate) use probe::*;
use state::*;
pub(crate) use system::*;

pub(crate) const SYSTEM: &str = "og.motion";
pub(crate) const AGENT: &str = "og.MotionAgent";
/// Below this speed a walkTo walks, above it runs (WalkGoalDesc.speed).
pub(crate) const RUN_SPEED: f32 = 2.5;
/// A walkTo's acceleration when its desc leaves it at 0, m/s².
const ACCELERATION: f32 = 4.0;
/// A walkTo ends within this distance of its target.
const ARRIVED: f32 = 1e-3;
/// The gravity of a world without og.Physics, m/s².
const GRAVITY: f32 = 9.81;
/// A controller jump that never leaves the ground (blocked, or not
/// grounded at takeoff) lands after this long in the air.
const STUCK: f32 = 0.2;

/// MotionGoalKind's variants, in the IDL's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Pose = 1,
    Place = 2,
    Face = 3,
    WalkTo = 4,
    Jump = 5,
}

const KIND_NAMES: [&str; 6] = ["none", "pose", "place", "face", "walkTo", "jump"];

impl Kind {
    fn from_u32(v: u32) -> Option<Kind> {
        Some(match v {
            1 => Kind::Pose,
            2 => Kind::Place,
            3 => Kind::Face,
            4 => Kind::WalkTo,
            5 => Kind::Jump,
            _ => return None,
        })
    }
}

/// A goal as the game queued it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Goal {
    pub kind: Kind,
    /// Index into [`POSES`].
    pub pose: u32,
    pub target: Vec3,
    pub heading: f32,
    pub duration: f32,
    pub speed: f32,
    pub arrival_heading: Option<f32>,
    pub deceleration: f32,
    /// How fast a walk gains speed, m/s² (infinite: at full speed in its
    /// first tick, as goals saved before MOTION5 did).
    pub acceleration: f32,
    /// A jump's apex over its takeoff, m (`height`), and the seconds before
    /// takeoff (`prepare`); its `duration` is the recovery after landing,
    /// `speed` its horizontal travel in m (`distance`) along `heading`.
    pub height: f32,
    pub prepare: f32,
}

/// Where a jump goal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JumpStage {
    Prepare,
    Air,
    Landed,
}

/// A jump goal's state in the simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Jump {
    pub stage: JumpStage,
    /// The takeoff speed, m/s: the gravity's for the goal's height.
    pub v0: f32,
    /// The gravity it falls by, m/s² (positive).
    pub gravity: f32,
    /// The takeoff height (the arc without a controller returns to it).
    pub base: f32,
    /// Seconds since takeoff, and since landing.
    pub air: f32,
    pub landed: f32,
    /// The vertical speed, m/s: the fixed arc's, or the controller's.
    pub velocity: f32,
    /// A controller has been seen off the ground since takeoff.
    pub left: bool,
    /// How far the Transform stood above the ground under it at takeoff
    /// (a controller's), m: what its feet keep below it in the air.
    pub rest: Option<f32>,
    /// Seconds until it lands, as the tick predicts it: the fixed arc's
    /// remaining flight, or a controller's ballistic arc traced to the
    /// ground under it ([`landing_in`]). `None` when there is no ground to
    /// trace (no physics, nothing below).
    pub eta: Option<f32>,
}

/// The goal being carried out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Current {
    pub goal: Goal,
    pub elapsed: f32,
    /// Where the Transform was, and its heading, when the goal started.
    pub start: Vec3,
    pub start_heading: f32,
    /// A jump goal's state.
    pub jump: Option<Jump>,
}

/// One agent's goals.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Agent {
    pub queue: VecDeque<Goal>,
    pub current: Option<Current>,
    pub completed: u32,
    /// The pose the character holds with no goal: the last finished goal's.
    pub hold: u32,
    /// Bumped whenever the current goal changes (a start, a finish, a clear):
    /// what the presenter watches to follow a new goal.
    pub serial: u32,
    pub performance: performance::Performance,
    /// The mover's speed over the ground, m/s: what the next walk or jump
    /// starts from. A walk into another walk or a jump keeps it through
    /// its target; a goal in place stops it.
    pub speed: f32,
}

/// The `og.motion` state: agents by entity bits (ordered, so saves are).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Goals {
    pub agents: BTreeMap<u64, Agent>,
}

/// og.motion on one engine: the goals its world's block saves, shared
/// with its system and its calls.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Arc<Mutex<Goals>>);

/// An engine's og.motion goals.
pub(crate) fn shared(eng: &Engine) -> Arc<Mutex<Goals>> {
    eng.modules.motion.0.clone()
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── the API ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
