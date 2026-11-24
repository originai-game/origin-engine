//! 1D Saint-Venant shallow water, finite-volume Rusanov fluxes. See
//! https://www.clawpack.org/riemann_book/html/Shallow_water.html for the
//! conserved depth/momentum and characteristic speeds u ± sqrt(g h).

use super::{Crossing, Floater};
use crate::schema::WaterLine;
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// A 2D water line's surface displacements and vertical velocities, left to right.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LineState {
    pub columns: u32,
    pub width: f32,
    pub depth: f32,
    pub heights: Vec<f32>,
    pub velocities: Vec<f32>,
    /// Depth-integrated horizontal momentum h*u (m²/s).
    pub momentum: Vec<f32>,
    pub under: IdMap<u64, bool>,
}

impl LineState {
    pub fn new(w: &WaterLine) -> LineState {
        let n = w.columns.clamp(4, 512) as usize;
        LineState {
            columns: w.columns,
            width: w.width,
            depth: w.depth,
            heights: vec![0.0; n],
            velocities: vec![0.0; n],
            momentum: vec![0.0; n],
            under: IdMap::new(),
        }
    }

    fn n(&self) -> usize {
        self.columns.clamp(4, 512) as usize
    }

    /// The surface height (relative to rest) at world x, linear between springs.
    pub fn height_at(&self, centre: Vec2, x: f32) -> f32 {
        self.sample(&self.heights, centre, x)
    }

    /// How fast the surface at world `x` is rising (m/s), as
    /// [`Self::height_at`] reads its height.
    pub fn velocity_at(&self, centre: Vec2, x: f32) -> f32 {
        self.sample(&self.velocities, centre, x)
    }

    pub fn flow_at(&self, centre: Vec2, x: f32) -> f32 {
        let depth = self.depth + self.height_at(centre, x);
        if depth > 1e-6 {
            self.sample(&self.momentum, centre, x) / depth
        } else {
            0.0
        }
    }

    /// The springs' heights or speeds at world `x`, between two springs;
    /// zero off the line.
    fn sample(&self, springs: &[f32], centre: Vec2, x: f32) -> f32 {
        let n = self.n();
        let step = self.width / (n - 1) as f32;
        let u = (x - (centre.x - self.width * 0.5)) / step;
        if !(u >= 0.0 && u <= (n - 1) as f32) {
            return 0.0;
        }
        let i = (u as usize).min(n - 2);
        let f = u - i as f32;
        springs[i] + (springs[i + 1] - springs[i]) * f
    }

    pub fn step(
        &mut self,
        w: &WaterLine,
        dt: f32,
        gravity: f32,
        centre: Vec2,
        floaters: &[Floater],
    ) -> Vec<Crossing> {
        let n = self.n();
        let step = self.width / (n - 1) as f32;
        let left_edge = centre.x - self.width * 0.5;
        self.depth = w.depth;
        let mut crossings = Vec::new();
        let mut seen = IdMap::new();
        if w.depth <= 0.0 || dt <= 0.0 {
            self.heights.fill(0.0);
            self.velocities.fill(0.0);
            self.momentum.fill(0.0);
            self.under.clear();
            return crossings;
        }
        let before = self.heights.clone();
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
                // Vertical body motion displaces the local water column.
                let i0 = ((f.min.x - left_edge) / step).floor().max(0.0) as usize;
                let i1 = (((f.max.x - left_edge) / step).ceil() as usize).min(n - 1);
                let width = (f.max.x - f.min.x).max(step);
                let kick = f.velocity.y * w.splash * width.sqrt() * dt;
                for height in self.heights.iter_mut().take(i1 + 1).skip(i0) {
                    *height = (*height + kick).max(-w.depth);
                }
            }
        }
        self.under = seen;
        let g = gravity.max(0.0);
        let state = |h: f32, m: f32| {
            let h = h.max(0.0);
            let u = if h > 1e-6 { m / h } else { 0.0 };
            (h, u, u.abs() + (g * h).sqrt())
        };
        let mut remaining = dt;
        let mut flux = vec![(0.0, 0.0); n + 1];
        while remaining > 0.0 {
            let speed = self
                .heights
                .iter()
                .zip(&self.momentum)
                .map(|(&h, &m)| state(w.depth + h, m).2)
                .fold(0.0, f32::max);
            let sub_dt = if speed > 0.0 {
                remaining.min(0.45 * step / speed)
            } else {
                remaining
            };
            for (i, flux) in flux.iter_mut().enumerate() {
                let l = i.saturating_sub(1);
                let r = i.min(n - 1);
                let ml = if i == 0 {
                    -self.momentum[0]
                } else {
                    self.momentum[l]
                };
                let mr = if i == n {
                    -self.momentum[n - 1]
                } else {
                    self.momentum[r]
                };
                let (hl, ul, sl) = state(w.depth + self.heights[l], ml);
                let (hr, ur, sr) = state(w.depth + self.heights[r], mr);
                let speed = sl.max(sr);
                *flux = (
                    (ml + mr - speed * (hr - hl)) * 0.5,
                    (ml * ul + mr * ur + 0.5 * g * (hl * hl + hr * hr) - speed * (mr - ml)) * 0.5,
                );
            }
            let keep = og_core::math::exp(-w.damping.max(0.0) * sub_dt);
            for i in 0..n {
                self.heights[i] =
                    (self.heights[i] - (flux[i + 1].0 - flux[i].0) * sub_dt / step).max(-w.depth);
                self.momentum[i] =
                    (self.momentum[i] - (flux[i + 1].1 - flux[i].1) * sub_dt / step) * keep;
                if w.depth + self.heights[i] <= 1e-6 {
                    self.momentum[i] = 0.0;
                }
            }
            remaining -= sub_dt;
        }
        for i in 0..n {
            self.velocities[i] = (self.heights[i] - before[i]) / dt;
        }
        crossings
    }
}
