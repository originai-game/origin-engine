//! The step and the scene queries, written once and compiled for each
//! profile: `dim3.rs` and `dim2.rs` include this file as their `sim` module,
//! with `rp` naming rapier3d's or rapier2d's prelude and the conversions
//! (`v`, `pose`, `unpose`, `angvel`, ...) defined beside it.

use super::{
    PROFILE_NAME, ShapeCastOptions, angvel, cache, flat, from_heightfield, from_mesh, pose,
    primitive, rp, unangvel, unpose, unv, v,
};
use super::{VehicleState, ext};
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
    if c.shape == SHAPE_HEIGHTFIELD {
        let field = shared.heightfields.get(&c.mesh).cloned().ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "og.Collider.mesh {} is not a height field registered with PhysicsSystem::set_heightfield",
                    c.mesh
                ),
            )
        })?;
        let built = from_heightfield(&field, s)?;
        let cached = cache(shared);
        if cached.len() >= SHAPE_CACHE_LIMIT {
            cached.clear();
        }
        cached.insert(key, built.clone());
        return Ok(built);
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

/// The contact pairs of one lane task when the step reads them back.
const PAIR_GRAIN: usize = 512;

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
pub(crate) struct Tracked {
    pub(crate) handle: rp::RigidBodyHandle,
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
    pub(crate) world: rp::PhysicsWorld,
    pub(crate) bodies: BTreeMap<u64, Tracked>,
    statics: BTreeMap<u64, TrackedStatic>,
    /// The fixed body world-anchored joints attach to.
    pub(crate) ground: Option<rp::RigidBodyHandle>,
    pub(crate) joints: BTreeMap<u64, ext::TrackedJoint>,
    pub(crate) vehicles: BTreeMap<u64, VehicleState>,
    pub(crate) ropes: BTreeMap<u64, crate::verlet::RopeState>,
    pub(crate) cloths: BTreeMap<u64, crate::verlet::ClothState>,
    pub(crate) fluids: BTreeMap<u64, crate::fluid::FluidState>,
    pub(crate) surfaces: BTreeMap<u64, crate::water::SurfaceState>,
    pub(crate) lines: BTreeMap<u64, crate::water::LineState>,
    pub(crate) ragdolls: BTreeMap<u64, super::RagdollState>,
    pub(crate) fluids3: BTreeMap<u64, crate::fluid3d::Fluid3State>,
}

/// Every body's pose at the start of the tick, by handle index (with the
/// handle, so a slot reused by a rebuilt body is not mistaken for it): how
/// far a platform moved this tick.
#[derive(Default)]
pub(crate) struct BodyPoses(Vec<Option<(rp::RigidBodyHandle, rp::Pose)>>);

impl BodyPoses {
    fn record(w: &rp::PhysicsWorld) -> BodyPoses {
        let mut poses = Vec::new();
        for (h, rb) in w.bodies.iter() {
            let i = h.into_raw_parts().0 as usize;
            if i >= poses.len() {
                poses.resize(i + 1, None);
            }
            poses[i] = Some((h, *rb.position()));
        }
        BodyPoses(poses)
    }

    pub(crate) fn get(&self, h: rp::RigidBodyHandle) -> Option<&rp::Pose> {
        match self.0.get(h.into_raw_parts().0 as usize)? {
            Some((handle, pose)) if *handle == h => Some(pose),
            _ => None,
        }
    }
}

