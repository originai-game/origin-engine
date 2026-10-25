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

use crate::schema_ext::{WaterLine, WaterSurface};
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// A height field's heights (relative to the rest level) and their vertical
/// velocities, row-major (`resolution` per row, rows along +z), and which
/// bodies were under the surface after the last step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SurfaceState {
    pub resolution: u32,
    pub size: f32,
    pub heights: Vec<f32>,
    pub velocities: Vec<f32>,
    pub under: IdMap<u64, bool>,
}

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

impl SurfaceState {
    pub fn new(w: &WaterSurface) -> SurfaceState {
        let n = w.resolution.clamp(8, 256) as usize;
        SurfaceState {
            resolution: w.resolution,
            size: w.size,
            heights: vec![0.0; n * n],
            velocities: vec![0.0; n * n],
            under: IdMap::new(),
        }
    }

    fn n(&self) -> usize {
        self.resolution.clamp(8, 256) as usize
    }

    /// The height (relative to rest) at a world xz, bilinear; `centre` is
    /// the entity's position. Zero outside the square.
    pub fn height_at(&self, centre: Vec3, x: f32, z: f32) -> f32 {
        let n = self.n();
        let cell = self.size / (n - 1) as f32;
        let u = (x - (centre.x - self.size * 0.5)) / cell;
        let v = (z - (centre.z - self.size * 0.5)) / cell;
        if !(u >= 0.0 && v >= 0.0 && u <= (n - 1) as f32 && v <= (n - 1) as f32) {
            return 0.0;
        }
        let (i, j) = ((u as usize).min(n - 2), (v as usize).min(n - 2));
        let (fu, fv) = (u - i as f32, v - j as f32);
        let h = |a: usize, b: usize| self.heights[b * n + a];
        let top = h(i, j) + (h(i + 1, j) - h(i, j)) * fu;
        let bottom = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * fu;
        top + (bottom - top) * fv
    }

    /// Bodies push and float, then the waves travel one tick.
    pub fn step(
        &mut self,
        w: &WaterSurface,
        dt: f32,
        gravity: f32,
        centre: Vec3,
        floaters: &[Floater],
    ) -> (Vec<WaterForce>, Vec<Crossing>) {
        let n = self.n();
        let cell = self.size / (n - 1) as f32;
        let half = self.size * 0.5;
        let mut forces = Vec::new();
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
            let depth = (level - f.min.y).clamp(0.0, (f.max.y - f.min.y).max(0.0));
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
                let push = f.velocity.y * w.splash * 0.06;
                for j in j0..=j1 {
                    for i in i0..=i1 {
                        self.velocities[j * n + i] += push;
                    }
                }
            }
            if f.dynamic {
                let area = (f.max.x - f.min.x) * (f.max.z - f.min.z);
                let volume = area * depth;
                let fraction = depth / (f.max.y - f.min.y).max(1e-6);
                forces.push(WaterForce {
                    entity: f.entity,
                    impulse: Vec3::Y * (w.buoyancy * volume * dt * (gravity / 9.81)),
                    keep: (1.0 - w.drag * fraction * dt).clamp(0.0, 1.0),
                });
            }
        }
        self.under = seen;
        // The wave equation, with the speed held under the grid's limit.
        let c = w.wave_speed.min(0.5 * cell / dt);
        let k = c * c / (cell * cell) * dt;
        let keep = per_tick(w.damping, dt);
        let h = &self.heights;
        let mut next_v = self.velocities.clone();
        for j in 0..n {
            for i in 0..n {
                let at = j * n + i;
                let left = if i > 0 { h[at - 1] } else { h[at] };
                let right = if i + 1 < n { h[at + 1] } else { h[at] };
                let up = if j > 0 { h[at - n] } else { h[at] };
                let down = if j + 1 < n { h[at + n] } else { h[at] };
                let lap = left + right + up + down - 4.0 * h[at];
                next_v[at] = (next_v[at] + lap * k) * keep;
            }
        }
        self.velocities = next_v;
        for (h, v) in self.heights.iter_mut().zip(&self.velocities) {
            *h = (*h + v * dt).clamp(-2.0, 2.0);
        }
        (forces, crossings)
    }
}

/// A 2D water line's spring heights and velocities, left to right.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LineState {
    pub columns: u32,
    pub width: f32,
    pub heights: Vec<f32>,
    pub velocities: Vec<f32>,
    pub under: IdMap<u64, bool>,
}

