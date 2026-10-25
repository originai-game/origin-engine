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
use crate::stats::{Laps, Phase};
use crate::{
    Hole, QueryFilter, RaycastHit, SHAPE_CACHE_LIMIT, ShapeCastDesc, ShapeCastHit, ShapeKey,
    Shared, StepOut,
};
use crate::{OverlapDesc, ShapeDesc};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
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
    if c.shape == SHAPE_HEIGHTFIELD
        && let Some(why) = shared.heightfields_absent
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.Collider.shape 6 (SHAPE_HEIGHTFIELD, a terrain's height field): {why}"),
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

/// A body's collider with its centre of mass moved by
/// `og.RigidBody.centerOfMass` (in the entity's frame, scaled like the
/// collider's offset): the shape's mass properties at the body's mass (or
/// the collider's density) with their centre shifted, the inertia kept.
/// Unchanged when the offset is zero.
fn centred(
    b: rp::ColliderBuilder,
    c: &Collider,
    body: &RigidBody,
    scale: Vec3,
    local: rp::Pose,
) -> Result<rp::ColliderBuilder> {
    let offset = body.center_of_mass;
    if offset == Vec3::ZERO {
        return Ok(b);
    }
    if !offset.is_finite() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("og.RigidBody.centerOfMass must be finite, got {offset}"),
        ));
    }
    let mut mp = if body.mass > 0.0 {
        let mut mp = b.shape.mass_properties(1.0);
        mp.set_mass(body.mass, true);
        mp
    } else {
        b.shape.mass_properties(c.density)
    };
    mp.local_com += local.rotation.inverse() * v(flat(offset * scale));
    Ok(b.mass_properties(mp))
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
    pub(crate) bodies: IdMap<u64, Tracked>,
    statics: IdMap<u64, TrackedStatic>,
    /// The fixed body world-anchored joints attach to.
    pub(crate) ground: Option<rp::RigidBodyHandle>,
    pub(crate) joints: IdMap<u64, ext::TrackedJoint>,
    pub(crate) vehicles: IdMap<u64, VehicleState>,
    pub(crate) ropes: IdMap<u64, crate::verlet::RopeState>,
    pub(crate) cloths: IdMap<u64, crate::verlet::ClothState>,
    pub(crate) fluids: IdMap<u64, crate::fluid::FluidState>,
    pub(crate) surfaces: IdMap<u64, crate::water::SurfaceState>,
    pub(crate) lines: IdMap<u64, crate::water::LineState>,
    pub(crate) ragdolls: IdMap<u64, super::RagdollState>,
    pub(crate) fluids3: IdMap<u64, crate::fluid3d::Fluid3State>,
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

/// Every body's linear and angular velocity before a substep, by handle
/// index (with the handle, as [`BodyPoses`]): the motions the solver meets
/// a new contact with. A body a controller moves is taken at the velocity
/// that carries it to its target, as rapier gives it inside the step.
pub(crate) type Motions = Vec<Option<(rp::RigidBodyHandle, Vec3, Vec3)>>;

fn motions_now(w: &rp::PhysicsWorld) -> Motions {
    let inv_dt = w.integration_parameters.inv_dt();
    let mut motions = Vec::new();
    for (h, rb) in w.bodies.iter() {
        let i = h.into_raw_parts().0 as usize;
        if i >= motions.len() {
            motions.resize(i + 1, None);
        }
        let vels = if rb.body_type() == rp::RigidBodyType::KinematicPositionBased {
            rp::RigidBodyPosition {
                position: *rb.position(),
                next_position: *rb.next_position(),
            }
            .interpolate_velocity(inv_dt, rb.local_center_of_mass())
        } else {
            *rb.vels()
        };
        motions[i] = Some((h, unv(vels.linvel), unangvel(vels.angvel)));
    }
    motions
}

/// The pairs touching after the substeps before the last, by their
/// colliders: the normal impulse they took so far and their relative
/// velocity as the first of those substeps began.
type Carried = BTreeMap<(u64, u64), (f32, Vec3)>;

