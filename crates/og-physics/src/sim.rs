//! The step and the scene queries, written once and compiled for each
//! profile: `dim3.rs` and `dim2.rs` include this file as their `sim` module,
//! with `rp` naming rapier3d's or rapier2d's prelude and the conversions
//! (`v`, `pose`, `unpose`, `angvel`, ...) defined beside it.

use super::{
    PROFILE_NAME, ShapeCastOptions, angvel, cache, flat, from_mesh, pose, primitive, rp, unangvel,
    unpose, unv, v,
};
use crate::gather::{Body, Scene};
use crate::schema::*;
use crate::{OverlapDesc, ShapeDesc};
use crate::{
    QueryFilter, RaycastHit, SHAPE_CACHE_LIMIT, ShapeCastDesc, ShapeCastHit, ShapeKey, Shared,
    StepOut,
};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, Result};
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(what: &str, value: f32) -> Error {
    Error::new(
        ErrorKind::Invalid,
        format!("og.Collider.{what} must be finite and at least 0, got {value}"),
    )
}

/// The shape of a collider at a transform scale: primitives are built, mesh
/// shapes come from the cache (or are built into it).
fn shape(c: &Collider, scale: Vec3, shared: &mut Shared) -> Result<rp::SharedShape> {
    for (what, x) in [("radius", c.radius), ("height", c.height)] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x));
        }
    }
    for x in c.size.to_array() {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid("size", x));
        }
    }
    let s = scale.abs();
    if !s.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Transform.scale must be finite, got {scale}"),
        ));
    }
    if let Some(shape) = primitive(c, s)? {
        return Ok(shape);
    }
    let key = ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: s.to_array().map(f32::to_bits),
    };
    if let Some(shape) = cache(shared).get(&key) {
        return Ok(shape.clone());
    }
    let mesh = shared.meshes.get(&c.mesh).cloned().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "og.Collider.mesh {} was not registered with PhysicsSystem::set_mesh",
                c.mesh
            ),
        )
    })?;
    let built = from_mesh(c.shape, &mesh, s).map_err(|e| e.context(format!("mesh {}", c.mesh)))?;
    let cached = cache(shared);
    if cached.len() >= SHAPE_CACHE_LIMIT {
        cached.clear();
    }
    cached.insert(key, built.clone());
    Ok(built)
}

fn groups(c: &Collider) -> rp::InteractionGroups {
    rp::InteractionGroups::new(
        rp::Group::from_bits_retain(c.layers),
        rp::Group::from_bits_retain(c.mask),
        rp::InteractionTestMode::And,
    )
}

/// A collider builder at `local` (relative to its body, or the world pose of
/// a static collider). `body_mass` > 0 overrides the density.
fn collider(
    c: &Collider,
    scale: Vec3,
    local: rp::Pose,
    entity: Entity,
    body_mass: f32,
    shared: &mut Shared,
) -> Result<rp::ColliderBuilder> {
    for (what, x) in [
        ("friction", c.friction),
        ("restitution", c.restitution),
        ("density", c.density),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(invalid(what, x).context(format!("entity {entity}")));
        }
    }
    let shape = shape(c, scale, shared).map_err(|e| e.context(format!("entity {entity}")))?;
    let mut b = rp::ColliderBuilder::new(shape)
        .position(local)
        .friction(c.friction)
        .restitution(c.restitution)
        .sensor(c.sensor != 0)
        .collision_groups(groups(c))
        .user_data(entity.to_bits() as u128);
    b = if body_mass > 0.0 {
        b.mass(body_mass)
    } else {
        b.density(c.density)
    };
    if c.sensor != 0 {
        // A sensor also notices kinematic and static colliders.
        b = b.active_collision_types(rp::ActiveCollisionTypes::all());
    }
    Ok(b)
}

fn local_pose(c: &Collider, scale: Vec3) -> rp::Pose {
    pose(c.offset * scale, c.rotation)
}

/// The overlap (m) below which a slow, supported body counts as resting;
/// deeper, it keeps moving so the solver can push it out.
const REST_PENETRATION: f32 = 0.002;

