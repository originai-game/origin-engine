//! Virtual character movement: recover overlaps, sweep carry, constrain the
//! desired move, then find support. Based on the constraint-plane approach in
//! Jolt CharacterVirtual (MIT); no simulation state lives outside components.

use super::{contact, flat, rp, unv, v};
use crate::{ContactHook, gather::Scene};
use og_core::glam::Vec3;

#[derive(Clone, Copy)]
pub(crate) struct Hit {
    pub handle: rp::ColliderHandle,
    pub normal: Vec3,
    pub point: Vec3,
    pub toi: f32,
}

#[derive(Default)]
pub(crate) struct Travel {
    /// Accumulated locally, never recovered by subtracting world positions.
    pub translation: Vec3,
    /// Recovery plus swept carry, separate from the controller's own motion.
    pub carried: Vec3,
    pub ground: Option<Hit>,
    pub sliding: bool,
    pub bumped: bool,
    pub crushed: bool,
    pub hits: Vec<Hit>,
}

pub(crate) struct Mover<'a> {
    pub qp: rp::QueryPipeline<'a>,
    pub shape: &'a dyn rp::Shape,
    pub rot: rp::Rotation,
    pub skin: f32,
    pub walkable: f32,
    pub step_height: f32,
    pub up: Vec3,
    pub scene: &'a Scene,
    pub entity: u64,
    pub one_way_mask: u32,
    pub dropping_through: u64,
}

pub(crate) fn collider_entity(c: &rp::Collider) -> u64 {
    c.user_data as u64
}

pub(crate) fn collider_body(c: &rp::Collider) -> u64 {
    let body = (c.user_data >> 64) as u64;
    if body == 0 { collider_entity(c) } else { body }
}