/// What the step's readout needs to report a hit: the motions before the
/// last substep and what the earlier ones carried.
struct Hits<'a> {
    motions: &'a Motions,
    carried: &'a Carried,
}

fn pair_key(pair: &rp::ContactPair) -> (u64, u64) {
    (
        pair.collider1.into_raw_parts().0 as u64 | (pair.collider1.into_raw_parts().1 as u64) << 32,
        pair.collider2.into_raw_parts().0 as u64 | (pair.collider2.into_raw_parts().1 as u64) << 32,
    )
}

/// A touching pair's hit in the substep just stepped: the normal impulse
/// the solver gave it (N·s, summed over its contact points) and collider
/// 1's velocity relative to collider 2 at the deepest point, each body's
/// velocity taken from `motions` (before the substep) with its angular
/// part about its centre of mass now. A collider without a body is still.
fn hit(
    pair: &rp::ContactPair,
    colliders: &rp::ColliderSet,
    bodies: &rp::RigidBodySet,
    motions: &Motions,
) -> (f32, Vec3) {
    let Some((_, contact)) = pair.find_deepest_contact() else {
        return (0.0, Vec3::ZERO);
    };
    let c1 = &colliders[pair.collider1];
    let point = unv(c1.position().transform_point(contact.local_p1));
    let at = |c: &rp::Collider| {
        let Some(h) = c.parent() else {
            return Vec3::ZERO;
        };
        match motions.get(h.into_raw_parts().0 as usize) {
            Some(Some((handle, linear, angular))) if *handle == h => {
                let com = unv(bodies[h].center_of_mass());
                *linear + angular.cross(point - com)
            }
            _ => Vec3::ZERO,
        }
    };
    let impulse = pair.total_impulse_magnitude().max(0.0);
    (impulse, at(c1) - at(&colliders[pair.collider2]))
}

/// After a substep but the last: each touching solid pair's impulse added
/// to what it carried, its first relative velocity kept; a pair no longer
/// touching drops out (a contact that starts again starts afresh).
fn carry(w: &rp::PhysicsWorld, motions: &Motions, carried: &mut Carried) {
    let mut next = Carried::new();
    for pair in w.narrow_phase.contact_pairs() {
        if !pair.has_any_active_contact() {
            continue;
        }
        let (impulse, velocity) = hit(pair, &w.colliders, &w.bodies, motions);
        let key = pair_key(pair);
        let entry = match carried.get(&key) {
            Some(&(earlier, first)) => (earlier + impulse, first),
            None => (impulse, velocity),
        };
        next.insert(key, entry);
    }
    *carried = next;
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
        Some(c) => {
            let local = local_pose(c, b.scale);
            let built = collider(c, b.scale, local, e, body.mass, shared)?;
            Some(centred(built, c, body, b.scale, local)?)
        }
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
/// All of it runs where og-lanes puts it: from [`crate::pool_bodies`] for
/// the pool's thread count, on the lanes pool (natively and in the lanes
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
    og_lanes::install_from(
        og_lanes::Site::Physics,
        bodies,
        crate::pool_bodies(og_lanes::threads()),
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
    let mut laps = Laps::start();
    // Poses before this tick moved anything (game code teleporting
    // kinematic platforms included): how far each body moved this tick.
    let poses = BodyPoses::record(&sim.world);
    let handles = sync(settings, dt, scene, shared, sim)?;
    let pushes = std::mem::take(&mut shared.pushes);
    apply_pushes(sim, &handles, &pushes, dt);
    laps.lap(Phase::Sync);
    ext::sync_joints(scene, sim, &handles)?;
    super::vehicles(scene, dt, sim, &handles, &mut out)?;
    laps.lap(Phase::Joints);
    super::ragdolls(scene, dt, sim, shared, &mut out)?;
    laps.lap(Phase::Ragdolls);
    ext::water(scene, settings, dt, sim, &handles, &mut out)?;
    laps.lap(Phase::Water);
    let pre = rest_poses(scene, sim, &handles);
    laps.lap(Phase::Readout);

    // Each substep starts from the bodies' motions recorded before it: what
    // the solver met a contact with. The pairs touching after a substep
    // but the last carry their impulse and first relative velocity on.
    let mut carried = BTreeMap::new();
    let mut motions = Vec::new();
    for k in 0..substeps {
        motions = motions_now(&sim.world);
        sim.world.step();
        if k + 1 < substeps {
            carry(&sim.world, &motions, &mut carried);
        }
    }
    laps.lap(Phase::Rapier);
    if split {
        laps.split(Phase::Rapier);
    }

    let hits = Hits {
        motions: &motions,
        carried: &carried,
    };
    let (touching, moved) = after_step(settings, scene, sim, &handles, pre, &hits, split);
    out.touching = touching;
    out.moved = moved;
    laps.lap(Phase::Readout);
    ext::characters(scene, settings, dt, sim, &poses, &mut out)?;
    super::platformers(scene, settings, dt, sim, &poses, &mut out)?;
    laps.lap(Phase::Controllers);
    ext::soft(scene, settings, dt, tick, sim, &mut out, &mut laps)?;
    ext::joint_values(sim, &mut out);
    for (e, j) in &out.joints {
        ext::joint_written(sim, *e, j);
    }
    laps.lap(Phase::Joints);
    out.laps = Some(laps);
    Ok(out)
}

/// Each body's rapier handle, in scene order, and by entity for the
/// passes that look bodies up (joints, vehicles, water), built on the
/// first lookup.
pub(crate) struct Handles<'s> {
    scene: &'s Scene,
    pub(crate) by_body: Vec<rp::RigidBodyHandle>,
    by_entity: std::sync::OnceLock<IdMap<u64, rp::RigidBodyHandle>>,
}

