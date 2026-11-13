//! The particle fluid's solver, in two or three dimensions: double density
//! relaxation (Clavet, Beaudoin and Poulin, "Particle-based viscoelastic
//! fluid simulation", SCA 2005). Each substep applies gravity and the
//! viscosity impulses, moves the particles, relaxes them towards the rest
//! density (pressure, and near pressure, which keeps them apart and gives
//! the surface its tension), takes the velocity from the displacement, and
//! pushes them out of solids, the terrain and the container.
//!
//! Why this and not PBF or WCSPH: the relaxation is position-based, so it
//! is stable at the 60 Hz tick with one or two substeps (velocity capped
//! at half the radius per substep, nothing tunnels), and costs two
//! neighbour passes a substep (density, then relaxation) besides the
//! viscosity. Position-based fluids need three or four constraint
//! iterations, each a neighbour pass, for the same incompressibility;
//! weakly compressible SPH needs a timestep a few times shorter than the
//! tick at these stiffnesses. Its surface tension comes free with the near
//! pressure.
//!
//! Fluids of one group (`group` on the component) step together: their
//! particles weigh their fluid's `density` and carry a mix over the group's
//! fluids that `mixing` evens out where they touch. A lone fluid is a group
//! of one (every particle the same mass, no mix).
//!
//! What a dimension chooses is [`Point`]: its vector, how many cells its
//! neighbour search visits, and how strongly unmixed fluids hold apart.
//! Everything else is this file for both.
//!
//! Determinism: the hash ([`Pairs`]) maps each cell of size `radius` to a
//! bucket; the buckets are filled by a counting sort in index order, a
//! particle's neighbouring cells visit their buckets once each in a fixed
//! order, and the pairs are grouped by their lower particle in the order
//! found, so every sum is taken in the same order on every host.
//! `+ - * /` and `sqrt` only. The particles are part of the og.physics
//! state block.

use super::pairs::Pairs;
use crate::verlet::Solids;
use og_core::glam::{Vec2, Vec3};
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

/// The time unit the stiffness numbers are tuned in: one 60 Hz tick.
const TICK: f32 = 1.0 / 60.0;

/// The most fluids one group simulates together (the lanes of a
/// particle's mix, and of the renderer's field).
pub(crate) const GROUP_SIZE: usize = 4;

/// The smallest radius a fluid simulates at (a smaller `radius` is this).
pub(crate) const MIN_RADIUS: f32 = 0.01;

/// How fast mixing evens out the mix of two touching particles, per tick
/// at `mixing` 1 and full overlap.
const MIX_RATE: f32 = 0.25;

/// How much of a particle's sliding velocity a solid it touches takes, per
/// tick (so a pool sloshing in a bowl comes to rest).
const WALL_FRICTION: f32 = 0.04;

/// A particle's position or velocity, and what its dimension chooses for
/// the solver.
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
    /// The vector's components.
    const AXES: usize;
    /// How many of the neighbour search's cells a particle visits: its own
    /// and the half of those around it that come after it.
    const FORWARD: usize;
    /// How strongly the boundary between unmixed fluids holds them apart
    /// (a multiple of the near pressure) at `mixing` 0.
    const TENSION: f32;

    fn dot(self, other: Self) -> f32;
    fn length(self) -> f32;
    fn length_squared(self) -> f32;
    /// The neighbour search's cell of this position, `inv` cells a unit.
    fn cell(self, inv: f32) -> [i32; 3];
    /// The position in the world, a 2D one in the plane at `depth`.
    fn world(self, depth: f32) -> Vec3;
    /// A world position or direction in this dimension.
    fn of_world(p: Vec3) -> Self;
    /// The particles one lane task takes in a per-particle pass.
    fn grain() -> usize;
}

impl Point for Vec2 {
    const ZERO: Vec2 = Vec2::ZERO;
    const AXES: usize = 2;
    const FORWARD: usize = 5;
    const TENSION: f32 = 1.0;

    fn dot(self, other: Vec2) -> f32 {
        Vec2::dot(self, other)
    }
    fn length(self) -> f32 {
        Vec2::length(self)
    }
    fn length_squared(self) -> f32 {
        Vec2::length_squared(self)
    }
    fn cell(self, inv: f32) -> [i32; 3] {
        [
            (self.x * inv).floor() as i32,
            (self.y * inv).floor() as i32,
            0,
        ]
    }
    fn world(self, depth: f32) -> Vec3 {
        Vec3::new(self.x, self.y, depth)
    }
    fn of_world(p: Vec3) -> Vec2 {
        Vec2::new(p.x, p.y)
    }
    fn grain() -> usize {
        crate::lanes::now().particles_2d
    }
}

