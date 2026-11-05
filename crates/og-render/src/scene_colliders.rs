//! The scene's colliders ([`crate::RenderScene::particle_colliders`]) as a
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
//! radius), 2 capsule (reserved), 3 oriented box (centre; three unit axes
//! with their half extents in w), 4 a height field's patch under the
//! boxes (world to local rows, local to world rows, then origin x and z,
//! the cell and the samples per row and rows (u32 bits, 16:16); x of the
//! head's y: its first sample's word; z: the field's vertical scale's
//! reciprocal).

use crate::particles::ParticleCollider;
use glam::Vec3;

/// The most samples a height field's patch has a side.
const PATCH: u32 = 64;

/// `colliders` near each of `bounds` (world boxes, min and max), packed
/// as the module docs say, and each box's list (its first word: a count
/// then record indices; `u32::MAX` when nothing is near it).
pub(crate) fn pack(
    colliders: &[ParticleCollider],
    bounds: &[(Vec3, Vec3)],
) -> (Vec<[f32; 4]>, Vec<u32>) {
    let meets = |c: &ParticleCollider, (lo, hi): (Vec3, Vec3)| -> bool {
        match c {
            ParticleCollider::Plane { .. } => true,
            ParticleCollider::Sphere { center, radius } => {
                (center.clamp(lo, hi) - *center).length() <= *radius
            }
            ParticleCollider::Box {
                transform,
                half_extents,
            } => {
                let c = transform.transform_point3(Vec3::ZERO);
                let r = (0..3)
                    .map(|k| transform.col(k).truncate().abs() * half_extents[k])
                    .fold(Vec3::ZERO, |s, v| s + v);
                (c - r).cmple(hi).all() && (c + r).cmpge(lo).all()
            }
            ParticleCollider::HeightField { .. } => true,
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
        let mut r = [[0.0f32; 4]; 8];
        let kind = |k: u32| f32::from_bits(k);
        match c {
            ParticleCollider::Plane { normal, offset } => {
                let n = normal.normalize_or_zero();
                if n == Vec3::ZERO {
                    continue;
                }
                r[0][0] = kind(0);
                r[1] = n.extend(*offset / normal.length()).to_array();
            }
            ParticleCollider::Sphere { center, radius } => {
                r[0][0] = kind(1);
                r[1] = center.extend(*radius).to_array();
            }
            ParticleCollider::Box {
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
            ParticleCollider::HeightField { data, transform } => {
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
                let (x0, z0) = (lo.x.max(-half.x), lo.z.max(-half.y));
                let (x1, z1) = (hi.x.min(half.x), hi.z.min(half.y));
                if x0 >= x1 || z0 >= z1 {
                    continue;
                }
                let source_cell = (data.size.x / (data.columns.max(2) - 1) as f32)
                    .min(data.size.y / (data.rows.max(2) - 1) as f32);
                let cell = source_cell.max((x1 - x0).max(z1 - z0) / (PATCH - 1) as f32);
                let side = (((x1 - x0) / cell).ceil() as u32 + 2).min(PATCH + 1);
                let rows = (((z1 - z0) / cell).ceil() as u32 + 2).min(PATCH + 1);
                r[0] = [
                    kind(4),
                    0.0, // the first height's word, set below
                    transform.col(1).truncate().length().max(1e-6).recip(),
                    0.0,
                ];
                let first = heights.len() as u32;
                r[0][1] = f32::from_bits(first);
                let sampler = crate::TerrainSampler::of(data);
                for z in 0..rows {
                    for x in 0..side {
                        let at = glam::Vec2::new(x0 + x as f32 * cell, z0 + z as f32 * cell);
                        heights.push(sampler.height(at.clamp(-half, half)));
                    }
                }
                let rows4 = inv.transpose().to_cols_array_2d();
                let fwd = transform.transpose().to_cols_array_2d();
                r[1..4].copy_from_slice(&rows4[..3]);
                r[4..7].copy_from_slice(&fwd[..3]);
                r[7] = [x0, z0, cell, f32::from_bits(side | rows << 16)];
            }
        }
        let index = records.len() as u32;
        records.push(r);
        for b in near {
            members[b].push(index);
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
        if r[0][0].to_bits() == 4 {
            r[0][1] = f32::from_bits(r[0][1].to_bits() + heights_at);
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    #[test]
    fn a_box_far_from_every_bound_is_left_out_and_lists_name_their_records() {
        let colliders = [
            ParticleCollider::Plane {
                normal: Vec3::Y * 2.0,
                offset: 1.0,
            },
            ParticleCollider::Box {
                transform: Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)),
                half_extents: Vec3::splat(0.5),
            },
            ParticleCollider::Sphere {
                center: Vec3::new(0.0, 1.0, 0.0),
                radius: 0.5,
            },
        ];
        let bounds = [
            (Vec3::splat(-1.0), Vec3::splat(1.0)),
            (Vec3::new(9.0, -1.0, -1.0), Vec3::new(11.0, 1.0, 1.0)),
        ];
        let (words, lists) = pack(&colliders, &bounds);
        // Plane (both), box (the second), sphere (the first): 3 records.
        assert_eq!(words[0][0].to_bits(), 3);
        let base = words[0][1].to_bits() as usize;
        let word = |w: usize| words[w / 4][w % 4].to_bits();
        let list = |l: u32| -> Vec<u32> {
            let at = base + l as usize;
            (0..word(at)).map(|k| word(at + 1 + k as usize)).collect()
        };
        assert_eq!(list(lists[0]), [0, 2]);
        assert_eq!(list(lists[1]), [0, 1]);
        // The plane's normal is unit and its offset scaled with it.
        assert_eq!(words[1 + 1], [0.0, 1.0, 0.0, 0.5]);
    }
}