impl Handles<'_> {
    /// The handle of the body of entity `key`.
    pub(crate) fn get(&self, key: &u64) -> Option<&rp::RigidBodyHandle> {
        self.by_entity
            .get_or_init(|| {
                self.scene
                    .bodies
                    .iter()
                    .map(|b| b.entity.to_bits())
                    .zip(self.by_body.iter().copied())
                    .collect()
            })
            .get(key)
    }
}

/// `f` on the ranges of the scene's bodies, by [`crate::Grains::bodies`] a
/// task on the lanes from two of them, the ranges' results joined in body
/// order.
fn by_bodies<R: Send>(
    n: usize,
    f: impl Fn(std::ops::Range<usize>) -> Vec<R> + Sync + Send,
) -> Vec<R> {
    let mut parts = crate::lanes::map_ranges(n, crate::lanes::now().bodies, f);
    if parts.len() == 1 {
        parts.pop().unwrap_or_default()
    } else {
        parts.into_iter().flatten().collect()
    }
}

fn simulated(b: &Body) -> bool {
    b.body.kind == BODY_DYNAMIC && b.control.is_none()
}

/// A kinematic body with a nonzero og.Velocity: the step moves it by it
/// and writes its Transform back.
fn moves_itself(b: &Body) -> bool {
    b.body.kind == BODY_KINEMATIC
        && b.control.is_none()
        && b.velocity
            .is_some_and(|v| v.linear != Vec3::ZERO || v.angular != Vec3::ZERO)
}

/// The dynamic bodies' poses before the step, for the rest rule, in body
/// order (by ranges of bodies on the lanes).
fn rest_poses(scene: &Scene, sim: &Sim, handles: &Handles<'_>) -> Vec<rp::Pose> {
    let bodies = &sim.world.bodies;
    by_bodies(scene.bodies.len(), |r| {
        scene.bodies[r.clone()]
            .iter()
            .zip(&handles.by_body[r])
            .filter(|(b, _)| simulated(b))
            .map(|(_, &h)| *bodies[h].position())
            .collect()
    })
}

/// What [`sync`] does to a body, decided from the body and what physics
/// last left it at (read only, by ranges of bodies on the lanes), then
/// done to the rapier world in body order.
enum Plan {
    /// Still the body it was: moved where game code or a controller put it
    /// (`next` kinematic target, or `position` teleport) and given a
    /// `velocity`, each when it changed.
    Keep {
        handle: rp::RigidBodyHandle,
        next: Option<rp::Pose>,
        position: Option<rp::Pose>,
        velocity: Option<(rp::Vector, rp::AngVector)>,
    },
    /// New, or its body, collider or scale changed: built afresh.
    Build,
}

