//! Origin Engine destruction, with no ECS, physics or renderer dependency:
//! what a prefractured asset holds ([`FractureAsset`]: its chunks, a
//! hierarchy of two levels and the bonds of its support graph, Blast's
//! model, `docs/design.md` §2.7), the state one destructible keeps while it
//! breaks ([`Instance`]: the health of every bond, its anchors and its
//! chunks), radial damage, the stress solver of NVIDIA Blast ([`stress`])
//! and the support graph's islands. og-runtime's `destruction` system
//! drives it from the world; the cook writes the asset (og-cook
//! `cook/fracture.rs`, the pieces from [`og_geom::fracture`]).
//!
//! Everything is f32 with `+ - * /` and `sqrt`, in fixed orders: the same
//! bits on every host, so a fracture replays.

pub mod stress;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use stress::{AngLin6, SolverBond, SolverNode, StressLimits, StressProcessor};

/// No chunk (a root's parent, a leaf's children).
pub const NONE: u32 = u32::MAX;

/// How much more damage a chunk takes before it shatters than a bond of
/// its cross-section before it breaks.
pub const SHATTER: f32 = 3.0;

/// The format version of [`FractureAsset::to_bytes`].
pub const ASSET_VERSION: u32 = 1;

/// One chunk of a prefractured asset, in the model's space.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    /// Its centre of mass.
    pub centroid: Vec3,
    pub volume: f32,
    /// Its vertices relative to its centroid (its collider is their hull).
    pub hull: Vec<Vec3>,
    /// The glTF node that draws it, at its centroid.
    pub node: u32,
    /// The chunk it is part of ([`NONE`] for a support chunk).
    pub parent: u32,
    /// Its children (`first`, `count`; count 0 for a leaf).
    pub children: (u32, u32),
    /// The area by which it rests on what holds the asset up (its anchor
    /// faces, `anchor` in the source's `fracture` block); 0 when none.
    pub anchor_area: f32,
    pub anchor_centroid: Vec3,
}

/// The surface two support chunks share (Blast's bond).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    pub a: u32,
    pub b: u32,
    pub centroid: Vec3,
    /// From a towards b.
    pub normal: Vec3,
    pub area: f32,
}

/// What the cook makes of a source with a `fracture` block: chunks (the
/// support chunks first, `0..support`, then their children, each parent's
/// contiguous), and the bonds between support chunks, sorted by (a, b).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FractureAsset {
    pub chunks: Vec<Chunk>,
    pub bonds: Vec<Bond>,
    pub support: u32,
    /// The intact model's bounds.
    pub min: Vec3,
    pub max: Vec3,
    /// The glTF material of the cut faces.
    pub interior: u32,
}

impl FractureAsset {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = ASSET_VERSION.to_le_bytes().to_vec();
        out.extend(bincode::serialize(self).expect("a fracture asset serialises"));
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<FractureAsset, String> {
        let (v, rest) = bytes
            .split_first_chunk::<4>()
            .ok_or("a fracture asset is cut short")?;
        let v = u32::from_le_bytes(*v);
        if v != ASSET_VERSION {
            return Err(format!(
                "a fracture asset of version {v}, not {ASSET_VERSION}: cook it again"
            ));
        }
        let a: FractureAsset = bincode::deserialize(rest).map_err(|e| e.to_string())?;
        a.check()?;
        Ok(a)
    }

    /// Every index in range and the support chunks first.
    pub fn check(&self) -> Result<(), String> {
        let n = self.chunks.len() as u32;
        if self.support == 0 || self.support > n {
            return Err(format!("{} support chunks of {n}", self.support));
        }
        for (i, c) in self.chunks.iter().enumerate() {
            let support = (i as u32) < self.support;
            if support != (c.parent == NONE) {
                return Err(format!("chunk {i}: support chunks come first"));
            }
            if c.children.1 > 0 && c.children.0.saturating_add(c.children.1) > n {
                return Err(format!("chunk {i}: its children are out of range"));
            }
        }
        for b in &self.bonds {
            if b.a >= b.b || b.b >= self.support {
                return Err(format!("bond {}-{}: not between support chunks", b.a, b.b));
            }
        }
        Ok(())
    }

    /// The bonds of each support chunk (indices into `bonds`).
    pub fn bonds_of(&self) -> Vec<Vec<u32>> {
        let mut out = vec![Vec::new(); self.support as usize];
        for (k, b) in self.bonds.iter().enumerate() {
            out[b.a as usize].push(k as u32);
            out[b.b as usize].push(k as u32);
        }
        out
    }
}

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
    ) -> u32 {
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
        sp.solve(&mut j, &v, iterations, 1e-3, false);
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
            let (sn, ss) = stress::bond_stress(j[k], normal, *area, disp.length());
            let mult = m.limits.damage(sn, ss);
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
