//! The 3D particle fluid (og.Fluid3D): its particles, the lattice they
//! start on, its emitter (new particles at `emitRate`, up to `maxCount`,
//! before the tick's step) and its sink (after it). The solver is
//! [`super::solve`], the 2D fluid's. The emitter's count and its carry are
//! part of the og.physics state block with the particles.

use super::solve::{self, Body, GROUP_SIZE, Rule, jitter};
use crate::schema::Fluid3D;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// The most particles one fluid holds.
pub(crate) const MAX_PARTICLES: u32 = 32768;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Fluid3State {
    pub count: u32,
    pub seed: u32,
    pub positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    /// The fluid's lane in its group (`u32::MAX`: in none).
    pub slot: u32,
    /// In a group, each particle's mix over the group's lanes (summing to
    /// 1; its own lane alone until it mixes); empty in none.
    pub mix: Vec<[f32; GROUP_SIZE]>,
    /// Particles the emitter has made (the jitter's index).
    pub emitted: u32,
    /// The emitter's fraction of a particle carried to the next tick.
    pub carry: f32,
}

impl Fluid3State {
    /// `count` particles on a jittered grid filling the spawn box from the
    /// bottom up.
    pub fn new(fluid: &Fluid3D, centre: Vec3) -> Fluid3State {
        let n = fluid
            .count
            .min(fluid.max_count.clamp(1, MAX_PARTICLES))
            .min(MAX_PARTICLES);
        let size = fluid.size.max(Vec3::splat(0.01));
        let mut positions = Vec::with_capacity(n as usize);
        if n > 0 {
            // A cubic lattice of the box's volume per particle.
            let cell = og_core::math::pow(size.x * size.y * size.z / n as f32, 1.0 / 3.0);
            let cols = ((size.x / cell).round() as u32).max(1);
            let deep = ((size.z / cell).round() as u32).max(1);
            let layer = cols * deep;
            let rows = n.div_ceil(layer).max(1);
            let step = Vec3::new(
                size.x / cols as f32,
                size.y / rows as f32,
                size.z / deep as f32,
            );
            let origin = centre - size * 0.5 + step * 0.5;
            for i in 0..n {
                let (x, z, y) = (i % cols, (i / cols) % deep, i / layer);
                let j = Vec3::new(
                    jitter(fluid.seed, i, 0) - 0.5,
                    jitter(fluid.seed, i, 1) - 0.5,
                    jitter(fluid.seed, i, 2) - 0.5,
                ) * step
                    * 0.3;
                positions.push(
                    origin + Vec3::new(x as f32 * step.x, y as f32 * step.y, z as f32 * step.z) + j,
                );
            }
        }
        Fluid3State {
            count: fluid.count,
            seed: fluid.seed,
            velocities: vec![Vec3::ZERO; positions.len()],
            positions,
            slot: u32::MAX,
            mix: Vec::new(),
            emitted: 0,
            carry: 0.0,
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

    /// The emitter's particles for a tick of `dt`: `emitRate · dt` (the
    /// fraction carried), each at a jittered point of the emitter's cube
    /// with the emitter's velocity, while fewer than `maxCount` live.
    pub fn emit(&mut self, fluid: &Fluid3D, centre: Vec3, dt: f32) {
        if fluid.emit_rate.is_nan() || fluid.emit_rate <= 0.0 {
            self.carry = 0.0;
            return;
        }
        self.carry += fluid.emit_rate.min(20000.0) * dt;
        let whole = self.carry.floor();
        self.carry -= whole;
        let cap = fluid.max_count.clamp(1, MAX_PARTICLES) as usize;
        let room = cap.saturating_sub(self.positions.len());
        let new = (whole as usize).min(room);
        let at = centre + fluid.emit_offset;
        let r = fluid.emit_radius.max(0.0);
        let velocity = if fluid.emit_velocity.is_finite() {
            fluid.emit_velocity
        } else {
            Vec3::ZERO
        };
        for _ in 0..new {
            let i = self.emitted;
            let k = Vec3::new(
                jitter(fluid.seed ^ 0x5bd1_e995, i, 0) - 0.5,
                jitter(fluid.seed ^ 0x5bd1_e995, i, 1) - 0.5,
                jitter(fluid.seed ^ 0x5bd1_e995, i, 2) - 0.5,
            ) * (2.0 * r);
            self.positions.push(at + k);
            self.velocities.push(velocity);
            if self.slot != u32::MAX {
                self.mix.push(solve::own(self.slot));
            }
            self.emitted = self.emitted.wrapping_add(1);
        }
    }

    /// Removes the particles inside the sink box, keeping the others in
    /// order.
    pub fn sink(&mut self, fluid: &Fluid3D, centre: Vec3) {
        let half = fluid.sink_size * 0.5;
        if !(half.x > 0.0 && half.y > 0.0 && half.z > 0.0) {
            return;
        }
        let at = centre + fluid.sink_offset;
        let (lo, hi) = (at - half, at + half);
        let inside = |p: &Vec3| p.cmpge(lo).all() && p.cmple(hi).all();
        if !self.positions.iter().any(inside) {
            return;
        }
        let mixed = self.mix.len() == self.positions.len();
        let mut keep = 0;
        for i in 0..self.positions.len() {
            if inside(&self.positions[i]) {
                continue;
            }
            self.positions[keep] = self.positions[i];
            self.velocities[keep] = self.velocities[i];
            if mixed {
                self.mix[keep] = self.mix[i];
            }
            keep += 1;
        }
        self.positions.truncate(keep);
        self.velocities.truncate(keep);
        if mixed {
            self.mix.truncate(keep);
        }
    }

    /// The fluid as the solver steps it, its container about `centre`.
    pub fn body(&mut self, fluid: &Fluid3D, centre: Vec3) -> Body<'_, Vec3> {
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