fn plan(b: &Body, tracked: &IdMap<u64, Tracked>, stale: &BTreeSet<u64>) -> Plan {
    let stale_mesh = |c: &Collider| {
        matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
            && stale.contains(&c.mesh)
    };
    match tracked.get(&b.entity.to_bits()) {
        Some(t)
            if t.body == b.body
                && t.collider == b.collider
                && t.scale == b.scale
                && !b.collider.as_ref().is_some_and(stale_mesh) =>
        {
            let kind = b.body.kind;
            let (mut next, mut position) = (None, None);
            if b.control.is_some() {
                // A controller moved it after the last step: the step
                // takes it there, pushing what is in the way.
                next = Some(pose(b.position, b.quaternion));
            } else if t.position != b.position || t.quaternion != b.quaternion {
                // Game code moved it (a kinematic body moving by its
                // velocity is where physics wrote it, so it is not moved).
                position = Some(pose(b.position, b.quaternion));
            }
            let velocity = (b.control.is_none()
                && (kind == BODY_KINEMATIC || (kind == BODY_DYNAMIC && t.velocity != b.velocity)))
                .then(|| {
                    let vel = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
                    (v(flat(vel.linear)), angvel(vel.angular))
                });
            Plan::Keep {
                handle: t.handle,
                next,
                position,
                velocity,
            }
        }
        _ => Plan::Build,
    }
}

/// Entity keys in ascending order (a scene's usually are already: then no
/// sort).
fn sorted_keys(keys: impl Iterator<Item = u64>) -> Vec<u64> {
    let mut keys: Vec<u64> = keys.collect();
    if !keys.is_sorted() {
        keys.sort_unstable();
    }
    keys
}

/// Keeps the entries of `map` whose key is in `live` (ascending), in one
/// walk of both; `gone` sees each other entry, in key order, as it leaves.
fn retain_live<T>(map: &mut IdMap<u64, T>, live: &[u64], mut gone: impl FnMut(&T)) {
    let mut at = 0;
    map.retain(|key, value| {
        while at < live.len() && live[at] < *key {
            at += 1;
        }
        let keep = live.get(at) == Some(key);
        if !keep {
            gone(value);
        }
        keep
    });
}

/// Brings the persistent rapier world in line with `scene` (bodies and
/// static colliders); returns each body's handle.
fn sync<'s>(
    settings: &Physics,
    dt: f32,
    scene: &'s Scene,
    shared: &mut Shared,
    sim: &mut Sim,
) -> Result<Handles<'s>> {
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
    let live = sorted_keys(scene.bodies.iter().map(|b| b.entity.to_bits()));
    retain_live(tracked, &live, |t| {
        w.remove_body(t.handle);
    });
    let live = sorted_keys(scene.statics.iter().map(|s| s.entity.to_bits()));
    retain_live(tracked_statics, &live, |t| {
        w.remove_collider(t.handle);
    });

    let stale = std::mem::take(&mut shared.stale_meshes);
    let stale_mesh = |c: &Collider| {
        matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
            && stale.contains(&c.mesh)
    };

    // What to do to each body, decided by ranges of bodies on the lanes;
    // then done in body order, so rapier sees its bodies touched, moved,
    // removed and inserted in the order one thread would.
    let plans = {
        let tracked = &*tracked;
        let stale = &stale;
        by_bodies(scene.bodies.len(), |r| {
            scene.bodies[r]
                .iter()
                .map(|b| plan(b, tracked, stale))
                .collect()
        })
    };
    let mut by_body = Vec::with_capacity(scene.bodies.len());
    for (b, plan) in scene.bodies.iter().zip(plans) {
        let h = match plan {
            Plan::Keep {
                handle,
                next,
                position,
                velocity,
            } => {
                let rb = &mut w.bodies[handle];
                if let Some(at) = next {
                    rb.set_next_kinematic_position(at);
                } else if let Some(at) = position {
                    rb.set_position(at, true);
                }
                if let Some((linear, angular)) = velocity {
                    rb.set_linvel(linear, true);
                    rb.set_angvel(angular, true);
                }
                handle
            }
            Plan::Build => {
                if let Some(t) = tracked.remove(&b.entity.to_bits()) {
                    w.remove_body(t.handle);
                }
                insert_body(w, b, shared)?
            }
        };
        by_body.push(h);
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
    Ok(Handles {
        scene,
        by_body,
        by_entity: std::sync::OnceLock::new(),
    })
}

