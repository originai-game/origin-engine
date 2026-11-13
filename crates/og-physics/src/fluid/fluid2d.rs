//! The 2D particle fluid (og.Fluid2D): its particles in the xy plane and
//! the lattice they start on. The solver is [`super::solve`], the 3D
//! fluid's.

use super::solve::{self, Body, GROUP_SIZE, Rule, jitter};
use crate::schema::Fluid2D;
use og_core::glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct FluidState {
    pub count: u32,
    pub seed: u32,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    /// The fluid's lane in its group (`u32::MAX`: in none).
    pub slot: u32,
    /// In a group, each particle's mix over the group's lanes (summing to
    /// 1; its own lane alone until it mixes); empty in none.
    pub mix: Vec<[f32; GROUP_SIZE]>,
}

impl FluidState {
    /// `count` particles on a jittered grid filling the spawn box.
    pub fn new(fluid: &Fluid2D, centre: Vec2) -> FluidState {
        let n = fluid.count.clamp(1, 8192);
        let size = fluid.size.max(Vec2::splat(0.01));
        // Columns and rows in the box's aspect.
        let aspect = size.x / size.y;
        let cols = ((n as f32 * aspect).sqrt().ceil() as u32).max(1);
        let rows = n.div_ceil(cols).max(1);
        let step = Vec2::new(size.x / cols as f32, size.y / rows as f32);
        let origin = centre - size * 0.5 + step * 0.5;
        let positions = (0..n)
            .map(|i| {
                let (x, y) = (i % cols, i / cols);
                let j = Vec2::new(
                    jitter(fluid.seed, i, 0) - 0.5,
                    jitter(fluid.seed, i, 1) - 0.5,
                ) * step
                    * 0.3;
                origin + Vec2::new(x as f32 * step.x, y as f32 * step.y) + j
            })
            .collect();
        FluidState {
            count: fluid.count,
            seed: fluid.seed,
            positions,
            velocities: vec![Vec2::ZERO; n as usize],
            slot: u32::MAX,
            mix: Vec::new(),
        }
    }

    /// Joins a group at lane `slot` ([`solve::join`]).
    pub fn join(&mut self, slot: u32) {
        solve::join(&mut self.slot, &mut self.mix, self.positions.len(), slot);
    }

    /// Leaves a group: the particles keep no mix.
    pub fn leave(&mut self) {
        solve::leave(&mut self.slot, &mut self.mix);
    }

    /// The fluid as the solver steps it, its container about `centre`.
    pub fn body(&mut self, fluid: &Fluid2D, centre: Vec2) -> Body<'_, Vec2> {
        Body {
            rule: Rule {
                substeps: fluid.substeps,
                radius: fluid.radius,
                gravity_scale: fluid.gravity_scale,
                viscosity: fluid.viscosity,
                stiffness: fluid.stiffness,
                near_stiffness: fluid.near_stiffness,
                rest_density: fluid.rest_density,
                density: fluid.density,
                mixing: fluid.mixing,
                bounds: fluid.bounds,
            },
            positions: &mut self.positions,
            velocities: &mut self.velocities,
            mix: &mut self.mix,
            slot: self.slot,
            centre,
        }
    }
}
