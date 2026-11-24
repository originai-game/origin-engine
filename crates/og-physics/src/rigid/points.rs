//! Deterministic point/solid manifolds and deferred two-way impulses.
use super::*;
use std::sync::Mutex;

pub(crate) struct Reaction {
    pub entity: u64,
    pub body: rp::RigidBodyHandle,
    pub point: Vec3,
    pub impulse: Vec3,
    pub displacement: Vec3,
}
pub(crate) type Reactions = Mutex<Vec<Reaction>>;

/// Flush only after every reader has finished. Sorting the complete records
/// makes the reduction order independent of job and lane scheduling.
pub(crate) fn apply_reactions(reactions: Reactions, bodies: &mut rp::RigidBodySet) {
    let mut records = reactions.into_inner().expect("reaction lock");
    records.sort_by_key(|r| {
        (
            r.entity,
            r.point.to_array().map(f32::to_bits),
            r.impulse.to_array().map(f32::to_bits),
            r.displacement.to_array().map(f32::to_bits),
        )
    });
    for r in records {
        if r.impulse == Vec3::ZERO && r.displacement == Vec3::ZERO {
            continue;
        }
        if let Some(rb) = bodies.get_mut(r.body)
            && rb.is_dynamic()
        {
            rb.apply_impulse_at_point(v(r.impulse), v(r.point), true);
            let mask = rb
                .mass_properties()
                .effective_inv_mass
                .map(|w| if w > 0.0 { 1.0 } else { 0.0 });
            rb.set_translation(rb.translation() + v(r.displacement) * mask, true);
        }
    }
}

pub(crate) struct PointWorld<'a> {
    pub world: &'a rp::PhysicsWorld,
    pub forces: &'a og_env::force::Forces,
    pub air: &'a og_env::air::Air,
    pub reactions: &'a Reactions,
}
pub(crate) struct Near<'a> {
    world: &'a PointWorld<'a>,
}
#[derive(Clone, Copy)]
pub(crate) struct PointContact {
    pub normal: Vec3,
    pub depth: f32,
    pub point: Vec3,
    pub velocity: Vec3,
    pub body: Option<rp::RigidBodyHandle>,
    pub entity: u64,
}
impl<'a> Near<'a> {
    pub fn new(world: &'a PointWorld<'a>) -> Self {
        Self { world }
    }
    pub fn manifold(&self, p: Vec3, radius: f32) -> Vec<PointContact> {
        let w = self.world.world;
        let pipeline = w.broad_phase.as_query_pipeline(
            w.narrow_phase.query_dispatcher(),
            &w.bodies,
            &w.colliders,
            rp::QueryFilter::default().exclude_sensors(),
        );
        // Query the predicted point, not a tick-old particle bound. There
        // is deliberately no speed cap that could make old bounds safe.
        let bounds = rp::Aabb::new(
            v(flat(p) - Vec3::splat(radius + 1e-6)),
            v(flat(p) + Vec3::splat(radius + 1e-6)),
        );
        let mut near: Vec<_> = pipeline
            .intersect_aabb_conservative(bounds)
            .filter(|(_, c)| c.collision_groups().memberships.bits() != 0)
            .map(|(h, _)| h)
            .collect();
        near.sort_by_key(|&h| {
            let c = &w.colliders[h];
            let index = c
                .parent()
                .and_then(|b| w.bodies[b].colliders().iter().position(|&x| x == h))
                .unwrap_or(0);
            (c.user_data as u64, index)
        });
        let mut contacts = Vec::new();
        for h in near {
            let c = &w.colliders[h];
            let projection = c.shape().project_point(c.position(), v(p), false);
            let q = unv(projection.point);
            let delta = flat(p) - q;
            let len = delta.length();
            let inside = (projection.is_inside && !surface_only(c))
                || heightfield_surface(c, p).is_some_and(|(q, n)| (p - q).dot(n) < 0.0)
                || (DIM == 2 && polyline_inside(c, p));
            let normal = if len > 1e-9 {
                delta * (if inside { -1.0 / len } else { 1.0 / len })
            } else {
                // At the surface, ask the shape's feature normal instead of
                // skipping contact (a resting particle has zero separation).
                let local = c.position().inverse_transform_point(v(p));
                let (projection, feature) = c.shape().project_local_point_and_get_feature(local);
                c.shape()
                    .feature_normal_at_point(projection.subshape, feature, local)
                    .map(|n| unv(c.position().rotation * n))
                    .unwrap_or(Vec3::Y)
            };
            let depth = if inside { radius + len } else { radius - len };
            if depth < -1e-6 {
                continue;
            }
            let body = c.parent();
            let velocity = body.map_or(Vec3::ZERO, |h| unv(w.bodies[h].velocity_at_point(v(q))));
            contacts.push(PointContact {
                normal,
                depth,
                point: q,
                velocity,
                body,
                entity: c.user_data as u64,
            });
        }
        contacts
    }
}