/// What the rest rule makes of a dynamic body after the step.
enum Rest {
    /// Supported and slow before and after: back to its pose before the
    /// step, its velocity dropped.
    Still(rp::Pose),
    Moving,
}

/// One body read back after the step (by ranges of bodies on the lanes):
/// its component values and, for a dynamic body, what the rest rule does.
type Read = (
    Option<Rest>,
    Option<(Vec3, Quat, Option<Velocity>)>,
    Tracked,
);

/// The touching pairs and the rest rule after the step: what each dynamic
/// body's component values become.
#[allow(clippy::type_complexity)]
fn after_step(
    settings: &Physics,
    scene: &Scene,
    sim: &mut Sim,
    handles: &Handles<'_>,
    before: Vec<rp::Pose>,
    hits: &Hits<'_>,
    split: bool,
) -> (
    Vec<(Contact, CollisionStarted)>,
    Vec<Option<(Vec3, Quat, Option<Velocity>)>>,
) {
    let Sim {
        world: w,
        bodies: tracked,
        ..
    } = sim;
    let grains = crate::lanes::now();
    // The pairs split only when the step is on the pool: below it, waking
    // the pool for them costs more than it saves (tests/evidence/P4/physics-3/).
    let pairs = if split { grains.pairs } else { usize::MAX };
    // The solid pairs: each one's contact (its entities in order, the
    // deepest point and normal, the hit), its deepest penetration and the
    // bodies it touches, by ranges of the contact graph's edges on the
    // lanes, joined in edge order: the narrow phase's pair order either way.
    let edges = w.narrow_phase.contact_graph().raw_graph().raw_edges();
    let (colliders, bodies) = (&w.colliders, &w.bodies);
    let read_pair = |pair: &rp::ContactPair| {
        let (c1, c2) = (&colliders[pair.collider1], &colliders[pair.collider2]);
        let (e1, e2) = (entity_of(c1).to_bits(), entity_of(c2).to_bits());
        let (mut point, mut normal) = (Vec3::ZERO, Vec3::ZERO);
        if let Some((manifold, contact)) = pair.find_deepest_contact() {
            point = unv(c1.position().transform_point(contact.local_p1));
            normal = unv(manifold.data.normal);
        }
        let (mut impulse, mut velocity) = hit(pair, colliders, bodies, hits.motions);
        if let Some(&(earlier, first)) = hits.carried.get(&pair_key(pair)) {
            impulse += earlier;
            velocity = first;
        }
        let (a, b, normal, velocity) = if e1 < e2 {
            (e1, e2, normal, velocity)
        } else {
            (e2, e1, -normal, -velocity)
        };
        let depth = pair
            .manifolds()
            .iter()
            .flat_map(|m| m.points.iter().map(|p| p.dist))
            .fold(f32::INFINITY, f32::min);
        let body = |c: &rp::Collider| c.parent().map(|p| p.into_raw_parts().0 as usize);
        (
            (
                Contact { a, b, sensor: 0 },
                CollisionStarted {
                    a,
                    b,
                    point,
                    normal,
                    impulse,
                    relative_velocity: velocity,
                },
            ),
            depth,
            [body(c1), body(c2)],
        )
    };
    let parts = crate::lanes::map_ranges(edges.len(), pairs, |r| {
        edges[r]
            .iter()
            .map(|e| &e.weight)
            .filter(|pair| pair.has_any_active_contact())
            .map(read_pair)
            .collect::<Vec<_>>()
    });
    // The deepest penetration of each body that has a solid contact, by
    // body handle index.
    let mut deepest: Vec<Option<f32>> = vec![None; w.bodies.len()];
    let mut touching = Vec::with_capacity(parts.iter().map(Vec::len).sum());
    for (contact, depth, parents) in parts.into_iter().flatten() {
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
            CollisionStarted::default(),
        ));
    }
    let touching = sorted(touching, pairs);

    // Each body read back by ranges of bodies on the lanes (rapier's
    // bodies read only), the rest rule decided there; then, in body order,
    // the resting bodies stilled in rapier and the tracked values kept.
    let rest = settings.rest_speed;
    let firsts: Vec<usize> = {
        let mut k = 0;
        scene
            .bodies
            .iter()
            .map(|b| {
                let at = k;
                k += usize::from(simulated(b));
                at
            })
            .collect()
    };
    let read: Vec<Read> = {
        let (bodies, deepest, before, firsts) = (&w.bodies, &deepest, &before, &firsts);
        by_bodies(scene.bodies.len(), |r| {
            scene.bodies[r.clone()]
                .iter()
                .zip(&handles.by_body[r.clone()])
                .zip(&firsts[r])
                .map(|((b, &h), &k)| read_body(b, h, bodies, deepest, before.get(k), rest))
                .collect()
        })
    };
    let mut moved = Vec::with_capacity(scene.bodies.len());
    let mut kept = Vec::with_capacity(scene.bodies.len());
    for (&h, (rest, out, t)) in handles.by_body.iter().zip(read) {
        if let Some(rest) = rest {
            // Touched in body order, as one thread reading them back would.
            let rb = &mut w.bodies[h];
            if let Rest::Still(pre) = rest {
                rb.set_position(pre, false);
                rb.set_linvel(v(Vec3::ZERO), false);
                rb.set_angvel(angvel(Vec3::ZERO), false);
            }
        }
        moved.push(out);
        kept.push(t);
    }
    keep_tracked(scene, tracked, kept);
    (touching, moved)
}

