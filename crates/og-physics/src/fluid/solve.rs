//! Position Based Fluids (Macklin & Müller, 2013). Every Jacobi pass reads
//! one immutable buffer; every particle gathers neighbours in index order.
//! Lane partitioning changes scheduling, never a floating-point reduction.

use super::pairs::Pairs;
use crate::xpbd::Solids;
use og_core::glam::{Vec2, Vec3};
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

pub(crate) const GROUP_SIZE: usize = 4;
pub(crate) const MIN_RADIUS: f32 = 0.01;

pub(crate) trait Point:
    Copy
    + Send
    + Sync
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign<f32>
    + Index<usize, Output = f32>
    + IndexMut<usize>
{
    const ZERO: Self;
    const AXES: usize;
    const FORWARD: usize;
    fn length_squared(self) -> f32;
    fn cell(self, inv: f32) -> [i32; 3];
    fn world(self, depth: f32) -> Vec3;
    fn of_world(p: Vec3) -> Self;
    fn grain() -> usize;
}

fn cell(x: f32, inv: f32) -> i32 {
    // Leave a cell on either side for neighbour offsets, even at huge coordinates.
    (x * inv).floor().clamp(-1_000_000_000.0, 1_000_000_000.0) as i32
}

impl Point for Vec2 {
    const ZERO: Self = Self::ZERO;
    const AXES: usize = 2;
    const FORWARD: usize = 5;
    fn length_squared(self) -> f32 {
        self.length_squared()
    }
    fn cell(self, inv: f32) -> [i32; 3] {
        [cell(self.x, inv), cell(self.y, inv), 0]
    }
    fn world(self, depth: f32) -> Vec3 {
        self.extend(depth)
    }
    fn of_world(p: Vec3) -> Self {
        p.truncate()
    }
    fn grain() -> usize {
        crate::lanes::now().particles_2d
    }
}

impl Point for Vec3 {
    const ZERO: Self = Self::ZERO;
    const AXES: usize = 3;
    const FORWARD: usize = 14;
    fn length_squared(self) -> f32 {
        self.length_squared()
    }
    fn cell(self, inv: f32) -> [i32; 3] {
        [cell(self.x, inv), cell(self.y, inv), cell(self.z, inv)]
    }
    fn world(self, _: f32) -> Vec3 {
        self
    }
    fn of_world(p: Vec3) -> Self {
        p
    }
    fn grain() -> usize {
        crate::lanes::now().particles_3d
    }
}

pub(crate) fn jitter(seed: u32, i: u32, k: u32) -> f32 {
    let mut h = seed
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add(i.wrapping_mul(0x9e37_79b1))
        .wrapping_add(k.wrapping_mul(0x85eb_ca77))
        .wrapping_add(0x3c6e_f372);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

pub(crate) fn own(slot: u32) -> [f32; GROUP_SIZE] {
    let mut w = [0.0; GROUP_SIZE];
    w[slot as usize] = 1.0;
    w
}
pub(crate) fn join(slot: &mut u32, mix: &mut Vec<[f32; GROUP_SIZE]>, n: usize, to: u32) {
    if *slot != to || mix.len() != n {
        *mix = vec![own(to); n];
        *slot = to;
    }
}
pub(crate) fn leave(slot: &mut u32, mix: &mut Vec<[f32; GROUP_SIZE]>) {
    *slot = u32::MAX;
    mix.clear();
}

pub(crate) struct Rule<V> {
    pub substeps: u32,
    pub solver_iterations: u32,
    pub radius: f32,
    pub gravity_scale: f32,
    pub viscosity: f32,
    pub stiffness: f32,
    pub near_stiffness: f32,
    pub vorticity: f32,
    pub rest_density: f32,
    pub density: f32,
    pub mixing: f32,
    pub bounds: V,
}
pub(crate) struct Body<'a, V> {
    pub rule: Rule<V>,
    pub positions: &'a mut Vec<V>,
    pub velocities: &'a mut Vec<V>,
    pub mix: &'a mut Vec<[f32; GROUP_SIZE]>,
    pub centre: V,
}

// Poly6 density and spiky gradient, as in PBF §3. Unnormalised kernels:
// restDensity is a particle count at unit material density, including self. The spiky
// gradient does not vanish as neighbours coincide. Its tie direction is
// antisymmetric, so even coincident particles exchange equal momentum.
fn kernel<V: Point>(d: V, h: f32, i: usize, j: usize) -> (f32, V) {
    let r = d.length_squared().sqrt();
    let q = (1.0 - r * r / (h * h)).max(0.0);
    let mut direction = V::ZERO;
    if r > h * 1e-6 {
        direction = d / r;
    } else {
        direction[(i.min(j) + i.max(j)) % V::AXES] = if i < j { 1.0 } else { -1.0 };
    }
    let near = (1.0 - r / h).max(0.0);
    (q * q * q, direction * (-3.0 * near * near / h))
}

