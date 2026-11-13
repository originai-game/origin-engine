//! The particle fluids (og.Fluid2D, og.Fluid3D): one solver
//! ([`solve`]) and one neighbour search ([`pairs`]) for both, and each
//! dimension's particles, the lattice they start on and what only it has
//! (the 3D fluid's emitter and sink).

mod fluid2d;
mod fluid3d;
mod pairs;
mod solve;

pub(crate) use fluid2d::FluidState;
pub(crate) use fluid3d::Fluid3State;
pub(crate) use solve::{Body, GROUP_SIZE, MIN_RADIUS, step};