/// Puts each body's [`Tracked`] (`kept`, in body order) in `tracked`,
/// which holds every body still in the rapier world and lacks those built
/// this tick: one walk of both in entity order, the new ones inserted
/// after it.
fn keep_tracked(scene: &Scene, tracked: &mut IdMap<u64, Tracked>, kept: Vec<Tracked>) {
    let keys: Vec<u64> = scene.bodies.iter().map(|b| b.entity.to_bits()).collect();
    let mut order: Vec<usize> = (0..keys.len()).collect();
    if !keys.is_sorted() {
        order.sort_unstable_by_key(|&i| keys[i]);
    }
    let mut kept: Vec<Option<Tracked>> = kept.into_iter().map(Some).collect();
    let mut order = order.into_iter().peekable();
    for (key, slot) in tracked.iter_mut() {
        // Bodies before `key` in entity order are new: inserted below.
        while let Some(&i) = order.peek() {
            if keys[i] >= *key {
                break;
            }
            order.next();
        }
        if let Some(i) = order.next_if(|&i| keys[i] == *key)
            && let Some(t) = kept[i].take()
        {
            *slot = t;
        }
    }
    for (key, t) in keys.into_iter().zip(kept) {
        if let Some(t) = t {
            tracked.insert(key, t);
        }
    }
}

/// The touching pairs sorted by their contact, ties (a ragdoll's bones on
/// one floor) in their order: what a stable sort leaves, made by sorting
/// each pair's key with its index (unique, so on the lanes or not the one
/// order) and gathering the pairs in it.
fn sorted(
    touching: Vec<(Contact, CollisionStarted)>,
    grain: usize,
) -> Vec<(Contact, CollisionStarted)> {
    let mut keys: Vec<(Contact, u32)> = touching
        .iter()
        .enumerate()
        .map(|(i, t)| (t.0, i as u32))
        .collect();
    if keys.is_sorted() {
        return touching;
    }
    og_lanes::sort(og_lanes::Site::Physics, &mut keys, grain);
    keys.into_iter()
        .map(|(_, i)| touching[i as usize])
        .collect()
}

