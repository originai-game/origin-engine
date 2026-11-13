//! Destruction, with no ECS, rapier or renderer dependency: the state one
//! destructible keeps while it breaks ([`Instance`]: the health of every
//! bond, its anchors and its chunks) over a prefractured asset
//! ([`FractureAsset`], og-physics-data), radial damage, the stress solver
//! of NVIDIA Blast ([`stress`]), the support graph's islands, and the 2D
//! pixel terrain ([`terrain2d`]). og-runtime's `destruction` system drives
//! it from the world.
//!
//! Everything is f32 with `+ - * /` and `sqrt`, in fixed orders: the same
//! bits on every host, so a fracture replays.

pub mod stress;
pub mod terrain2d;

use glam::Vec3;
pub use og_physics_data::fracture::{ASSET_VERSION, Bond, Chunk, FractureAsset, NO_MATERIAL, NONE};
use serde::{Deserialize, Serialize};
use stress::{AngLin6, SolverBond, SolverNode, StressLimits, StressProcessor};

/// How much more damage a chunk takes before it shatters than a bond of
/// its cross-section before it breaks.
pub const SHATTER: f32 = 3.0;

/// How one destructible's material answers damage and load.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// Damage a square metre of bond takes before it breaks (N·s/m²: an
    /// impulse of that many newton-seconds through 1 m² breaks it).
    pub toughness: f32,
    /// The stress solver's limits (Pa).
    pub limits: StressLimits,
    /// kg/m³: the chunks' mass for the stress solver.
    pub density: f32,
}

/// A radial damage: `amount` at `point` (the destructible's local space),
/// all of it within `min_radius`, falling linearly to nothing at
/// `max_radius` (Blast's `NvBlastExtRadialDamageDesc` falloff).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Damage {
    pub point: Vec3,
    pub min_radius: f32,
    pub max_radius: f32,
    pub amount: f32,
}

impl Damage {
    pub fn at(&self, p: Vec3) -> f32 {
        let d = (p - self.point).length();
        if d <= self.min_radius {
            self.amount
        } else if d >= self.max_radius {
            0.0
        } else {
            self.amount * (self.max_radius - d) / (self.max_radius - self.min_radius)
        }
    }
}

/// A fixed scramble of `x` (Chris Wellons's lowbias32, public domain):
/// a chunk's own variation, the same on every run and host.
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// Where something holds up a chunk of an island no anchor holds: a loose
/// island resting on a pier, a pile or the ground, for
/// [`Instance::stress`]. In the model's space, as the chunk's own points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Support {
    pub chunk: u32,
    /// Where it bears on the chunk.
    pub point: Vec3,
    /// The area it bears on (m², before the destructible's scale).
    pub area: f32,
}

/// One connected group of support chunks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Island {
    /// Its support chunks, ascending; the first names it.
    pub chunks: Vec<u32>,
    /// Held by an intact anchor bond.
    pub anchored: bool,
}

/// The state of one destructible while it breaks: world state (saved,
/// hashed, restored by its owner's state block).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    /// Per bond: its remaining health, 1 intact to 0 broken (the share of
    /// its area that still holds: Blast's bond health as effective area).
    pub bonds: Vec<f32>,
    /// Per support chunk: the health of its anchor bond (0 none or broken).
    pub anchors: Vec<f32>,
    /// Per chunk: damage taken (N·s), which shatters a loose chunk with
    /// children into them once past [`Instance::chunk_health`].
    pub chunk_damage: Vec<f32>,
    /// Per support chunk: shattered into its children.
    pub shattered: Vec<bool>,
    /// Damage or a break since the stress solver last found the graph at
    /// rest: it runs again.
    pub dirty: bool,
}

