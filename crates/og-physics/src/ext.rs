//! Physics beyond rigid bodies, written once and compiled for each profile
//! (like `sim.rs`), run by `sim::step` around the rigid-body step, a
//! module per system: joints, the character controller, ropes and cloth,
//! the particle fluids, water surfaces and lines, and buoyancy.
//! `mover.rs` is the controllers' move against the world (both
//! profiles); `platformer.rs` (2D), `vehicle2.rs` / `vehicle3.rs` and
//! `ragdoll.rs` (both profiles, through each one's joint) hold what the profiles do differently.

use super::mover::Mover;
use super::sim::{BodyPoses, Sim};
use super::{
    DIM, angvel, flat, heightfield_surface, joint_frame, pose, rp, surface_only, unangvel, unpose,
    unv, v,
};
use crate::fluid::FluidState;
use crate::gather::Scene;
use crate::schema::*;
use crate::schema_ext::*;
use crate::verlet::{ClothState, RopeState, Solids};
use crate::water::{Floater, LineState, SurfaceState, WaterForce};
use crate::{ControlOut, StepOut};
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn entity_of(c: &rp::Collider) -> u64 {
    c.user_data as u64
}

#[path = "ext/buoyancy.rs"]
mod buoyancy;
#[path = "ext/carry.rs"]
mod carry;
#[path = "ext/characters.rs"]
mod characters;
#[path = "ext/fluids.rs"]
mod fluids;
#[path = "ext/joints.rs"]
mod joints;
#[path = "ext/points.rs"]
mod points;
#[path = "ext/soft.rs"]
mod soft;
#[path = "ext/surfaces.rs"]
mod surfaces;

pub(crate) use buoyancy::*;
pub(crate) use carry::*;
pub(crate) use characters::*;
use fluids::*;
pub(crate) use joints::*;
pub(crate) use points::*;
pub(crate) use soft::*;
pub(crate) use surfaces::*;
