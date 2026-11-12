//! og.WaterLine: its line of springs, splashed by and pushing the bodies in it.

use super::{Crossing, Floater, WaterForce};
use crate::schema_ext::WaterLine;
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

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
                for velocity in self.velocities.iter_mut().take(i1 + 1).skip(i0) {
                    *velocity += kick;
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