impl Point for Vec3 {
    const ZERO: Vec3 = Vec3::ZERO;
    const AXES: usize = 3;
    const FORWARD: usize = 14;
    /// A tenth of the 2D value: a particle has about four times the
    /// neighbours in 3D, and at 2D's strength the boundary traps a light
    /// layer under a heavy one, the heavy fluid never sinking through.
    const TENSION: f32 = 0.1;

    fn dot(self, other: Vec3) -> f32 {
        Vec3::dot(self, other)
    }
    fn length(self) -> f32 {
        Vec3::length(self)
    }
    fn length_squared(self) -> f32 {
        Vec3::length_squared(self)
    }
    fn cell(self, inv: f32) -> [i32; 3] {
        [
            (self.x * inv).floor() as i32,
            (self.y * inv).floor() as i32,
            (self.z * inv).floor() as i32,
        ]
    }
    fn world(self, _: f32) -> Vec3 {
        self
    }
    fn of_world(p: Vec3) -> Vec3 {
        p
    }
    fn grain() -> usize {
        crate::lanes::now().particles_3d
    }
}

/// A hash of (seed, i, k) to 0..1: the spawn jitter.
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

/// The mix of a particle that is all its own fluid (lane `slot`).
pub(crate) fn own(slot: u32) -> [f32; GROUP_SIZE] {
    let mut own = [0.0f32; GROUP_SIZE];
    own[slot as usize] = 1.0;
    own
}

/// Joins a group at lane `to`: every one of the fluid's `particles` gets a
/// mix of only its own fluid, when it has none or was in another lane.
pub(crate) fn join(slot: &mut u32, mix: &mut Vec<[f32; GROUP_SIZE]>, particles: usize, to: u32) {
    if *slot != to || mix.len() != particles {
        *mix = vec![own(to); particles];
        *slot = to;
    }
}

/// Leaves a group: the particles keep no mix.
pub(crate) fn leave(slot: &mut u32, mix: &mut Vec<[f32; GROUP_SIZE]>) {
    *slot = u32::MAX;
    mix.clear();
}

/// The numbers of a fluid's component the solver reads.
pub(crate) struct Rule<V> {
    pub substeps: u32,
    pub radius: f32,
    pub gravity_scale: f32,
    pub viscosity: f32,
    pub stiffness: f32,
    pub near_stiffness: f32,
    pub rest_density: f32,
    pub density: f32,
    pub mixing: f32,
    /// The container's size about the fluid's centre (an axis of 0: open).
    pub bounds: V,
}

/// One fluid of a step: its rule, its particles, its lane in its group
/// (`u32::MAX`: in none, and `mix` empty) and the centre of its container.
pub(crate) struct Body<'a, V> {
    pub rule: Rule<V>,
    pub positions: &'a mut Vec<V>,
    pub velocities: &'a mut Vec<V>,
    pub mix: &'a mut Vec<[f32; GROUP_SIZE]>,
    pub slot: u32,
    pub centre: V,
}