/// A body after the step: the rest rule for a dynamic one (`pre` its pose
/// before the step) and the values its components and [`Tracked`] take.
fn read_body(
    b: &Body,
    h: rp::RigidBodyHandle,
    bodies: &rp::RigidBodySet,
    deepest: &[Option<f32>],
    pre: Option<&rp::Pose>,
    rest: f32,
) -> Read {
    let (rest_rule, out) = if simulated(b) {
        let pre = pre.expect("one pre-step pose per dynamic body");
        let rb = &bodies[h];
        let velocity = unlocked(
            Velocity {
                linear: unv(rb.linvel()),
                angular: unangvel(rb.angvel()),
            },
            b.body.lock,
        );
        // Resting: supported with at most REST_PENETRATION of overlap, and
        // slow before and after the step. It keeps its pose and loses its
        // residual velocity, in the components and in rapier.
        let prior = unlocked(b.velocity.unwrap_or_default(), b.body.lock);
        let supported = deepest
            .get(h.into_raw_parts().0 as usize)
            .copied()
            .flatten()
            .is_some_and(|d| d >= -REST_PENETRATION);
        if rest > 0.0 && supported && slow(&prior, rest) && slow(&velocity, rest) {
            (
                Some(Rest::Still(*pre)),
                Some((b.position, b.quaternion, Some(Velocity::default()))),
            )
        } else {
            let (position, quaternion) = unpose(rb.position(), b.position);
            (
                Some(Rest::Moving),
                Some((position, quaternion, Some(velocity))),
            )
        }
    } else if moves_itself(b) {
        // A kinematic body moving by its velocity: where the step took it.
        let (position, quaternion) = unpose(bodies[h].position(), b.position);
        (None, Some((position, quaternion, None)))
    } else {
        (None, None)
    };
    let (position, quaternion, velocity) = match out {
        Some((p, q, vel)) => (p, q, vel),
        None => (b.position, b.quaternion, b.velocity),
    };
    let tracked = Tracked {
        handle: h,
        body: b.body,
        collider: b.collider,
        scale: b.scale,
        position,
        quaternion,
        velocity,
    };
    (rest_rule, out, tracked)
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
        og_core::sort::sort_by_key(&mut hits, |e| e.to_bits());
        hits.dedup();
        Ok(hits)
    }
}

// ---- forces and impulses --------------------------------------------------------

/// Applies what game code pushed the bodies with since the last step
/// (og.physics.applyForce and friends), in the order it asked: a force or
/// torque over this step (as the impulse it gives in `dt`), an impulse at
/// once. Only dynamic bodies move; the others ignore it.
pub(crate) fn apply_pushes(sim: &mut Sim, handles: &Handles<'_>, pushes: &[crate::Push], dt: f32) {
    use crate::PushKind;
    for p in pushes {
        let Some(&h) = handles.get(&p.entity) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let scale = match p.kind {
            PushKind::Force | PushKind::Torque => dt,
            PushKind::Impulse | PushKind::TorqueImpulse => 1.0,
        };
        let x = p.vector * scale;
        match p.kind {
            PushKind::Force | PushKind::Impulse => match p.point {
                Some(at) => rb.apply_impulse_at_point(v(flat(x)), v(at), true),
                None => rb.apply_impulse(v(flat(x)), true),
            },
            PushKind::Torque | PushKind::TorqueImpulse => rb.apply_torque_impulse(angvel(x), true),
        }
    }
}

// ---- the saved simulation ------------------------------------------------------

/// Whether a collider's shape is built from a registered asset (a mesh or a
/// height field), so a save carries its key instead of its geometry.
fn from_asset(c: &Collider) -> bool {
    matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
}

fn asset_key(c: &Collider, scale: Vec3) -> ShapeKey {
    ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: scale.abs().to_array().map(f32::to_bits),
    }
}

