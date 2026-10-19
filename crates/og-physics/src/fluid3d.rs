//! The 3D particle fluid: double density relaxation (Clavet, Beaudoin and
//! Poulin, "Particle-based viscoelastic fluid simulation", SCA 2005) in
//! three dimensions, the 2D fluid's method (`fluid.rs`) over a spatial
//! hash. Each substep applies gravity and the viscosity impulses, moves the
//! particles, relaxes them towards the rest density (pressure, and near
//! pressure, which keeps them apart and gives the surface its tension),
//! pushes them out of solids, the terrain and the container, and takes the
//! velocity from the displacement. Each tick first runs the emitter (new
//! particles at `emitRate`, up to `maxCount`) and ends with the sink.
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
//! Fluids of one group (`Fluid3D.group`) step together, as in 2D: their
//! particles weigh their fluid's `density` and carry a mix over the group's
//! fluids that `mixing` evens out where they touch. A lone fluid is a group
//! of one (every particle the same mass, no mix).
//!
//! Determinism: the hash maps each cell of size `radius` to a bucket; the
//! buckets are filled by a counting sort in index order, a particle's 27
//! neighbouring cells visit their buckets in ascending bucket order once
//! each, and its neighbours are sorted, so every sum is taken in the same
//! order on every host. `+ - * /` and `sqrt` only. The particles, the
//! emitter's count and its carry are part of the og.physics state block.

use crate::fluid::GROUP_SIZE;
use crate::schema_ext::Fluid3D;
use crate::verlet::Solids;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// The time unit the stiffness numbers are tuned in: one 60 Hz tick.
const TICK: f32 = 1.0 / 60.0;

/// The most particles one fluid holds.
pub(crate) const MAX_PARTICLES: u32 = 32768;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Fluid3State {
    pub count: u32,
    pub seed: u32,
    pub positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    /// The fluid's lane in its group (`u32::MAX`: in none).
    pub slot: u32,
    /// In a group, each particle's mix over the group's lanes (summing to
    /// 1; its own lane alone until it mixes); empty in none.
    pub mix: Vec<[f32; GROUP_SIZE]>,
    /// Particles the emitter has made (the jitter's index).
    pub emitted: u32,
    /// The emitter's fraction of a particle carried to the next tick.
    pub carry: f32,
}

