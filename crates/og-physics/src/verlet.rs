//! Verlet ropes and cloth: points integrated by position (velocity is the
//! last step's displacement), held together by distance constraints solved
//! in a fixed order, pinned to their entities and pushed out of solid
//! colliders. Profile-independent: the step hands in a [`Solids`] that
//! answers collisions in the profile's world, and 2D keeps every point at
//! its entity's z.
//!
//! Determinism: `+ - * /` and `sqrt` on f32 in a fixed order, the gust
//! noise from an integer hash of (point, tick); the state is the points,
//! saved with the rest of the simulation.

use crate::schema_ext::{Cloth, PIN_LEFT, PIN_TOP, PIN_TOP_CORNERS, PIN_TOP_RINGS, Rope};
use og_core::glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// What points collide with: solid colliders near them.
pub(crate) trait Solids {
    /// `p` moved out of every solid it is closer than `radius` to, and the
    /// last surface normal pushed along (`None`: nothing touched).
    fn push_out(&self, p: Vec3, radius: f32) -> Option<(Vec3, Vec3)>;
}

/// A rope's points as the last step left them, and the component they were
/// built for (a change of `segments` or `length` rebuilds the rope).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RopeState {
    pub segments: u32,
    pub length: f32,
    pub points: Vec<Vec3>,
    pub previous: Vec<Vec3>,
}

/// Drops z in 2D (the profile's `flat`) but keeps the entity's.
fn planar(p: Vec3, flat: bool, z: f32) -> Vec3 {
    if flat { Vec3::new(p.x, p.y, z) } else { p }
}

/// Pulls `a` and `b` to `rest` apart, `wa`/`wb` their inverse masses (0 pinned).
fn satisfy(points: &mut [Vec3], a: usize, b: usize, rest: f32, wa: f32, wb: f32) {
    let w = wa + wb;
    if w <= 0.0 {
        return;
    }
    let d = points[b] - points[a];
    let len = d.length();
    if len <= 1e-9 {
        return;
    }
    let k = (len - rest) / (len * w);
    points[a] += d * (k * wa);
    points[b] -= d * (k * wb);
}

impl RopeState {
    /// A rope hanging straight down from `start` (or straight to `end`).
    pub fn new(rope: &Rope, start: Vec3, end: Option<Vec3>) -> RopeState {
        let n = rope.segments.clamp(1, 256) as usize;
        let length = rope.length.max(0.0);
        let to = end.unwrap_or(start - Vec3::Y * length);
        let points: Vec<Vec3> = (0..=n)
            .map(|i| start + (to - start) * (i as f32 / n as f32))
            .collect();
        RopeState {
            segments: rope.segments,
            length: rope.length,
            previous: points.clone(),
            points,
        }
    }

    /// One tick: integrate, then `iterations` passes of pins, links and
    /// collisions.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        rope: &Rope,
        dt: f32,
        gravity: Vec3,
        start: Vec3,
        end: Option<Vec3>,
        flat: bool,
        solids: &dyn Solids,
    ) {
        let n = self.points.len();
        let z = start.z;
        let accel = gravity * rope.gravity_scale + rope.wind;
        let keep = rope.damping.clamp(0.0, 1.0);
        let last = n - 1;
        for i in 0..n {
            let pinned = i == 0 || (i == last && end.is_some());
            let p = self.points[i];
            if pinned {
                self.previous[i] = p;
                continue;
            }
            let v = (p - self.previous[i]) * keep;
            self.previous[i] = p;
            self.points[i] = planar(p + v + accel * (dt * dt), flat, z);
        }
        let rest = rope.length.max(0.0) / (n - 1) as f32;
        let radius = rope.radius.max(0.0);
        for _ in 0..rope.iterations.clamp(1, 64) {
            self.points[0] = start;
            if let Some(e) = end {
                self.points[last] = e;
            }
            for i in 0..last {
                let wa = if i == 0 { 0.0 } else { 1.0 };
                let wb = if i + 1 == last && end.is_some() {
                    0.0
                } else {
                    1.0
                };
                satisfy(&mut self.points, i, i + 1, rest, wa, wb);
            }
            if radius > 0.0 {
                for i in 1..n {
                    if i == last && end.is_some() {
                        continue;
                    }
                    if let Some((p, _)) = solids.push_out(self.points[i], radius) {
                        self.points[i] = planar(p, flat, z);
                    }
                }
            }
        }
        self.points[0] = start;
        if let Some(e) = end {
            self.points[last] = e;
        }
    }
}

