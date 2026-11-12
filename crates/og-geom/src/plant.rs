//! A plant grown from numbers: a trunk, up to three levels of branches,
//! leaves and flowers, so a tree, a shrub or a flower is a handful of
//! parameters and a seed instead of an authored mesh. The parameter model
//! and the growth rules are ez-tree's (MIT, Daniel Greenheck 2024;
//! third_party/vegetation/ez-tree.LICENSE): each branch is a run of
//! sections that wander (`gnarliness`), turn (`twist`) and bend towards a
//! direction (`force`), and grows children at stratified places and turns
//! round it; a deciduous branch also continues from its own end, an
//! evergreen's children shorten towards its top. Rewritten here in
//! metres (ez-tree's presets are in units of about [`UNIT`] m), on
//! quaternions, with the engine's seeded [`Rng`] and [`trig`], so a seed
//! grows the same plant on every host; flowers, leaf colours and the wind
//! attributes are this file's.
//!
//! Besides the two meshes it returns what og-render's foliage wind reads
//! of each vertex (`foliage.rs` `FoliageVertex`): the pivot its limb sways
//! about (where the limb leaves the trunk) with the limb's stiffness, and
//! for a leaf or a petal the point it is attached at and how much it
//! flutters. A limb and everything growing from it share one pivot and
//! one stiffness, so they bend as one piece and stay joined.

use crate::{Mesh, trig};
use glam::{DQuat, DVec3};
use og_core::rng::Rng;

/// The trunk and three levels of branches.
pub const LEVELS: usize = 4;

/// What one ez-tree unit is in metres: its presets (a trunk 37 long and
/// 1.4 thick for a medium oak) are about this scale, and the two rules
/// that read a radius (`gnarliness` and `force`) read it in these units.
pub const UNIT: f64 = 0.2;

/// The most branches and leaves a plant grows.
pub const MAX_BRANCHES: u64 = 20_000;
pub const MAX_LEAVES: u64 = 120_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Growth {
    /// Every branch continues from its own end, a level deeper.
    Deciduous,
    /// Branches end in a point and are shorter the higher they leave
    /// their parent: a conifer's cone.
    Evergreen,
}

