//! What a loose island at rest stands on.

use super::pose::Pose;
use crate::destruction::{FractureAsset, Support};
use glam::Vec3;

/// The supports of loose island `chunks` (its body `body`, at rest): for
/// each chunk, what a ray down from its centre meets within its depth
/// below it and 5 cm more, other than the island itself.
pub fn supports_of(
    q: &crate::PhysicsQuery,
    a: &FractureAsset,
    pose: &[Option<Pose>],
    chunks: &[u32],
    body: u64,
    gravity: Vec3,
) -> Vec<Support> {
    let down = gravity.normalize_or_zero();
    if down == Vec3::ZERO {
        return Vec::new();
    }
    let mut out = Vec::new();
    for &c in chunks {
        let Some(cp) = pose[c as usize] else {
            continue;
        };
        let ch = &a.chunks[c as usize];
        let depth = ch
            .hull
            .iter()
            .map(|h| (cp.1 * (cp.2 * *h)).dot(down))
            .fold(0.0, f32::max);
        let hits = q
            .raycast_all(&crate::RaycastAllDesc {
                ray: crate::RaycastDesc {
                    origin: cp.0,
                    direction: down,
                    max_distance: depth + 0.05,
                    filter: crate::QueryFilter::default(),
                },
                max_hits: 16,
            })
            .unwrap_or_default();
        let Some(hit) = hits.iter().find(|h| h.body.to_bits() != body) else {
            continue;
        };
        let s = cp.2.max_element().max(1e-6);
        let r = crate::destruction::cube_root(ch.volume);
        out.push(Support {
            chunk: c,
            point: ch.centroid + cp.1.inverse() * (hit.point - cp.0) / s,
            area: r * r,
        });
    }
    out
}