fn body_kind(b: &Body) -> Result<rp::RigidBodyType> {
    let e = b.entity;
    let kind = match b.body.kind {
        _ if b.control.is_some() => rp::RigidBodyType::KinematicPositionBased,
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
///
/// All of it runs where og-lanes puts it: from [`crate::BODY_GRAIN`] bodies
/// for every lane thread, on the lanes pool (natively and in the lanes
/// wasm), where rapier's `parallel` build splits its broad phase, narrow
/// phase and solver and the contact readout splits by pairs; below that,
/// and with lanes off, on the calling thread, where rapier finds one thread
/// and runs serially. Rapier computes the same bits on any number of
/// threads (`enhanced-determinism`, its solver's layout fixed for eight
/// workers whatever the pool), so the world is the same either way.
pub(crate) fn step(
    settings: &Physics,
    dt: f32,
    tick: u64,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<StepOut> {
    // The bodies this tick steps, or last tick's when ragdolls and vehicles
    // added their own.
    let bodies = scene.bodies.len().max(sim.world.bodies.len());
    og_lanes::install(
        og_lanes::Site::Physics,
        bodies,
        crate::BODY_GRAIN,
        |split| step_here(settings, dt, tick, scene, shared, sim, split),
    )
}

/// [`step`] on the thread `install` chose; `split` when that is the pool.
fn step_here(
    settings: &Physics,
    dt: f32,
    tick: u64,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
    split: bool,
) -> Result<StepOut> {
    let substeps = settings.substeps.clamp(1, 16);
    let mut out = StepOut::default();
    // Poses before this tick moved anything (game code teleporting
    // kinematic platforms included): how far each body moved this tick.
    let poses = BodyPoses::record(&sim.world);
    let handles = sync(settings, dt, scene, shared, sim)?;
    ext::sync_joints(scene, sim, &handles)?;
    super::vehicles(scene, dt, sim, &handles, &mut out)?;
    super::ragdolls(scene, dt, sim, shared, &mut out)?;
    ext::water(scene, settings, dt, sim, &handles, &mut out)?;
    let pre = rest_poses(scene, sim, &handles);

    for _ in 0..substeps {
        sim.world.step();
    }

    let (touching, moved) = after_step(settings, scene, sim, &handles, pre, split);
    out.touching = touching;
    out.moved = moved;
    ext::characters(scene, settings, dt, sim, &poses, &mut out)?;
    super::platformers(scene, settings, dt, sim, &poses, &mut out)?;
    ext::soft(scene, settings, dt, tick, sim, &mut out)?;
    ext::joint_values(sim, &mut out);
    for (e, j) in &out.joints {
        ext::joint_written(sim, *e, j);
    }
    Ok(out)
}

/// The dynamic bodies' poses before the step, for the rest rule.
fn rest_poses(
    scene: &Scene,
    sim: &Sim,
    handles: &BTreeMap<u64, rp::RigidBodyHandle>,
) -> Vec<rp::Pose> {
    scene
        .bodies
        .iter()
        .filter(|b| b.body.kind == BODY_DYNAMIC && b.control.is_none())
        .map(|b| *sim.world.bodies[handles[&b.entity.to_bits()]].position())
        .collect()
}

/// Brings the persistent rapier world in line with `scene` (bodies and
/// static colliders); returns each body's handle by entity.
fn sync(
    settings: &Physics,
    dt: f32,
    scene: &Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<BTreeMap<u64, rp::RigidBodyHandle>> {
    let substeps = settings.substeps.clamp(1, 16);
    let Sim {
        world: w,
        bodies: tracked,
        statics: tracked_statics,
        ..
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
    let stale_mesh = |c: &Collider| {
        matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
            && stale.contains(&c.mesh)
    };

    let mut handles = BTreeMap::new();
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
                if b.control.is_some() {
                    // A controller moved it after the last step: the step
                    // takes it there, pushing what is in the way.
                    rb.set_next_kinematic_position(pose(b.position, b.quaternion));
                } else if kind == BODY_KINEMATIC
                    || t.position != b.position
                    || t.quaternion != b.quaternion
                {
                    rb.set_position(pose(b.position, b.quaternion), true);
                }
                if b.control.is_none()
                    && (kind == BODY_KINEMATIC
                        || (kind == BODY_DYNAMIC && t.velocity != b.velocity))
                {
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
        handles.insert(key, h);
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

    // Bodies that are no longer bodies (now static colliders or gone) were
    // removed above; a body re-made this tick keeps its joints out until
    // they are rebuilt.
    Ok(handles)
}

/// The touching pairs and the rest rule after the step: what each dynamic
/// body's component values become.
#[allow(clippy::type_complexity)]
fn after_step(
    settings: &Physics,
    scene: &Scene,
    sim: &mut Sim,
    handles: &BTreeMap<u64, rp::RigidBodyHandle>,
    before: Vec<rp::Pose>,
    split: bool,
) -> (
    Vec<(Contact, Vec3, Vec3)>,
    Vec<Option<(Vec3, Quat, Option<Velocity>)>>,
) {
    let Sim {
        world: w,
        bodies: tracked,
        ..
    } = sim;
    // The solid pairs: each one's contact (its entities in order, the
    // deepest point and normal), its deepest penetration and the bodies it
    // touches, by pair on the lanes when the step is on them; in pair order
    // either way.
    let pairs: Vec<&rp::ContactPair> = w
        .narrow_phase
        .contact_pairs()
        .filter(|pair| pair.has_any_active_contact())
        .collect();
    let grain = if split { PAIR_GRAIN } else { pairs.len() };
    let colliders = &w.colliders;
    let read = og_lanes::map(og_lanes::Site::Physics, &pairs, grain, |pair| {
        let (c1, c2) = (&colliders[pair.collider1], &colliders[pair.collider2]);
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
        let depth = pair
            .manifolds()
            .iter()
            .flat_map(|m| m.points.iter().map(|p| p.dist))
            .fold(f32::INFINITY, f32::min);
        let body = |c: &rp::Collider| c.parent().map(|p| p.into_raw_parts().0 as usize);
        (
            (Contact { a, b, sensor: 0 }, point, normal),
            depth,
            [body(c1), body(c2)],
        )
    });
    // The deepest penetration of each body that has a solid contact, by
    // body handle index.
    let mut deepest: Vec<Option<f32>> = Vec::new();
    let mut touching = Vec::with_capacity(read.len());
    for (contact, depth, parents) in read {
        touching.push(contact);
        for i in parents.into_iter().flatten() {
            if i >= deepest.len() {
                deepest.resize(i + 1, None);
            }
            deepest[i] = Some(deepest[i].map_or(depth, |x| x.min(depth)));
        }
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

    let rest = settings.rest_speed;
    let mut before = before.into_iter();
    let mut moved = Vec::with_capacity(scene.bodies.len());
    for b in &scene.bodies {
        let h = handles[&b.entity.to_bits()];
        let out = (b.body.kind == BODY_DYNAMIC && b.control.is_none()).then(|| {
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
                return (b.position, b.quaternion, Some(Velocity::default()));
            }
            let (position, quaternion) = unpose(rb.position(), b.position);
            (position, quaternion, Some(velocity))
        });
        let (position, quaternion, velocity) = match out {
            Some((p, q, vel)) => (p, q, vel),
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
    (touching, moved)
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
