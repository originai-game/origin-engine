//! What a fractured prop draws: its chunks, crumbs and debris.

use super::*;

/// A fractured prop as extraction draws it: its og.Model's entity, and
/// each chunk's entity with the glTF node that draws it, its centre in
/// the model and its outline group (og_render::InstanceFx::outline): the
/// chunks still bonded into one island share one, so an inked look lines
/// the island round and its cracks only once they open.
pub(crate) struct Drawn {
    pub entity: u64,
    pub chunks: Vec<(u64, u32, Vec3, u32)>,
    /// Its fine pieces, the renderer's to move (og_render::DebrisPiece).
    pub debris: Vec<DrawnDebris>,
    /// The grit its broken joints shed, and the glTF material of its
    /// mortar (og_physics::destruction::NO_MATERIAL for none).
    pub crumbs: Vec<DrawnCrumb>,
    pub mortar: u32,
}

/// A broken joint's grit as extraction hands it to the renderer: an id
/// stable while it lives, the glTF node of the piece on its first side
/// (whose cut faces the clay chips are drawn with), the tick it broke, the
/// joint's centre and normal then, half its face's extent (m: a joint's
/// centre lies inside the prop, its crack at the face's edges), and what
/// the blow gave it.
pub(crate) struct DrawnCrumb {
    pub id: u64,
    pub node: u32,
    pub tick: u64,
    pub point: Vec3,
    pub normal: Vec3,
    pub half: f32,
    pub velocity: Vec3,
}

/// A fine piece as extraction hands it to the renderer: an id stable while
/// it lives, the glTF node that draws it and its centre in the model, the
/// tick it broke off with its pose (centre of mass, rotation, scale) and
/// velocities then, and the points it collides at (its hull's vertices
/// about its centre of mass, at most og_render::DEBRIS_POINTS).
pub(crate) struct DrawnDebris {
    pub id: u64,
    pub node: u32,
    pub centre: Vec3,
    pub tick: u64,
    pub pose: (Vec3, Quat, Vec3),
    pub velocity: Vec3,
    pub angular: Vec3,
    pub points: Vec<Vec3>,
}

/// Engine `engine`'s fractured props.
#[inline(never)]
pub(crate) fn drawn(engine: u32) -> Vec<Drawn> {
    let Ok(m) = module(engine) else {
        return Vec::new();
    };
    let assets = lock(&m.assets);
    let mut st = lock(&m.state);
    let State {
        props,
        orphan_chunks,
        ..
    } = &mut *st;
    for p in props.values_mut().chain(orphan_chunks.values_mut()) {
        if let Some(a) = assets.get(&p.asset) {
            p.cache_islands(a);
        }
    }
    st.props
        .iter()
        .chain(&st.orphan_chunks)
        .filter(|(_, p)| p.fractured)
        .filter_map(|(&k, p)| {
            let a = assets.get(&p.asset)?;
            // Each support chunk's island (by its first chunk); a child
            // chunk, loose on its own, its own group.
            let group = |c: usize| {
                let of = p.island_cache.1[c];
                // Never 0 (no group): mixed with the prop, so two props'
                // islands are two objects.
                let h = (k ^ (k >> 32)) as u32 ^ of.wrapping_mul(0x9e37_79b9);
                (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d).max(1)
            };
            let debris = st
                .debris
                .iter()
                .filter(|d| d.prop == k)
                .filter_map(|d| {
                    let c = a.chunks.get(d.chunk as usize)?;
                    Some(DrawnDebris {
                        id: scramble(k.rotate_left(17) ^ (u64::from(d.chunk) << 1 | 1)),
                        node: c.node,
                        centre: c.centroid,
                        tick: d.tick,
                        pose: pose_of_frame(&d.pose),
                        velocity: Vec3::from(d.velocity),
                        angular: Vec3::from(d.angular),
                        points: c.debris_points.clone(),
                    })
                })
                .collect();
            Some(Drawn {
                entity: k,
                chunks: p
                    .entities
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| **e != 0)
                    .map(|(c, &e)| (e, a.chunks[c].node, a.chunks[c].centroid, group(c)))
                    .collect(),
                debris,
                crumbs: st
                    .crumbs
                    .iter()
                    .filter(|c| c.prop == k)
                    .filter_map(|c| {
                        let b = a.bonds.get(c.bond as usize)?;
                        Some(DrawnCrumb {
                            id: scramble(k.rotate_left(29) ^ (u64::from(c.bond) << 2 | 2)),
                            node: a.chunks.get(b.a as usize)?.node,
                            tick: c.tick,
                            point: Vec3::from(c.point),
                            normal: Vec3::from(c.normal),
                            half: 0.5 * b.area.max(0.0).sqrt() * pose_of_frame(&p.frame).2.x.abs(),
                            velocity: Vec3::from(c.velocity),
                        })
                    })
                    .collect(),
                mortar: a.mortar,
            })
        })
        .collect()
}

/// Stress and support diagnostics use the same chunk frames as extraction.
#[inline(never)]
pub(crate) fn debug_lines(
    engine: u32,
    h: &mut crate::present::extract::Hierarchy,
) -> Vec<(Vec3, Vec3, [f32; 4])> {
    let Ok(m) = module(engine) else {
        return Vec::new();
    };
    let assets = lock(&m.assets);
    let st = lock(&m.state);
    let mut lines = Vec::new();
    for (&k, p) in st.props.iter().chain(&st.orphan_chunks) {
        let Some(a) = assets.get(&p.asset) else {
            continue;
        };
        let mut point = |c: u32, x: Vec3| {
            let i = c as usize;
            if p.entities[i] != 0 {
                h.world_of(p.entities[i])
                    .transform_point3(x - a.chunks[i].centroid)
            } else {
                h.world_of(k).transform_point3(x)
            }
        };
        let peak = p
            .inst
            .impulses
            .iter()
            .filter(|(kind, _, _)| *kind == 0)
            .map(|(_, _, j)| j.lin.length())
            .fold(1e-6f32, f32::max);
        for (i, b) in a.bonds.iter().enumerate() {
            if p.inst.bonds[i] <= 0.0 {
                continue;
            }
            let stress = p
                .inst
                .impulses
                .iter()
                .find(|(kind, ix, _)| *kind == 0 && *ix as usize == i)
                .map_or(0.0, |(_, _, j)| j.lin.length() / peak);
            lines.push((
                point(b.a, a.chunks[b.a as usize].centroid),
                point(b.b, a.chunks[b.b as usize].centroid),
                [stress, 1.0 - stress, 0.1, 1.0],
            ));
        }
        for (i, c) in a.chunks.iter().take(a.support as usize).enumerate() {
            if p.inst.anchors[i] > 0.0 && p.anchors {
                lines.push((
                    point(i as u32, c.anchor_centroid),
                    point(i as u32, c.anchor_centroid + c.anchor_normal * 0.25),
                    [0.0, 1.0, 1.0, 1.0],
                ));
            }
        }
        for s in &p.inst.supports {
            lines.push((
                point(s.chunk, a.chunks[s.chunk as usize].centroid),
                point(s.chunk, s.point),
                [1.0, 0.0, 1.0, 1.0],
            ));
        }
    }
    lines
}
