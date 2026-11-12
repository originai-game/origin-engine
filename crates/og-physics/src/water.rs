//! Water that bodies disturb: a height field for 3D (the wave equation on a
//! grid) and a line of springs for 2D side-scrollers (each column pulled to
//! its rest height, waves passed to the neighbours). Both are stepped by the
//! og.physics system after the rigid-body step, splashed by the bodies that
//! cross them (og.Splash) and push those bodies up (buoyancy) and slow them
//! (drag) before the next one. Their heights are part of the og.physics
//! state block.
//!
//! Determinism: `+ - * /` on f32 in index order, no transcendental
//! functions (damping per tick is precomputed from per-second factors with
//! og-core's deterministic `pow`).

use og_core::glam::Vec3;

mod body;
mod line;
mod surface;
mod wakes;

pub use body::WaterBody;
pub(crate) use line::LineState;
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

fn per_tick(per_second: f32, dt: f32) -> f32 {
    og_core::math::pow(per_second.clamp(0.0, 1.0), dt)
}