impl Instance {
    pub fn new(a: &FractureAsset) -> Instance {
        Instance {
            bonds: vec![1.0; a.bonds.len()],
            anchors: a.chunks[..a.support as usize]
                .iter()
                .map(|c| if c.anchor_area > 0.0 { 1.0 } else { 0.0 })
                .collect(),
            chunk_damage: vec![0.0; a.chunks.len()],
            shattered: vec![false; a.support as usize],
            dirty: false,
        }
    }

    /// Whether nothing has broken yet.
    pub fn intact(&self) -> bool {
        self.bonds.iter().all(|&h| h > 0.0) && !self.shattered.iter().any(|&s| s)
    }

    /// The damage a chunk takes before it shatters into its children: its
    /// cross-section (volume^(2/3), scaled by `scale`²) times toughness,
    /// times [`SHATTER`]: a hit that breaks the bonds round a chunk does
    /// not by itself break the chunk.
    pub fn chunk_health(a: &FractureAsset, chunk: u32, scale: f32, toughness: f32) -> f32 {
        SHATTER * sq_cube_root(a.chunks[chunk as usize].volume) * scale * scale * toughness
    }

    /// Applies a radial damage to every intact bond, anchor and chunk it
    /// reaches (`scale`: the destructible's uniform scale, so areas are in
    /// square metres). `place` maps a point of the model's space, on chunk
    /// `c` (a bond's first chunk, an anchor's, a chunk's own), to the
    /// damage's space: the chunks of a broken prop have moved apart, each
    /// with its own transform. Answers whether anything took damage; a
    /// bond whose health reaches 0 is broken.
    pub fn damage(
        &mut self,
        a: &FractureAsset,
        d: &Damage,
        scale: f32,
        m: &Material,
        place: &dyn Fn(u32, Vec3) -> Vec3,
    ) -> bool {
        let mut any = false;
        let tough = m.toughness.max(1e-6);
        for (k, b) in a.bonds.iter().enumerate() {
            let h = &mut self.bonds[k];
            if *h <= 0.0 {
                continue;
            }
            let amount = d.at(place(b.a, b.centroid));
            if amount > 0.0 {
                *h = (*h - amount / (b.area * scale * scale * tough)).max(0.0);
                any = true;
            }
        }
        for (i, c) in a.chunks[..a.support as usize].iter().enumerate() {
            let h = &mut self.anchors[i];
            if *h <= 0.0 {
                continue;
            }
            let amount = d.at(place(i as u32, c.anchor_centroid));
            if amount > 0.0 {
                *h = (*h - amount / (c.anchor_area * scale * scale * tough)).max(0.0);
                any = true;
            }
        }
        for (i, c) in a.chunks.iter().enumerate() {
            let amount = d.at(place(i as u32, c.centroid));
            if amount > 0.0 {
                self.chunk_damage[i] += amount;
                any = true;
            }
        }
        self.dirty |= any;
        any
    }

