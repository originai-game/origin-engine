//! og.WaterSurface: its height field, splashed by and pushing the bodies in it.

use super::{Crossing, Floater};
use crate::schema::WaterSurface;
use og_core::IdMap;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// A height field's heights (relative to the rest level) and their vertical
/// velocities, row-major (`resolution` per row, rows along +z), and which
/// bodies were under the surface after the last step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SurfaceState {
    pub resolution: u32,
    pub size: f32,
    pub depth: f32,
    pub heights: Vec<f32>,
    pub velocities: Vec<f32>,
    pub under: IdMap<u64, bool>,
}

impl SurfaceState {
    pub fn new(w: &WaterSurface) -> SurfaceState {
        let n = w.resolution.clamp(8, 256) as usize;
        SurfaceState {
            resolution: w.resolution,
            size: w.size,
            depth: w.depth,
            heights: vec![0.0; n * n],
            velocities: vec![0.0; n * n],
            under: IdMap::new(),
        }
    }

    fn n(&self) -> usize {
        self.resolution.clamp(8, 256) as usize
    }

    /// The distance between two heights.
    pub fn cell(&self) -> f32 {
        self.size / (self.n() - 1) as f32
    }

    /// The height (relative to rest) at a world xz, bilinear; `centre` is
    /// the entity's position. Zero outside the square.
    pub fn height_at(&self, centre: Vec3, x: f32, z: f32) -> f32 {
        self.sample(&self.heights, centre, x, z)
    }

    /// How fast the height at a world xz is rising (m/s), as
    /// [`Self::height_at`] reads the height.
    pub fn velocity_at(&self, centre: Vec3, x: f32, z: f32) -> f32 {
        self.sample(&self.velocities, centre, x, z)
    }

    /// One of the grids at a world xz, bilinear; zero outside the square.
    fn sample(&self, grid: &[f32], centre: Vec3, x: f32, z: f32) -> f32 {
        let n = self.n();
        let cell = self.size / (n - 1) as f32;
        let u = (x - (centre.x - self.size * 0.5)) / cell;
        let v = (z - (centre.z - self.size * 0.5)) / cell;
        if !(u >= 0.0 && v >= 0.0 && u <= (n - 1) as f32 && v <= (n - 1) as f32) {
            return 0.0;
        }
        let (i, j) = ((u as usize).min(n - 2), (v as usize).min(n - 2));
        let (fu, fv) = (u - i as f32, v - j as f32);
        let h = |a: usize, b: usize| grid[b * n + a];
        let top = h(i, j) + (h(i + 1, j) - h(i, j)) * fu;
        let bottom = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * fu;
        top + (bottom - top) * fv
    }

    /// Bodies push and float, then the waves travel one tick.
    pub fn step(
        &mut self,
        w: &WaterSurface,
        dt: f32,
        centre: Vec3,
        floaters: &[Floater],
    ) -> Vec<Crossing> {
        let n = self.n();
        let cell = self.size / (n - 1) as f32;
        let half = self.size * 0.5;
        self.depth = w.depth;
        let mut crossings = Vec::new();
        let mut seen = IdMap::new();
        for f in floaters {
            if f.max.x < centre.x - half
                || f.min.x > centre.x + half
                || f.max.z < centre.z - half
                || f.min.z > centre.z + half
            {
                continue;
            }
            // Crossings and buoyancy use the rest level: a body's own wake
            // must not carry the surface down with it.
            let level = centre.y;
            let under = f.centre.y < level;
            let was = self.under.get(&f.entity).copied().unwrap_or(under);
            seen.insert(f.entity, under);
            if under != was {
                crossings.push(Crossing {
                    entity: f.entity,
                    point: Vec3::new(
                        f.centre.x,
                        level + self.height_at(centre, f.centre.x, f.centre.z),
                        f.centre.z,
                    ),
                    speed: f.velocity.y,
                });
            }
            let depth = (level.min(f.max.y) - f.min.y.max(level - w.depth)).max(0.0);
            if depth <= 0.0 {
                continue;
            }
            // While it cuts the surface it pushes the cells under its
            // footprint by its vertical speed: the splash and the wake.
            if f.min.y < level && f.max.y > level {
                let (i0, i1) = (
                    ((f.min.x - (centre.x - half)) / cell).floor().max(0.0) as usize,
                    (((f.max.x - (centre.x - half)) / cell).ceil() as usize).min(n - 1),
                );
                let (j0, j1) = (
                    ((f.min.z - (centre.z - half)) / cell).floor().max(0.0) as usize,
                    (((f.max.z - (centre.z - half)) / cell).ceil() as usize).min(n - 1),
                );
                for j in j0..=j1 {
                    for i in i0..=i1 {
                        let x = centre.x - half + i as f32 * cell;
                        let z = centre.z - half + j as f32 * cell;
                        let thickness = |x, z| f.column_depth(x, z, level, w.depth);
                        let submerged = thickness(x, z);
                        let dx =
                            (thickness(x + cell * 0.5, z) - thickness(x - cell * 0.5, z)) / cell;
                        let dz =
                            (thickness(x, z + cell * 0.5) - thickness(x, z - cell * 0.5)) / cell;
                        let push = (if submerged > 0.0 { f.velocity.y } else { 0.0 }
                            - f.velocity.x * dx
                            - f.velocity.z * dz)
                            * w.splash
                            * dt;
                        self.velocities[j * n + i] += push;
                    }
                }
            }
        }
        self.under = seen;
        // CFL substeps preserve the requested speed, including fine grids.
        let c = w.wave_speed.max(0.0);
        let steps = (2.0 * c * dt / cell).ceil().max(1.0) as u32;
        let sub_dt = dt / steps as f32;
        let k = c * c / (cell * cell) * sub_dt;
        for _ in 0..steps {
            let h = &self.heights;
            for j in 0..n {
                for i in 0..n {
                    let at = j * n + i;
                    let left = h[j * n + i.saturating_sub(1)];
                    let right = h[j * n + (i + 1).min(n - 1)];
                    let up = h[j.saturating_sub(1) * n + i];
                    let down = h[(j + 1).min(n - 1) * n + i];
                    let lap = left + right + up + down - 4.0 * h[at];
                    let edge = i.min(j).min(n - 1 - i).min(n - 1 - j) as f32 * cell;
                    let sponge = if w.sponge_width > 0.0 {
                        (1.0 - edge / w.sponge_width).max(0.0)
                    } else {
                        0.0
                    };
                    let keep = og_core::math::exp(
                        -(w.damping.max(0.0) + w.sponge_damping.max(0.0) * sponge * sponge)
                            * sub_dt,
                    );
                    self.velocities[at] = (self.velocities[at] + lap * k) * keep;
                }
            }
            for (h, v) in self.heights.iter_mut().zip(&self.velocities) {
                *h += v * sub_dt;
            }
        }
        crossings
    }
}