impl Mover<'_> {
    pub fn hook(&self, c: &rp::Collider) -> Option<&ContactHook> {
        self.scene
            .hooks
            .get(&collider_entity(c))
            .or_else(|| self.scene.hooks.get(&collider_body(c)))
    }

    pub fn one_way(&self, c: &rp::Collider) -> Vec3 {
        let explicit = self
            .hook(c)
            .map_or(Vec3::ZERO, |h| flat(h.one_way))
            .normalize_or_zero();
        if explicit != Vec3::ZERO {
            return explicit;
        }
        if c.collision_groups().memberships.bits() & self.one_way_mask != 0 {
            unv(c.position().rotation * v(self.up)).normalize_or_zero()
        } else {
            Vec3::ZERO
        }
    }

    fn allowed(&self, c: &rp::Collider) -> bool {
        let (e, b) = (collider_entity(c), collider_body(c));
        if self.dropping_through != 0 && self.dropping_through == e {
            return false;
        }
        let ignores = |h: &ContactHook, a, b| {
            h.ignore
                .as_slice()
                .iter()
                .any(|&x| x != 0 && (x == a || x == b))
        };
        !self
            .hook(c)
            .is_some_and(|h| ignores(h, self.entity, self.entity))
            && !self
                .scene
                .hooks
                .get(&self.entity)
                .is_some_and(|h| ignores(h, e, b))
    }

    /// Recast past rejected surfaces, including grazing hits. A grazing
    /// floor can never authorize travelling through a wall behind it.
    pub fn cast(&self, from: Vec3, dir: Vec3, dist: f32) -> Option<Hit> {
        if dist <= 0.0 {
            return None;
        }
        let mut ignored = Vec::new();
        loop {
            let predicate = |h, c: &rp::Collider| {
                !ignored.contains(&h)
                    && self.allowed(c)
                    && self.qp.filter.predicate.is_none_or(|p| p(h, c))
            };
            let qp = rp::QueryPipeline {
                filter: rp::QueryFilter {
                    predicate: Some(&predicate),
                    ..self.qp.filter
                },
                ..self.qp
            };
            let (handle, hit) = qp.cast_shape(
                &rp::Pose::from_parts(v(from), self.rot),
                v(dir),
                self.shape,
                super::ShapeCastOptions {
                    max_time_of_impact: dist,
                    target_distance: 0.0,
                    stop_at_penetration: false,
                    compute_impact_geometry_on_penetration: true,
                },
            )?;
            let n = unv(hit.normal1);
            let c = &self.qp.colliders[handle];
            let ow = self.one_way(c).normalize_or_zero();
            // A one-way surface blocks only its allowed face, approached
            // from outside. Starting underneath never catches its top.
            let side = if ow != Vec3::ZERO {
                self.shape
                    .as_support_map()
                    .map_or((from - unv(hit.witness1)).dot(ow), |shape| {
                        let at = rp::Pose::from_parts(v(from), self.rot);
                        (unv(shape.support_point(&at, v(-ow))) - unv(hit.witness1)).dot(ow)
                    })
            } else {
                0.0
            };
            if -dir.dot(n) <= 1e-4
                || (ow != Vec3::ZERO
                    && (n.dot(ow) < std::f32::consts::FRAC_1_SQRT_2
                        || dir.dot(ow) >= 0.0
                        || side < -self.skin))
            {
                ignored.push(handle);
                continue;
            }
            return Some(Hit {
                handle,
                normal: n,
                point: unv(hit.witness1),
                toi: hit.time_of_impact,
            });
        }
    }

    /// Four projected Gauss-Seidel MTD passes. Keep the recovery normals:
    /// opposed solids with residual penetration are a crush, not support.
    fn recover(&self, centre: Vec3, out: &mut Travel) {
        let mut normals: Vec<Vec3> = Vec::new();
        for _ in 0..4 {
            let at = rp::Pose::from_parts(v(centre + out.translation), self.rot);
            let mut corrections = Vec::new();
            for (h, c) in self.qp.intersect_shape(at, self.shape) {
                if !self.allowed(c) || self.one_way(c) != Vec3::ZERO {
                    continue;
                }
                if let Ok(Some(p)) = contact(&at, self.shape, c.position(), c.shape(), 0.0)
                    && p.dist < -1e-5
                {
                    corrections.push((h, -unv(p.normal1), unv(p.point2), -p.dist));
                }
            }
            if corrections.is_empty() {
                return;
            }
            corrections.sort_by(|a, b| {
                b.3.total_cmp(&a.3)
                    .then(a.0.into_raw_parts().cmp(&b.0.into_raw_parts()))
            });
            if let Some((handle, normal, point, depth)) = corrections.into_iter().next() {
                normals.push(normal);
                out.translation += normal * (depth + self.skin);
                out.hits.push(Hit {
                    handle,
                    normal,
                    point,
                    toi: 0.0,
                });
            }
        }
        let at = rp::Pose::from_parts(v(centre + out.translation), self.rot);
        out.crushed = self.qp.intersect_shape(at, self.shape).any(|(_, c)| {
            self.allowed(c)
                && self.one_way(c) == Vec3::ZERO
                && contact(&at, self.shape, c.position(), c.shape(), 0.0)
                    .ok()
                    .flatten()
                    .is_some_and(|p| {
                        p.dist < -1e-5 && normals.iter().any(|n| n.dot(-unv(p.normal1)) < -0.95)
                    })
        });
    }

    /// A bounded active plane set. Two independent planes leave a crease;
    /// a third plane stops only if it blocks that remaining degree of freedom.
    fn slide(&self, centre: Vec3, motion: Vec3, walking: bool, out: &mut Travel) -> Vec3 {
        let mut delta = Vec3::ZERO;
        let mut rem = flat(motion);
        let mut planes: Vec<Vec3> = Vec::new();
        for _ in 0..8 {
            let len = rem.length();
            if len < 1e-6 {
                break;
            }
            let dir = rem / len;
            let Some(hit) = self.cast(centre + delta, dir, len + self.skin) else {
                delta += rem;
                break;
            };
            let n = hit.normal;
            out.bumped |= dir.dot(self.up) > 0.0 && n.dot(self.up) < -1e-4;
            out.hits.push(hit);
            let advance = (hit.toi - self.skin).clamp(0.0, len);
            delta += dir * advance;
            let left = rem - dir * advance;
            if planes.iter().all(|p| p.dot(n) < 0.999) {
                planes.push(n);
            }
            let mut next = left - n * left.dot(n).min(0.0);
            let ny = n.dot(self.up);
            if walking && ny >= self.walkable && ny > 1e-4 {
                let across = left - self.up * left.dot(self.up);
                next = across - self.up * (across.dot(n) / ny);
            } else if walking && ny > 0.0 && next.dot(self.up) > left.dot(self.up).max(0.0) {
                // A steep face must not turn running into a wall climb.
                let wall = (n - self.up * ny).normalize_or_zero();
                next = left - wall * left.dot(wall).min(0.0);
            }
            for a in 0..planes.len() {
                if next.dot(planes[a]) >= -1e-5 {
                    continue;
                }
                let crease = planes[a].cross(n).normalize_or_zero();
                next = crease * left.dot(crease);
                if planes.iter().any(|p| next.dot(*p) < -1e-5) {
                    next = Vec3::ZERO;
                }
            }
            if next.dot(motion) <= 0.0 {
                break;
            }
            rem = flat(next);
        }
        delta
    }

    fn settle(&self, centre: Vec3, fall: f32, snap: f32, out: &mut Travel) -> Vec3 {
        let Some(hit) = self.cast(centre, -self.up, fall + snap + self.skin) else {
            return -self.up * fall;
        };
        let travel = (hit.toi - self.skin).max(0.0);
        if hit.normal.dot(self.up) >= self.walkable {
            out.hits.push(hit);
            out.ground = Some(hit);
            return -self.up * travel;
        }
        // A capsule touching an edge can have a steep normal. Accept only
        // the same collider's nearby top, never an unrelated floor above it.
        let horizontal = hit.normal - self.up * hit.normal.dot(self.up);
        let origin = hit.point - horizontal.normalize_or_zero() * 0.002 + self.up * 0.05;
        let ray = rp::Ray::new(v(origin), v(-self.up));
        if let Some((h, top)) = self.qp.cast_ray_and_get_normal(&ray, 0.06, true)
            && h == hit.handle
            && unv(top.normal).dot(self.up) >= self.walkable
            && (0.0..=0.06).contains(&top.time_of_impact)
        {
            out.ground = Some(Hit {
                normal: unv(top.normal),
                ..hit
            });
            out.hits.push(hit);
            return -self.up * travel;
        }
        if travel > fall {
            return -self.up * fall;
        }
        out.sliding = true;
        out.hits.push(hit);
        let down = -self.up * travel;
        down + self.slide(centre + down, -self.up * (fall - travel), true, out)
    }

    pub fn travel(
        &self,
        centre: Vec3,
        carry: Vec3,
        desired: Vec3,
        snap: f32,
        grounded: bool,
    ) -> Travel {
        let mut out = Travel::default();
        self.recover(centre, &mut out);
        let direction = carry.normalize_or_zero();
        let carry = carry - direction * out.translation.dot(direction).clamp(0.0, carry.length());
        let carried = self.slide(centre + out.translation, carry, false, &mut out);
        out.translation += carried;
        let start = centre + out.translation;
        out.carried = out.translation;
        let vertical = desired.dot(self.up);
        let across = desired - self.up * vertical;
        if vertical > 0.0 {
            // Keep vertical hits distinct from a side move's quantized pose.
            let rise = self.slide(start, self.up * vertical, false, &mut out);
            out.translation += rise;
            let horizontal = self.slide(centre + out.translation, across, true, &mut out);
            out.translation += horizontal;
            return out;
        }
        let lift = self
            .cast(start, self.up, self.skin * 3.0)
            .map_or(self.skin * 2.0, |h| {
                (h.toi - self.skin).clamp(0.0, self.skin * 2.0)
            });
        let low_start = out.translation;
        let mut low = Travel::default();
        let horizontal = self.slide(start + self.up * lift, across, true, &mut low);
        let down = self.settle(
            start + self.up * lift + horizontal,
            lift - vertical,
            snap,
            &mut low,
        );
        low.translation = self.up * lift + horizontal + down;
        let blocked_by_dynamic = low.hits.iter().any(|h| {
            self.qp.colliders[h.handle]
                .parent()
                .is_some_and(|b| self.qp.bodies[b].is_dynamic())
        });
        if grounded
            && self.step_height > lift
            && across.length_squared() > 1e-12
            && !blocked_by_dynamic
        {
            let high = self
                .cast(start, self.up, self.step_height + self.skin)
                .map_or(self.step_height, |h| {
                    (h.toi - self.skin).clamp(0.0, self.step_height)
                });
            let mut step = Travel::default();
            let h = self.slide(start + self.up * high, across, true, &mut step);
            let d = self.settle(start + self.up * high + h, high - vertical, snap, &mut step);
            step.translation = self.up * high + h + d;
            if step.ground.is_some_and(|g| {
                !self.qp.colliders[g.handle]
                    .parent()
                    .is_some_and(|b| self.qp.bodies[b].is_dynamic())
            }) && step.translation.dot(across) > low.translation.dot(across) + 1e-5
                && step.translation.dot(self.up) <= self.step_height
            {
                low = step;
            }
        }
        out.translation = low_start + low.translation;
        out.ground = low.ground;
        out.sliding = low.sliding;
        out.hits.extend(low.hits);
        out
    }

    pub fn travel_free(&self, centre: Vec3, carry: Vec3, desired: Vec3) -> Travel {
        let mut out = Travel::default();
        self.recover(centre, &mut out);
        let direction = carry.normalize_or_zero();
        let carry = carry - direction * out.translation.dot(direction).clamp(0.0, carry.length());
        let carried = self.slide(centre + out.translation, carry, false, &mut out);
        out.translation += carried;
        let moved = self.slide(centre + out.translation, desired, false, &mut out);
        out.carried = out.translation;
        out.translation += moved;
        out
    }
}