    /// The chunks of anchored `islands` that nothing holds from below:
    /// masonry stands by its weight alone (Heyman's no-tension masonry), so
    /// a block stays only with its centre of mass within the middle third of
    /// what bears it (the masonry rule that keeps a joint closed), the
    /// standing blocks under it across a bed joint (a bond facing down,
    /// within 45° of `-up`: a cracked joint bears as well as a sound one,
    /// the brick still sits on the brick) or a foundation (an intact anchor
    /// under it). A block held only from its side, by the blocks beside it
    /// or a tie-in, or reaching past its bearing (a whole brick over air),
    /// is listed.
    pub fn unsupported(&self, a: &FractureAsset, islands: &[Island], up: Vec3) -> Vec<u32> {
        let (u, v) = up.any_orthonormal_pair();
        let span = |c: usize| {
            let ch = &a.chunks[c];
            let mut r = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
            for h in &ch.hull {
                let p = ch.centroid + *h;
                r[0] = r[0].min(p.dot(u));
                r[1] = r[1].max(p.dot(u));
                r[2] = r[2].min(p.dot(v));
                r[3] = r[3].max(p.dot(v));
            }
            r
        };
        let mut standing = vec![false; a.support as usize];
        for c in islands
            .iter()
            .filter(|i| i.anchored)
            .flat_map(|i| i.chunks.iter())
        {
            standing[*c as usize] = true;
        }
        let mut out = Vec::new();
        for c in islands
            .iter()
            .filter(|i| i.anchored)
            .flat_map(|i| i.chunks.iter())
        {
            let c = *c as usize;
            let ch = &a.chunks[c];
            // A foundation: an intact anchor lying under the block (one
            // anchor stands for all of a chunk's faces on the world, so a
            // corner block's lies under it and to its side).
            let below = ch.hull.iter().map(|h| -h.dot(up)).fold(0.0f32, f32::max);
            if self.anchors[c] > 0.0
                && ch.anchor_area > 0.0
                && (ch.centroid - ch.anchor_centroid).dot(up) > 0.25 * below
            {
                continue;
            }
            let own = span(c);
            // The bearing: the overlap of this block and each block under
            // it, across both horizontal axes.
            let mut bear = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
            for b in &a.bonds {
                if b.a as usize != c && b.b as usize != c {
                    continue;
                }
                let (o, down) = if b.a as usize == c {
                    (b.b as usize, b.normal)
                } else {
                    (b.a as usize, -b.normal)
                };
                if !standing[o] || down.dot(-up) < 0.7 {
                    continue;
                }
                let s = span(o);
                bear[0] = bear[0].min(s[0].max(own[0]));
                bear[1] = bear[1].max(s[1].min(own[1]));
                bear[2] = bear[2].min(s[2].max(own[2]));
                bear[3] = bear[3].max(s[3].min(own[3]));
            }
            // The middle-third rule: its weight within the middle third of
            // its bearing on both axes (a joint pressed off-centre further
            // than that opens at its far edge).
            // Each block's joint its own (mortar laid well or badly, a
            // brick a little long or short): its tolerance a fixed share of
            // the middle third's, 0.6 to 1.4 of it, so the piers step back
            // unevenly, a course holding a brick longer or shorter, rather
            // than in a drawn staircase. At most 0.23 of the bearing, so no
            // block reaches half its length past what bears it.
            let give = 0.6 + 0.8 * (mix(c as u32) >> 8) as f32 / (1u32 << 24) as f32;
            let (cu, cv) = (ch.centroid.dot(u), ch.centroid.dot(v));
            let third = |c: f32, lo: f32, hi: f32| {
                hi >= lo && (c - 0.5 * (lo + hi)).abs() <= give * (hi - lo) / 6.0 + 1e-3
            };
            if !(third(cu, bear[0], bear[1]) && third(cv, bear[2], bear[3])) {
                out.push(c as u32);
            }
        }
        out
    }

    /// Frees chunk `c` from the graph: every bond it has and its anchor
    /// break, so it falls on its own.
    pub fn free(&mut self, a: &FractureAsset, c: u32) {
        for (k, b) in a.bonds.iter().enumerate() {
            if b.a == c || b.b == c {
                self.bonds[k] = 0.0;
            }
        }
        self.anchors[c as usize] = 0.0;
        self.dirty = true;
    }