/// Winding of a closed 2D polyline in its own plane, independent of edge
/// insertion order; open polylines remain two-sided surfaces.
fn polyline_inside(c: &rp::Collider, p: Vec3) -> bool {
    let Some(line) = c.shape().as_polyline() else {
        return false;
    };
    let p = c.position().inverse_transform_point(v(p));
    let mut degree = vec![0usize; line.vertices().len()];
    let mut winding = 0i32;
    for &[a, b] in line.indices() {
        degree[a as usize] += 1;
        degree[b as usize] += 1;
        let a = line.vertices()[a as usize];
        let b = line.vertices()[b as usize];
        let side = (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y);
        if a.y <= p.y && b.y > p.y && side > 0.0 {
            winding += 1;
        }
        if a.y > p.y && b.y <= p.y && side < 0.0 {
            winding -= 1;
        }
    }
    degree.iter().all(|&n| n == 2) && winding != 0
}
impl Solids for Near<'_> {
    fn acceleration(&self, p: Vec3) -> Vec3 {
        flat(self.world.forces.at(p))
    }
    fn air(&self, p: Vec3) -> Vec3 {
        flat(self.world.air.at(p))
    }
    fn inverse_mass(&self, entity: u64) -> f32 {
        self.world
            .world
            .bodies
            .iter()
            .find(|(_, rb)| rb.user_data as u64 == entity && rb.is_dynamic())
            .map_or(0.0, |(_, rb)| {
                let mass = rb.mass();
                if mass > 0.0 { 1.0 / mass } else { 0.0 }
            })
    }
    fn reaction(&self, entity: u64, point: Vec3, impulse: Vec3, displacement: Vec3) {
        if let Some((h, _)) = self
            .world
            .world
            .bodies
            .iter()
            .find(|(_, rb)| rb.user_data as u64 == entity && rb.is_dynamic())
        {
            self.world
                .reactions
                .lock()
                .expect("reaction lock")
                .push(Reaction {
                    entity,
                    body: h,
                    point,
                    impulse,
                    displacement,
                });
        }
    }
    fn collide_point(
        &self,
        p: &mut Vec3,
        velocity: &mut Vec3,
        radius: f32,
        friction: f32,
        mass: f32,
        _dt: f32,
    ) {
        if radius <= 0.0 {
            return;
        }
        // Re-query after each projection to retain every wall in a corner.
        for _ in 0..4 {
            let contacts = self.manifold(*p, radius);
            let mut projected = false;
            for c in contacts {
                if c.depth > 0.0 {
                    *p += c.normal * c.depth;
                    projected = true;
                }
                let relative = *velocity - c.velocity;
                let along = relative.dot(c.normal);
                let next = c.velocity
                    + c.normal * along.max(0.0)
                    + (relative - c.normal * along) * (1.0 - friction);
                if mass > 0.0
                    && let Some(body) = c.body
                    && self.world.world.bodies[body].is_dynamic()
                {
                    self.world
                        .reactions
                        .lock()
                        .expect("reaction lock")
                        .push(Reaction {
                            entity: c.entity,
                            body,
                            point: c.point,
                            impulse: (*velocity - next) * mass,
                            displacement: Vec3::ZERO,
                        });
                }
                *velocity = next;
            }
            if !projected {
                break;
            }
        }
    }
    fn push_out(&self, p: Vec3, radius: f32) -> Option<(Vec3, Vec3)> {
        let mut at = p;
        let mut normal = Vec3::ZERO;
        for _ in 0..4 {
            let mut moved = false;
            for c in self.manifold(at, radius) {
                if c.depth > 0.0 {
                    at += c.normal * c.depth;
                    normal += c.normal;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        (normal.length_squared() > 0.0).then_some((at, normal.normalize_or_zero()))
    }
}
