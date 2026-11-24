//! The scene's colliders ([`crate::RenderScene::colliders`]) as a
//! compute kernel reads them: [`pack`] writes each collider near one of a
//! set of boxes as a record, with a list per box of the records near it,
//! and `shaders/scene_colliders.wgsl` pushes a point out of a list's
//! colliders. GPU visual simulations that never touch the simulation
//! (the debris layer, `debris.rs`) collide with the world through it.
//!
//! The format (the GPU visual-body solver's, `gpu/visual-cloth-softbody`,
//! which this module is shared with):
//! word 0: the records' count and the first word after them (u32 bits);
//! then each record, 8 vec4s: kind (u32 bits in x) and its shape; then the
//! lists, each a count and record indices; then the height fields'
//! samples. Kinds: 0 plane (unit normal, offset), 1 sphere (centre,
//! radius), 2 capsule (one end and the radius, the other end), 3 oriented box (centre; three unit axes
//! with their half extents in w), 4 a height field's patch under the
//! boxes (world to local rows, local to world rows, then origin x and z,
//! the cell and the samples per row and rows (u32 bits, 16:16); x of the
//! head's y: its first sample's word; z: the z cell size; w: the first
//! hole-mask word). Kind 5 is a local cylinder (inverse rows, radius and
//! half-height); kind 6 a local convex (inverse rows, tail plane offset
//! and count). The tail's planes are outward (normal, offset).

use crate::particles::SceneCollider;
use glam::Vec3;

/// The most samples a height field's patch has a side.
const PATCH: u32 = 64;