    /// The islands of the support graph: support chunks (not shattered)
    /// joined by intact bonds, each anchored when one of its chunks keeps
    /// an intact anchor; ordered by their first chunk.
    pub fn islands(&self, a: &FractureAsset) -> Vec<Island> {
        let n = a.support as usize;
        let mut parent: Vec<u32> = (0..n as u32).collect();
        fn find(p: &mut [u32], mut i: u32) -> u32 {
            while p[i as usize] != i {
                p[i as usize] = p[p[i as usize] as usize];
                i = p[i as usize];
            }
            i
        }
        for (k, b) in a.bonds.iter().enumerate() {
            if self.bonds[k] > 0.0 && !self.shattered[b.a as usize] && !self.shattered[b.b as usize]
            {
                let (x, y) = (find(&mut parent, b.a), find(&mut parent, b.b));
                if x != y {
                    parent[x.max(y) as usize] = x.min(y);
                }
            }
        }
        let mut out: Vec<Island> = Vec::new();
        let mut at: Vec<usize> = vec![usize::MAX; n];
        for i in 0..n as u32 {
            if self.shattered[i as usize] {
                continue;
            }
            let r = find(&mut parent, i) as usize;
            if at[r] == usize::MAX {
                at[r] = out.len();
                out.push(Island {
                    chunks: Vec::new(),
                    anchored: false,
                });
            }
            let isl = &mut out[at[r]];
            isl.chunks.push(i);
            isl.anchored |= self.anchors[i as usize] > 0.0;
        }
        out
    }

