//! Small-step XPBD. Physical stiffness k defines compliance 1/k; zero
//! stiffness disables a constraint. Multipliers live for one substep only.
//! Point velocities and inverse masses, not last-step displacements, are state.

use crate::schema::{Cloth, PIN_LEFT, PIN_TOP, PIN_TOP_CORNERS, PIN_TOP_RINGS, Rope};
use og_core::glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn keep_per_substep(damping: f32, dt: f32, substeps: u32) -> f32 {
    og_core::math::damping_keep(damping, dt / substeps.max(1) as f32)
}

pub(crate) trait Solids: Sync {
    fn acceleration(&self, _p: Vec3) -> Vec3 {
        Vec3::ZERO
    }
    fn air(&self, _p: Vec3) -> Vec3 {
        Vec3::ZERO
    }
    fn reaction(&self, _entity: u64, _point: Vec3, _impulse: Vec3, _displacement: Vec3) {}
    fn inverse_mass(&self, _entity: u64) -> f32 {
        0.0
    }
    fn collide_point(
        &self,
        p: &mut Vec3,
        v: &mut Vec3,
        radius: f32,
        friction: f32,
        _mass: f32,
        _dt: f32,
    ) {
        if let Some((q, n)) = self.push_out(*p, radius) {
            *p = q;
            let along = v.dot(n);
            *v = n * along.max(0.0) + (*v - n * along) * (1.0 - friction);
        }
    }
    fn push_out(&self, p: Vec3, radius: f32) -> Option<(Vec3, Vec3)>;
}

fn planar(p: Vec3, flat: bool, z: f32) -> Vec3 {
    if flat { Vec3::new(p.x, p.y, z) } else { p }
}

/// One XPBD distance constraint; returns its signed impulse on b.
#[allow(clippy::too_many_arguments)]
pub(crate) fn link(
    points: &mut [Vec3],
    a: usize,
    b: usize,
    rest: f32,
    (wa, wb): (f32, f32),
    stiffness: f32,
    h: f32,
    lambda: &mut f32,
) -> Vec3 {
    if stiffness <= 0.0 || wa + wb == 0.0 {
        return Vec3::ZERO;
    }
    let d = points[b] - points[a];
    let len = d.length();
    if len <= 1e-9 {
        return Vec3::ZERO;
    }
    let alpha = 1.0 / (stiffness * h * h);
    let dl = (-(len - rest) - alpha * *lambda) / (wa + wb + alpha);
    *lambda += dl;
    let correction = d * (dl / len);
    points[a] -= correction * wa;
    points[b] += correction * wb;
    correction / h
}

pub(crate) fn masses(points: &[Vec3], triangles: &[[u32; 3]]) -> Vec<f32> {
    let mut mass = vec![0.0; points.len()];
    for &[a, b, c] in triangles {
        let area = (points[b as usize] - points[a as usize])
            .cross(points[c as usize] - points[a as usize])
            .length()
            / 6.0;
        for i in [a, b, c] {
            mass[i as usize] += area;
        }
    }
    mass.into_iter()
        .map(|m| if m > 0.0 { 1.0 / m } else { 0.0 })
        .collect()
}

/// Quadratic aerodynamic force resolved along drag and lift directions.
pub(crate) fn aero(relative: Vec3, normal: Vec3, drag: f32, lift: f32) -> Vec3 {
    let speed = relative.length();
    if speed <= 1e-9 {
        return Vec3::ZERO;
    }
    let direction = relative / speed;
    let projected = normal.dot(direction);
    relative * (drag.max(0.0) * speed * projected.abs())
        + (normal - direction * projected) * (lift.max(0.0) * speed * speed * projected)
}

