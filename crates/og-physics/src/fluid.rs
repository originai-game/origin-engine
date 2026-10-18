//! The 2D particle fluid: double density relaxation (Clavet, Beaudoin and
//! Poulin, "Particle-based viscoelastic fluid simulation", SCA 2005) in the
//! xy plane. Each substep applies gravity and the viscosity impulses,
//! moves the particles, relaxes them towards the rest density (pressure
//! and near pressure), pushes them out of solids and the container, and
//! takes the velocity from the displacement.
//!
//! Determinism: neighbours come from a uniform grid of cell size `radius`
//! whose particles are sorted by (cell, index); pairs are visited in that
//! order and the relaxation updates positions in place in index order, so
//! every sum is taken in the same order on every host. `+ - * /` and
//! `sqrt` only. The particles are part of the og.physics state block.

use crate::schema_ext::Fluid2D;
use crate::verlet::Solids;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// The time unit the stiffness numbers are tuned in: one 60 Hz tick.
const TICK: f32 = 1.0 / 60.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct FluidState {
    pub count: u32,
    pub seed: u32,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
}

/// A hash of (seed, i, k) to 0..1: the spawn jitter.
fn jitter(seed: u32, i: u32, k: u32) -> f32 {
    let mut h = seed
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add(i.wrapping_mul(0x9e37_79b1))
        .wrapping_add(k.wrapping_mul(0x85eb_ca77));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// The neighbour grid of one substep: particle indices sorted by cell.
struct Grid {
    cell: f32,
    /// (cell x, cell y, particle) sorted.
    sorted: Vec<(i32, i32, u32)>,
}

impl Grid {
    fn build(positions: &[Vec2], cell: f32) -> Grid {
        let inv = 1.0 / cell;
        let mut sorted: Vec<(i32, i32, u32)> = positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    (p.x * inv).floor() as i32,
                    (p.y * inv).floor() as i32,
                    i as u32,
                )
            })
            .collect();
        sorted.sort_unstable_by_key(|&(x, y, i)| (y, x, i));
        Grid { cell, sorted }
    }

    /// The range of `sorted` in cell (`x`, `y`).
    fn range(&self, x: i32, y: i32) -> &[(i32, i32, u32)] {
        let lo = self
            .sorted
            .partition_point(|&(cx, cy, _)| (cy, cx) < (y, x));
        let hi = self
            .sorted
            .partition_point(|&(cx, cy, _)| (cy, cx) <= (y, x));
        &self.sorted[lo..hi]
    }

    /// Every pair (i, j) with j > i closer than the cell size, as flat lists
    /// per particle (`start[i]..start[i + 1]` into `pairs`), j ascending.
    fn neighbours(&self, positions: &[Vec2]) -> (Vec<u32>, Vec<u32>) {
        let n = positions.len();
        let h2 = self.cell * self.cell;
        let inv = 1.0 / self.cell;
        let mut start = Vec::with_capacity(n + 1);
        let mut pairs = Vec::new();
        let mut found: Vec<u32> = Vec::new();
        for (i, p) in positions.iter().enumerate() {
            start.push(pairs.len() as u32);
            let (cx, cy) = ((p.x * inv).floor() as i32, (p.y * inv).floor() as i32);
            found.clear();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    for &(_, _, j) in self.range(cx + dx, cy + dy) {
                        if j as usize > i && (positions[j as usize] - *p).length_squared() < h2 {
                            found.push(j);
                        }
                    }
                }
            }
            found.sort_unstable();
            pairs.extend_from_slice(&found);
        }
        start.push(pairs.len() as u32);
        (start, pairs)
    }
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
        }
    }

    /// One tick of `fluid.substeps` substeps. `centre` is the entity's
    /// position (the container's centre), `z` the plane collisions are
    /// tested in.
    pub fn step(
        &mut self,
        fluid: &Fluid2D,
        dt: f32,
        gravity: Vec2,
        centre: Vec2,
        z: f32,
        solids: &dyn Solids,
    ) {
        let substeps = fluid.substeps.clamp(1, 8);
        let h = fluid.radius.max(0.01);
        let ds = dt / substeps as f32;
        // Displacements in Clavet's units: time in ticks, so the numbers
        // mean the same at any substep count.
        let t = ds / TICK;
        let g = gravity * fluid.gravity_scale;
        let (k, k_near, rho0) = (fluid.stiffness, fluid.near_stiffness, fluid.rest_density);
        let sigma = fluid.viscosity;
        let r_particle = h * 0.25;
        let n = self.positions.len();
        let mut previous = vec![Vec2::ZERO; n];
        for _ in 0..substeps {
            for v in &mut self.velocities {
                *v += g * ds;
            }
            let grid = Grid::build(&self.positions, h);
            let (start, pairs) = grid.neighbours(&self.positions);
            // Viscosity: radial impulses between approaching pairs.
            if sigma > 0.0 {
                for i in 0..n {
                    for &j in &pairs[start[i] as usize..start[i + 1] as usize] {
                        let j = j as usize;
                        let d = self.positions[j] - self.positions[i];
                        let r = d.length();
                        if r <= 1e-9 {
                            continue;
                        }
                        let q = r / h;
                        let dir = d / r;
                        let u = (self.velocities[i] - self.velocities[j]).dot(dir);
                        if u > 0.0 {
                            let impulse = dir * (t * (1.0 - q) * sigma * u * 0.5);
                            self.velocities[i] -= impulse;
                            self.velocities[j] += impulse;
                        }
                    }
                }
            }
            previous.copy_from_slice(&self.positions);
            let cap = 0.5 * h / ds;
            for (p, v) in self.positions.iter_mut().zip(self.velocities.iter_mut()) {
                let len = v.length();
                if len > cap {
                    *v *= cap / len;
                }
                *p += *v * ds;
            }
            // Density relaxation, Gauss-Seidel in index order over the
            // pairs found before the move (both directions).
            let mut density = vec![0.0f32; n];
            let mut near = vec![0.0f32; n];
            for i in 0..n {
                for &j in &pairs[start[i] as usize..start[i + 1] as usize] {
                    let r = (self.positions[j as usize] - self.positions[i]).length();
                    let q = 1.0 - r / h;
                    if q > 0.0 {
                        let (q2, q3) = (q * q, q * q * q);
                        density[i] += q2;
                        density[j as usize] += q2;
                        near[i] += q3;
                        near[j as usize] += q3;
                    }
                }
            }
            for i in 0..n {
                let pressure = k * (density[i] - rho0);
                let near_pressure = k_near * near[i];
                let mut dx = Vec2::ZERO;
                for &j in &pairs[start[i] as usize..start[i + 1] as usize] {
                    let j = j as usize;
                    let d = self.positions[j] - self.positions[i];
                    let r = d.length();
                    if r <= 1e-9 {
                        continue;
                    }
                    let q = 1.0 - r / h;
                    if q <= 0.0 {
                        continue;
                    }
                    let disp = (d / r) * (t * t * h * (pressure * q + near_pressure * q * q) * 0.5);
                    self.positions[j] += disp;
                    dx -= disp;
                }
                self.positions[i] += dx;
            }
            // Solids and the container.
            let half = fluid.bounds * 0.5;
            for p in self.positions.iter_mut() {
                if let Some((q, _)) = solids.push_out(Vec3::new(p.x, p.y, z), r_particle) {
                    *p = Vec2::new(q.x, q.y);
                }
                if half.x > 0.0 {
                    p.x = p.x.clamp(centre.x - half.x, centre.x + half.x);
                }
                if half.y > 0.0 {
                    p.y = p.y.clamp(centre.y - half.y, centre.y + half.y);
                }
            }
            // The velocity from the displacement, capped at half the radius
            // per substep (stable, and nothing tunnels through a wall).
            let cap = 0.5 * h / ds;
            for ((v, p), q) in self
                .velocities
                .iter_mut()
                .zip(&self.positions)
                .zip(&previous)
            {
                let u = (*p - *q) / ds;
                let len = u.length();
                *v = if len > cap { u * (cap / len) } else { u };
            }
        }
    }
}
