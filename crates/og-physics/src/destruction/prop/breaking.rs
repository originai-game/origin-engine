//! One prop's tick without the world: its hits wear its bonds, masonry
//! nothing bears comes loose, wedged islands give way and broken joints
//! shed grit.

use super::debris::{CRUMBS_A_TICK, Crumb};
use super::pose::{Pose, place_on};
use super::state::{CAUSE_FRACTURE, CAUSE_STRESS, Prop};
use crate::destruction::{Damage, FractureAsset, Instance, Island, Material};
use glam::Vec3;

/// How often (in ticks) a loose island at rest is solved on what holds it.
pub const REST_EVERY: u64 = 6;

/// The most blocks of no-tension masonry that fly loose as one piece: one
/// (mortar holds nothing pulled apart, so what breaks loose is its blocks,
/// each a cluster of bricks, never a run of them bonded into a beam).
pub const RUN: usize = 1;

/// Below this speed (m/s) a loose island is at rest.
pub const REST_SPEED: f32 = 0.5;

/// Ticks a loose island stays where it broke free before, at rest with
/// nothing under it, it counts as wedged (one just let go is at rest
/// too, for the tick before it falls).
pub const WEDGED_AFTER: u64 = 30;

/// An og.Destructible's settings, as the tick reads them.
pub struct Settings {
    pub mat: Material,
    pub threshold: f32,
    pub radius: f32,
    pub anchored: bool,
    pub break_fx: u64,
    pub hit_fx: u64,
    pub debris: u64,
}

/// What one run does to a prop besides its bonds.
#[derive(Default)]
pub struct Hits {
    pub damage: Vec<(Damage, u32)>,
    pub fracture: bool,
}

/// One prop as it breaks: its key, asset, settings and chunk poses.
#[derive(Clone, Copy)]
pub struct Breaking<'a> {
    pub key: u64,
    pub a: &'a FractureAsset,
    pub s: &'a Settings,
    pub pose: &'a [Option<Pose>],
}

/// Model-space point `at` of chunk `c` in the world: on the chunk's pose,
/// else on the prop's frame.
pub fn place_chunk(at: Breaking<'_>, frame: &Pose, c: u32, x: Vec3) -> Vec3 {
    let ch = &at.a.chunks[c as usize];
    match at.pose[c as usize] {
        Some(ps) => place_on(&ps, ch.centroid, x),
        None => place_on(frame, Vec3::ZERO, x),
    }
}

/// The prop's islands; none anchored unless its anchors hold.
pub fn islands_of(inst: &Instance, a: &FractureAsset, anchors: bool) -> Vec<Island> {
    let mut isl = inst.islands(a);
    if !anchors {
        isl.iter_mut().for_each(|i| i.anchored = false);
    }
    isl
}

/// The prop's hits on its bonds, and a forced fracture: the cause of the
/// first that broke something, and the chunks with children past their
/// health, which shatter into them.
pub fn wear(
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    h: &Hits,
    scale: f32,
) -> (Option<u32>, Vec<u32>) {
    let (a, s) = (at.a, at.s);
    let mut cause = None;
    let place_fn = |c: u32, x: Vec3| place_chunk(at, frame, c, x);
    for (d, c) in &h.damage {
        if p.inst.damage(a, d, scale, &s.mat, &place_fn) && cause.is_none() {
            cause = Some(*c);
        }
    }
    if h.fracture {
        p.inst.bonds.iter_mut().for_each(|b| *b = 0.0);
        cause = Some(CAUSE_FRACTURE);
    }
    // A chunk with children past its health shatters into them.
    let mut shatter = Vec::new();
    for c in 0..a.support as usize {
        let ch = &a.chunks[c];
        if !p.inst.shattered[c]
            && ch.children.1 > 0
            && p.inst.chunk_damage[c] >= Instance::chunk_health(a, c as u32, scale, s.mat.toughness)
        {
            p.inst.shattered[c] = true;
            p.inst.dirty = true;
            shatter.push(c as u32);
        }
    }
    (cause, shatter)
}