/// `v` without the components its body's `lock` bits forbid.
fn unlocked(v: Velocity, lock: u32) -> Velocity {
    let keep = |bit: u32, x: f32| if lock & bit != 0 { 0.0 } else { x };
    Velocity {
        linear: Vec3::new(
            keep(LOCK_TRANSLATION_X, v.linear.x),
            keep(LOCK_TRANSLATION_Y, v.linear.y),
            keep(LOCK_TRANSLATION_Z, v.linear.z),
        ),
        angular: Vec3::new(
            keep(LOCK_ROTATION_X, v.angular.x),
            keep(LOCK_ROTATION_Y, v.angular.y),
            keep(LOCK_ROTATION_Z, v.angular.z),
        ),
    }
}

fn slow(v: &Velocity, limit: f32) -> bool {
    v.linear.length_squared() < limit * limit && v.angular.length_squared() < limit * limit
}

fn entity_of(c: &rp::Collider) -> Entity {
    Entity::from_bits(c.user_data as u64)
}

/// A body as the last tick left it: its rapier handle and the component
/// values physics last read or wrote. A component that no longer equals
/// these was changed by something else, and the rapier body follows it.
#[derive(Clone, Serialize, Deserialize)]
struct Tracked {
    handle: rp::RigidBodyHandle,
    body: RigidBody,
    collider: Option<Collider>,
    scale: Vec3,
    position: Vec3,
    quaternion: Quat,
    velocity: Option<Velocity>,
}

/// A static collider (no body) as the last tick left it.
#[derive(Clone, Serialize, Deserialize)]
struct TrackedStatic {
    handle: rp::ColliderHandle,
    collider: Collider,
    scale: Vec3,
    position: Vec3,
    quaternion: Quat,
}

/// The rapier world kept between ticks (contacts, warm starts, broad
/// phase), with what each entity's components were when physics last
/// touched them. Saved whole in the world's `og.physics` state block.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Sim {
    world: rp::PhysicsWorld,
    bodies: BTreeMap<u64, Tracked>,
    statics: BTreeMap<u64, TrackedStatic>,
}

fn body_kind(b: &Body) -> Result<rp::RigidBodyType> {
    let e = b.entity;
    let kind = match b.body.kind {
        BODY_DYNAMIC => rp::RigidBodyType::Dynamic,
        BODY_STATIC => rp::RigidBodyType::Fixed,
        BODY_KINEMATIC => rp::RigidBodyType::KinematicVelocityBased,
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "entity {e}: og.RigidBody.kind {other} is not a kind (0 dynamic, 1 static, 2 kinematic)"
                ),
            ));
        }
    };
    for (what, x) in [
        ("mass", b.body.mass),
        ("linearDamping", b.body.linear_damping),
        ("angularDamping", b.body.angular_damping),
    ] {
        if !(x >= 0.0 && x.is_finite()) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("entity {e}: og.RigidBody.{what} must be finite and at least 0, got {x}"),
            ));
        }
    }
    Ok(kind)
}

/// Adds `b` to the rapier world at its transform and velocity.
fn insert_body(
    w: &mut rp::PhysicsWorld,
    b: &Body,
    shared: &mut Shared,
) -> Result<rp::RigidBodyHandle> {
    let (e, body) = (b.entity, &b.body);
    let kind = body_kind(b)?;
    let collider = match &b.collider {
        Some(c) => Some(collider(
            c,
            b.scale,
            local_pose(c, b.scale),
            e,
            body.mass,
            shared,
        )?),
        None => None,
    };
    let mut builder = rp::RigidBodyBuilder::new(kind)
        .pose(pose(b.position, b.quaternion))
        .linear_damping(body.linear_damping)
        .angular_damping(body.angular_damping)
        .gravity_scale(body.gravity_scale)
        .locked_axes(rp::LockedAxes::from_bits_truncate((body.lock & 63) as u8))
        .ccd_enabled(body.ccd != 0)
        // Resting is decided by the rest rule below, not by rapier's sleep.
        .can_sleep(false)
        .user_data(e.to_bits() as u128);
    if kind != rp::RigidBodyType::Fixed {
        let vel = unlocked(b.velocity.unwrap_or_default(), body.lock);
        builder = builder
            .linvel(v(flat(vel.linear)))
            .angvel(angvel(vel.angular));
    }
    if collider.is_none() && kind == rp::RigidBodyType::Dynamic {
        builder = builder.additional_mass(if body.mass > 0.0 { body.mass } else { 1.0 });
    }
    let h = w.bodies.insert(builder);
    if let Some(c) = collider {
        w.colliders.insert_with_parent(c, h, &mut w.bodies);
    }
    Ok(h)
}