/// A hash of (seed, i, k) to 0..1: the spawn jitter.
fn jitter(seed: u32, i: u32, k: u32) -> f32 {
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

fn cell_of(p: Vec3, inv: f32) -> [i32; 3] {
    [
        (p.x * inv).floor() as i32,
        (p.y * inv).floor() as i32,
        (p.z * inv).floor() as i32,
    ]
}

fn bucket_of(c: [i32; 3], mask: u32) -> u32 {
    let h = (c[0] as u32).wrapping_mul(0x8da6_b343)
        ^ (c[1] as u32).wrapping_mul(0xd816_3841)
        ^ (c[2] as u32).wrapping_mul(0xcb1a_b31f);
    (h ^ (h >> 16)) & mask
}

/// The cell itself and the 13 of the 26 around it that come after it
/// (z, then y, then x first).
const FORWARD: [[i32; 3]; 14] = [
    [0, 0, 0],
    [1, 0, 0],
    [-1, 1, 0],
    [0, 1, 0],
    [1, 1, 0],
    [-1, -1, 1],
    [0, -1, 1],
    [1, -1, 1],
    [-1, 0, 1],
    [0, 0, 1],
    [1, 0, 1],
    [-1, 1, 1],
    [0, 1, 1],
    [1, 1, 1],
];

/// The neighbour pairs of one substep: for each particle `i`, the
/// particles `j > i` closer than the radius, in a fixed order
/// (`start[i]..start[i + 1]` into `pairs`).
pub(crate) struct Pairs {
    pub start: Vec<u32>,
    pub pairs: Vec<u32>,
}

impl Pairs {
    /// Builds the spatial hash (a bucket per cell of size `h`, at least
    /// twice as many buckets as particles) and finds the pairs: each
    /// particle against its own cell and the 13 cells after it (half the
    /// 26 around, so each pair is met once), the bucket's other cells
    /// (hash collisions) told apart by their coordinates.
    pub fn find(positions: &[Vec3], h: f32) -> Pairs {
        let n = positions.len();
        let inv = 1.0 / h;
        let table = (2 * n).max(16).next_power_of_two();
        let mask = table as u32 - 1;
        let cells: Vec<[i32; 3]> = positions.iter().map(|p| cell_of(*p, inv)).collect();
        // Counting sort: each bucket's particles in index order, their
        // cells and positions packed alongside.
        let mut first = vec![0u32; table + 1];
        for c in &cells {
            first[bucket_of(*c, mask) as usize + 1] += 1;
        }
        for k in 0..table {
            first[k + 1] += first[k];
        }
        let mut fill = first.clone();
        let mut sorted = vec![0u32; n];
        let mut packed = vec![(Vec3::ZERO, [0i32; 3]); n];
        for (i, c) in cells.iter().enumerate() {
            let b = bucket_of(*c, mask) as usize;
            sorted[fill[b] as usize] = i as u32;
            packed[fill[b] as usize] = (positions[i], *c);
            fill[b] += 1;
        }
        let h2 = h * h;
        // Each pair once, as (lower, higher); then grouped by the lower.
        let mut found: Vec<(u32, u32)> = Vec::with_capacity(n * 24);
        for (i, p) in positions.iter().enumerate() {
            let c = cells[i];
            for &[dx, dy, dz] in &FORWARD {
                let cell = [c[0] + dx, c[1] + dy, c[2] + dz];
                let b = bucket_of(cell, mask) as usize;
                let range = first[b] as usize..first[b + 1] as usize;
                let own = dx == 0 && dy == 0 && dz == 0;
                for (&j, (q, cj)) in sorted[range.clone()].iter().zip(&packed[range]) {
                    if *cj != cell || (own && j as usize <= i) {
                        continue;
                    }
                    if (*q - *p).length_squared() < h2 {
                        let i = i as u32;
                        found.push(if i < j { (i, j) } else { (j, i) });
                    }
                }
            }
        }
        let mut start = vec![0u32; n + 1];
        for &(i, _) in &found {
            start[i as usize + 1] += 1;
        }
        for k in 0..n {
            start[k + 1] += start[k];
        }
        let mut fill = start.clone();
        let mut pairs = vec![0u32; found.len()];
        for &(i, j) in &found {
            pairs[fill[i as usize] as usize] = j;
            fill[i as usize] += 1;
        }
        Pairs { start, pairs }
    }

    fn of(&self, i: usize) -> &[u32] {
        &self.pairs[self.start[i] as usize..self.start[i + 1] as usize]
    }
}

impl Fluid3State {
    /// `count` particles on a jittered grid filling the spawn box from the
    /// bottom up.
    pub fn new(fluid: &Fluid3D, centre: Vec3) -> Fluid3State {
        let n = fluid
            .count
            .min(fluid.max_count.clamp(1, MAX_PARTICLES))
            .min(MAX_PARTICLES);
        let size = fluid.size.max(Vec3::splat(0.01));
        let mut positions = Vec::with_capacity(n as usize);
        if n > 0 {
            // A cubic lattice of the box's volume per particle.
            let cell = og_core::math::pow(size.x * size.y * size.z / n as f32, 1.0 / 3.0);
            let cols = ((size.x / cell).round() as u32).max(1);
            let deep = ((size.z / cell).round() as u32).max(1);
            let layer = cols * deep;
            let rows = n.div_ceil(layer).max(1);
            let step = Vec3::new(
                size.x / cols as f32,
                size.y / rows as f32,
                size.z / deep as f32,
            );
            let origin = centre - size * 0.5 + step * 0.5;
            for i in 0..n {
                let (x, z, y) = (i % cols, (i / cols) % deep, i / layer);
                let j = Vec3::new(
                    jitter(fluid.seed, i, 0) - 0.5,
                    jitter(fluid.seed, i, 1) - 0.5,
                    jitter(fluid.seed, i, 2) - 0.5,
                ) * step
                    * 0.3;
                positions.push(
                    origin + Vec3::new(x as f32 * step.x, y as f32 * step.y, z as f32 * step.z) + j,
                );
            }
        }
        Fluid3State {
            count: fluid.count,
            seed: fluid.seed,
            velocities: vec![Vec3::ZERO; positions.len()],
            positions,
            slot: u32::MAX,
            mix: Vec::new(),
            emitted: 0,
            carry: 0.0,
        }
    }

    /// Gives every particle a mix of only its own fluid (lane `slot`), when
    /// it has none or was in another lane.
    pub fn join(&mut self, slot: u32) {
        if self.slot != slot || self.mix.len() != self.positions.len() {
            let mut own = [0.0f32; GROUP_SIZE];
            own[slot as usize] = 1.0;
            self.mix = vec![own; self.positions.len()];
            self.slot = slot;
        }
    }

    /// Leaves a group: the particles keep no mix.
    pub fn leave(&mut self) {
        self.slot = u32::MAX;
        self.mix.clear();
    }

    /// The emitter's particles for a tick of `dt`: `emitRate · dt` (the
    /// fraction carried), each at a jittered point of the emitter's cube
    /// with the emitter's velocity, while fewer than `maxCount` live.
    pub fn emit(&mut self, fluid: &Fluid3D, centre: Vec3, dt: f32) {
        if fluid.emit_rate.is_nan() || fluid.emit_rate <= 0.0 {
            self.carry = 0.0;
            return;
        }
        self.carry += fluid.emit_rate.min(20000.0) * dt;
        let whole = self.carry.floor();
        self.carry -= whole;
        let cap = fluid.max_count.clamp(1, MAX_PARTICLES) as usize;
        let room = cap.saturating_sub(self.positions.len());
        let new = (whole as usize).min(room);
        let at = centre + fluid.emit_offset;
        let r = fluid.emit_radius.max(0.0);
        let velocity = if fluid.emit_velocity.is_finite() {
            fluid.emit_velocity
        } else {
            Vec3::ZERO
        };
        for _ in 0..new {
            let i = self.emitted;
            let k = Vec3::new(
                jitter(fluid.seed ^ 0x5bd1_e995, i, 0) - 0.5,
                jitter(fluid.seed ^ 0x5bd1_e995, i, 1) - 0.5,
                jitter(fluid.seed ^ 0x5bd1_e995, i, 2) - 0.5,
            ) * (2.0 * r);
            self.positions.push(at + k);
            self.velocities.push(velocity);
            if self.slot != u32::MAX {
                let mut own = [0.0f32; GROUP_SIZE];
                own[self.slot as usize] = 1.0;
                self.mix.push(own);
            }
            self.emitted = self.emitted.wrapping_add(1);
        }
    }

    /// Removes the particles inside the sink box, keeping the others in
    /// order.
    pub fn sink(&mut self, fluid: &Fluid3D, centre: Vec3) {
        let half = fluid.sink_size * 0.5;
        if !(half.x > 0.0 && half.y > 0.0 && half.z > 0.0) {
            return;
        }
        let at = centre + fluid.sink_offset;
        let (lo, hi) = (at - half, at + half);
        let inside = |p: &Vec3| p.cmpge(lo).all() && p.cmple(hi).all();
        if !self.positions.iter().any(inside) {
            return;
        }
        let mixed = self.mix.len() == self.positions.len();
        let mut keep = 0;
        for i in 0..self.positions.len() {
            if inside(&self.positions[i]) {
                continue;
            }
            self.positions[keep] = self.positions[i];
            self.velocities[keep] = self.velocities[i];
            if mixed {
                self.mix[keep] = self.mix[i];
            }
            keep += 1;
        }
        self.positions.truncate(keep);
        self.velocities.truncate(keep);
        if mixed {
            self.mix.truncate(keep);
        }
    }
}

/// How fast mixing evens out the mix of two touching particles, per tick
/// at `mixing` 1 and full overlap.
const MIX_RATE: f32 = 0.25;

/// How strongly the boundary between unmixed fluids holds them apart (a
/// multiple of the near pressure) at `mixing` 0. A tenth of the 2D value:
/// a particle has about four times the neighbours in 3D, and at 2D's
/// strength the boundary traps a light layer under a heavy one, the
/// heavy fluid never sinking through.
const TENSION: f32 = 0.1;

/// How much of a particle's sliding velocity a solid it touches takes, per
/// tick (so a pool sloshing in a bowl comes to rest).
const WALL_FRICTION: f32 = 0.04;

/// One fluid of a step: its component, its particles and the centre of its
/// container.
pub(crate) struct Member<'a> {
    pub fluid: &'a Fluid3D,
    pub state: &'a mut Fluid3State,
    pub centre: Vec3,
}