    /// Runs the stress solver over the islands that are held up (anchored,
    /// or with a chunk among `supports`) under `gravity` (local space,
    /// m/s²) and damages the bonds and anchors it finds past their elastic
    /// limits (bonds only: anchors and supports carry what they are given;
    /// damage breaks anchors). `islands` is [`Instance::islands`] or some
    /// of them. Answers how many bonds broke; clears `dirty` when nothing
    /// took damage (the graph is at rest).
    ///
    /// `budget` is the tick's solver work left, in bond-iterations (one
    /// iteration over one bond): a solve takes what it used, and none
    /// starts once it is spent, leaving `dirty` for the next tick. A tick's
    /// stress work is so at most its budget plus one solve's cap (six
    /// iterations a bond), whatever breaks.
    #[allow(clippy::too_many_arguments)]
    pub fn stress(
        &mut self,
        a: &FractureAsset,
        islands: &[Island],
        supports: &[Support],
        gravity: Vec3,
        scale: f32,
        m: &Material,
        iterations: u32,
        budget: &mut u64,
    ) -> u32 {
        if *budget == 0 {
            return 0;
        }
        let s2 = scale * scale;
        let mut node_of = vec![NONE; a.support as usize];
        // Every chunk of a held island, then the world under each anchor
        // and support.
        let mut nodes = Vec::new();
        let held = |i: &&Island| i.anchored || supports.iter().any(|s| i.chunks.contains(&s.chunk));
        for isl in islands.iter().filter(held) {
            for &c in &isl.chunks {
                let ch = &a.chunks[c as usize];
                let mass = ch.volume * s2 * scale * m.density;
                node_of[c as usize] = nodes.len() as u32;
                nodes.push(SolverNode {
                    com: ch.centroid * scale,
                    mass,
                    // A solid sphere's of the same volume: 2/5 m r², r² ≈
                    // (3V/4π)^(2/3); its cube root by Newton's method.
                    inertia: 0.4 * mass * sq_cube_root(ch.volume * 0.238_732_4) * s2,
                });
            }
        }
        if nodes.is_empty() {
            self.dirty = false;
            return 0;
        }
        // (bond kind, index): 0 a bond, 1 an anchor.
        let mut which: Vec<(u8, u32, Vec3, f32)> = Vec::new();
        let mut bonds: Vec<SolverBond> = Vec::new();
        for (k, b) in a.bonds.iter().enumerate() {
            let (na, nb) = (node_of[b.a as usize], node_of[b.b as usize]);
            if self.bonds[k] <= 0.0 || na == NONE || nb == NONE {
                continue;
            }
            bonds.push(SolverBond {
                nodes: [na, nb],
                centroid: b.centroid * scale,
            });
            which.push((0, k as u32, b.normal, b.area * self.bonds[k] * s2));
        }
        for (i, c) in a.chunks[..a.support as usize].iter().enumerate() {
            let n = node_of[i];
            if n == NONE || self.anchors[i] <= 0.0 {
                continue;
            }
            // The world, as a static node mirrored through the anchor face
            // (Blast bonds a chunk to the world; the mirror gives the bond
            // a length for its bending, as a bond between two chunks has).
            let world = nodes.len() as u32;
            nodes.push(SolverNode {
                com: (c.anchor_centroid * 2.0 - c.centroid) * scale,
                mass: 0.0,
                inertia: 0.0,
            });
            bonds.push(SolverBond {
                nodes: [world, n],
                centroid: c.anchor_centroid * scale,
            });
            // From the world towards the chunk.
            let normal = (c.centroid - c.anchor_centroid).normalize_or_zero();
            which.push((1, i as u32, normal, c.anchor_area * self.anchors[i] * s2));
        }
        // What a loose island rests on: the world as an anchor that never
        // breaks (kind 2).
        for sp in supports {
            let n = node_of[sp.chunk as usize];
            if n == NONE {
                continue;
            }
            let c = &a.chunks[sp.chunk as usize];
            let world = nodes.len() as u32;
            nodes.push(SolverNode {
                com: (sp.point * 2.0 - c.centroid) * scale,
                mass: 0.0,
                inertia: 0.0,
            });
            bonds.push(SolverBond {
                nodes: [world, n],
                centroid: sp.point * scale,
            });
            let normal = (c.centroid - sp.point).normalize_or_zero();
            which.push((2, sp.chunk, normal, sp.area * s2));
        }
        let sp = StressProcessor::prepare(&nodes, &bonds);
        let v: Vec<AngLin6> = nodes
            .iter()
            .map(|n| AngLin6 {
                ang: Vec3::ZERO,
                lin: if n.mass > 0.0 { gravity } else { Vec3::ZERO },
            })
            .collect();
        let mut j = vec![AngLin6::default(); bonds.len()];
        let taken = sp.solve(&mut j, &v, iterations, 1e-3, false);
        *budget = budget.saturating_sub(u64::from(taken.unsigned_abs()) * bonds.len() as u64);
        let mut broke = 0;
        let mut damaged = false;
        for (k, (kind, idx, normal, area)) in which.iter().enumerate() {
            let bond = bonds[k];
            let (n0, n1) = (nodes[bond.nodes[0] as usize], nodes[bond.nodes[1] as usize]);
            let disp = n1.com - n0.com;
            let normal = if normal.dot(disp) < 0.0 {
                -*normal
            } else {
                *normal
            };
            let mult = m
                .limits
                .damage(stress::bond_stress(j[k], normal, *area, disp.length()));
            if mult <= 0.0 {
                continue;
            }
            // Only bonds between chunks give way under load: an anchor is
            // the world the prop is built into (a foundation, the walls a
            // wall is tied into), as strong as it is, and a support is
            // what a loose island rests on. Damage breaks anchors.
            let h = match kind {
                0 => &mut self.bonds[*idx as usize],
                _ => continue,
            };
            damaged = true;
            // Blast: the bond loses health × multiplier; at or past the
            // fatal limit (multiplier ≥ 1) it breaks.
            *h = if mult >= 1.0 {
                0.0
            } else {
                (*h - *h * mult).max(0.0)
            };
            if *h < 1e-3 {
                *h = 0.0;
                broke += 1;
            }
        }
        self.dirty = damaged;
        broke
    }
}

/// The cube root of `x` ≥ 0 without `powf` (not the same bits on every
/// host): Kahan's bit estimate, then Newton's method.
pub fn cube_root(x: f32) -> f32 {
    if x <= 0.0 || !x.is_finite() {
        return 0.0;
    }
    let mut r = f32::from_bits(x.to_bits() / 3 + 709_921_077);
    for _ in 0..4 {
        r = (2.0 * r + x / (r * r)) / 3.0;
    }
    r
}

/// x^(2/3), as [`cube_root`].
fn sq_cube_root(x: f32) -> f32 {
    let r = cube_root(x);
    r * r
}

#[cfg(test)]
mod tests;