/// No-tension masonry (tension at most a tenth of a strength above 0;
/// strength 0 turns stress off, and with it this): every REST_EVERY ticks
/// a block of what stands that nothing bears from below comes loose,
/// whole, and what it bore is judged again the next time; and a loose run
/// of mortared blocks longer than RUN comes apart at its joints the tick
/// it breaks loose.
pub fn break_masonry(
    tick: u64,
    gravity: Vec3,
    at: Breaking<'_>,
    frame: &Pose,
    p: &mut Prop,
    before: &[f32],
    cause: &mut Option<u32>,
) {
    let (a, s) = (at.a, at.s);
    let masonry = s.mat.limits.shear_elastic.is_finite()
        && s.mat.limits.tension_elastic <= 0.1 * s.mat.limits.shear_elastic;
    let anchors = p.anchors;
    if anchors && tick.is_multiple_of(REST_EVERY) && masonry {
        let up = -(frame.1.inverse() * gravity).normalize_or_zero();
        for c in p.inst.unsupported(a, &islands_of(&p.inst, a, anchors), up) {
            p.inst.free(a, c);
            cause.get_or_insert(CAUSE_STRESS);
        }
    }
    // No-tension masonry in the air holds no bending: a loose run of
    // mortared blocks longer than RUN comes apart at its joints the
    // tick it breaks loose, into its blocks (each its own cluster of
    // bricks), so the top of a blown wall goes up as a spray of
    // blocks, not a sheet that bends away whole or a beam of courses
    // lying across the hole.
    if masonry && (0..before.len()).any(|i| before[i] > 0.0 && p.inst.bonds[i] <= 0.0) {
        let mut run = vec![false; a.chunks.len()];
        for isl in islands_of(&p.inst, a, anchors)
            .into_iter()
            .filter(|i| !i.anchored && i.chunks.len() > RUN)
        {
            isl.chunks.iter().for_each(|&c| run[c as usize] = true);
        }
        for (k, b) in a.bonds.iter().enumerate() {
            if run[b.a as usize] && p.inst.bonds[k] > 0.0 {
                p.inst.bonds[k] = 0.0;
                p.inst.dirty = true;
            }
        }
    }
}

/// The grit the joints broken this tick shed (spread over at most
/// CRUMBS_A_TICK of them), with the mass of the piece beside each.
pub fn shed_crumbs(
    tick: u64,
    at: Breaking<'_>,
    frame: &Pose,
    broken_now: &[bool],
    scale: f32,
    crumbs: &mut Vec<(Crumb, f32)>,
) {
    let (a, s) = (at.a, at.s);
    let fresh: Vec<usize> = (0..broken_now.len()).filter(|&i| broken_now[i]).collect();
    let step = fresh.len().div_ceil(CRUMBS_A_TICK).max(1);
    for &i in fresh.iter().step_by(step) {
        let b = &a.bonds[i];
        let rot = at.pose[b.a as usize].map_or(frame.1, |ps| ps.1);
        let v = a.chunks[b.a as usize].volume * scale * scale * scale;
        crumbs.push((
            Crumb {
                prop: at.key,
                bond: i as u32,
                tick,
                point: place_chunk(at, frame, b.a, b.centroid).to_array(),
                normal: (rot * b.normal).to_array(),
                velocity: [0.0; 3],
            },
            v * s.mat.density,
        ));
    }
}

/// Wedged loose island `chunks` gives way: its bonds break, and each chunk
/// with pieces shatters into them (`shatter` gathers those).
pub fn crumble(a: &FractureAsset, inst: &mut Instance, chunks: &[u32], shatter: &mut Vec<u32>) {
    for (k, b) in a.bonds.iter().enumerate() {
        if chunks.contains(&b.a) && chunks.contains(&b.b) {
            inst.bonds[k] = 0.0;
        }
    }
    for &c in chunks {
        let i = c as usize;
        if a.chunks[i].children.1 > 0 && !inst.shattered[i] {
            inst.shattered[i] = true;
            shatter.push(c);
        }
    }
    inst.dirty = true;
}