/// One tick of a fluid alone or of the fluids of one group (in entity
/// order, their lanes `0..4`, each joined to its lane first): their
/// particles push, drag and (by `mixing`) mix as one fluid whose particles
/// weigh their fluids' `density`, so a heavier fluid sinks under a lighter
/// one. Pressure comes from every neighbour (number density), each
/// particle's own stiffness against its own rest density; a pair's
/// displacement and viscous impulse are shared in inverse proportion to
/// the particles' masses; unmixed particles of different fluids are held
/// apart at their boundary by an extra near pressure (`TENSION`, gone at
/// `mixing` 1); and touching particles even out their mix at
/// `mixing × MIX_RATE` per tick, their mass following it.
pub(crate) fn step(members: &mut [Member<'_>], dt: f32, gravity: Vec3, solids: &dyn Solids) {
    for m in members.iter_mut() {
        m.state.emit(m.fluid, m.centre, dt);
    }
    let grouped = members.len() > 1;
    let substeps = members
        .iter()
        .map(|m| m.fluid.substeps.clamp(1, 8))
        .max()
        .unwrap_or(1);
    let ds = dt / substeps as f32;
    let t = ds / TICK;
    let h = members
        .iter()
        .map(|m| m.fluid.radius.max(0.02))
        .fold(0.0f32, f32::max);
    // The particles of every member, in member order.
    let mut owner: Vec<u8> = Vec::new();
    let mut positions: Vec<Vec3> = Vec::new();
    let mut velocities: Vec<Vec3> = Vec::new();
    let mut mix: Vec<[f32; GROUP_SIZE]> = Vec::new();
    for (k, m) in members.iter().enumerate() {
        owner.extend(std::iter::repeat_n(k as u8, m.state.positions.len()));
        positions.extend_from_slice(&m.state.positions);
        velocities.extend_from_slice(&m.state.velocities);
        if grouped {
            mix.extend_from_slice(&m.state.mix);
        }
    }
    let n = positions.len();
    if n > 0 {
        let mut lane_density = [1.0f32; GROUP_SIZE];
        if grouped {
            for m in members.iter() {
                lane_density[m.state.slot as usize] = m.fluid.density.max(0.05);
            }
        }
        let mass_of = |w: &[f32; GROUP_SIZE]| -> f32 {
            let mut m = 0.0;
            for l in 0..GROUP_SIZE {
                m += w[l] * lane_density[l];
            }
            m.max(0.05)
        };
        let fluid_of = |i: usize| members[owner[i] as usize].fluid;
        let mixing_of = |i: usize| fluid_of(i).mixing.clamp(0.0, 1.0);
        let mut mass: Vec<f32> = if grouped {
            mix.iter().map(mass_of).collect()
        } else {
            vec![1.0; n]
        };
        let sigma: Vec<f32> = (0..n).map(|i| fluid_of(i).viscosity.max(0.0)).collect();
        let mut previous = vec![Vec3::ZERO; n];
        let mut rho = vec![0.0f32; n];
        let mut near = vec![0.0f32; n];
        let cap = 0.5 * h / ds;
        for _ in 0..substeps {
            for (i, v) in velocities.iter_mut().enumerate() {
                *v += gravity * (fluid_of(i).gravity_scale * ds);
            }
            let pairs = Pairs::find(&positions, h);
            // Viscosity: radial impulses between approaching pairs, shared
            // by mass.
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
            // Number density over every neighbour.
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
            // before the move.
            for i in 0..n {
                let f = fluid_of(i);
                let pressure = f.stiffness * (rho[i] - f.rest_density);
                let near_pressure = f.near_stiffness * near[i];
                let mut dx = Vec3::ZERO;
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
                        // How unlike the two particles are (0: the same
                        // mix), and how much of that the boundary holds.
                        let same: f32 = mix[i].iter().zip(&mix[j]).map(|(a, b)| a * b).sum();
                        let unlike = (1.0 - same).clamp(0.0, 1.0);
                        let hold = unlike * (1.0 - 0.5 * (mixing_of(i) + mixing_of(j))) * TENSION;
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
                // Mixing: touching particles even out their mix, then
                // their mass.
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
            // The velocity from the displacement, capped at half the
            // radius per substep (stable, and nothing tunnels a wall).
            for ((v, p), q) in velocities.iter_mut().zip(&positions).zip(&previous) {
                let u = (*p - *q) / ds;
                let len = u.length();
                *v = if len > cap { u * (cap / len) } else { u };
            }
            // Solids, the terrain and each member's container: pushed out,
            // the velocity into them dropped (pushing out adds none, so
            // a resting particle does not bounce) and sliding slowed.
            for i in 0..n {
                let m = &members[owner[i] as usize];
                let r_particle = m.fluid.radius.max(0.02) * 0.25;
                let (p, v) = (&mut positions[i], &mut velocities[i]);
                if let Some((q, normal)) = solids.push_out(*p, r_particle) {
                    *p = q;
                    let along = v.dot(normal);
                    let slide = *v - normal * along;
                    *v = normal * along.max(0.0) + slide * (1.0 - WALL_FRICTION * t);
                }
                let half = m.fluid.bounds * 0.5;
                for k in 0..3 {
                    if half[k] > 0.0 {
                        let (lo, hi) = (m.centre[k] - half[k], m.centre[k] + half[k]);
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
        }
        // Back into each member.
        let mut at = 0;
        for m in members.iter_mut() {
            let count = m.state.positions.len();
            m.state
                .positions
                .copy_from_slice(&positions[at..at + count]);
            m.state
                .velocities
                .copy_from_slice(&velocities[at..at + count]);
            if grouped {
                m.state.mix.copy_from_slice(&mix[at..at + count]);
            }
            at += count;
        }
    }
    for m in members.iter_mut() {
        m.state.sink(m.fluid, m.centre);
    }
}