/// One tick of a fluid alone or of the fluids of one group (in entity
/// order, their lanes `0..4`, each joined to its lane first): their
/// particles push, drag and (by `mixing`) mix as one fluid whose particles
/// weigh their fluids' `density`, so a heavier fluid sinks under a lighter
/// one. Pressure comes from every neighbour (number density), each
/// particle's own stiffness against its own rest density; a pair's
/// displacement and viscous impulse are shared in inverse proportion to
/// the particles' masses; unmixed particles of different fluids are held
/// apart at their boundary by an extra near pressure ([`Point::TENSION`],
/// gone at `mixing` 1); and touching particles even out their mix at
/// `mixing × MIX_RATE` per tick, their mass following it. `depth` is the
/// plane a 2D fluid meets the solids in.
pub(crate) fn step<V: Point>(
    bodies: &mut [Body<'_, V>],
    dt: f32,
    gravity: V,
    depth: f32,
    solids: &dyn Solids,
) {
    let grouped = bodies.len() > 1;
    let substeps = bodies
        .iter()
        .map(|b| b.rule.substeps.clamp(1, 8))
        .max()
        .unwrap_or(1);
    let ds = dt / substeps as f32;
    // Displacements in Clavet's units: time in ticks, so the numbers mean
    // the same at any substep count.
    let t = ds / TICK;
    let h = bodies
        .iter()
        .map(|b| b.rule.radius.max(MIN_RADIUS))
        .fold(0.0f32, f32::max);
    // The particles of every body, in body order.
    let mut owner: Vec<u8> = Vec::new();
    let mut positions: Vec<V> = Vec::new();
    let mut velocities: Vec<V> = Vec::new();
    let mut mix: Vec<[f32; GROUP_SIZE]> = Vec::new();
    for (k, b) in bodies.iter().enumerate() {
        owner.extend(std::iter::repeat_n(k as u8, b.positions.len()));
        positions.extend_from_slice(b.positions);
        velocities.extend_from_slice(b.velocities);
        if grouped {
            mix.extend_from_slice(b.mix);
        }
    }
    let n = positions.len();
    if n == 0 {
        return;
    }
    let mut lane_density = [1.0f32; GROUP_SIZE];
    if grouped {
        for b in bodies.iter() {
            lane_density[b.slot as usize] = b.rule.density.max(0.05);
        }
    }
    let mass_of = |w: &[f32; GROUP_SIZE]| -> f32 {
        let mut m = 0.0;
        for l in 0..GROUP_SIZE {
            m += w[l] * lane_density[l];
        }
        m.max(0.05)
    };
    let rule_of = |i: usize| &bodies[owner[i] as usize].rule;
    let mixing_of = |i: usize| rule_of(i).mixing.clamp(0.0, 1.0);
    let mut mass: Vec<f32> = if grouped {
        mix.iter().map(mass_of).collect()
    } else {
        vec![1.0; n]
    };
    let sigma: Vec<f32> = (0..n).map(|i| rule_of(i).viscosity.max(0.0)).collect();
    let mut previous = vec![V::ZERO; n];
    let mut rho = vec![0.0f32; n];
    let mut near = vec![0.0f32; n];
    let cap = 0.5 * h / ds;
    for _ in 0..substeps {
        for (i, v) in velocities.iter_mut().enumerate() {
            *v += gravity * (rule_of(i).gravity_scale * ds);
        }
        let pairs = Pairs::find(&positions, h);
        // Viscosity: radial impulses between approaching pairs, shared by
        // mass.
        for i in 0..n {
            let (pi, si) = (positions[i], sigma[i]);
            for &j in pairs.of(i) {
                let j = j as usize;
                let s = 0.5 * (si + sigma[j]);
                let d = positions[j] - pi;
                let r2 = d.length_squared();
                if s <= 0.0 || r2 <= 1e-18 {
                    continue;
                }
                let r = r2.sqrt();
                let dir = d / r;
                let u = (velocities[i] - velocities[j]).dot(dir);
                if u > 0.0 {
                    let impulse = dir * (t * (1.0 - r / h) * s * u);
                    let share = mass[j] / (mass[i] + mass[j]);
                    velocities[i] -= impulse * share;
                    velocities[j] += impulse * (1.0 - share);
                }
            }
        }
        previous.copy_from_slice(&positions);
        for (p, v) in positions.iter_mut().zip(velocities.iter_mut()) {
            let len = v.length();
            if len > cap {
                *v *= cap / len;
            }
            *p += *v * ds;
        }
        // Number density over every neighbour. Each particle's sums could
        // be gathered on the lanes bit for bit (its pairs with the
        // particles before it, then its own: the order this scatter visits
        // them), but the pairs' transpose that needs costs what the split
        // saves: 8,000 particles in 3D on four lanes 8.37 against 8.45 ms a
        // tick, 2,000 1.56 against 1.84. So it stays this scatter.
        rho.iter_mut().for_each(|r| *r = 0.0);
        near.iter_mut().for_each(|r| *r = 0.0);
        for i in 0..n {
            for &j in pairs.of(i) {
                let r = (positions[j as usize] - positions[i]).length();
                let q = 1.0 - r / h;
                if q > 0.0 {
                    let (q2, q3) = (q * q, q * q * q);
                    rho[i] += q2;
                    rho[j as usize] += q2;
                    near[i] += q3;
                    near[j as usize] += q3;
                }
            }
        }
        // Relaxation, Gauss-Seidel in index order over the pairs found
        // before the move: each pair moves both particles at once and the
        // next pair reads where they now are. Particle i's pairs read what
        // particle i - 1's wrote whenever the two are neighbours, as
        // lattice, emission and pouring order make them nearly always, so
        // the chain that must run in this order is as long as the fluid:
        // only another order (Jacobi, or two colours) splits it, and that
        // moves every particle's bits and the world hash. Viscosity and
        // mixing are passes of the same kind, over velocities and mixes,
        // and stay serial too.
        for i in 0..n {
            let f = rule_of(i);
            let pressure = f.stiffness * (rho[i] - f.rest_density);
            let near_pressure = f.near_stiffness * near[i];
            let mut dx = V::ZERO;
            for &j in pairs.of(i) {
                let j = j as usize;
                let d = positions[j] - positions[i];
                let r = d.length();
                if r <= 1e-9 {
                    continue;
                }
                let q = 1.0 - r / h;
                if q <= 0.0 {
                    continue;
                }
                let mut push = pressure * q + near_pressure * q * q;
                if grouped {
                    // How unlike the two particles are (0: the same mix),
                    // and how much of that the boundary holds.
                    let same: f32 = mix[i].iter().zip(&mix[j]).map(|(a, b)| a * b).sum();
                    let unlike = (1.0 - same).clamp(0.0, 1.0);
                    let hold = unlike * (1.0 - 0.5 * (mixing_of(i) + mixing_of(j))) * V::TENSION;
                    push += f.near_stiffness * hold * q;
                }
                let disp = (d / r) * (t * t * h * push);
                let total = mass[i] + mass[j];
                positions[j] += disp * (mass[i] / total);
                dx -= disp * (mass[j] / total);
            }
            positions[i] += dx;
        }
        if grouped {
            // Mixing: touching particles even out their mix, then their
            // mass.
            for i in 0..n {
                for &j in pairs.of(i) {
                    let j = j as usize;
                    let rate = 0.5 * (mixing_of(i) + mixing_of(j)) * MIX_RATE * t;
                    if rate <= 0.0 {
                        continue;
                    }
                    let r = (positions[j] - positions[i]).length();
                    let q = 1.0 - r / h;
                    if q <= 0.0 {
                        continue;
                    }
                    let k = (rate * q).min(0.5);
                    // j > i: both halves of the pair, mutably.
                    let (low, high) = mix.split_at_mut(j);
                    for (a, b) in low[i].iter_mut().zip(high[0].iter_mut()) {
                        let delta = (*b - *a) * k;
                        *a += delta;
                        *b -= delta;
                    }
                }
            }
            for (m, w) in mass.iter_mut().zip(&mix) {
                *m = mass_of(w);
            }
        }
        // The velocity from the displacement, capped at half the radius
        // per substep (stable, and nothing tunnels a wall).
        for ((v, p), q) in velocities.iter_mut().zip(&positions).zip(&previous) {
            let u = (*p - *q) / ds;
            let len = u.length();
            *v = if len > cap { u * (cap / len) } else { u };
        }
        // Solids, the terrain and each body's container: pushed out, the
        // velocity into them dropped (pushing out adds none, so a resting
        // particle does not bounce) and sliding slowed. Each particle
        // alone, so by ranges on the lanes from two grains.
        let shared: &[Body<'_, V>] = bodies;
        let owner = &owner;
        crate::lanes::for_each_chunk(&mut positions, &mut velocities, V::grain(), |at, ps, vs| {
            for (k, (p, v)) in ps.iter_mut().zip(vs.iter_mut()).enumerate() {
                let b = &shared[owner[at + k] as usize];
                let r_particle = b.rule.radius.max(MIN_RADIUS) * 0.25;
                if let Some((q, normal)) = solids.push_out(p.world(depth), r_particle) {
                    *p = V::of_world(q);
                    let normal = V::of_world(normal);
                    let along = v.dot(normal);
                    let slide = *v - normal * along;
                    *v = normal * along.max(0.0) + slide * (1.0 - WALL_FRICTION * t);
                }
                let half = b.rule.bounds * 0.5;
                for k in 0..V::AXES {
                    if half[k] > 0.0 {
                        let (lo, hi) = (b.centre[k] - half[k], b.centre[k] + half[k]);
                        if p[k] < lo {
                            p[k] = lo;
                            v[k] = v[k].max(0.0);
                        } else if p[k] > hi {
                            p[k] = hi;
                            v[k] = v[k].min(0.0);
                        }
                    }
                }
            }
        });
    }
    // Back into each body.
    let mut at = 0;
    for b in bodies.iter_mut() {
        let count = b.positions.len();
        b.positions.copy_from_slice(&positions[at..at + count]);
        b.velocities.copy_from_slice(&velocities[at..at + count]);
        if grouped {
            b.mix.copy_from_slice(&mix[at..at + count]);
        }
        at += count;
    }
}