pub(crate) fn gust(p: Vec3, tick: u64) -> f32 {
    let phase = p.x * 0.73 + p.y * 0.37 + p.z * 0.53 + tick as f32 / 24.0;
    og_core::math::sin(phase)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Bend {
    pub vertices: [u32; 4],
    rest: f32,
}

fn angle(p: [Vec3; 4]) -> f32 {
    let [a, b, c, d] = p;
    let e = b - a;
    let n1 = e.cross(c - a).normalize_or_zero();
    let n2 = e.cross(d - a).normalize_or_zero();
    og_core::math::atan2(e.normalize_or_zero().dot(n1.cross(n2)), n1.dot(n2))
}

pub(crate) fn bends(points: &[Vec3], triangles: &[[u32; 3]]) -> Vec<Bend> {
    let mut edges: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for &[a, b, c] in triangles {
        for (a, b, c) in [(a, b, c), (b, c, a), (c, a, b)] {
            edges.entry((a.min(b), a.max(b))).or_default().push(c);
        }
    }
    edges
        .into_iter()
        .filter_map(|((a, b), far)| {
            let [c, d] = far[..] else {
                return None;
            };
            let vertices = [a, b, c, d];
            Some(Bend {
                vertices,
                rest: angle(vertices.map(|i| points[i as usize])),
            })
        })
        .collect()
}

pub(crate) fn bend(
    points: &mut [Vec3],
    w: &[f32],
    bend: &Bend,
    stiffness: f32,
    h: f32,
    lambda: &mut f32,
) {
    if stiffness <= 0.0 {
        return;
    }
    let ids = bend.vertices.map(|i| i as usize);
    let [a, b, c, d] = ids.map(|i| points[i]);
    let e = b - a;
    let len2 = e.length_squared();
    let n1 = e.cross(c - a);
    let n2 = e.cross(d - a);
    if len2 < 1e-12 || n1.length_squared() < 1e-12 || n2.length_squared() < 1e-12 {
        return;
    }
    let qc = n1 * (-len2.sqrt() / n1.length_squared());
    let qd = n2 * (len2.sqrt() / n2.length_squared());
    let tc = (c - a).dot(e) / len2;
    let td = (d - a).dot(e) / len2;
    let gradients = [
        qc * (tc - 1.0) + qd * (td - 1.0),
        qc * (-tc) + qd * (-td),
        qc,
        qd,
    ];
    let mut error = angle([a, b, c, d]) - bend.rest;
    if error > std::f32::consts::PI {
        error -= std::f32::consts::TAU;
    }
    if error < -std::f32::consts::PI {
        error += std::f32::consts::TAU;
    }
    let alpha = 1.0 / (stiffness * h * h);
    let mut denom = alpha;
    for k in 0..4 {
        denom += w[ids[k]] * gradients[k].length_squared();
    }
    let dl = (-error - alpha * *lambda) / denom;
    *lambda += dl;
    for k in 0..4 {
        points[ids[k]] += gradients[k] * (w[ids[k]] * dl);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RopeState {
    pub segments: u32,
    pub points: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    pub inv_mass: Vec<f32>,
    broken: Vec<bool>,
}
impl RopeState {
    pub fn new(rope: &Rope, start: Vec3, end: Option<Vec3>) -> Self {
        let n = rope.segments.clamp(1, 256) as usize;
        let to = end.unwrap_or(start - Vec3::Y * rope.length.max(0.0));
        Self {
            segments: rope.segments,
            points: (0..=n)
                .map(|i| start + (to - start) * (i as f32 / n as f32))
                .collect(),
            velocities: vec![Vec3::ZERO; n + 1],
            inv_mass: vec![0.0; n + 1],
            broken: vec![false; n],
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        rope: &Rope,
        dt: f32,
        gravity: Vec3,
        entity: u64,
        start: Vec3,
        end: Option<Vec3>,
        flat: bool,
        solids: &dyn Solids,
    ) -> [Vec3; 2] {
        let mut reactions = [Vec3::ZERO; 2];
        if dt <= 0.0 || rope.substeps == 0 {
            return reactions;
        }
        let steps = rope.substeps.min(16);
        let h = dt / steps as f32;
        let last = self.points.len() - 1;
        self.inv_mass.fill(if rope.mass > 0.0 {
            1.0 / rope.mass
        } else {
            0.0
        });
        self.inv_mass[0] = if rope.coupling {
            solids.inverse_mass(entity)
        } else {
            0.0
        };
        if end.is_some() {
            self.inv_mass[last] = if rope.coupling {
                solids.inverse_mass(rope.end)
            } else {
                0.0
            };
        }
        let from = [self.points[0], self.points[last]];
        let mut displacement = [Vec3::ZERO; 2];
        let mut reactive_velocity = [Vec3::ZERO; 2];
        for s in 0..steps {
            let before = self.points.clone();
            let t = (s + 1) as f32 / steps as f32;
            displacement[0] += reactive_velocity[0] * h;
            self.points[0] = from[0] + (start - from[0]) * t + displacement[0];
            if let Some(end) = end {
                displacement[1] += reactive_velocity[1] * h;
                self.points[last] = from[1] + (end - from[1]) * t + displacement[1];
            }
            for (i, &w) in self.inv_mass.iter().enumerate() {
                if w == 0.0 || i == 0 || (i == last && end.is_some()) {
                    continue;
                }
                self.velocities[i] *= keep_per_substep(rope.damping, dt, steps);
                self.velocities[i] += (gravity * rope.gravity_scale
                    + rope.wind
                    + solids.acceleration(self.points[i])
                    + solids.air(self.points[i]))
                    * h;
                self.points[i] = planar(self.points[i] + self.velocities[i] * h, flat, start.z);
            }
            let mut lambda = vec![0.0; last];
            for _ in 0..rope.iterations.min(64) {
                for (i, l) in lambda.iter_mut().enumerate() {
                    if self.broken[i] {
                        continue;
                    }
                    let impulse = link(
                        &mut self.points,
                        i,
                        i + 1,
                        rope.length.max(0.0) / last as f32,
                        (self.inv_mass[i], self.inv_mass[i + 1]),
                        rope.stiffness,
                        h,
                        l,
                    );
                    if i == 0 {
                        reactions[0] -= impulse;
                        reactive_velocity[0] -= impulse * self.inv_mass[0];
                        displacement[0] -= impulse * (self.inv_mass[0] * h);
                    }
                    if i + 1 == last && end.is_some() {
                        reactions[1] += impulse;
                        reactive_velocity[1] += impulse * self.inv_mass[last];
                        displacement[1] += impulse * (self.inv_mass[last] * h);
                    }
                    if rope.break_force > 0.0 && l.abs() / (h * h) > rope.break_force {
                        self.broken[i] = true;
                    }
                }
            }
            for (i, old) in before.iter().enumerate() {
                self.velocities[i] = (self.points[i] - *old) / h;
                if self.inv_mass[i] == 0.0
                    || i == 0
                    || (i == last && end.is_some())
                    || rope.radius <= 0.0
                {
                    continue;
                }
                solids.collide_point(
                    &mut self.points[i],
                    &mut self.velocities[i],
                    rope.radius,
                    0.0,
                    if rope.coupling { rope.mass } else { 0.0 },
                    h,
                );
                self.points[i] = planar(self.points[i], flat, start.z);
            }
        }
        if rope.coupling {
            solids.reaction(entity, start, reactions[0], displacement[0]);
            if let Some(end) = end {
                solids.reaction(rope.end, end, reactions[1], displacement[1]);
            }
        }
        reactions
    }
}

pub(crate) fn pinned(pins: u32, x: u32, y: u32, cols: u32) -> bool {
    (pins & PIN_TOP != 0 && y == 0)
        || (pins & PIN_TOP_CORNERS != 0 && y == 0 && (x == 0 || x + 1 == cols))
        || (pins & PIN_LEFT != 0 && x == 0)
        || (pins & PIN_TOP_RINGS != 0 && y == 0 && (x.is_multiple_of(4) || x + 1 == cols))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ClothState {
    pub columns: u32,
    pub rows: u32,
    pub spacing: f32,
    pub pins: u32,
    pub points: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    pub inv_mass: Vec<f32>,
    triangles: Vec<[u32; 3]>,
    bends: Vec<Bend>,
    tethers: Vec<(usize, usize, f32)>,
}
impl ClothState {
    pub fn rest(cloth: &Cloth, origin: Vec3, rotation: Quat, x: u32, y: u32) -> Vec3 {
        origin + rotation * Vec3::new(x as f32 * cloth.spacing, -(y as f32) * cloth.spacing, 0.0)
    }
    pub fn new(cloth: &Cloth, origin: Vec3, rotation: Quat) -> Self {
        let (cols, rows) = (cloth.columns.clamp(2, 64), cloth.rows.clamp(2, 64));
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        for y in 0..rows {
            for x in 0..cols {
                points.push(Self::rest(cloth, origin, rotation, x, y));
                if x + 1 < cols && y + 1 < rows {
                    let a = y * cols + x;
                    triangles.extend([[a, a + cols, a + 1], [a + 1, a + cols, a + cols + 1]]);
                }
            }
        }
        let inv_mass = masses(&points, &triangles);
        let bends = bends(&points, &triangles);
        let mut tethers = Vec::new();
        let pins: Vec<usize> = (0..points.len())
            .filter(|&i| pinned(cloth.pins, i as u32 % cols, i as u32 / cols, cols))
            .collect();
        for i in 0..points.len() {
            if let Some(&anchor) = pins.iter().min_by(|&&a, &&b| {
                (points[a] - points[i])
                    .length_squared()
                    .total_cmp(&(points[b] - points[i]).length_squared())
            }) && anchor != i
            {
                tethers.push((anchor, i, (points[anchor] - points[i]).length()));
            }
        }
        Self {
            columns: cloth.columns,
            rows: cloth.rows,
            spacing: cloth.spacing,
            pins: cloth.pins,
            velocities: vec![Vec3::ZERO; points.len()],
            points,
            inv_mass,
            triangles,
            bends,
            tethers,
        }
    }
    pub fn dims(&self) -> (u32, u32) {
        (self.columns.clamp(2, 64), self.rows.clamp(2, 64))
    }
    pub fn hold(&mut self, cloth: &Cloth, origin: Vec3, rotation: Quat) {
        let (cols, rows) = self.dims();
        for y in 0..rows {
            for x in 0..cols {
                self.points[(y * cols + x) as usize] = Self::rest(cloth, origin, rotation, x, y);
            }
        }
        self.velocities.fill(Vec3::ZERO);
    }
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
        if dt <= 0.0 || cloth.substeps == 0 {
            return;
        }
        let (cols, _) = self.dims();
        let steps = cloth.substeps.min(16);
        let h = dt / steps as f32;
        let mut w = self.inv_mass.clone();
        for (i, wi) in w.iter_mut().enumerate() {
            *wi = if cloth.density <= 0.0
                || pinned(self.pins, i as u32 % cols, i as u32 / cols, cols)
            {
                0.0
            } else {
                *wi / cloth.density
            };
        }
        let mut edges = BTreeSet::new();
        for &[a, b, c] in &self.triangles {
            for (a, b) in [(a, b), (b, c), (c, a)] {
                edges.insert((a.min(b) as usize, a.max(b) as usize));
            }
        }
        let edges: Vec<_> = edges.into_iter().collect();
        for _ in 0..steps {
            let before = self.points.clone();
            let mut normals = vec![Vec3::ZERO; self.points.len()];
            for &[a, b, c] in &self.triangles {
                let n = (self.points[b as usize] - self.points[a as usize])
                    .cross(self.points[c as usize] - self.points[a as usize]);
                for i in [a, b, c] {
                    normals[i as usize] += n;
                }
            }
            for i in 0..self.points.len() {
                if pinned(self.pins, i as u32 % cols, i as u32 / cols, cols) {
                    self.points[i] =
                        Self::rest(cloth, origin, rotation, i as u32 % cols, i as u32 / cols);
                    continue;
                }
                if w[i] == 0.0 {
                    continue;
                }
                let p = self.points[i];
                let air = (cloth.wind + solids.air(p)) * (1.0 + cloth.gust * gust(p, tick));
                // In 2D use the in-plane perpendicular rather than dropping z normals.
                let normal = if flat {
                    let tangent = self.points[(i + 1).min(self.points.len() - 1)] - p;
                    Vec3::new(-tangent.y, tangent.x, 0.0).normalize_or_zero()
                } else {
                    normals[i].normalize_or_zero()
                };
                let acceleration = gravity * cloth.gravity_scale
                    + solids.acceleration(p)
                    + aero(air - self.velocities[i], normal, cloth.drag, cloth.lift)
                        / cloth.density;
                self.velocities[i] = self.velocities[i]
                    * keep_per_substep(cloth.damping, dt, steps)
                    + acceleration * h;
                self.points[i] = planar(p + self.velocities[i] * h, flat, origin.z);
            }
            let mut lambda = vec![0.0; edges.len()];
            let mut bending = vec![0.0; self.bends.len()];
            let mut tether = vec![0.0; self.tethers.len()];
            for _ in 0..cloth.iterations.min(64) {
                for (k, &(a, b)) in edges.iter().enumerate() {
                    let dx = (a as u32 % cols) as f32 - (b as u32 % cols) as f32;
                    let dy = (a as u32 / cols) as f32 - (b as u32 / cols) as f32;
                    link(
                        &mut self.points,
                        a,
                        b,
                        cloth.spacing * (dx * dx + dy * dy).sqrt(),
                        (w[a], w[b]),
                        cloth.stiffness,
                        h,
                        &mut lambda[k],
                    );
                }
                for (k, b) in self.bends.iter().enumerate() {
                    bend(&mut self.points, &w, b, cloth.bending, h, &mut bending[k]);
                }
                for (k, &(a, b, rest)) in self.tethers.iter().enumerate() {
                    let d = self.points[b] - self.points[a];
                    let len = d.length();
                    if cloth.tether > 0.0 && len > 1e-9 && w[a] + w[b] > 0.0 {
                        let alpha = 1.0 / (cloth.tether * h * h);
                        let dl = (-(len - rest) - alpha * tether[k]) / (w[a] + w[b] + alpha);
                        // An attachment limits extension, never pushes a
                        // folded sheet away from its pin.
                        let next = (tether[k] + dl).min(0.0);
                        let correction = d * ((next - tether[k]) / len);
                        tether[k] = next;
                        self.points[a] -= correction * w[a];
                        self.points[b] += correction * w[b];
                    }
                }
            }
            // Rebuild and project the self-contact set once per small step,
            // after elastic constraints, like the solid contact stage below.
            if cloth.thickness > 0.0 {
                self_collide(&mut self.points, &self.triangles, &w, cloth.thickness);
            }
            for i in 0..self.points.len() {
                self.velocities[i] = (self.points[i] - before[i]) / h;
                if w[i] > 0.0 && cloth.radius > 0.0 {
                    solids.collide_point(
                        &mut self.points[i],
                        &mut self.velocities[i],
                        cloth.radius,
                        cloth.friction.clamp(0.0, 1.0),
                        1.0 / w[i],
                        h,
                    );
                }
                self.points[i] = planar(self.points[i], flat, origin.z);
            }
        }
    }
}

/// Closest point on a triangle, with barycentric weights (Ericson 2005).
fn closest(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> (Vec3, [f32; 3]) {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, [1.0, 0.0, 0.0]);
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, [0.0, 1.0, 0.0]);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return (a + ab * v, [1.0 - v, v, 0.0]);
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, [0.0, 0.0, 1.0]);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let v = d2 / (d2 - d6);
        return (a + ac * v, [1.0 - v, 0.0, v]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let v = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (b + (c - b) * v, [0.0, 1.0 - v, v]);
    }
    let denom = va + vb + vc;
    if denom.abs() < 1e-20 {
        return (a, [1.0, 0.0, 0.0]);
    }
    let v = vb / denom;
    let z = vc / denom;
    (a + ab * v + ac * z, [1.0 - v - z, v, z])
}

/// Vertex/triangle and edge/edge contacts. Sorted spatial cells cull boxes;
/// projections distribute the correction over all four inverse masses.
pub(crate) fn self_collide(points: &mut [Vec3], triangles: &[[u32; 3]], w: &[f32], thickness: f32) {
    let cell = |p: Vec3| {
        p.to_array()
            .map(|x| (x / thickness).floor().clamp(-1e8, 1e8) as i32)
    };
    let mut vertices: BTreeMap<[i32; 3], Vec<usize>> = BTreeMap::new();
    for (i, &p) in points.iter().enumerate() {
        vertices.entry(cell(p)).or_default().push(i);
    }
    let mut candidates = BTreeSet::new();
    for (t, &ids) in triangles.iter().enumerate() {
        let [a, b, c] = ids.map(|i| points[i as usize]);
        let lo = cell(a.min(b).min(c) - Vec3::splat(thickness));
        let hi = cell(a.max(b).max(c) + Vec3::splat(thickness));
        for (key, list) in vertices.range([lo[0], i32::MIN, i32::MIN]..=[hi[0], i32::MAX, i32::MAX])
        {
            if key[1] < lo[1] || key[1] > hi[1] || key[2] < lo[2] || key[2] > hi[2] {
                continue;
            }
            for &i in list {
                if !ids.contains(&(i as u32)) {
                    candidates.insert((i, t));
                }
            }
        }
    }
    for (i, t) in candidates {
        let ids = triangles[t].map(|i| i as usize);
        let [a, b, c] = ids.map(|j| points[j]);
        let (q, bary) = closest(points[i], a, b, c);
        let d = points[i] - q;
        let len = d.length();
        if len >= thickness {
            continue;
        }
        let normal = if len > 1e-9 {
            d / len
        } else {
            (b - a).cross(c - a).normalize_or_zero()
        };
        let mut denom = w[i];
        for k in 0..3 {
            denom += w[ids[k]] * bary[k] * bary[k];
        }
        if denom <= 0.0 {
            continue;
        }
        let correction = normal * ((thickness - len) / denom);
        points[i] += correction * w[i];
        for k in 0..3 {
            points[ids[k]] -= correction * (w[ids[k]] * bary[k]);
        }
    }
    let mut edges = BTreeSet::new();
    for &[a, b, c] in triangles {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            edges.insert((a.min(b) as usize, a.max(b) as usize));
        }
    }
    let edges: Vec<_> = edges.into_iter().collect();
    // At least an edge long: every expanded edge box occupies at most
    // four cells per axis, even when an unconstrained sheet stretches.
    let size = edges.iter().fold(thickness, |size, &(a, b)| {
        size.max((points[a] - points[b]).length())
    });
    let cell = |p: Vec3| {
        p.to_array()
            .map(|x| (x / size).floor().clamp(-1e8, 1e8) as i32)
    };
    let mut grid: BTreeMap<[i32; 3], Vec<usize>> = BTreeMap::new();
    let mut pairs = BTreeSet::new();
    for (i, &(a, b)) in edges.iter().enumerate() {
        let lo = cell(points[a].min(points[b]) - Vec3::splat(thickness));
        let hi = cell(points[a].max(points[b]) + Vec3::splat(thickness));
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    let list = grid.entry([x, y, z]).or_default();
                    for &j in list.iter() {
                        let (c, d) = edges[j];
                        if a != c && a != d && b != c && b != d {
                            pairs.insert((j, i));
                        }
                    }
                    list.push(i);
                }
            }
        }
    }
    for (i, j) in pairs {
        let (a, b) = edges[i];
        let (c, d) = edges[j];
        let u = points[b] - points[a];
        let v = points[d] - points[c];
        let r = points[a] - points[c];
        let (uu, vv, uv) = (u.dot(u), v.dot(v), u.dot(v));
        if uu <= 1e-12 || vv <= 1e-12 {
            continue;
        }
        let denom = uu * vv - uv * uv;
        let mut s = if denom > 1e-12 {
            ((uv * v.dot(r) - u.dot(r) * vv) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let t = ((uv * s + v.dot(r)) / vv).clamp(0.0, 1.0);
        s = ((uv * t - u.dot(r)) / uu).clamp(0.0, 1.0);
        let delta = points[a] + u * s - points[c] - v * t;
        let len = delta.length();
        if len >= thickness {
            continue;
        }
        let normal = if len > 1e-9 {
            delta / len
        } else {
            let n = u.cross(v);
            if n.length_squared() > 1e-12 {
                n.normalize()
            } else {
                u.cross(if u.x.abs() < u.y.abs() {
                    Vec3::X
                } else {
                    Vec3::Y
                })
                .normalize_or_zero()
            }
        };
        let ids = [a, b, c, d];
        let weights = [1.0 - s, s, -(1.0 - t), -t];
        let mut denom = 0.0;
        for k in 0..4 {
            denom += w[ids[k]] * weights[k] * weights[k];
        }
        if denom <= 0.0 {
            continue;
        }
        for k in 0..4 {
            points[ids[k]] += normal * ((thickness - len) * w[ids[k]] * weights[k] / denom);
        }
    }
}
