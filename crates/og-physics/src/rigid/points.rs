//! Solid shapes near a set of points: what ropes, cloth and fluids collide with.

use super::*;

/// The solid colliders whose boxes meet a region, for points that collide.
pub(crate) struct Near<'a> {
    colliders: &'a rp::ColliderSet,
    near: Vec<rp::ColliderHandle>,
}

impl<'a> Near<'a> {
    /// Every solid collider whose box (grown by `margin`) meets `min`..`max`,
    /// in collider-set order.
    pub fn around(colliders: &'a rp::ColliderSet, min: Vec3, max: Vec3, margin: f32) -> Near<'a> {
        let lo = flat(min) - Vec3::splat(margin);
        let hi = flat(max) + Vec3::splat(margin);
        let near = colliders
            .iter()
            .filter(|(_, c)| !c.is_sensor() && c.collision_groups().memberships.bits() != 0)
            .filter(|(_, c)| {
                let b = c.compute_aabb();
                let (bmin, bmax) = (unv(b.mins), unv(b.maxs));
                (0..DIM).all(|k| bmin[k] <= hi[k] && bmax[k] >= lo[k])
            })
            .map(|(h, _)| h)
            .collect();
        Near { colliders, near }
    }
}

impl Solids for Near<'_> {
    fn push_out(&self, p: Vec3, radius: f32) -> Option<(Vec3, Vec3)> {
        let mut at = p;
        let mut hit = None;
        for &h in &self.near {
            let c = &self.colliders[h];
            if let Some((q, n)) = heightfield_surface(c, at) {
                // Over a height field: below its surface (or within
                // `radius` above it) goes up to `radius` above it.
                let depth = (at - q).dot(n);
                if depth < radius {
                    at += n * (radius - depth);
                    hit = Some(n);
                }
                continue;
            }
            if c.shape().as_heightfield().is_some() {
                continue;
            }
            // A mesh level is most of a cloth's or rope's step: the nearest
            // of its triangles to a point in the air is found only to be
            // left alone (`len >= radius`). Bounded at `radius`, the search
            // stops at the BVH's first boxes; the triangle it finds within
            // it is the same, so the step's arithmetic is unchanged.
            let proj = if surface_only(c) {
                match c
                    .shape()
                    .project_point_with_max_dist(c.position(), v(at), false, radius)
                {
                    Some(proj) => proj,
                    None => continue,
                }
            } else {
                c.shape().project_point(c.position(), v(at), false)
            };
            let q = unv(proj.point);
            let d = flat(at) - q;
            let len = d.length();
            let n = if proj.is_inside {
                if len > 1e-9 { -d / len } else { Vec3::Y }
            } else if len < radius && len > 1e-9 {
                d / len
            } else {
                continue;
            };
            let moved = q + n * radius;
            at = if DIM == 2 {
                Vec3::new(moved.x, moved.y, p.z)
            } else {
                moved
            };
            hit = Some(n);
        }
        hit.map(|n| (at, n))
    }
}