/// `colliders` near each of `bounds` (world boxes, min and max), packed
/// as the module docs say, and each box's list (its first word: a count
/// then record indices; `u32::MAX` when nothing is near it).
pub(crate) fn pack(
    colliders: &[SceneCollider],
    bounds: &[(Vec3, Vec3)],
) -> (Vec<[f32; 4]>, Vec<u32>) {
    let meets = |c: &SceneCollider, (lo, hi): (Vec3, Vec3)| -> bool {
        match c {
            SceneCollider::Plane { .. } => true,
            SceneCollider::Sphere { center, radius } => {
                (center.clamp(lo, hi) - *center).length() <= *radius
            }
            SceneCollider::Capsule { a, b, radius } => {
                let r = Vec3::splat(*radius);
                (a.min(*b) - r).cmple(hi).all() && (a.max(*b) + r).cmpge(lo).all()
            }
            SceneCollider::Box {
                transform,
                half_extents,
            } => {
                let c = transform.transform_point3(Vec3::ZERO);
                let r = (0..3)
                    .map(|k| transform.col(k).truncate().abs() * half_extents[k])
                    .fold(Vec3::ZERO, |s, v| s + v);
                (c - r).cmple(hi).all() && (c + r).cmpge(lo).all()
            }
            SceneCollider::HeightField { .. }
            | SceneCollider::Cylinder { .. }
            | SceneCollider::Convex { .. } => true,
        }
    };
    let mut records: Vec<[[f32; 4]; 8]> = Vec::new();
    let mut heights: Vec<f32> = Vec::new();
    let mut members: Vec<Vec<u32>> = vec![Vec::new(); bounds.len()];
    for c in colliders {
        let near: Vec<usize> = (0..bounds.len()).filter(|&b| meets(c, bounds[b])).collect();
        if near.is_empty() {
            continue;
        }
        // A separate terrain patch per body, never a union across distant bodies.
        let groups = if matches!(c, SceneCollider::HeightField { .. }) {
            near.into_iter().map(|b| vec![b]).collect::<Vec<_>>()
        } else {
            vec![near]
        };
        for near in groups {
            let mut r = [[0.0f32; 4]; 8];
            let kind = |k: u32| f32::from_bits(k);
            match c {
                SceneCollider::Plane { normal, offset } => {
                    let n = normal.normalize_or_zero();
                    if n == Vec3::ZERO {
                        continue;
                    }
                    r[0][0] = kind(0);
                    r[1] = n.extend(*offset / normal.length()).to_array();
                }
                SceneCollider::Sphere { center, radius } => {
                    r[0][0] = kind(1);
                    r[1] = center.extend(*radius).to_array();
                }
                SceneCollider::Capsule { a, b, radius } => {
                    r[0][0] = kind(2);
                    r[1] = a.extend(*radius).to_array();
                    r[2] = b.extend(0.0).to_array();
                }
                SceneCollider::Box {
                    transform,
                    half_extents,
                } => {
                    r[0][0] = kind(3);
                    r[1] = transform
                        .transform_point3(Vec3::ZERO)
                        .extend(0.0)
                        .to_array();
                    for k in 0..3 {
                        let axis = transform.col(k).truncate();
                        let len = axis.length();
                        r[2 + k] = (axis / len.max(1e-12))
                            .extend(half_extents[k] * len)
                            .to_array();
                    }
                }
                SceneCollider::HeightField { data, transform } => {
                    // A patch of the field under the boxes, in its own space:
                    // at most PATCH samples a side.
                    let inv = transform.inverse();
                    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                    for &b in &near {
                        let (blo, bhi) = bounds[b];
                        for k in 0..8 {
                            let corner = Vec3::new(
                                if k & 1 == 0 { blo.x } else { bhi.x },
                                if k & 2 == 0 { blo.y } else { bhi.y },
                                if k & 4 == 0 { blo.z } else { bhi.z },
                            );
                            let l = inv.transform_point3(corner);
                            lo = lo.min(l);
                            hi = hi.max(l);
                        }
                    }
                    let half = data.size * 0.5;
                    if lo.x >= half.x || lo.z >= half.y || hi.x <= -half.x || hi.z <= -half.y {
                        continue;
                    }
                    let source_cell = data.size
                        / glam::Vec2::new(
                            (data.columns.max(2) - 1) as f32,
                            (data.rows.max(2) - 1) as f32,
                        );
                    let start = ((glam::Vec2::new(lo.x, lo.z) + half) / source_cell)
                        .floor()
                        .max(glam::Vec2::ZERO);
                    let end = ((glam::Vec2::new(hi.x, hi.z) + half) / source_cell)
                        .ceil()
                        .min(glam::Vec2::new(
                            (data.columns - 1) as f32,
                            (data.rows - 1) as f32,
                        ));
                    let extent = (end - start).max(glam::Vec2::ONE);
                    let side = (extent.x as u32 + 1).min(PATCH);
                    let rows = (extent.y as u32 + 1).min(PATCH);
                    let cell = extent * source_cell
                        / glam::Vec2::new((side - 1) as f32, (rows - 1) as f32);
                    let corner = start * source_cell - half;
                    let (x0, z0) = (corner.x, corner.y);
                    r[0] = [
                        kind(4),
                        0.0, // the first height's word, set below
                        cell.y,
                        0.0,
                    ];
                    let first = heights.len() as u32;
                    r[0][1] = f32::from_bits(first);
                    let sampler = crate::TerrainSampler::of(data);
                    for z in 0..rows {
                        for x in 0..side {
                            let at =
                                glam::Vec2::new(x0 + x as f32 * cell.x, z0 + z as f32 * cell.y);
                            heights.push(sampler.height(at.clamp(-half, half)));
                        }
                    }
                    r[0][3] = f32::from_bits(heights.len() as u32);
                    for z in 0..rows {
                        for x in 0..side {
                            let at = glam::Vec2::new(
                                x0 + (x as f32 + 0.5) * cell.x,
                                z0 + (z as f32 + 0.5) * cell.y,
                            );
                            heights.push(if data.hole(at) { 1.0 } else { 0.0 });
                        }
                    }
                    let rows4 = inv.transpose().to_cols_array_2d();
                    let fwd = transform.transpose().to_cols_array_2d();
                    r[1..4].copy_from_slice(&rows4[..3]);
                    r[4..7].copy_from_slice(&fwd[..3]);
                    r[7] = [x0, z0, cell.x, f32::from_bits(side | rows << 16)];
                }
                SceneCollider::Cylinder {
                    transform,
                    radius,
                    half_height,
                } => {
                    r[0][0] = kind(5);
                    let inv = transform.inverse().transpose().to_cols_array_2d();
                    r[1..4].copy_from_slice(&inv[..3]);
                    r[4] = [*radius, *half_height, 0.0, 0.0];
                }
                SceneCollider::Convex { transform, planes } => {
                    r[0][0] = kind(6);
                    r[0][1] = f32::from_bits(heights.len() as u32);
                    r[0][2] = f32::from_bits(planes.len() as u32);
                    let inv = transform.inverse().transpose().to_cols_array_2d();
                    r[1..4].copy_from_slice(&inv[..3]);
                    heights.extend(planes.iter().flat_map(|p| p.to_array()));
                }
            }
            let index = records.len() as u32;
            records.push(r);
            for b in near {
                members[b].push(index);
            }
        }
    }
    // Words after the records: the lists, then the heights.
    let base = 4 + records.len() * 32;
    let mut tail: Vec<u32> = Vec::new();
    let lists = members
        .iter()
        .map(|m| {
            if m.is_empty() {
                return u32::MAX;
            }
            let at = tail.len() as u32;
            tail.push(m.len() as u32);
            tail.extend(m);
            at
        })
        .collect();
    let heights_at = (base + tail.len()) as u32;
    let mut out =
        Vec::with_capacity(1 + records.len() * 8 + (tail.len() + heights.len()).div_ceil(4));
    out.push([
        f32::from_bits(records.len() as u32),
        f32::from_bits(base as u32),
        0.0,
        0.0,
    ]);
    for mut r in records {
        if matches!(r[0][0].to_bits(), 4 | 6) {
            r[0][1] = f32::from_bits(r[0][1].to_bits() + heights_at);
        }
        if r[0][0].to_bits() == 4 {
            r[0][3] = f32::from_bits(r[0][3].to_bits() + heights_at);
        }
        out.extend(r);
    }
    let mut words: Vec<f32> = tail.into_iter().map(f32::from_bits).collect();
    words.extend(heights);
    while !words.len().is_multiple_of(4) {
        words.push(0.0);
    }
    out.extend(words.as_chunks::<4>().0.iter().copied());
    (out, lists)
}