/// A cloth's points (row-major, `columns` per row) and the layout they were
/// built for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ClothState {
    pub columns: u32,
    pub rows: u32,
    pub spacing: f32,
    pub pins: u32,
    pub points: Vec<Vec3>,
    pub previous: Vec<Vec3>,
}

/// Whether point (`x`, `y`) of a `cols` × `rows` grid is pinned by `pins`.
pub(crate) fn pinned(pins: u32, x: u32, y: u32, cols: u32) -> bool {
    (pins & PIN_TOP != 0 && y == 0)
        || (pins & PIN_TOP_CORNERS != 0 && y == 0 && (x == 0 || x + 1 == cols))
        || (pins & PIN_LEFT != 0 && x == 0)
        || (pins & PIN_TOP_RINGS != 0 && y == 0 && (x.is_multiple_of(4) || x + 1 == cols))
}

/// A hash of (a, b, c) to -1..1: the gust noise.
fn noise(a: u32, b: u32, c: u32) -> f32 {
    let mut h = a
        .wrapping_mul(0x9e37_79b1)
        .wrapping_add(b.wrapping_mul(0x85eb_ca77))
        .wrapping_add(c.wrapping_mul(0xc2b2_ae3d));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 23) as f32 - 1.0
}

/// Linear interpolation of the gust noise over ticks, so it changes smoothly.
fn gust(x: u32, y: u32, tick: u64) -> f32 {
    let period = 24u64;
    let k = (tick / period) as u32;
    let t = (tick % period) as f32 / period as f32;
    let (cx, cy) = (x / 3, y / 3);
    let a = noise(cx, cy, k);
    let b = noise(cx, cy, k.wrapping_add(1));
    a + (b - a) * (t * t * (3.0 - 2.0 * t))
}

impl ClothState {
    /// The rest position of point (`x`, `y`): along the entity's local x and down its local -y.
    pub fn rest(cloth: &Cloth, origin: Vec3, rotation: Quat, x: u32, y: u32) -> Vec3 {
        origin + rotation * Vec3::new(x as f32 * cloth.spacing, -(y as f32) * cloth.spacing, 0.0)
    }

    pub fn new(cloth: &Cloth, origin: Vec3, rotation: Quat) -> ClothState {
        let (cols, rows) = (cloth.columns.clamp(2, 64), cloth.rows.clamp(2, 64));
        let mut points = Vec::with_capacity((cols * rows) as usize);
        for y in 0..rows {
            for x in 0..cols {
                points.push(Self::rest(cloth, origin, rotation, x, y));
            }
        }
        ClothState {
            columns: cloth.columns,
            rows: cloth.rows,
            spacing: cloth.spacing,
            pins: cloth.pins,
            previous: points.clone(),
            points,
        }
    }

    /// The grid's size as built.
    pub fn dims(&self) -> (u32, u32) {
        (self.columns.clamp(2, 64), self.rows.clamp(2, 64))
    }

    /// The normal at a point from its neighbours (for the wind's push).
    fn normal(&self, x: u32, y: u32) -> Vec3 {
        let (cols, rows) = self.dims();
        let at = |x: u32, y: u32| self.points[(y * cols + x) as usize];
        let (x0, x1) = (x.saturating_sub(1), (x + 1).min(cols - 1));
        let (y0, y1) = (y.saturating_sub(1), (y + 1).min(rows - 1));
        let dx = at(x1, y) - at(x0, y);
        let dy = at(x, y1) - at(x, y0);
        dx.cross(dy).normalize_or_zero()
    }