pub(crate) fn step<V: Point>(
    bodies: &mut [Body<'_, V>],
    dt: f32,
    gravity: V,
    depth: f32,
    solids: &dyn Solids,
) {
    if dt <= 0.0 {
        return;
    }
    let substeps = bodies
        .iter()
        .map(|b| b.rule.substeps.clamp(1, 8))
        .max()
        .unwrap_or(1);
    let iterations = bodies
        .iter()
        .map(|b| b.rule.solver_iterations.min(64))
        .max()
        .unwrap_or(0);
    let ds = dt / substeps as f32;
    let h = bodies
        .iter()
        .map(|b| b.rule.radius.max(MIN_RADIUS))
        .fold(0.0f32, f32::max);
    let grouped = bodies.len() > 1;
    let mut owner = Vec::new();
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    let mut mix = Vec::new();
    for (k, b) in bodies.iter().enumerate() {
        owner.extend(std::iter::repeat_n(k, b.positions.len()));
        positions.extend_from_slice(b.positions);
        velocities.extend_from_slice(b.velocities);
        mix.extend(if grouped {
            b.mix.clone()
        } else {
            vec![own(k as u32); b.positions.len()]
        });
    }
    let n = positions.len();
    if n == 0 {
        return;
    }
    // Material interpolation follows mix, not the particle's original owner.
    let property = |w: &[f32; GROUP_SIZE], f: fn(&Rule<V>) -> f32| {
        let mut value = 0.0;
        for (k, b) in bodies.iter().enumerate() {
            value += w[k] * f(&b.rule);
        }
        value
    };
    let mut lambda = vec![0.0; n];
    let mut delta = vec![V::ZERO; n];
    for _ in 0..substeps {
        let mass: Vec<f32> = mix
            .iter()
            .map(|w| property(w, |r| r.density).max(0.05))
            .collect();
        let rest: Vec<f32> = mix
            .iter()
            .enumerate()
            // Material density scales BOTH mass and target density. Otherwise
            // a density-2 particle with restDensity 2 already fills its entire
            // target alone: every neighbour is expelled to the kernel edge,
            // eliminating the overlap needed for a liquid interface to mix.
            .map(|(i, w)| property(w, |r| r.rest_density) * mass[i])
            .collect();
        let previous = positions.clone();
        for i in 0..n {
            let at = positions[i].world(depth);
            let field = V::of_world(solids.acceleration(at) + solids.air(at));
            velocities[i] += (gravity * property(&mix[i], |r| r.gravity_scale) + field) * ds;
            positions[i] += velocities[i] * ds;
        }
        for iteration in 0..iterations {
            // Rebuild after prediction AND projection: no stale half-pairs.
            let neighbours = Pairs::find(&positions, h).neighbours(n);
            crate::lanes::for_each_chunk(&mut lambda, &mut delta, V::grain(), |at, ls, _| {
                for (k, l) in ls.iter_mut().enumerate() {
                    let i = at + k;
                    let strength = property(&mix[i], |r| r.stiffness).clamp(0.0, 1.0);
                    if rest[i] <= 0.0
                        || strength == 0.0
                        || iteration >= bodies[owner[i]].rule.solver_iterations
                    {
                        *l = 0.0;
                        continue;
                    }
                    let mut rho = mass[i];
                    let mut grad_i = V::ZERO;
                    let mut denom = 0.0;
                    for &j in &neighbours[i] {
                        let (w, g) = kernel(positions[i] - positions[j], h, i, j);
                        rho += mass[j] * w;
                        let g = g * (mass[j] / rest[i]);
                        grad_i += g;
                        denom += g.length_squared() / mass[j];
                    }
                    denom += grad_i.length_squared() / mass[i];
                    // A free liquid surface has missing neighbours, not
                    // negative pressure that pulls isolated drops together.
                    *l = -strength * (rho / rest[i] - 1.0).max(0.0) / (denom + 1e-6 / (h * h));
                }
            });
            let mut unused = vec![0.0; n];
            crate::lanes::for_each_chunk(&mut delta, &mut unused, V::grain(), |at, dx, _| {
                for (k, out) in dx.iter_mut().enumerate() {
                    let i = at + k;
                    *out = V::ZERO;
                    if rest[i] <= 0.0 {
                        continue;
                    }
                    for &j in &neighbours[i] {
                        if rest[j] <= 0.0 {
                            continue;
                        }
                        let (w, grad) = kernel(positions[i] - positions[j], h, i, j);
                        let ratio = w / (0.91 * 0.91 * 0.91);
                        let ratio2 = ratio * ratio;
                        let artificial = |p: usize| {
                            if iteration < bodies[owner[p]].rule.solver_iterations {
                                property(&mix[p], |r| r.near_stiffness)
                            } else {
                                0.0
                            }
                        };
                        let corr = -0.5 * h * h * (artificial(i) + artificial(j)) * ratio2 * ratio2;
                        let symmetric =
                            lambda[i] * mass[j] / rest[i] + lambda[j] * mass[i] / rest[j];
                        *out += grad * ((symmetric + corr) / mass[i]);
                    }
                }
            });
            for i in 0..n {
                positions[i] += delta[i];
            }
        }
        for i in 0..n {
            velocities[i] = (positions[i] - previous[i]) / ds;
        }
        let neighbours = Pairs::find(&positions, h).neighbours(n);
        // Symmetric, mass-weighted XSPH: one pair coefficient has a common
        // normalization on both ends. Each row is convex (stable at any
        // viscosity), and m_i dv_i + m_j dv_j = 0 even for mixed masses.
        let before = velocities.clone();
        let viscosity_weight: Vec<f32> = (0..n)
            .map(|i| {
                let mut sum = 0.0;
                for &j in &neighbours[i] {
                    let (w, _) = kernel(positions[i] - positions[j], h, i, j);
                    let s = 0.5
                        * (property(&mix[i], |r| r.viscosity) + property(&mix[j], |r| r.viscosity))
                            .max(0.0)
                        * ds;
                    sum += s * w * mass[j];
                }
                sum
            })
            .collect();
        let omega: Vec<Vec3> = (0..n)
            .map(|i| {
                let mut curl = Vec3::ZERO;
                for &j in &neighbours[i] {
                    let (_, g) = kernel(positions[i] - positions[j], h, i, j);
                    curl += (before[j] - before[i]).world(0.0).cross(g.world(0.0));
                }
                curl
            })
            .collect();
        let mut next_mix = mix.clone();
        crate::lanes::for_each_chunk(&mut velocities, &mut next_mix, V::grain(), |at, vs, ws| {
            for (k, v) in vs.iter_mut().enumerate() {
                let i = at + k;
                let mut sum = V::ZERO;
                let mut eta = Vec3::ZERO;
                let mut mixed = [0.0; GROUP_SIZE];
                let mut mix_weight = 0.0;
                for &j in &neighbours[i] {
                    let (w, grad) = kernel(positions[i] - positions[j], h, i, j);
                    let s = 0.5
                        * (property(&mix[i], |r| r.viscosity) + property(&mix[j], |r| r.viscosity))
                            .max(0.0)
                        * ds;
                    sum += (before[j] - before[i])
                        * (s * w * mass[j] / (1.0 + viscosity_weight[i].max(viscosity_weight[j])));
                    eta += grad.world(0.0) * omega[j].length();
                    let rate = 0.5
                        * (property(&mix[i], |r| r.mixing) + property(&mix[j], |r| r.mixing))
                            .clamp(0.0, 1.0)
                        * ds
                        * w;
                    mix_weight += rate;
                    for (l, m) in mixed.iter_mut().enumerate() {
                        *m += rate * mix[j][l];
                    }
                }
                *v = before[i] + sum;
                let len = eta.length();
                if len > 1e-9 {
                    *v += V::of_world((eta / len).cross(omega[i]))
                        * (ds * property(&mix[i], |r| r.vorticity).max(0.0));
                }
                for (l, m) in ws[k].iter_mut().enumerate() {
                    *m = (mix[i][l] + mixed[l]) / (1.0 + mix_weight);
                }
            }
        });
        mix = next_mix;
        for i in 0..n {
            let b = &bodies[owner[i]];
            let radius = b.rule.radius.max(MIN_RADIUS) * 0.25;
            let mut p = positions[i].world(depth);
            let mut v = velocities[i].world(0.0);
            solids.collide_point(&mut p, &mut v, radius, 0.04, mass[i], ds);
            positions[i] = V::of_world(p);
            velocities[i] = V::of_world(v);
            let half = b.rule.bounds * 0.5;
            for axis in 0..V::AXES {
                if half[axis] <= 0.0 {
                    continue;
                }
                let lo = b.centre[axis] - half[axis];
                let hi = b.centre[axis] + half[axis];
                if positions[i][axis] < lo {
                    positions[i][axis] = lo;
                    velocities[i][axis] = velocities[i][axis].max(0.0);
                }
                if positions[i][axis] > hi {
                    positions[i][axis] = hi;
                    velocities[i][axis] = velocities[i][axis].min(0.0);
                }
            }
        }
    }
    let mut at = 0;
    for b in bodies.iter_mut() {
        let end = at + b.positions.len();
        b.positions.copy_from_slice(&positions[at..end]);
        b.velocities.copy_from_slice(&velocities[at..end]);
        if grouped {
            b.mix.copy_from_slice(&mix[at..end]);
        }
        at = end;
    }
}
