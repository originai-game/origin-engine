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

/// A body cutting a surface in one tick, for the renderer: an
/// og.WaterSurface's ripples, an og.Water sea's hull wave suppression. Its
/// entity; its footprint in the surface's frame (xz relative to the
/// entity), as a box (`min`, `max`) and as the rectangle it turns with
/// (`centre`, `half` its half extents along its own axes, `axis` its own x
/// in xz, unit); how deep it reaches below the rest level (on a sea, below
/// the water at its centre) and its vertical speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterCut {
    pub entity: u64,
    pub min: Vec2,
    pub max: Vec2,
    pub centre: Vec2,
    pub half: Vec2,
    pub axis: Vec2,
    pub depth: f32,
    pub speed: f32,
}

impl WaterCut {
    /// A cut whose footprint is its box, unturned.
    pub fn boxed(entity: u64, min: Vec2, max: Vec2, depth: f32, speed: f32) -> WaterCut {
        WaterCut {
            entity,
            min,
            max,
            centre: (min + max) * 0.5,
            half: (max - min) * 0.5,
            axis: Vec2::X,
            depth,
            speed,
        }
    }
}

/// What the renderer reads of the bodies in a water: of an
/// og.WaterSurface (its ripple layer), the bodies that cut it in each of
/// the last [`Wakes::TICKS`] ticks and the wave settings they travel at; of
/// an og.Water sea, the og.Buoyancy hulls in it (its hull wave
/// suppression). Not world state: never saved, never read
/// by the tick. A world made or restored starts a new `epoch`, so the
/// ripples drawn from it start again too.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wakes {
    pub epoch: u64,
    /// The waves' speed as the height field runs them (held under its grid's
    /// limit), m/s.
    pub wave_speed: f32,
    /// The fraction of a wave kept after a second.
    pub damping: f32,
    pub splash: f32,
    /// The tick's length, s.
    pub dt: f32,
    /// (tick, the bodies that cut the surface after it), oldest first.
    pub ticks: std::collections::VecDeque<(u64, Vec<WaterCut>)>,
}

impl Wakes {
    /// How many ticks are kept: the most the ripples catch up in one frame.
    pub const TICKS: usize = 8;

    /// Records `tick`'s cuts: the floaters that straddle the rest level of
    /// a surface of side `size` at `centre`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &mut self,
        w: &WaterSurface,
        size: f32,
        cell: f32,
        dt: f32,
        tick: u64,
        centre: Vec3,
        floaters: &[Floater],
    ) {
        self.wave_speed = w.wave_speed.min(0.5 * cell / dt.max(1e-6));
        self.damping = w.damping.clamp(0.0, 1.0);
        self.splash = w.splash;
        self.dt = dt;
        let half = size * 0.5;
        let cuts = floaters
            .iter()
            .filter(|f| {
                f.min.y < centre.y
                    && f.max.y > centre.y
                    && f.max.x >= centre.x - half
                    && f.min.x <= centre.x + half
                    && f.max.z >= centre.z - half
                    && f.min.z <= centre.z + half
            })
            .map(|f| {
                WaterCut::boxed(
                    f.entity,
                    Vec2::new(f.min.x - centre.x, f.min.z - centre.z),
                    Vec2::new(f.max.x - centre.x, f.max.z - centre.z),
                    centre.y - f.min.y,
                    f.velocity.y,
                )
            })
            .collect();
        self.push(tick, cuts);
    }

    /// Records `tick`'s og.Buoyancy hulls in an og.Water sea (`cuts`, from
    /// og.Buoyancy's step): no ripples travel on a sea, so no wave settings.
    pub(crate) fn record_sea(&mut self, dt: f32, tick: u64, cuts: Vec<WaterCut>) {
        self.wave_speed = 0.0;
        self.damping = 0.0;
        self.splash = 0.0;
        self.dt = dt;
        self.push(tick, cuts);
    }

    fn push(&mut self, tick: u64, cuts: Vec<WaterCut>) {
        static EPOCHS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        // A tick that does not follow the last one (the world went back):
        // the ripples start again.
        if self.epoch == 0 || self.ticks.back().is_some_and(|(t, _)| *t >= tick) {
            self.epoch = EPOCHS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.ticks.clear();
        }
        self.ticks.push_back((tick, cuts));
        while self.ticks.len() > Self::TICKS {
            self.ticks.pop_front();
        }
    }
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

    /// The distance between two heights.
    pub fn cell(&self) -> f32 {
        self.size / (self.n() - 1) as f32
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wakes_keep_the_last_ticks_cuts_and_a_world_that_went_back_starts_a_new_epoch() {
        let w = WaterSurface::default();
        let crate_box = Floater {
            entity: 7,
            min: Vec3::new(0.5, -0.25, 1.0),
            max: Vec3::new(1.5, 0.75, 2.0),
            centre: Vec3::new(1.0, 0.25, 1.5),
            velocity: Vec3::new(0.0, -2.0, 0.0),
            dynamic: true,
        };
        let above = Floater {
            entity: 8,
            min: Vec3::new(0.0, 1.0, 0.0),
            max: Vec3::new(1.0, 2.0, 1.0),
            ..crate_box
        };
        let centre = Vec3::new(10.0, 0.0, -4.0);
        let moved = |f: Floater| Floater {
            min: f.min + centre,
            max: f.max + centre,
            centre: f.centre + centre,
            ..f
        };
        let mut wakes = Wakes::default();
        for tick in 1..=12 {
            wakes.record(
                &w,
                16.0,
                0.25,
                1.0 / 60.0,
                tick,
                centre,
                &[moved(crate_box), moved(above)],
            );
        }
        let epoch = wakes.epoch;
        assert!(epoch != 0);
        let ticks: Vec<u64> = wakes.ticks.iter().map(|(t, _)| *t).collect();
        assert_eq!(ticks, (5..=12).collect::<Vec<u64>>());
        // Only the body across the rest level, in the surface's frame.
        let (_, cuts) = wakes.ticks.back().unwrap();
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].entity, 7);
        assert_eq!(cuts[0].min, Vec2::new(0.5, 1.0));
        assert_eq!(cuts[0].max, Vec2::new(1.5, 2.0));
        assert_eq!(cuts[0].depth, 0.25);
        // The world went back (a restore, a rollback): a new epoch, the old ticks gone.
        wakes.record(&w, 16.0, 0.25, 1.0 / 60.0, 9, centre, &[moved(crate_box)]);
        assert_ne!(wakes.epoch, epoch);
        assert_eq!(wakes.ticks.len(), 1);
    }
}