/// Brings the persistent rapier world in line with `scene`, steps it
/// `substeps` times and reads the dynamic bodies and the touching pairs
/// back. On an error `sim` is left half-updated; the caller drops it.
pub(crate) fn step(
    settings: &Physics,
    dt: f32,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<StepOut> {
    let substeps = settings.substeps.clamp(1, 16);
    let Sim {
        world: w,
        bodies: tracked,
        statics: tracked_statics,
    } = sim;
    w.gravity = v(settings.gravity);
    w.integration_parameters.dt = dt / substeps as f32;
    w.integration_parameters.num_solver_iterations =
        settings.solver_iterations.clamp(1, 32) as usize;

    // What is gone leaves first, in entity order.
    let live: BTreeSet<u64> = scene.bodies.iter().map(|b| b.entity.to_bits()).collect();
    tracked.retain(|e, t| {
        let keep = live.contains(e);
        if !keep {
            w.remove_body(t.handle);
        }
        keep
    });
    let live: BTreeSet<u64> = scene.statics.iter().map(|s| s.entity.to_bits()).collect();
    tracked_statics.retain(|e, t| {
        let keep = live.contains(e);
        if !keep {
            w.remove_collider(t.handle);
        }
        keep
    });

    let stale = std::mem::take(&mut shared.stale_meshes);
    let stale_mesh =
        |c: &Collider| matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH) && stale.contains(&c.mesh);

    // Pre-step poses of the dynamic bodies, for the rest rule.
    let mut before = Vec::new();
    let mut handles = Vec::with_capacity(scene.bodies.len());
    for b in &scene.bodies {
        let key = b.entity.to_bits();
        let h = match tracked.get(&key) {
            Some(t)
                if t.body == b.body
                    && t.collider == b.collider
                    && t.scale == b.scale
                    && !b.collider.as_ref().is_some_and(stale_mesh) =>
            {
                let kind = b.body.kind;
                let rb = &mut w.bodies[t.handle];
                if kind == BODY_KINEMATIC
                    || t.position != b.position
                    || t.quaternion != b.quaternion
                {
                    rb.set_position(pose(b.position, b.quaternion), true);
                }
                if kind == BODY_KINEMATIC || (kind == BODY_DYNAMIC && t.velocity != b.velocity) {
                    let vel = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
                    rb.set_linvel(v(flat(vel.linear)), true);
                    rb.set_angvel(angvel(vel.angular), true);
                }
                t.handle
            }
            // New, or its body, collider or scale changed: built afresh.
            _ => {
                if let Some(t) = tracked.remove(&key) {
                    w.remove_body(t.handle);
                }
                insert_body(w, b, shared)?
            }
        };
        if b.body.kind == BODY_DYNAMIC {
            before.push(*w.bodies[h].position());
        }
        handles.push(h);
    }
    for s in &scene.statics {
        let key = s.entity.to_bits();
        let at = pose(s.position, s.quaternion) * local_pose(&s.collider, s.scale);
        match tracked_statics.get_mut(&key) {
            Some(t)
                if t.collider == s.collider && t.scale == s.scale && !stale_mesh(&s.collider) =>
            {
                if t.position != s.position || t.quaternion != s.quaternion {
                    w.colliders[t.handle].set_position(at);
                    t.position = s.position;
                    t.quaternion = s.quaternion;
                }
            }
            _ => {
                if let Some(t) = tracked_statics.remove(&key) {
                    w.remove_collider(t.handle);
                }
                let c = collider(&s.collider, s.scale, at, s.entity, 0.0, shared)?;
                let handle = w.colliders.insert(c);
                tracked_statics.insert(
                    key,
                    TrackedStatic {
                        handle,
                        collider: s.collider,
                        scale: s.scale,
                        position: s.position,
                        quaternion: s.quaternion,
                    },
                );
            }
        }
    }

    for _ in 0..substeps {
        w.step();
    }

    let mut touching = Vec::new();
    for pair in w.narrow_phase.contact_pairs() {
        if !pair.has_any_active_contact() {
            continue;
        }
        let (c1, c2) = (&w.colliders[pair.collider1], &w.colliders[pair.collider2]);
        let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
        let (mut point, mut normal) = (Vec3::ZERO, Vec3::ZERO);
        if let Some((manifold, contact)) = pair.find_deepest_contact() {
            point = unv(c1.position().transform_point(contact.local_p1));
            normal = unv(manifold.data.normal);
        }
        let (a, b, normal) = if e1 < e2 {
            (e1, e2, normal)
        } else {
            (e2, e1, -normal)
        };
        touching.push((Contact { a, b, sensor: 0 }, point, normal));
    }
    for (h1, h2, intersecting) in w.narrow_phase.intersection_pairs() {
        if !intersecting {
            continue;
        }
        let (c1, c2) = (&w.colliders[h1], &w.colliders[h2]);
        let (mut a, mut b) = (
            (entity_of(c1).to_bits(), c1.is_sensor()),
            (entity_of(c2).to_bits(), c2.is_sensor()),
        );
        if a.0 > b.0 {
            std::mem::swap(&mut a, &mut b);
        }
        let sensor = u32::from(a.1) | (u32::from(b.1) << 1);
        touching.push((
            Contact {
                a: a.0,
                b: b.0,
                sensor,
            },
            Vec3::ZERO,
            Vec3::ZERO,
        ));
    }
    touching.sort_by_key(|t| t.0);

    // The deepest penetration of each body that has a solid contact, by
    // body handle index.
    let mut deepest: Vec<Option<f32>> = Vec::new();
    for pair in w.narrow_phase.contact_pairs() {
        if !pair.has_any_active_contact() {
            continue;
        }
        let depth = pair
            .manifolds()
            .iter()
            .flat_map(|m| m.points.iter().map(|p| p.dist))
            .fold(f32::INFINITY, f32::min);
        for c in [pair.collider1, pair.collider2] {
            if let Some(parent) = w.colliders[c].parent() {
                let i = parent.into_raw_parts().0 as usize;
                if i >= deepest.len() {
                    deepest.resize(i + 1, None);
                }
                deepest[i] = Some(deepest[i].map_or(depth, |x| x.min(depth)));
            }
        }
    }
    let rest = settings.rest_speed;
    let mut before = before.into_iter();
    let mut moved = Vec::with_capacity(scene.bodies.len());
    for (b, &h) in scene.bodies.iter().zip(&handles) {
        let out = (b.body.kind == BODY_DYNAMIC).then(|| {
            let pre = before.next().expect("one pre-step pose per dynamic body");
            let rb = &mut w.bodies[h];
            let velocity = unlocked(
                Velocity {
                    linear: unv(rb.linvel()),
                    angular: unangvel(rb.angvel()),
                },
                b.body.lock,
            );
            // Resting: supported with at most REST_PENETRATION of overlap,
            // and slow before and after the step. It keeps its pose and
            // loses its residual velocity, in the components and in rapier.
            let prior = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
            let supported = deepest
                .get(h.into_raw_parts().0 as usize)
                .copied()
                .flatten()
                .is_some_and(|d| d >= -REST_PENETRATION);
            if rest > 0.0 && supported && slow(&prior, rest) && slow(&velocity, rest) {
                rb.set_position(pre, false);
                rb.set_linvel(v(Vec3::ZERO), false);
                rb.set_angvel(angvel(Vec3::ZERO), false);
                return (b.position, b.quaternion, Velocity::default());
            }
            let (position, quaternion) = unpose(rb.position(), b.position);
            (position, quaternion, velocity)
        });
        let (position, quaternion, velocity) = match out {
            Some((p, q, vel)) => (p, q, Some(vel)),
            None => (b.position, b.quaternion, b.velocity),
        };
        tracked.insert(
            b.entity.to_bits(),
            Tracked {
                handle: h,
                body: b.body,
                collider: b.collider,
                scale: b.scale,
                position,
                quaternion,
                velocity,
            },
        );
        moved.push(out);
    }
    Ok(StepOut { moved, touching })
}