/// `sim`'s bytes without the geometry of its mesh, convex-hull and
/// height-field colliders: meshes are assets, so a save carries each such
/// shape's key and [`load`] rebuilds it from the asset. Everything else is
/// rapier's own serialisation, byte for byte.
pub(crate) fn save(sim: &Sim) -> (Vec<Hole>, Vec<u8>) {
    let fits = "a physics world always serializes into memory";
    let mut full = Vec::new();
    crate::encode(&mut full, sim).expect(fits);
    let w = &sim.world;
    // Which colliders' shapes come from assets, by handle.
    let mut keyed: BTreeMap<(u32, u32), ShapeKey> = BTreeMap::new();
    for t in sim.bodies.values() {
        if let Some(c) = t.collider.as_ref().filter(|c| from_asset(c))
            && let Some(&h) = w.bodies.get(t.handle).and_then(|rb| rb.colliders().first())
        {
            keyed.insert(h.into_raw_parts(), asset_key(c, t.scale));
        }
    }
    for t in sim.statics.values() {
        if from_asset(&t.collider) {
            keyed.insert(t.handle.into_raw_parts(), asset_key(&t.collider, t.scale));
        }
    }
    if keyed.is_empty() {
        return (Vec::new(), full);
    }
    // The colliders' bytes start after the world's fields before them
    // (bincode writes a struct's fields, and a tuple's, one after another):
    // measured by writing those fields again with the same encoder rather
    // than a size pass, which would compile their serialisation twice.
    let mut scratch = Vec::new();
    let before = (
        &w.gravity,
        &w.integration_parameters,
        &w.islands,
        &w.broad_phase,
        &w.narrow_phase,
        &w.bodies,
    );
    crate::encode(&mut scratch, &before).expect(fits);
    let start = scratch.len();
    scratch.clear();
    crate::encode(&mut scratch, &w.colliders).expect(fits);
    let end = start + scratch.len();
    let mut holes = Vec::new();
    let mut out = Vec::with_capacity(full.len());
    let mut at = start;
    out.extend_from_slice(&full[..start]);
    for (h, c) in w.colliders.iter() {
        let Some(key) = keyed.get(&h.into_raw_parts()) else {
            continue;
        };
        scratch.clear();
        crate::encode(&mut scratch, c.shared_shape()).expect(fits);
        let shape = &scratch;
        // The shape is the collider's second field: the first match past
        // the previous collider is it.
        let Some(found) = full[at..end]
            .windows(shape.len())
            .position(|bytes| bytes == shape.as_slice())
        else {
            continue;
        };
        out.extend_from_slice(&full[at..at + found]);
        holes.push(Hole {
            at: out.len() as u64,
            key: *key,
        });
        at += found + shape.len();
    }
    out.extend_from_slice(&full[at..]);
    (holes, out)
}

/// The simulation [`save`] wrote, its asset shapes rebuilt from the meshes
/// and height fields registered now (the same shapes a step builds, from
/// the shape cache when it has them). Refused, naming the handle, for a
/// mesh no longer registered.
pub(crate) fn load(bytes: &[u8], holes: &[Hole], shared: &mut Shared) -> Result<Sim> {
    use bincode::Options;
    let parse = |e: bincode::Error| {
        Error::new(
            ErrorKind::Parse,
            format!("the rapier world does not decode: {e}"),
        )
    };
    let codec = crate::codec;
    if holes.is_empty() {
        return codec().deserialize(bytes).map_err(parse);
    }
    let mut full = Vec::with_capacity(bytes.len());
    let mut from = 0;
    for hole in holes {
        let at = (hole.at as usize).clamp(from, bytes.len());
        full.extend_from_slice(&bytes[from..at]);
        from = at;
        let c = Collider {
            shape: hole.key.shape,
            mesh: hole.key.mesh,
            ..Collider::default()
        };
        let scale = Vec3::from_array(hole.key.scale.map(f32::from_bits));
        let built = shape(&c, scale, shared).map_err(|e| {
            e.context(
                "the save's colliders are built from meshes and height fields, which are assets: load or register them before restoring",
            )
        })?;
        crate::encode(&mut full, &built).map_err(parse)?;
    }
    full.extend_from_slice(&bytes[from..]);
    codec().deserialize(&full).map_err(parse)
}