impl LineState {
    pub fn new(w: &WaterLine) -> LineState {
        let n = w.columns.clamp(4, 512) as usize;
        LineState {
            columns: w.columns,
            width: w.width,
            heights: vec![0.0; n],
            velocities: vec![0.0; n],
            under: IdMap::new(),
        }
    }

    fn n(&self) -> usize {
        self.columns.clamp(4, 512) as usize
    }

    /// The surface height (relative to rest) at world x, linear between springs.
    pub fn height_at(&self, centre: Vec2, x: f32) -> f32 {
        let n = self.n();
        let step = self.width / (n - 1) as f32;
        let u = (x - (centre.x - self.width * 0.5)) / step;
        if !(u >= 0.0 && u <= (n - 1) as f32) {
            return 0.0;
        }
        let i = (u as usize).min(n - 2);
        let f = u - i as f32;
        self.heights[i] + (self.heights[i + 1] - self.heights[i]) * f
    }

    pub fn step(
        &mut self,
        w: &WaterLine,
        dt: f32,
        gravity: f32,
        centre: Vec2,
        floaters: &[Floater],
    ) -> (Vec<WaterForce>, Vec<Crossing>) {
        let n = self.n();
        let step = self.width / (n - 1) as f32;
        let left_edge = centre.x - self.width * 0.5;
        let mut forces = Vec::new();
        let mut crossings = Vec::new();
        let mut seen = IdMap::new();
        for f in floaters {
            if f.max.x < left_edge || f.min.x > left_edge + self.width {
                continue;
            }
            if f.max.y < centre.y - w.depth {
                continue;
            }
            let level = centre.y + self.height_at(centre, f.centre.x);
            let under = f.centre.y < level;
            let was = self.under.get(&f.entity).copied().unwrap_or(under);
            seen.insert(f.entity, under);
            if under != was {
                crossings.push(Crossing {
                    entity: f.entity,
                    point: Vec3::new(f.centre.x, level, f.centre.z),
                    speed: f.velocity.y,
                });
                // A crossing kicks the springs under the body.
                let i0 = ((f.min.x - left_edge) / step).floor().max(0.0) as usize;
                let i1 = (((f.max.x - left_edge) / step).ceil() as usize).min(n - 1);
                let width = (f.max.x - f.min.x).max(step);
                let kick = f.velocity.y * w.splash * width.sqrt() * 8.0;
                for i in i0..=i1 {
                    self.velocities[i] += kick;
                }
            }
            let depth = (level - f.min.y).clamp(0.0, (f.max.y - f.min.y).max(0.0));
            if depth > 0.0 && f.dynamic {
                let area = (f.max.x - f.min.x) * depth;
                let fraction = depth / (f.max.y - f.min.y).max(1e-6);
                forces.push(WaterForce {
                    entity: f.entity,
                    impulse: Vec3::Y * (w.buoyancy * area * dt * (gravity / 9.81)),
                    keep: (1.0 - w.drag * fraction * dt).clamp(0.0, 1.0),
                });
            }
        }
        self.under = seen;
        // Springs, then the waves passed to the neighbours (eight passes).
        for (h, v) in self.heights.iter_mut().zip(self.velocities.iter_mut()) {
            let a = -w.tension * *h - w.damping * *v;
            *v += a * dt;
            *h = (*h + *v * dt).clamp(-w.depth.max(0.1), w.depth.max(0.1) * 2.0);
        }
        let spread = w.spread.clamp(0.0, 0.5);
        let mut left = vec![0.0f32; n];
        let mut right = vec![0.0f32; n];
        for _ in 0..8 {
            for i in 0..n {
                if i > 0 {
                    left[i] = spread * (self.heights[i] - self.heights[i - 1]);
                    self.velocities[i - 1] += left[i];
                }
                if i + 1 < n {
                    right[i] = spread * (self.heights[i] - self.heights[i + 1]);
                    self.velocities[i + 1] += right[i];
                }
            }
            for i in 0..n {
                if i > 0 {
                    self.heights[i - 1] += left[i] * dt;
                }
                if i + 1 < n {
                    self.heights[i + 1] += right[i] * dt;
                }
            }
        }
        (forces, crossings)
    }
}