    /// One tick of `substeps` steps: integrate with gravity and a gusting
    /// wind pushed along the local normal, then `iterations` passes of pins,
    /// links (structural and shear, in a fixed order), self-collision (with
    /// `thickness`) and collisions.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        cloth: &Cloth,
        dt: f32,
        tick: u64,
        gravity: Vec3,
        origin: Vec3,
        rotation: Quat,
        flat: bool,
        solids: &dyn Solids,
    ) {
        let (cols, rows) = self.dims();
        let z = origin.z;
        let substeps = cloth.substeps.clamp(1, 16);
        let ds = dt / substeps as f32;
        // The velocity kept per tick, spread over the substeps.
        let damping = cloth.damping.clamp(0.0, 1.0);
        let keep = if substeps == 1 {
            damping
        } else {
            og_core::math::pow(damping, 1.0 / substeps as f32)
        };
        let g = gravity * cloth.gravity_scale;
        let wind = cloth.wind;
        let wind_len = wind.length();
        let thickness = cloth.thickness.max(0.0);
        let s = cloth.spacing.max(0.001);
        let diag = s * std::f32::consts::SQRT_2;
        let radius = cloth.radius.max(0.0);
        let w = |x: u32, y: u32| {
            if pinned(self.pins, x, y, cols) {
                0.0
            } else {
                1.0
            }
        };
        for _ in 0..substeps {
            let mut accel = vec![Vec3::ZERO; self.points.len()];
            for y in 0..rows {
                for x in 0..cols {
                    let i = (y * cols + x) as usize;
                    let mut a = g;
                    if wind_len > 0.0 {
                        let w = wind * (1.0 + cloth.gust * gust(x, y, tick));
                        // Wind pushes a surface along its normal, more when it faces the wind.
                        let n = self.normal(x, y);
                        let along = n * n.dot(w);
                        a += along + w * 0.15;
                    }
                    accel[i] = a;
                }
            }
            for y in 0..rows {
                for x in 0..cols {
                    let i = (y * cols + x) as usize;
                    let p = self.points[i];
                    if pinned(self.pins, x, y, cols) {
                        self.previous[i] = p;
                        continue;
                    }
                    let v = (p - self.previous[i]) * keep;
                    self.previous[i] = p;
                    let mut d = v + accel[i] * (ds * ds);
                    // With self-collision a point moves at most half the
                    // thickness a step, so it cannot jump through a layer.
                    if thickness > 0.0 {
                        let len = d.length();
                        if len > thickness * 0.5 {
                            d *= thickness * 0.5 / len;
                        }
                    }
                    self.points[i] = planar(p + d, flat, z);
                }
            }
            let near = if thickness > 0.0 {
                self.near_pairs(thickness)
            } else {
                Vec::new()
            };
            for _ in 0..cloth.iterations.clamp(1, 64) {
                for y in 0..rows {
                    for x in 0..cols {
                        if pinned(self.pins, x, y, cols) {
                            self.points[(y * cols + x) as usize] =
                                Self::rest(cloth, origin, rotation, x, y);
                        }
                    }
                }
                for y in 0..rows {
                    for x in 0..cols {
                        let i = (y * cols + x) as usize;
                        if x + 1 < cols {
                            satisfy(&mut self.points, i, i + 1, s, w(x, y), w(x + 1, y));
                        }
                        if y + 1 < rows {
                            let j = i + cols as usize;
                            satisfy(&mut self.points, i, j, s, w(x, y), w(x, y + 1));
                            if x + 1 < cols {
                                satisfy(&mut self.points, i, j + 1, diag, w(x, y), w(x + 1, y + 1));
                            }
                            if x > 0 {
                                satisfy(&mut self.points, i, j - 1, diag, w(x, y), w(x - 1, y + 1));
                            }
                        }
                    }
                }
                for &(i, j) in &near {
                    let (xi, yi) = (i % cols, i / cols);
                    let (xj, yj) = (j % cols, j / cols);
                    separate(
                        &mut self.points,
                        i as usize,
                        j as usize,
                        thickness,
                        w(xi, yi),
                        w(xj, yj),
                    );
                }
                if radius > 0.0 {
                    for y in 0..rows {
                        for x in 0..cols {
                            if pinned(self.pins, x, y, cols) {
                                continue;
                            }
                            let i = (y * cols + x) as usize;
                            if let Some((p, _)) = solids.push_out(self.points[i], radius) {
                                self.points[i] = planar(p, flat, z);
                            }
                        }
                    }
                }
            }
            // Friction: a point resting on a solid loses part of its slide.
            let friction = cloth.friction.clamp(0.0, 1.0);
            if friction > 0.0 && radius > 0.0 {
                for (i, p) in self.points.iter().enumerate() {
                    if let Some((_, n)) = solids.push_out(*p, radius * 1.05) {
                        let v = *p - self.previous[i];
                        let slide = v - n * n.dot(v);
                        self.previous[i] += slide * friction;
                    }
                }
            }
        }
    }

    /// The pairs of points (i < j, in order) closer than 1.5 × `thickness`
    /// whose rest distance is at least `thickness` (nearer neighbours are
    /// held by the links), from a grid of cell `thickness` sorted by cell
    /// and index.
    fn near_pairs(&self, thickness: f32) -> Vec<(u32, u32)> {
        let (cols, _) = self.dims();
        let reach = thickness * 1.5;
        let inv = 1.0 / reach;
        let cell = |p: Vec3| {
            (
                (p.x * inv).floor() as i32,
                (p.y * inv).floor() as i32,
                (p.z * inv).floor() as i32,
            )
        };
        let mut sorted: Vec<((i32, i32, i32), u32)> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let (x, y, z) = cell(*p);
                ((z, y, x), i as u32)
            })
            .collect();
        sorted.sort_unstable();
        let rest_far = |i: u32, j: u32| {
            let (dx, dy) = (
                (i % cols) as f32 - (j % cols) as f32,
                (i / cols) as f32 - (j / cols) as f32,
            );
            self.spacing.max(0.001) * (dx * dx + dy * dy).sqrt() >= thickness
        };
        // The occupied cells, each a run of `sorted`; then per cell the runs
        // of the 27 cells around it, searched once for all its points.
        let mut cells: Vec<((i32, i32, i32), usize, usize)> = Vec::new();
        for (k, &(key, _)) in sorted.iter().enumerate() {
            match cells.last_mut() {
                Some(c) if c.0 == key => c.2 = k + 1,
                _ => cells.push((key, k, k + 1)),
            }
        }
        let mut pairs = Vec::new();
        let mut around: Vec<(usize, usize)> = Vec::with_capacity(27);
        for &((cz, cy, cx), lo, hi) in &cells {
            around.clear();
            for dz in -1..=1 {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let key = (cz + dz, cy + dy, cx + dx);
                        if let Ok(c) = cells.binary_search_by(|c| c.0.cmp(&key)) {
                            around.push((cells[c].1, cells[c].2));
                        }
                    }
                }
            }
            for &(_, i) in &sorted[lo..hi] {
                let p = self.points[i as usize];
                for &(a, b) in &around {
                    for &(_, j) in &sorted[a..b] {
                        if j > i
                            && (self.points[j as usize] - p).length_squared() < reach * reach
                            && rest_far(i, j)
                        {
                            pairs.push((i, j));
                        }
                    }
                }
            }
        }
        pairs.sort_unstable();
        pairs
    }
}

/// Pushes `a` and `b` apart to at least `thickness`, `wa`/`wb` their
/// inverse masses (0 pinned).
fn separate(points: &mut [Vec3], a: usize, b: usize, thickness: f32, wa: f32, wb: f32) {
    let w = wa + wb;
    if w <= 0.0 {
        return;
    }
    let d = points[b] - points[a];
    let len = d.length();
    if len >= thickness || len <= 1e-9 {
        return;
    }
    let k = (len - thickness) / (len * w);
    points[a] += d * (k * wa);
    points[b] -= d * (k * wb);
}
