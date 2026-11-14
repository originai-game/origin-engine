//! Water and the bodies in it. Every kind answers the same questions the
//! same way: where its surface is at a point ([`level`], and [`ripple`] for
//! the waves bodies raised), what crossed it this tick (og.Splash) and what
//! cuts it now (the renderer's disturbance, [`Wakes`]).
//!
//! - og.Water ([`WaterBody`]): a lake, a sea or a river, a function of the
//!   place and the time. Bodies cross and cut it ([`submersion`]); it lifts
//!   only the hulls of og.Buoyancy.
//! - og.WaterSurface ([`SurfaceState`]): a height field for 3D (the wave
//!   equation on a grid) that the bodies crossing it splash, with a box
//!   buoyancy of its own.
//! - og.WaterLine ([`LineState`]): a line of springs for 2D side-scrollers
//!   (each column pulled to its rest height, waves passed to the
//!   neighbours), with the same box buoyancy.
//!
//! All are stepped by the og.physics system before the rigid-body step.
//! The heights and which bodies are under are part of the og.physics state
//! block.
//!
//! Determinism: the fields are `+ - * /` on f32 in index order, no
//! transcendental functions (what a wave keeps a tick is
//! `og_core::math::damping_keep` of the surface's rate); a sea is
//! og-core's own deterministic function.

use og_core::glam::Vec3;

mod body;
mod line;
mod query;
pub(crate) mod submersion;
mod surface;
mod wakes;

pub use body::WaterBody;
pub(crate) use line::LineState;
pub use query::WaterAt;
pub(crate) use query::{at, level, ripple};
pub(crate) use submersion::Under;
pub(crate) use surface::SurfaceState;
pub use wakes::{Wakes, WaterCut};

/// A body as water sees it: its box, velocity and whether it floats.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Floater {
    pub entity: u64,
    pub min: Vec3,
    pub max: Vec3,
    pub centre: Vec3,
    pub velocity: Vec3,
    /// Dynamic bodies float and slow down; kinematic ones (characters) only splash.
    pub dynamic: bool,
    /// It floats by its own hull (og.Buoyancy): the water takes its splash
    /// and leaves its lift and its cut to that.
    pub hull: bool,
}

/// What water does to a body this step: an impulse and a velocity factor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WaterForce {
    pub entity: u64,
    pub impulse: Vec3,
    /// Multiplies the body's velocity (drag), 0..1.
    pub keep: f32,
}

/// A crossing of the surface, for og.Splash.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Crossing {
    pub entity: u64,
    pub point: Vec3,
    pub speed: f32,
}