/// The colliders of a scene, placed for queries (no bodies, no stepping).
pub(crate) struct QueryWorld {
    bodies: rp::RigidBodySet,
    colliders: rp::ColliderSet,
    broad: rp::BroadPhaseBvh,
    narrow: rp::NarrowPhase,
}

fn keep(c: &rp::Collider, f: &QueryFilter) -> bool {
    (f.mask == 0 || c.collision_groups().memberships.bits() & f.mask != 0)
        && f.exclude.is_none_or(|e| e.to_bits() != c.user_data as u64)
}

fn query_shape(d: &ShapeDesc, shared: &mut Shared) -> Result<rp::SharedShape> {
    shape(&d.collider(), Vec3::ONE, shared)
}

impl QueryWorld {
    pub(crate) fn build(scene: &Scene, shared: &mut Shared) -> Result<QueryWorld> {
        let params = rp::IntegrationParameters::default();
        let mut q = QueryWorld {
            bodies: rp::RigidBodySet::new(),
            colliders: rp::ColliderSet::new(),
            broad: rp::BroadPhaseBvh::new(),
            narrow: rp::NarrowPhase::new(),
        };
        let mut placed = Vec::with_capacity(scene.bodies.len() + scene.statics.len());
        for b in &scene.bodies {
            if let Some(c) = &b.collider {
                placed.push((b.entity, b.position, b.quaternion, b.scale, c));
            }
        }
        for s in &scene.statics {
            placed.push((s.entity, s.position, s.quaternion, s.scale, &s.collider));
        }
        for (e, position, quaternion, scale, c) in placed {
            let at = pose(position, quaternion) * local_pose(c, scale);
            let built = collider(c, scale, at, e, 0.0, shared)?.build();
            let aabb = built.compute_aabb();
            let h = q.colliders.insert(built);
            q.broad.set_aabb(&params, h, aabb);
        }
        Ok(q)
    }