/// One level of branches: 0 the trunk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    /// Its length (m).
    pub length: f64,
    /// The trunk's radius at its foot (m); a branch's as a share of its
    /// parent's radius where it leaves it.
    pub radius: f64,
    /// Sections along it and sides round it.
    pub sections: u32,
    pub segments: u32,
    /// How much of its radius is gone by its end (0..1).
    pub taper: f64,
    /// How far each section wanders from the last (radians, more on thin
    /// branches).
    pub gnarliness: f64,
    /// The turn round its own axis each section (radians).
    pub twist: f64,
    /// The branches a branch of this level grows (unused at the last level).
    pub children: u32,
    /// Its angle from its parent (degrees; unused for the trunk).
    pub angle: f64,
    /// Where along its parent the first of them may grow (0..1; unused
    /// for the trunk).
    pub start: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafShape {
    /// A pointed leaf cupped along its midrib: six vertices, no texture.
    Blade,
    /// A quad for a leaf texture (uv 0..1, the stem at the bottom).
    Card,
    /// Two crossed quads.
    Cross,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leaves {
    pub shape: LeafShape,
    /// Leaves along each last-level branch (a deciduous one adds one at
    /// its end, where they grow that far).
    pub count: u32,
    /// The part of the branch they grow along (0..1 each): 0 to 0.2 is a
    /// rosette at a stem's foot, 0.5 to 1 leaves a bough's inner half bare.
    pub start: f64,
    pub end: f64,
    /// Their angle from the branch (degrees).
    pub angle: f64,
    /// How far a leaf's face turns from the branch's side towards the
    /// sky (0..1): 0 as it grows, 1 flat to the light, as most broadleaves
    /// hold theirs.
    pub facing: f64,
    /// The rows the leaves stand in round the branch: 0 anywhere round
    /// it; 2 in two rows, one to each side of it and level with the
    /// ground (a palm's or a fern's frond, a conifer's flat spray); more
    /// are spaced evenly round it from there.
    pub rows: u32,
    /// A leaf's length (m) and its width as a share of it.
    pub size: f64,
    pub width: f64,
    /// Each leaf's size varies by this share either way (0..1).
    pub size_variance: f64,
    /// Linear colours: each leaf takes one between the two, lighter or
    /// darker by `color_variation` (0..1).
    pub color: [f32; 3],
    pub color2: [f32; 3],
    pub color_variation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flowers {
    /// The share of last-level branch ends that flower (0..1).
    pub share: f64,
    /// More flowers along each last-level branch.
    pub count: u32,
    pub petals: u32,
    /// A petal's length (m) and its width as a share of it.
    pub size: f64,
    pub width: f64,
    /// How far the petals lift from flat (degrees: 0 a daisy, 60 a tulip).
    pub cup: f64,
    /// Linear colours: each flower takes one between the two.
    pub color: [f32; 3],
    pub color2: [f32; 3],
    pub center_color: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plant {
    pub seed: u32,
    pub growth: Growth,
    /// Levels of branches past the trunk, 0 to 3.
    pub levels: u32,
    pub level: [Level; LEVELS],
    /// The direction branches bend towards (away, with a negative
    /// strength: a conifer's drooping boughs) and how fast.
    pub force_direction: DVec3,
    pub force_strength: f64,
    pub leaves: Option<Leaves>,
    pub flowers: Option<Flowers>,
}

impl Default for Level {
    fn default() -> Self {
        Level {
            length: 1.0,
            radius: 0.6,
            sections: 4,
            segments: 4,
            taper: 0.6,
            gnarliness: 0.1,
            twist: 0.0,
            children: 3,
            angle: 50.0,
            start: 0.3,
        }
    }
}

impl Default for Leaves {
    fn default() -> Self {
        Leaves {
            shape: LeafShape::Blade,
            count: 10,
            start: 0.1,
            end: 1.0,
            angle: 45.0,
            facing: 0.8,
            rows: 0,
            size: 0.4,
            width: 0.5,
            size_variance: 0.4,
            color: [0.09, 0.24, 0.04],
            color2: [0.17, 0.33, 0.06],
            color_variation: 0.3,
        }
    }
}

impl Default for Flowers {
    fn default() -> Self {
        Flowers {
            share: 1.0,
            count: 0,
            petals: 5,
            size: 0.04,
            width: 0.6,
            cup: 20.0,
            color: [1.0, 1.0, 1.0],
            color2: [1.0, 1.0, 1.0],
            center_color: [1.0, 0.7, 0.05],
        }
    }
}

impl Default for Plant {
    /// A small broadleaf tree, about 6 m.
    fn default() -> Self {
        let level =
            |length, radius, sections, segments, gnarliness, children, angle, start| Level {
                length,
                radius,
                sections,
                segments,
                gnarliness,
                children,
                angle,
                start,
                ..Level::default()
            };
        Plant {
            seed: 1,
            growth: Growth::Deciduous,
            levels: 2,
            level: [
                level(3.2, 0.16, 6, 6, 0.04, 5, 0.0, 0.0),
                level(2.0, 0.6, 5, 4, 0.1, 3, 55.0, 0.45),
                level(1.2, 0.6, 3, 3, 0.12, 2, 50.0, 0.3),
                level(0.6, 0.6, 2, 3, 0.1, 0, 45.0, 0.2),
            ],
            force_direction: DVec3::Y,
            force_strength: 0.01,
            leaves: Some(Leaves::default()),
            flowers: None,
        }
    }
}

/// A grown plant: its wood and what grows on it (leaves and flowers, in
/// vertex colours), each with a vertex's wind attributes beside it
/// (`[pivot x y z, stiffness, anchor x y z, flutter]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlantMesh {
    pub wood: Mesh,
    pub wood_wind: Vec<[f32; 8]>,
    pub leaves: Mesh,
    pub leaf_wind: Vec<[f32; 8]>,
    pub branches: usize,
    pub leaf_count: usize,
    pub flower_count: usize,
}

struct Section {
    origin: DVec3,
    q: DQuat,
    radius: f64,
}

/// What a limb and everything on it sways about.
#[derive(Clone, Copy)]
struct Sway {
    pivot: DVec3,
    stiffness: f64,
}

struct Bud {
    origin: DVec3,
    q: DQuat,
    length: f64,
    radius: f64,
    level: usize,
    sections: u32,
    segments: u32,
    sway: Sway,
}

struct Limb {
    sections: Vec<Section>,
    segments: u32,
    base_radius: f64,
    sway: Sway,
}

struct Leaf {
    origin: DVec3,
    q: DQuat,
    size: f64,
    sway: Sway,
}

/// A turn of `angle` round the unit `axis`.
fn turn(axis: DVec3, angle: f64) -> DQuat {
    let (s, c) = trig::sincos(0.5 * angle);
    DQuat::from_xyzw(axis.x * s, axis.y * s, axis.z * s, c)
}

/// Between two orientations, without a transcendental.
fn between(a: DQuat, b: DQuat, t: f64) -> DQuat {
    let b = if a.dot(b) < 0.0 { -b } else { b };
    (a * (1.0 - t) + b * t).normalize()
}

/// A place `at` (0..1) along a branch's sections.
fn along(sections: &[Section], at: f64) -> Section {
    let last = sections.len() - 1;
    let f = at.clamp(0.0, 1.0) * last as f64;
    let i = (f.floor() as usize).min(last);
    let (a, b) = (&sections[i], &sections[(i + 1).min(last)]);
    let t = f - i as f64;
    Section {
        origin: a.origin.lerp(b.origin, t),
        q: between(a.q, b.q, t),
        radius: a.radius + (b.radius - a.radius) * t,
    }
}

/// `0..count` shuffled (Fisher-Yates).
fn shuffled(count: usize, rng: &mut Rng) -> Vec<usize> {
    let mut v: Vec<usize> = (0..count).collect();
    for k in (1..count).rev() {
        let r = ((rng.next_f64() * (k + 1) as f64).floor() as usize).min(k);
        v.swap(k, r);
    }
    v
}

/// `count` places along a branch from `start` to `end` and turns round it, both
/// stratified (a slot each, jittered) and paired at random, so they are
/// spread without a spiral: each as (where along, the orientation of
/// something leaving the branch there at `angle` degrees).
fn spread(
    sections: &[Section],
    count: u32,
    (start, end): (f64, f64),
    angle: f64,
    rows: u32,
    rng: &mut Rng,
) -> Vec<(f64, Section)> {
    let count = count as usize;
    let offset = rng.next_f64();
    let start = start.clamp(0.0, 1.0);
    let step = (end.clamp(start, 1.0) - start) / count.max(1) as f64;
    let slots = shuffled(count, rng);
    (0..count)
        .map(|i| {
            let at = start + (i as f64 + rng.next_f64()) * step;
            let mut s = along(sections, at);
            let jitter = rng.range(-0.5, 0.5);
            let round = if rows == 0 {
                trig::TAU * (offset + (slots[i] as f64 + jitter) / count as f64)
            } else {
                // In rows, taken in turn along the branch: the first to
                // its side, where the ground's level crosses it (anywhere
                // round a branch that stands upright).
                let side = (s.q * DVec3::Y).cross(DVec3::Y);
                let first = if side.length_squared() < 1e-6 {
                    trig::TAU * offset
                } else {
                    let local = s.q.inverse() * side.normalize();
                    trig::atan2(local.x, local.z)
                };
                first + trig::TAU * (i as u32 % rows) as f64 / f64::from(rows) + 0.3 * jitter
            };
            s.q = (s.q * turn(DVec3::Y, round) * turn(DVec3::X, angle * trig::PI / 180.0))
                .normalize();
            (at, s)
        })
        .collect()
}

/// How many branches and leaves `p` grows.
fn counts(p: &Plant) -> (u64, u64) {
    let levels = (p.levels as usize).min(LEVELS - 1);
    let own = u64::from(p.growth == Growth::Deciduous);
    let (mut at, mut branches) = (1u64, 1u64);
    for level in p.level.iter().take(levels) {
        at = at.saturating_mul(u64::from(level.children) + own);
        branches = branches.saturating_add(at);
    }
    let per = p.leaves.map_or(0, |l| u64::from(l.count) + own)
        + p.flowers
            .map_or(0, |f| (u64::from(f.count) + 1) * (u64::from(f.petals) + 1));
    (branches, at.saturating_mul(per))
}

/// Grows `p`. `detail` (0..1, 1 as written) draws the same plant with
/// fewer sections, sides and leaves (each leaf larger, to cover as much).
pub fn plant(p: &Plant, detail: f64) -> Result<PlantMesh, String> {
    let levels = (p.levels as usize).min(LEVELS - 1);
    let (branches, leaves) = counts(p);
    if branches > MAX_BRANCHES || leaves > MAX_LEAVES {
        return Err(format!(
            "grows {branches} branches and {leaves} leaves and petals (at most {MAX_BRANCHES} and {MAX_LEAVES}): fewer children, levels or leaves"
        ));
    }
    let mut rng = Rng::new(p.seed);
    // Colours and flowers draw from their own sequences, so changing one
    // leaves the plant's shape alone.
    let mut colour = rng.fork("plant colour");
    let mut flowering = rng.fork("plant flowers");
    let mut rolling = rng.fork("plant leaf roll");
    let target = p.force_direction.normalize_or(DVec3::Y);
    let trunk = p.level[0];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(Bud {
        origin: DVec3::ZERO,
        q: DQuat::IDENTITY,
        length: trunk.length.max(0.0),
        radius: trunk.radius.max(1e-4),
        level: 0,
        sections: trunk.sections.max(1),
        segments: trunk.segments.max(3),
        sway: Sway {
            pivot: DVec3::ZERO,
            stiffness: 1.0,
        },
    });
    let mut limbs: Vec<Limb> = Vec::new();
    let mut leaf_list: Vec<Leaf> = Vec::new();
    // Flowers: where, and whether along the branch (smaller) or at its end.
    let mut flower_list: Vec<Leaf> = Vec::new();
    while let Some(bud) = queue.pop_front() {
        let l = p.level[bud.level];
        let last_level = bud.level == levels;
        let step = bud.length / bud.sections as f64;
        let mut sections = Vec::with_capacity(bud.sections as usize + 1);
        let (mut origin, mut q) = (bud.origin, bud.q);
        for i in 0..=bud.sections {
            let t = i as f64 / bud.sections as f64;
            let radius = if i == bud.sections && last_level {
                1e-4
            } else if p.growth == Growth::Deciduous {
                bud.radius * (1.0 - l.taper.clamp(0.0, 1.0) * t)
            } else {
                bud.radius * (1.0 - t)
            }
            .max(1e-4);
            sections.push(Section { origin, q, radius });
            origin += q * DVec3::Y * step;
            // Thin branches wander more (at most four times as much).
            let units = radius / UNIT;
            let g = (1.0 / units.sqrt()).clamp(1.0, 4.0) * l.gnarliness;
            let (gx, gz) = (rng.range(-g, g), rng.range(-g, g));
            q = q * turn(DVec3::X, gx) * turn(DVec3::Z, gz) * turn(DVec3::Y, l.twist);
            // Towards the force's direction, faster the thinner, and
            // never past it.
            let up = q * DVec3::Y;
            let axis = up.cross(target);
            let sin_full = axis.length();
            if sin_full > 1e-6 && p.force_strength != 0.0 {
                let full = trig::atan2(sin_full, up.dot(target));
                let by = (p.force_strength / units).clamp(-full, full);
                q = turn(axis / sin_full, by) * q;
            }
            q = q.normalize();
        }
        let end = sections
            .last()
            .map(|s| (s.origin, s.q, s.radius))
            .expect("a section");
        // A limb leaving the trunk sways about where it leaves it, stiffer
        // the thicker it is; everything on it sways with it.
        let child_sway = |origin: DVec3, radius: f64| {
            if bud.level == 0 {
                Sway {
                    pivot: origin,
                    stiffness: (0.3 + 0.6 * radius / trunk.radius.max(1e-4)).clamp(0.3, 0.9),
                }
            } else {
                bud.sway
            }
        };
        if p.growth == Growth::Deciduous {
            if !last_level {
                queue.push_back(Bud {
                    origin: end.0,
                    q: end.1,
                    length: p.level[bud.level + 1].length.max(0.0),
                    radius: end.2,
                    level: bud.level + 1,
                    // Its parent's counts: it grows from its end ring.
                    sections: bud.sections,
                    segments: bud.segments,
                    sway: child_sway(end.0, end.2),
                });
            } else if let Some(lv) = p.leaves.as_ref().filter(|lv| lv.end >= 1.0) {
                let v = lv.size_variance.clamp(0.0, 1.0);
                leaf_list.push(Leaf {
                    origin: end.0,
                    q: end.1,
                    size: lv.size * (1.0 + rng.range(-v, v)),
                    sway: bud.sway,
                });
            }
        }
        if last_level {
            if let Some(lv) = &p.leaves {
                let v = lv.size_variance.clamp(0.0, 1.0);
                for (_, s) in spread(
                    &sections,
                    lv.count,
                    (lv.start, lv.end),
                    lv.angle,
                    lv.rows,
                    &mut rng,
                ) {
                    leaf_list.push(Leaf {
                        origin: s.origin,
                        q: s.q,
                        size: lv.size * (1.0 + rng.range(-v, v)),
                        sway: bud.sway,
                    });
                }
            }
            if let Some(f) = &p.flowers {
                if flowering.next_f64() < f.share {
                    flower_list.push(Leaf {
                        origin: end.0,
                        q: end.1,
                        size: f.size,
                        sway: bud.sway,
                    });
                }
                for (_, s) in spread(&sections, f.count, (0.2, 1.0), 70.0, 0, &mut flowering) {
                    flower_list.push(Leaf {
                        origin: s.origin,
                        q: s.q,
                        size: f.size,
                        sway: bud.sway,
                    });
                }
            }
        } else {
            let c = p.level[bud.level + 1];
            for (at, s) in spread(&sections, l.children, (c.start, 1.0), c.angle, 0, &mut rng) {
                let radius = (c.radius * s.radius).max(1e-4);
                queue.push_back(Bud {
                    origin: s.origin,
                    q: s.q,
                    length: c.length.max(0.0)
                        * if p.growth == Growth::Evergreen {
                            1.0 - at
                        } else {
                            1.0
                        },
                    radius,
                    level: bud.level + 1,
                    sections: c.sections.max(1),
                    segments: c.segments.max(3),
                    sway: child_sway(s.origin, radius),
                });
            }
        }
        limbs.push(Limb {
            sections,
            segments: bud.segments,
            base_radius: bud.radius,
            sway: bud.sway,
        });
    }

    let detail = if detail.is_finite() {
        detail.clamp(1.0 / 16.0, 1.0)
    } else {
        1.0
    };
    let stride = (1.0 / detail).round().max(1.0) as usize;
    let mut out = PlantMesh {
        branches: limbs.len(),
        ..PlantMesh::default()
    };
    for limb in &limbs {
        wood(&mut out, limb, stride, detail);
    }
    if let Some(lv) = &p.leaves {
        // Fewer leaves cover as much: each as much larger in area.
        let scale = (stride as f64).sqrt();
        for leaf in leaf_list.iter().step_by(stride) {
            let u = colour.next_f64() as f32;
            let shade =
                1.0 + lv.color_variation.clamp(0.0, 1.0) * (colour.next_f64() as f32 - 0.5) * 0.6;
            let c: [f32; 3] =
                std::array::from_fn(|k| (lv.color[k] + (lv.color2[k] - lv.color[k]) * u) * shade);
            let wind = wind_of(leaf.sway, leaf.origin, 1.0);
            let (len, width) = (leaf.size * scale, leaf.size * scale * lv.width);
            let q = skyward(leaf.q, lv.facing, rolling.range(-0.5, 0.5));
            match lv.shape {
                LeafShape::Blade => blade(&mut out, leaf.origin, q, len, width, LEAF, c, wind),
                LeafShape::Card => card(&mut out, leaf.origin, q, len, width, c, wind),
                LeafShape::Cross => {
                    card(&mut out, leaf.origin, leaf.q, len, width, c, wind);
                    let q = leaf.q * turn(DVec3::Y, 0.5 * trig::PI);
                    card(&mut out, leaf.origin, q, len, width, c, wind);
                }
            }
            out.leaf_count += 1;
        }
    }
    if let Some(f) = &p.flowers {
        let petals = f.petals.clamp(1, 32);
        let lift = -(0.5 * trig::PI - f.cup.clamp(0.0, 90.0) * trig::PI / 180.0);
        for flower in &flower_list {
            let u = colour.next_f64() as f32;
            let c: [f32; 3] = std::array::from_fn(|k| f.color[k] + (f.color2[k] - f.color[k]) * u);
            let wind = wind_of(flower.sway, flower.origin, 0.6);
            let offset = trig::TAU * colour.next_f64();
            for k in 0..petals {
                let round = offset + trig::TAU * f64::from(k) / f64::from(petals);
                let q = flower.q * turn(DVec3::Y, round) * turn(DVec3::X, lift);
                blade(
                    &mut out,
                    flower.origin,
                    q,
                    flower.size,
                    flower.size * f.width,
                    PETAL,
                    c,
                    wind,
                );
            }
            centre(
                &mut out,
                flower.origin,
                flower.q,
                flower.size,
                f.center_color,
                wind,
            );
            out.flower_count += 1;
        }
    }
    Ok(out)
}

/// `q` rolled about its own length so its face turns `facing` (0..1) of
/// the way to the sky, and `jitter` radians past that.
fn skyward(q: DQuat, facing: f64, jitter: f64) -> DQuat {
    let (y, z) = (q * DVec3::Y, q * DVec3::Z);
    // The sky's direction in the plane the face can turn in.
    let flat = DVec3::Y - y * y.y;
    let facing = facing.clamp(0.0, 1.0);
    if flat.length_squared() < 1e-8 || facing == 0.0 {
        return q;
    }
    let flat = flat.normalize();
    let roll = trig::atan2(z.cross(flat).dot(y), z.dot(flat));
    (turn(y, roll * facing + jitter * facing) * q).normalize()
}

fn wind_of(sway: Sway, anchor: DVec3, flutter: f32) -> [f32; 8] {
    let (p, a) = (sway.pivot.as_vec3(), anchor.as_vec3());
    [p.x, p.y, p.z, sway.stiffness as f32, a.x, a.y, a.z, flutter]
}

/// One limb's rings, every `stride`th section (its ends always), as a
/// tube open at both ends: uv u round it (a whole number of metres, so a
/// bark texture meets itself), v along it in metres.
fn wood(out: &mut PlantMesh, limb: &Limb, stride: usize, detail: f64) {
    let segments = ((f64::from(limb.segments) * detail).round() as usize).max(3);
    let wraps = (trig::TAU * limb.base_radius).round().max(1.0);
    let last = limb.sections.len() - 1;
    let mut rings: Vec<usize> = (0..=last).step_by(stride).collect();
    if rings.last() != Some(&last) {
        rings.push(last);
    }
    let m = &mut out.wood;
    let first = m.positions.len() as u32;
    let wind = wind_of(limb.sway, DVec3::ZERO, 0.0);
    let wind = [wind[0], wind[1], wind[2], wind[3], 0.0, 0.0, 0.0, 0.0];
    let mut v = 0.0;
    for (k, &i) in rings.iter().enumerate() {
        let s = &limb.sections[i];
        if k > 0 {
            v += s.origin.distance(limb.sections[rings[k - 1]].origin);
        }
        for j in 0..=segments {
            let (sin, cos) = trig::sincos(trig::TAU * (j % segments) as f64 / segments as f64);
            let n = s.q * DVec3::new(cos, 0.0, sin);
            m.positions
                .push((s.origin + n * s.radius).as_vec3().to_array());
            m.normals.push(n.as_vec3().to_array());
            m.uvs
                .push([(j as f64 / segments as f64 * wraps) as f32, v as f32]);
            out.wood_wind.push(wind);
        }
    }
    let n = segments as u32 + 1;
    for i in 0..rings.len() as u32 - 1 {
        for j in 0..segments as u32 {
            let a = first + i * n + j;
            let (b, c, d) = (a + 1, a + n, a + n + 1);
            m.indices.extend([a, c, b, b, c, d]);
        }
    }
}

/// Vertices of a leaf-like piece at `origin` turned by `q`: x across it,
/// y along it, z its face.
fn piece(
    out: &mut PlantMesh,
    origin: DVec3,
    q: DQuat,
    verts: &[(DVec3, DVec3, [f32; 2], f32)],
    tris: &[u32],
    color: [f32; 3],
    wind: [f32; 8],
) {
    let m = &mut out.leaves;
    let first = m.positions.len() as u32;
    for (p, n, uv, shade) in verts {
        m.positions.push((origin + q * *p).as_vec3().to_array());
        m.normals
            .push((q * *n).normalize_or(DVec3::Y).as_vec3().to_array());
        m.uvs.push(*uv);
        m.colors
            .push([color[0] * shade, color[1] * shade, color[2] * shade, 1.0]);
        out.leaf_wind.push(wind);
    }
    m.indices.extend(tris.iter().map(|i| first + i));
}

/// A blade's edge at two heights, as (along, across, shade), and how far
/// along its tip is.
type Outline = ([(f64, f64, f32); 2], f64);
/// A leaf: widest under its middle, a pointed tip.
const LEAF: Outline = ([(0.3, 0.92, 0.95), (0.68, 0.74, 1.05)], 1.0);
/// A petal: widest near its end, which is blunt.
const PETAL: Outline = ([(0.45, 0.62, 0.95), (0.84, 1.0, 1.03)], 1.0);

/// A leaf or petal cupped along its midrib, darker at its stem: six
/// vertices, four triangles.
#[allow(clippy::too_many_arguments)]
fn blade(
    out: &mut PlantMesh,
    origin: DVec3,
    q: DQuat,
    length: f64,
    width: f64,
    (rows, tip): Outline,
    color: [f32; 3],
    wind: [f32; 8],
) {
    let (w, cup) = (0.5 * width, 0.12 * width);
    let mut verts = vec![(DVec3::ZERO, DVec3::Z, [0.5, 1.0], 0.7)];
    for (along, across, shade) in rows {
        for side in [-1.0, 1.0] {
            verts.push((
                DVec3::new(side * across * w, along * length, cup * across),
                DVec3::new(-side * 0.35, 0.0, 1.0),
                [0.5 + 0.5 * (side * across) as f32, 1.0 - along as f32],
                shade,
            ));
        }
    }
    verts.push((
        DVec3::new(0.0, tip * length, -cup),
        DVec3::new(0.0, 0.2, 1.0),
        [0.5, 0.0],
        1.12,
    ));
    piece(
        out,
        origin,
        q,
        &verts,
        &[0, 2, 1, 1, 2, 4, 1, 4, 3, 3, 4, 5],
        color,
        wind,
    );
}

/// A quad for a leaf texture, its stem at the bottom of the image.
fn card(
    out: &mut PlantMesh,
    origin: DVec3,
    q: DQuat,
    length: f64,
    width: f64,
    color: [f32; 3],
    wind: [f32; 8],
) {
    let w = 0.5 * width;
    piece(
        out,
        origin,
        q,
        &[
            (DVec3::new(-w, length, 0.0), DVec3::Z, [0.0, 0.0], 1.0),
            (DVec3::new(-w, 0.0, 0.0), DVec3::Z, [0.0, 1.0], 1.0),
            (DVec3::new(w, 0.0, 0.0), DVec3::Z, [1.0, 1.0], 1.0),
            (DVec3::new(w, length, 0.0), DVec3::Z, [1.0, 0.0], 1.0),
        ],
        &[0, 1, 2, 0, 2, 3],
        color,
        wind,
    );
}

/// A flower's centre: a hexagon facing along its stem, a little above
/// the petals' roots.
fn centre(
    out: &mut PlantMesh,
    origin: DVec3,
    q: DQuat,
    size: f64,
    color: [f32; 3],
    wind: [f32; 8],
) {
    let (r, lift) = (0.22 * size, 0.06 * size);
    let mut verts = vec![(DVec3::new(0.0, lift * 1.5, 0.0), DVec3::Y, [0.5, 0.5], 1.0)];
    let mut tris = Vec::new();
    for k in 0..6u32 {
        let (sin, cos) = trig::sincos(trig::TAU * f64::from(k) / 6.0);
        verts.push((
            DVec3::new(cos * r, lift, sin * r),
            DVec3::new(0.4 * cos, 1.0, 0.4 * sin),
            [0.5 + 0.5 * cos as f32, 0.5 + 0.5 * sin as f32],
            0.85,
        ));
        tris.extend([0, 1 + (k + 1) % 6, 1 + k]);
    }
    piece(out, origin, q, &verts, &tris, color, wind);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(m: &Mesh) -> ([f32; 3], [f32; 3]) {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in &m.positions {
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
        (lo, hi)
    }

    /// Ways this could fail: a seed grows another plant on another run (a
    /// hash map's order, libm), two seeds grow the same one, or the
    /// colours' and flowers' draws move the wood.
    #[test]
    fn a_seed_names_one_plant_and_colours_and_flowers_leave_its_shape_alone() {
        let p = Plant::default();
        let a = plant(&p, 1.0).unwrap();
        assert_eq!(a, plant(&p, 1.0).unwrap());
        let other = plant(&Plant { seed: 2, ..p }, 1.0).unwrap();
        assert_ne!(a.wood.positions, other.wood.positions);
        let mut flowered = p;
        flowered.flowers = Some(Flowers::default());
        flowered.leaves = Some(Leaves {
            color: [1.0, 0.0, 0.0],
            ..Leaves::default()
        });
        let f = plant(&flowered, 1.0).unwrap();
        assert_eq!(a.wood, f.wood);
        assert_eq!(
            a.leaves.positions[..],
            f.leaves.positions[..a.leaves.positions.len()]
        );
        assert!(f.flower_count > 0 && a.flower_count == 0);
    }

    /// Ways this could fail: the counts are off (a level too many or too
    /// few, the deciduous end branch missed), a tube faces inwards, the
    /// plant grows sideways or underground, or attributes disagree in
    /// length.
    #[test]
    fn the_default_tree_stands_on_its_root_with_the_branches_its_numbers_say() {
        let p = Plant::default();
        let m = plant(&p, 1.0).unwrap();
        // 1 trunk, 5 + 1 limbs, each 3 + 1 twigs.
        assert_eq!(m.branches, 1 + 6 + 24);
        assert_eq!(m.leaf_count, 24 * (10 + 1));
        assert_eq!(counts(&p), (31, 24 * 11));
        assert_eq!(m.wood.positions.len(), m.wood_wind.len());
        assert_eq!(m.leaves.positions.len(), m.leaf_wind.len());
        assert_eq!(m.leaves.positions.len(), m.leaves.colors.len());
        assert_eq!(m.leaves.triangle_count(), m.leaf_count * 4);
        let (lo, hi) = bounds(&m.wood);
        assert!(lo[1] > -0.3 && hi[1] > 4.0 && hi[1] < 9.0, "{lo:?} {hi:?}");
        assert!(lo[0] < -0.5 && hi[0] > 0.5 && lo[2] < -0.5 && hi[2] > 0.5);
        // The trunk's tube faces out: its first triangle's face normal
        // points away from its axis.
        let v = |i: usize| DVec3::from(m.wood.positions[m.wood.indices[i] as usize].map(f64::from));
        let face = (v(1) - v(0)).cross(v(2) - v(0));
        let out = (v(0) + v(1) + v(2)) / 3.0;
        assert!(face.dot(DVec3::new(out.x, 0.0, out.z)) > 0.0);
        // The enclosed volume is positive too (every tube, open ends and all).
        assert!(m.wood.volume() > 0.0);
    }

    /// Ways this could fail: the trunk sways about a pivot of its own
    /// (the foliage shader bends it about the root already), a limb's
    /// twigs or leaves take another pivot or stiffness than the limb (they
    /// would come apart in wind), a leaf's anchor is not on its own leaf,
    /// or wood flutters.
    #[test]
    fn a_limb_and_all_that_grows_on_it_sway_as_one_piece() {
        let m = plant(&Plant::default(), 1.0).unwrap();
        let rigid = m.wood_wind.iter().filter(|w| w[3] == 1.0).count();
        // The trunk's 7 rings of 7 vertices, and nothing else.
        assert_eq!(rigid, 7 * 7);
        assert!(m.wood_wind.iter().all(|w| w[4..] == [0.0; 4]));
        let mut pivots: Vec<[u32; 4]> = m
            .wood_wind
            .iter()
            .map(|w| [w[0], w[1], w[2], w[3]].map(f32::to_bits))
            .collect();
        pivots.sort_unstable();
        pivots.dedup();
        // The root and the six limbs' own.
        assert_eq!(pivots.len(), 7);
        for w in &m.leaf_wind {
            let key = [w[0], w[1], w[2], w[3]].map(f32::to_bits);
            assert!(pivots.contains(&key), "a leaf sways about no limb's pivot");
            assert!(w[3] < 1.0 && w[7] == 1.0);
        }
        // A blade's first vertex is its stem: the anchor itself.
        for (p, w) in m.leaves.positions.iter().zip(&m.leaf_wind).step_by(6) {
            assert_eq!(p[..], w[4..7]);
        }
    }

    /// Ways this could fail: an evergreen keeps a deciduous end branch,
    /// its boughs do not shorten towards the top, a stem with no levels
    /// grows no flower, or the caps let a runaway plant through.
    #[test]
    fn an_evergreen_is_a_cone_a_bare_stem_flowers_and_a_runaway_is_refused() {
        let mut pine = Plant {
            growth: Growth::Evergreen,
            levels: 1,
            force_strength: -0.003,
            ..Plant::default()
        };
        pine.level[0] = Level {
            length: 8.0,
            radius: 0.2,
            sections: 10,
            segments: 6,
            children: 40,
            gnarliness: 0.02,
            ..Level::default()
        };
        pine.level[1] = Level {
            length: 3.0,
            radius: 0.4,
            angle: 100.0,
            start: 0.25,
            gnarliness: 0.02,
            ..Level::default()
        };
        let m = plant(&pine, 1.0).unwrap();
        assert_eq!(m.branches, 41);
        let reach = |from: f32, to: f32| {
            m.leaves
                .positions
                .iter()
                .filter(|p| p[1] >= from && p[1] < to)
                .map(|p| (p[0] * p[0] + p[2] * p[2]).sqrt())
                .fold(0.0, f32::max)
        };
        assert!(
            reach(2.0, 3.5) > 1.6 * reach(6.5, 8.5),
            "{} {}",
            reach(2.0, 3.5),
            reach(6.5, 8.5)
        );

        let mut daisy = Plant {
            levels: 0,
            leaves: None,
            flowers: Some(Flowers::default()),
            ..Plant::default()
        };
        daisy.level[0] = Level {
            length: 0.3,
            radius: 0.004,
            gnarliness: 0.0,
            ..Level::default()
        };
        let m = plant(&daisy, 1.0).unwrap();
        assert_eq!((m.branches, m.flower_count), (1, 1));
        // Five petals of four triangles and a six-triangle centre.
        assert_eq!(m.leaves.triangle_count(), 5 * 4 + 6);
        let (lo, hi) = bounds(&m.leaves);
        assert!(lo[1] > 0.25 && hi[1] < 0.36, "{lo:?} {hi:?}");

        let mut runaway = Plant {
            levels: 3,
            ..Plant::default()
        };
        for l in &mut runaway.level {
            l.children = 100;
        }
        assert!(plant(&runaway, 1.0).unwrap_err().contains("branches"));
    }

    /// Ways this could fail: leaves in rows stand round the branch as
    /// they do without (a frond that is a bottle brush), or all to one
    /// side of it.
    #[test]
    fn leaves_in_two_rows_lie_level_to_both_sides_of_a_branch() {
        let frond = |rows| {
            let mut p = Plant {
                levels: 1,
                force_strength: 0.0,
                leaves: Some(Leaves {
                    rows,
                    count: 24,
                    start: 0.3,
                    angle: 90.0,
                    facing: 1.0,
                    size: 0.3,
                    size_variance: 0.0,
                    ..Leaves::default()
                }),
                ..Plant::default()
            };
            p.level[0] = Level {
                length: 2.0,
                gnarliness: 0.0,
                children: 1,
                ..p.level[0]
            };
            p.level[1] = Level {
                length: 2.0,
                gnarliness: 0.0,
                angle: 90.0,
                ..p.level[1]
            };
            // The one branch out of the trunk's side, below the leader
            // that carries the trunk on from 2 m (its leaves from 2.6 m).
            let m = plant(&p, 1.0).unwrap();
            let on: Vec<[f32; 3]> = m
                .leaves
                .positions
                .iter()
                .copied()
                .filter(|p| p[1] < 2.3)
                .collect();
            assert!(on.len() > 60, "{}", on.len());
            on
        };
        let spread = |on: &[[f32; 3]]| {
            let (lo, hi) = on.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                (lo.min(p[1]), hi.max(p[1]))
            });
            hi - lo
        };
        let (round, rows) = (frond(0), frond(2));
        assert!(spread(&round) > 0.4, "{}", spread(&round));
        assert!(spread(&rows) < 0.12, "{}", spread(&rows));
        // To both sides: across the branch (the narrow way of the leaves'
        // footprint on the ground), as many one side as the other.
        let n = rows.len() as f32;
        let (cx, cz) = rows
            .iter()
            .fold((0.0, 0.0), |(x, z), p| (x + p[0] / n, z + p[2] / n));
        let (mut xx, mut xz, mut zz) = (0.0f32, 0.0f32, 0.0f32);
        for p in &rows {
            let (x, z) = (p[0] - cx, p[2] - cz);
            xx += x * x;
            xz += x * z;
            zz += z * z;
        }
        let along = 0.5 * og_core::math::atan2(2.0 * xz, xx - zz);
        let (ax, az) = (-og_core::math::sin(along), og_core::math::cos(along));
        let left = rows
            .iter()
            .filter(|p| (p[0] - cx) * ax + (p[2] - cz) * az > 0.0)
            .count() as f32;
        assert!(left > 0.35 * n && left < 0.65 * n, "{left} of {n}");
    }

    /// Ways this could fail: a lower detail grows another plant (its
    /// branches end elsewhere) or draws no fewer triangles.
    #[test]
    fn less_detail_draws_the_same_plant_with_fewer_triangles() {
        let p = Plant::default();
        let (full, half) = (plant(&p, 1.0).unwrap(), plant(&p, 0.5).unwrap());
        assert!(half.wood.triangle_count() * 3 < full.wood.triangle_count() * 2);
        assert!(half.leaf_count * 2 <= full.leaf_count + 1);
        let (a, b) = (bounds(&full.wood), bounds(&half.wood));
        for k in 0..3 {
            assert!((a.0[k] - b.0[k]).abs() < 0.15 && (a.1[k] - b.1[k]).abs() < 0.15);
        }
    }
}