    fn with<T>(&self, f: &QueryFilter, run: impl FnOnce(&rp::QueryPipeline<'_>) -> T) -> T {
        let predicate = |_: rp::ColliderHandle, c: &rp::Collider| keep(c, f);
        let mut filter = rp::QueryFilter::new().predicate(&predicate);
        if !f.include_sensors {
            filter = filter.exclude_sensors();
        }
        let pipeline = self.broad.as_query_pipeline(
            self.narrow.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            filter,
        );
        run(&pipeline)
    }

    pub(crate) fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max: f32,
        f: &QueryFilter,
    ) -> Result<Option<RaycastHit>> {
        let dir = crate::direction(flat(direction), "RaycastDesc", PROFILE_NAME)?;
        let ray = rp::Ray::new(v(origin), v(dir));
        Ok(self.with(f, |p| {
            p.cast_ray_and_get_normal(&ray, max, true)
                .map(|(h, hit)| RaycastHit {
                    entity: entity_of(&self.colliders[h]),
                    distance: hit.time_of_impact,
                    point: unv(ray.point_at(hit.time_of_impact)),
                    normal: unv(hit.normal),
                })
        }))
    }

    pub(crate) fn shape_cast(
        &self,
        d: &ShapeCastDesc,
        max: f32,
        shared: &mut Shared,
    ) -> Result<Option<ShapeCastHit>> {
        let dir = crate::direction(flat(d.direction), "ShapeCastDesc", PROFILE_NAME)?;
        let shape = query_shape(&d.shape, shared)?;
        let at = pose(d.position, d.rotation);
        let options = ShapeCastOptions::with_max_time_of_impact(max);
        Ok(self.with(&d.filter, |p| {
            p.cast_shape(&at, v(dir), shape.as_ref(), options)
                .map(|(h, hit)| ShapeCastHit {
                    entity: entity_of(&self.colliders[h]),
                    distance: hit.time_of_impact,
                    point: unv(hit.witness1),
                    normal: unv(hit.normal1),
                })
        }))
    }

    pub(crate) fn overlap(&self, d: &OverlapDesc, shared: &mut Shared) -> Result<Vec<Entity>> {
        let shape = query_shape(&d.shape, shared)?;
        let at = pose(d.position, d.rotation);
        let mut hits = self.with(&d.filter, |p| {
            p.intersect_shape(at, shape.as_ref())
                .map(|(_, c)| entity_of(c))
                .collect::<Vec<_>>()
        });
        hits.sort_by_key(|e| e.to_bits());
        hits.dedup();
        Ok(hits)
    }
}
