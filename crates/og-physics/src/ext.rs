//! Physics beyond rigid bodies, written once and compiled for each profile
//! (like `sim.rs`): joints, the character controller, Verlet ropes and
//! cloth, the 2D fluid and water, run by `sim::step` around the rigid-body
//! step. `mover.rs` is the controllers' move against the world (both
//! profiles); `platformer.rs` (2D), `vehicle2.rs` / `vehicle3.rs` and
//! `ragdoll.rs` (both profiles, through each one's joint) hold what the profiles do differently.

use super::mover::Mover;
use super::sim::{BodyPoses, Sim};
use super::{
    DIM, angvel, flat, heightfield_surface, joint_frame, pose, rp, surface_only, unangvel, unpose,
    unv, v,
};
use crate::fluid::FluidState;
use crate::gather::Scene;
use crate::schema::*;
use crate::schema_ext::*;
use crate::verlet::{ClothState, RopeState, Solids};
use crate::water::{Floater, LineState, SurfaceState, WaterForce};
use crate::{ControlOut, StepOut};
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn entity_of(c: &rp::Collider) -> u64 {
    c.user_data as u64
}

// ---- collisions for points ----------------------------------------------------

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

// ---- joints ---------------------------------------------------------------

/// A joint as the last tick built it.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TrackedJoint {
    handle: rp::ImpulseJointHandle,
    joint: Joint,
    first: rp::RigidBodyHandle,
    second: rp::RigidBodyHandle,
}

/// What the tracked joints hold that [`Sim`]'s own bytes leave out
/// (og.Joint's breakForce and breakTorque are serde-skipped, so a world
/// without a breakable joint saves as before): the two as bits, by joint
/// entity, for the joints with either. Without it a restored world's
/// record of such a joint differs from its component, which the next sync
/// takes for an edit and wakes its bodies for.
pub(crate) fn joint_section(joints: &IdMap<u64, TrackedJoint>) -> Option<(u16, Vec<u8>)> {
    let list: Vec<(u64, [u32; 2])> = joints
        .iter()
        .filter(|(_, t)| t.joint.break_force != 0.0 || t.joint.break_torque != 0.0)
        .map(|(&e, t)| {
            (
                e,
                [
                    t.joint.break_force.to_bits(),
                    t.joint.break_torque.to_bits(),
                ],
            )
        })
        .collect();
    if list.is_empty() {
        return None;
    }
    let mut bytes = Vec::new();
    crate::encode(&mut bytes, &list).expect("joints serialize into memory");
    Some((super::sim::EXT_JOINTS, bytes))
}

/// Puts [`joint_section`]'s values back on the restored joints.
pub(crate) fn restore_joint_section(
    joints: &mut IdMap<u64, TrackedJoint>,
    sections: &[(u16, &[u8])],
) -> Result<()> {
    use bincode::Options;
    for &(tag, bytes) in sections {
        if tag != super::sim::EXT_JOINTS {
            continue;
        }
        let list: Vec<(u64, [u32; 2])> = crate::codec().deserialize(bytes).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                format!("the physics block's joint section does not decode: {e}"),
            )
        })?;
        for (e, [force, torque]) in list {
            if let Some(t) = joints.get_mut(&e) {
                t.joint.break_force = f32::from_bits(force);
                t.joint.break_torque = f32::from_bits(torque);
            }
        }
    }
    Ok(())
}

/// Moves the joints anchored to the world (no `other`) by `s.d`: the
/// ground body's frame of each, and the og.Joint it was built from (whose
/// `otherAnchor` the shift moves the same way).
pub(crate) fn shift_joints(sim: &mut Sim, s: &og_ecs::Shift) {
    let Some(ground) = sim.ground else {
        return;
    };
    for (_, t) in sim.joints.iter_mut() {
        if t.second != ground {
            continue;
        }
        t.joint.other_anchor = s.moved(t.joint.other_anchor);
        if let Some(j) = sim.world.impulse_joints.get_mut(t.handle, false) {
            let a = j.data.local_anchor1();
            j.data.set_local_anchor1(v(s.moved(unv(a))));
        }
    }
}

/// The world anchor joints without `other` attach to: a fixed body at the
/// origin, made the first time one needs it.
pub(crate) fn ground(sim: &mut Sim) -> rp::RigidBodyHandle {
    if let Some(h) = sim.ground {
        return h;
    }
    let h = sim
        .world
        .bodies
        .insert(rp::RigidBodyBuilder::fixed().user_data(0).can_sleep(false));
    sim.ground = Some(h);
    h
}

/// The joint's structural part (what rebuilds it), without its motor, its
/// spring and its output.
fn structure(j: &Joint) -> Joint {
    Joint {
        motor_target: 0.0,
        motor_stiffness: 0.0,
        motor_damping: 0.0,
        motor_max_force: 0.0,
        stiffness: 0.0,
        damping: 0.0,
        value: 0.0,
        break_force: 0.0,
        break_torque: 0.0,
        ..*j
    }
}

/// The free axis of a hinge or slider.
fn free_axis(kind: u32) -> rp::JointAxis {
    match kind {
        JOINT_PRISMATIC => rp::JointAxis::LinX,
        _ => rp::JointAxis::AngX,
    }
}

fn set_drive(data: &mut rp::GenericJoint, j: &Joint) {
    match j.kind {
        JOINT_REVOLUTE | JOINT_PRISMATIC => {
            let axis = free_axis(j.kind);
            match j.motor {
                MOTOR_VELOCITY => {
                    data.set_motor_velocity(axis, j.motor_target, j.motor_stiffness.max(0.0));
                }
                MOTOR_POSITION => {
                    data.set_motor_position(
                        axis,
                        j.motor_target,
                        j.motor_stiffness.max(0.0),
                        j.motor_damping.max(0.0),
                    );
                }
                _ => {
                    if data.motor(axis).is_some() {
                        data.set_motor(axis, 0.0, 0.0, 0.0, 0.0);
                        data.set_motor_max_force(axis, 0.0);
                    }
                    return;
                }
            }
            data.set_motor_max_force(
                axis,
                if j.motor_max_force > 0.0 {
                    j.motor_max_force
                } else {
                    f32::MAX
                },
            );
        }
        JOINT_SPRING => {
            let mut s = rp::SpringJointBuilder::new(
                j.rest_length.max(0.0),
                j.stiffness.max(0.0),
                j.damping.max(0.0),
            )
            .build();
            let anchors = (data.local_anchor1(), data.local_anchor2());
            s.data.set_local_anchor1(anchors.0);
            s.data.set_local_anchor2(anchors.1);
            s.data.set_contacts_enabled(data.contacts_enabled());
            *data = s.data;
        }
        _ => {}
    }
}

/// The rapier joint for `j`, whose rapier body 1 is the other body (at
/// `other`) and body 2 this one (at `this`), so a motor turns this body
/// relative to the other and a positive angle is this body's turn about
/// `axis`. The other body's frame is placed where this one's is now, so a
/// joint holds the pose it was made in (a hinge's angle is 0 there); a
/// fixed joint also keeps the bodies' current offset (it welds them as they
/// are; `otherAnchor` is not used).
fn joint_data(j: &Joint, other: &rp::Pose, this: &rp::Pose) -> Result<rp::GenericJoint> {
    let mine = v(j.anchor);
    let theirs = v(j.other_anchor);
    let mut data: rp::GenericJoint = match j.kind {
        JOINT_SPRING => rp::SpringJointBuilder::new(
            j.rest_length.max(0.0),
            j.stiffness.max(0.0),
            j.damping.max(0.0),
        )
        .local_anchor1(theirs)
        .local_anchor2(mine)
        .build()
        .into(),
        JOINT_ROPE => rp::RopeJointBuilder::new(j.length.max(0.0))
            .local_anchor1(theirs)
            .local_anchor2(mine)
            .build()
            .into(),
        JOINT_FIXED | JOINT_REVOLUTE | JOINT_PRISMATIC | JOINT_BALL => {
            let locked = match j.kind {
                JOINT_FIXED => rp::JointAxesMask::LOCKED_FIXED_AXES,
                JOINT_PRISMATIC => rp::JointAxesMask::LOCKED_PRISMATIC_AXES,
                JOINT_BALL => super::ball_axes(),
                _ => rp::JointAxesMask::LOCKED_REVOLUTE_AXES,
            };
            let frame2 = rp::Pose::from_parts(mine, joint_frame(j.axis));
            let placed = other.inverse() * *this * frame2;
            let frame1 = if j.kind == JOINT_FIXED {
                placed
            } else {
                rp::Pose::from_parts(theirs, placed.rotation)
            };
            let mut b = rp::GenericJointBuilder::new(locked)
                .local_frame1(frame1)
                .local_frame2(frame2);
            if j.limits && matches!(j.kind, JOINT_REVOLUTE | JOINT_PRISMATIC) {
                if j.min.partial_cmp(&j.max).is_none_or(|o| o.is_gt()) {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("og.Joint.min {} is above max {}", j.min, j.max),
                    ));
                }
                b = b.limits(free_axis(j.kind), [j.min, j.max]);
            }
            b.build()
        }
        other => {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "og.Joint.kind {other} is not a joint (0 fixed, 1 hinge, 2 slider, 3 spring, 4 rope, 5 ball)"
                ),
            ));
        }
    };
    data.set_contacts_enabled(j.collide);
    set_drive(&mut data, j);
    Ok(data)
}

/// Brings the joints in line with the scene: gone ones leave, new or
/// structurally changed ones are built, motor and spring changes update in
/// place. `handles` maps a body's entity to its rapier body.
pub(crate) fn sync_joints(
    scene: &Scene,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
) -> Result<()> {
    let live: BTreeSet<u64> = og_core::sort::set(scene.joints.iter().map(|(e, _)| e.to_bits()));
    let gone: Vec<u64> = sim
        .joints
        .keys()
        .filter(|e| !live.contains(e))
        .copied()
        .collect();
    for e in gone {
        if let Some(t) = sim.joints.remove(&e) {
            sim.world.impulse_joints.remove(t.handle, true);
        }
    }
    for (e, j) in &scene.joints {
        let key = e.to_bits();
        if j.broken {
            // Broken: it holds nothing until game code mends it.
            if let Some(t) = sim.joints.remove(&key) {
                sim.world.impulse_joints.remove(t.handle, true);
            }
            continue;
        }
        let first_entity = if j.body == 0 { key } else { j.body };
        let first = handles.get(&first_entity).copied();
        let second = if j.other == 0 {
            Some(ground(sim))
        } else {
            handles.get(&j.other).copied()
        };
        let (Some(first), Some(second)) = (first, second) else {
            if let Some(t) = sim.joints.remove(&key) {
                sim.world.impulse_joints.remove(t.handle, true);
            }
            continue;
        };
        if first == second {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("og.Joint of entity {e} joins a body to itself"),
            ));
        }
        let alive = |t: &TrackedJoint, sim: &Sim| sim.world.impulse_joints.get(t.handle).is_some();
        match sim.joints.get(&key) {
            Some(t)
                if t.first == first
                    && t.second == second
                    && structure(&t.joint) == structure(j)
                    && alive(t, sim) =>
            {
                if t.joint != *j {
                    let handle = t.handle;
                    if let Some(joint) = sim.world.impulse_joints.get_mut(handle, true) {
                        set_drive(&mut joint.data, j);
                    }
                    sim.joints.get_mut(&key).expect("tracked").joint = *j;
                }
            }
            _ => {
                if let Some(t) = sim.joints.remove(&key) {
                    sim.world.impulse_joints.remove(t.handle, true);
                }
                let (this, other) = (
                    *sim.world.bodies[first].position(),
                    *sim.world.bodies[second].position(),
                );
                let data = joint_data(j, &other, &this)
                    .map_err(|err| err.context(format!("entity {e}")))?;
                // Rapier's body 1 is the other body, body 2 this one.
                let handle = sim.world.impulse_joints.insert(second, first, data, true);
                sim.joints.insert(
                    key,
                    TrackedJoint {
                        handle,
                        joint: *j,
                        first,
                        second,
                    },
                );
            }
        }
    }
    Ok(())
}

/// After a step: removes, in entity order, the joints whose solver load
/// passed their breakForce or breakTorque, each with its event and its
/// component as it is now, broken. `dt` is the solver's own substep (the
/// step's dt over its solver iterations, rapier's internal substeps): the
/// impulse rapier writes back on the joint's locked axes, limits and
/// motors is the last one's, so a load held through the step counts (a
/// weight, gravity, a body resting or pressing on it, a pull along other
/// joints) and a shock the step's first solver substep absorbs does not
/// (an impulse, and applyForce, which reaches the body as one).
pub(crate) fn break_joints(sim: &mut Sim, dt: f32, out: &mut StepOut) {
    if dt <= 0.0 {
        return;
    }
    let mut gone = Vec::new();
    for (&e, t) in &sim.joints {
        let j = &t.joint;
        if j.break_force <= 0.0 && j.break_torque <= 0.0 {
            continue;
        }
        let Some(joint) = sim.world.impulse_joints.get(t.handle) else {
            continue;
        };
        let (linear, angular) = super::joint_load(joint, &sim.world.bodies);
        let (force, torque) = (linear / dt, angular / dt);
        if (j.break_force > 0.0 && force > j.break_force)
            || (j.break_torque > 0.0 && torque > j.break_torque)
        {
            gone.push((e, force, torque));
        }
    }
    for (e, force, torque) in gone {
        let Some(t) = sim.joints.remove(&e) else {
            continue;
        };
        sim.world.impulse_joints.remove(t.handle, true);
        let j = Joint {
            broken: true,
            ..t.joint
        };
        out.broken.push(crate::JointBroken {
            joint: e,
            body: if j.body == 0 { e } else { j.body },
            other: j.other,
            force,
            torque,
        });
        out.joints.push((Entity::from_bits(e), j));
    }
}

/// Each joint's value after the step: the hinge angle, the slide, or the
/// anchors' distance.
pub(crate) fn joint_values(sim: &Sim, out: &mut StepOut) {
    for (&e, t) in &sim.joints {
        let Some(joint) = sim.world.impulse_joints.get(t.handle) else {
            continue;
        };
        let (b1, b2) = (&sim.world.bodies[t.second], &sim.world.bodies[t.first]);
        let f1 = *b1.position() * joint.data.local_frame1;
        let f2 = *b2.position() * joint.data.local_frame2;
        let value = match t.joint.kind {
            JOINT_REVOLUTE => super::hinge_angle(&f1, &f2),
            JOINT_PRISMATIC => {
                let rel = f1.inverse() * f2;
                unv(rel.translation).x
            }
            _ => (unv(f2.translation) - unv(f1.translation)).length(),
        };
        if value != t.joint.value {
            out.joints
                .push((Entity::from_bits(e), Joint { value, ..t.joint }));
        }
    }
}

/// Records that joint `e`'s component now carries `value` (what physics
/// wrote back), so the next tick does not see it as changed.
pub(crate) fn joint_written(sim: &mut Sim, e: Entity, joint: &Joint) {
    if let Some(t) = sim.joints.get_mut(&e.to_bits()) {
        t.joint = *joint;
    }
}

// ---- moving a kinematic shape ----------------------------------------------

/// Where a point on a body moved during the step (the body's motion from
/// its pre-step pose), for riding moving platforms.
pub(crate) fn carried(
    poses: &BodyPoses,
    w: &rp::PhysicsWorld,
    body: rp::RigidBodyHandle,
    point: Vec3,
) -> Vec3 {
    let Some(before) = poses.get(body) else {
        return Vec3::ZERO;
    };
    let after = *w.bodies[body].position();
    let moved = after * before.inverse();
    flat(unv(moved.transform_point(v(point)))) - flat(point)
}

/// The body a collider's entity is, by the tracked bodies.
pub(crate) fn body_of(sim: &Sim, entity: u64) -> Option<rp::RigidBodyHandle> {
    sim.bodies.get(&entity).map(|t| t.handle)
}

/// Pushes the dynamic bodies a controller walked into, one rule for both
/// controllers: each body is asked to take (most of) the controller's speed
/// into it along the contact normal, an impulse of its mass times that
/// speed; the bodies of one tick share at most `strength` (N) times `dt`,
/// each its share of what it asked for, so a light body is shoved at the
/// controller's speed and a heavy one only as far as the force beats its
/// friction (a tonne on the ground holds), and two bodies walked into at
/// once split the push instead of the first taking it all. A body hit more
/// than once in the tick asks once, its largest.
pub(crate) fn push(
    w: &mut rp::PhysicsWorld,
    hits: &[(rp::ColliderHandle, Vec3)],
    velocity: Vec3,
    strength: f32,
    dt: f32,
) {
    let budget = strength.max(0.0) * dt;
    if budget <= 0.0 {
        return;
    }
    // (body, direction, impulse asked), in hit order.
    let mut asks: Vec<(rp::RigidBodyHandle, Vec3, f32)> = Vec::new();
    for &(h, normal) in hits {
        let Some(parent) = w.colliders.get(h).and_then(|c| c.parent()) else {
            continue;
        };
        let rb = &w.bodies[parent];
        if !rb.is_dynamic() {
            continue;
        }
        let into = -normal;
        let speed = velocity.dot(into) - unv(rb.linvel()).dot(into);
        if speed <= 0.0 {
            continue;
        }
        let ask = speed * rb.mass() * 0.8;
        match asks.iter_mut().find(|a| a.0 == parent) {
            Some(a) if a.2 < ask => *a = (parent, into, ask),
            Some(_) => {}
            None => asks.push((parent, into, ask)),
        }
    }
    let asked: f32 = asks.iter().map(|a| a.2).sum();
    let share = if asked > budget { budget / asked } else { 1.0 };
    for (body, into, ask) in asks {
        w.bodies[body].apply_impulse(v(flat(into * (ask * share))), true);
    }
}

// ---- the character controller ---------------------------------------------

/// Moves every og.CharacterController after the step, against the world as
/// it now is: ride the platform it stood on, take the input and gravity,
/// jump on the rising edge of `jump`, slide, climb, step and snap.
pub(crate) fn characters(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    sim: &mut Sim,
    poses: &BodyPoses,
    out: &mut StepOut,
) -> Result<()> {
    for (k, b) in scene.bodies.iter().enumerate() {
        let Some(crate::gather::Control::Character(mut c)) = b.control else {
            continue;
        };
        let Some(t) = sim.bodies.get(&b.entity.to_bits()) else {
            continue;
        };
        let handle = t.handle;
        let rb = &sim.world.bodies[handle];
        let Some(&ch) = rb.colliders().first() else {
            continue;
        };
        let collider = &sim.world.colliders[ch];
        let shape = collider.shared_shape().clone();
        let local = *collider
            .position_wrt_parent()
            .unwrap_or(&rp::Pose::IDENTITY);
        let body_at = pose(b.position, b.quaternion);
        let at = body_at * local;
        // What blocks it: the solids its layers and mask interact with.
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(handle)
            .exclude_sensors()
            .groups(collider.collision_groups());
        // Ride what it stood on.
        let foot = unv(at.translation);
        let ride = match body_of(sim, c.ground) {
            Some(g) if c.grounded => carried(poses, &sim.world, g, foot),
            _ => Vec3::ZERO,
        };
        let g = flat(settings.gravity) * c.gravity_scale;
        let mut vertical = if c.gravity_scale > 0.0 {
            c.velocity.y
        } else {
            0.0
        };
        c.coyote = if c.grounded {
            c.coyote_time
        } else {
            (c.coyote - dt).max(0.0)
        };
        let mut jumped = false;
        if c.jump && !c.jump_was_held && c.coyote > 0.0 && c.gravity_scale > 0.0 {
            vertical = c.jump_speed;
            c.coyote = 0.0;
            jumped = true;
        }
        if c.gravity_scale > 0.0 {
            vertical = (vertical + g.y * dt).max(-c.max_fall_speed);
            if c.grounded && !jumped && vertical < 0.0 {
                // Pressed into the ground, so snapping and slopes hold.
                vertical = vertical.max(g.y * dt);
            }
        }
        let across = if c.gravity_scale > 0.0 {
            Vec3::new(c.movement.x, 0.0, c.movement.z)
        } else {
            c.movement
        };
        let desired = flat((across + Vec3::Y * vertical) * dt);
        let start = foot + ride;
        let mover = Mover {
            qp: sim.world.query_pipeline_with_filter(filter),
            shape: &*shape,
            rot: at.rotation,
            skin: c.skin.max(0.001),
            walkable: og_core::math::cos(c.max_slope.clamp(0.0, 89.0).to_radians()),
            step_height: c.step_height.max(0.0),
        };
        let moved = if c.gravity_scale > 0.0 {
            // Snapping holds a character that stood on the ground down
            // slopes and steps; one in the air (falling from a jump or off
            // a ledge) lands where it meets the ground, not pulled down to
            // ground within snapDistance in one tick (rapier's own
            // controller snaps only when grounded at the start, too).
            let snap = if jumped || vertical > 0.0 || !c.grounded {
                0.0
            } else {
                c.snap_distance
            };
            mover.travel(start, desired, snap, c.grounded && !jumped)
        } else {
            mover.travel_free(start, desired)
        };
        let end = start + moved.translation;
        let was_grounded = c.grounded;
        let grounded = moved.ground.is_some() && !jumped && vertical <= 0.0;
        if moved.translation.y < desired.y - 1e-4 && vertical > 0.0 {
            vertical = 0.0; // a ceiling
        }
        if grounded && vertical < 0.0 {
            vertical = 0.0;
        }
        let ground = if grounded {
            moved
                .ground
                .map(|(h, n)| (entity_of(&sim.world.colliders[h]), n))
        } else {
            None
        };
        let speed_in = c.velocity.y;
        c.grounded = grounded;
        c.sliding = moved.sliding;
        c.velocity = flat(Vec3::new(
            moved.translation.x / dt,
            vertical,
            moved.translation.z / dt,
        )) + if c.gravity_scale > 0.0 {
            Vec3::ZERO
        } else {
            Vec3::new(0.0, moved.translation.y / dt, 0.0)
        };
        c.ground_normal = ground.map_or(Vec3::ZERO, |g| g.1);
        c.ground = ground.map_or(0, |g| g.0);
        c.jump_was_held = c.jump;
        // The body's new pose: its collider's displacement applied to the body.
        let new_body = rp::Pose::from_parts(v(end), at.rotation) * local.inverse();
        let (position, quaternion) = unpose(&new_body, b.position);
        out.moved[k] = Some((position, quaternion, None));
        if c.push_bodies {
            push(
                &mut sim.world,
                &moved.hits,
                desired / dt,
                c.push_strength,
                dt,
            );
        }
        if jumped {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_JUMPED,
                speed: vertical,
            });
        } else if grounded && !was_grounded {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_LANDED,
                speed: speed_in,
            });
        } else if !grounded && was_grounded {
            out.events.push(CharacterEvent {
                entity: b.entity.to_bits(),
                kind: CHARACTER_LEFT_GROUND,
                speed: vertical,
            });
        }
        out.controls.push((b.entity, ControlOut::Character(c)));
    }
    Ok(())
}

// ---- ropes, cloth, fluids, water -----------------------------------------

/// Where a tracked body or collider entity stands now (rope ends).
fn position_of(scene: &Scene, entity: u64) -> Option<Vec3> {
    scene
        .bodies
        .iter()
        .find(|b| b.entity.to_bits() == entity)
        .map(|b| b.position)
        .or_else(|| {
            scene
                .statics
                .iter()
                .find(|s| s.entity.to_bits() == entity)
                .map(|s| s.position)
        })
}

/// The bodies' final positions this tick, by entity (controllers moved).
fn final_position(scene: &Scene, out: &StepOut, entity: u64) -> Option<Vec3> {
    for (k, b) in scene.bodies.iter().enumerate() {
        if b.entity.to_bits() == entity {
            return Some(out.moved[k].map_or(b.position, |m| m.0));
        }
    }
    position_of(scene, entity)
}

/// Steps every rope and cloth, fluid and water after the bodies moved.
///
/// Each rope, each cloth and each fluid (alone, or its group's set) steps
/// from its own state and the solid colliders around it, read only, so
/// they step side by side on the lanes once their work gives two tasks of
/// a grain (`crate::lanes`), each one exactly as it would alone; building
/// and rebuilding them stays serial, in entity order.
#[allow(clippy::too_many_arguments)]
pub(crate) fn soft(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    shared: &mut crate::Shared,
    sim: &mut Sim,
    out: &mut StepOut,
    laps: &mut crate::stats::Laps,
) -> Result<()> {
    use crate::lanes::{Soft, soft_each};
    use crate::stats::Phase;
    let grains = crate::lanes::now();
    let flat2 = DIM == 2;
    let gravity = flat(settings.gravity);
    // The world's wind pushes each beside its own.
    let wind = flat(shared.wind);
    // Ropes.
    let live: BTreeSet<u64> = og_core::sort::set(scene.ropes.iter().map(|r| r.0.to_bits()));
    sim.ropes.retain(|e, _| live.contains(e));
    let mut ropes: IdMap<u64, (&Rope, Vec3, Option<Vec3>)> = IdMap::new();
    for (e, position, rope) in &scene.ropes {
        let start = flat_keep(*position + rope.offset, position.z, flat2);
        let end = if rope.end != 0 {
            final_position(scene, out, rope.end)
                .map(|p| flat_keep(p + rope.end_offset, position.z, flat2))
        } else {
            None
        };
        let key = e.to_bits();
        let rebuild = sim
            .ropes
            .get(&key)
            .is_none_or(|s| s.segments != rope.segments || s.length != rope.length);
        if rebuild {
            sim.ropes.insert(key, RopeState::new(rope, start, end));
        }
        ropes.insert(key, (rope, start, end));
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim.ropes.iter_mut().map(|(k, s)| (s, &ropes[k])).collect();
    let work = jobs
        .iter()
        .map(|(s, (rope, ..))| s.points.len() * rope.iterations.clamp(1, 64) as usize)
        .sum();
    soft_each(Soft::Ropes, &mut jobs, work, grains.rope_work, |job| {
        let (rope, start, end) = *job.1;
        let state = &mut *job.0;
        let (min, max) = bounds(&state.points, rope.length);
        let near = Near::around(colliders, min, max, rope.radius * 2.0 + 0.1);
        let rope = &Rope {
            wind: rope.wind + wind,
            ..*rope
        };
        state.step(rope, dt, gravity, start, end, flat2, &near);
    });
    // Cloth.
    let live: BTreeSet<u64> = og_core::sort::set(scene.cloths.iter().map(|c| c.0.to_bits()));
    sim.cloths.retain(|e, _| live.contains(e));
    let mut cloths: IdMap<u64, (&Cloth, Vec3, Quat)> = IdMap::new();
    for (e, position, rotation, cloth) in &scene.cloths {
        let key = e.to_bits();
        let rotation = if flat2 { Quat::IDENTITY } else { *rotation };
        let rebuild = sim.cloths.get(&key).is_none_or(|s| {
            s.columns != cloth.columns
                || s.rows != cloth.rows
                || s.spacing != cloth.spacing
                || s.pins != cloth.pins
        });
        if rebuild {
            sim.cloths
                .insert(key, ClothState::new(cloth, *position, rotation));
        }
        if cloth.simulation == SIMULATION_VISUAL {
            // The renderer simulates it; the tick carries its rest shape.
            sim.cloths
                .get_mut(&key)
                .expect("built above")
                .hold(cloth, *position, rotation);
            continue;
        }
        cloths.insert(key, (cloth, *position, rotation));
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim
        .cloths
        .iter_mut()
        .filter_map(|(k, s)| cloths.get(k).map(|c| (s, c)))
        .collect();
    let work = jobs
        .iter()
        .map(|(s, (cloth, ..))| {
            s.points.len()
                * cloth.iterations.clamp(1, 64) as usize
                * cloth.substeps.clamp(1, 16) as usize
        })
        .sum();
    soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
        let (cloth, position, rotation) = *job.1;
        let state = &mut *job.0;
        let size = cloth.spacing * (cloth.columns.max(cloth.rows)) as f32;
        let (min, max) = bounds(&state.points, size);
        let near = Near::around(colliders, min, max, cloth.radius * 2.0 + 0.1);
        let cloth = &Cloth {
            wind: cloth.wind + wind,
            ..*cloth
        };
        state.step(cloth, dt, tick, gravity, position, rotation, flat2, &near);
    });
    // Soft volumes (og.SoftBody), kept in the block's trailer.
    let live: BTreeSet<u64> = og_core::sort::set(scene.softs.iter().map(|s| s.0.to_bits()));
    sim.ext.softs.retain(|e, _| live.contains(e));
    let mut softs: IdMap<u64, (&SoftBody, Vec3, Quat)> = IdMap::new();
    for (e, position, rotation, body) in &scene.softs {
        let key = e.to_bits();
        let rebuild = sim
            .ext
            .softs
            .get(&key)
            .is_none_or(|s| s.key != crate::softbody::SoftKey::of(body));
        if rebuild {
            let mesh = (body.shape == SOFT_MESH)
                .then(|| super::sim::mesh(shared, body.mesh))
                .flatten();
            let state =
                crate::softbody::SoftState::new(body, *position, *rotation, flat2, mesh.as_deref())
                    .map_err(|e| e.context(format!("og.SoftBody of entity {key}")))?;
            sim.ext.softs.insert(key, state);
        }
        if body.simulation == SIMULATION_VISUAL {
            sim.ext
                .softs
                .get_mut(&key)
                .expect("built above")
                .hold(*position, *rotation);
        } else {
            softs.insert(key, (body, *position, *rotation));
        }
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim
        .ext
        .softs
        .iter_mut()
        .filter_map(|(k, s)| softs.get(k).map(|b| (s, b)))
        .collect();
    let work = jobs
        .iter()
        .map(|(s, (body, ..))| {
            s.points.len()
                * body.iterations.clamp(1, 64) as usize
                * body.substeps.clamp(1, 16) as usize
        })
        .sum();
    soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
        let (body, position, rotation) = *job.1;
        let state = &mut *job.0;
        let (min, max) = bounds(&state.points, 0.0);
        let near = Near::around(colliders, min, max, body.thickness * 2.0 + 0.5);
        state.step(body, dt, gravity, position, rotation, &near);
    });
    for (e, _, _, body) in &scene.softs {
        let s = &sim.ext.softs[&e.to_bits()];
        let volume = if body.simulation == SIMULATION_VISUAL {
            1.0
        } else if s.rest_volume > 0.0 {
            s.volume() / s.rest_volume
        } else {
            1.0
        };
        out.softs.push((
            *e,
            SoftBody {
                volume,
                center: s.center(),
                ..*body
            },
        ));
    }
    // Garments (og.Garment, 3D), each towards the pose the host gave it
    // this tick; one given none keeps its points.
    let live: BTreeSet<u64> = og_core::sort::set(scene.garments.iter().map(|g| g.0.to_bits()));
    sim.ext.garments.retain(|e, _| live.contains(e));
    let poses = std::mem::take(&mut shared.garment_poses);
    if !flat2 {
        let mut garments: IdMap<u64, (&Garment, &crate::GarmentPose)> = IdMap::new();
        for (e, garment) in &scene.garments {
            let key = e.to_bits();
            let (Some((rig_key, rig)), Some(pose)) =
                (shared.garment_rigs.get(&key), poses.get(&key))
            else {
                continue;
            };
            let n = rig.rest.len();
            if pose.targets.len() != n || pose.normals.len() != n {
                return Err(og_core::Error::new(
                    og_core::ErrorKind::Invalid,
                    format!(
                        "og.Garment of entity {key}: a pose of {} targets and {} normals for a rig of {n} points",
                        pose.targets.len(),
                        pose.normals.len()
                    ),
                ));
            }
            let rebuild = sim
                .ext
                .garments
                .get(&key)
                .is_none_or(|s| s.key != (*rig_key, garment.pin_top.to_bits()));
            if rebuild {
                sim.ext.garments.insert(
                    key,
                    crate::garment::GarmentState::new(garment, *rig_key, rig, pose),
                );
            }
            if garment.simulation == SIMULATION_VISUAL {
                sim.ext
                    .garments
                    .get_mut(&key)
                    .expect("built above")
                    .hold(pose);
            } else {
                garments.insert(key, (garment, pose));
            }
        }
        let colliders = &sim.world.colliders;
        let mut jobs: Vec<_> = sim
            .ext
            .garments
            .iter_mut()
            .filter_map(|(k, s)| garments.get(k).map(|g| (s, *g)))
            .collect();
        let work = jobs
            .iter()
            .map(|(s, (g, _))| {
                s.points.len()
                    * g.iterations.clamp(1, 64) as usize
                    * g.substeps.clamp(1, 16) as usize
            })
            .sum();
        soft_each(Soft::Cloths, &mut jobs, work, grains.cloth_work, |job| {
            let (garment, pose) = job.1;
            let state = &mut *job.0;
            let (min, max) = bounds(&state.points, garment.max_distance);
            let near = Near::around(colliders, min, max, garment.radius * 2.0 + 0.1);
            let garment = &Garment {
                wind: garment.wind + wind,
                ..*garment
            };
            state.step(garment, pose, dt, tick, gravity, &near);
        });
    }
    laps.lap(Phase::Soft);
    // Fluids: each alone, or with the others of its group (the first four
    // in entity order); each set steps from its own particles.
    let live: BTreeSet<u64> = og_core::sort::set(scene.fluids.iter().map(|f| f.0.to_bits()));
    sim.fluids.retain(|e, _| live.contains(e));
    let g = Vec2::new(settings.gravity.x, settings.gravity.y);
    let mut groups: IdMap<u32, Vec<usize>> = IdMap::new();
    for (k, (e, position, fluid)) in scene.fluids.iter().enumerate() {
        let key = e.to_bits();
        let centre = Vec2::new(position.x, position.y);
        let rebuild = sim
            .fluids
            .get(&key)
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed);
        if rebuild {
            sim.fluids.insert(key, FluidState::new(fluid, centre));
        }
        if fluid.group != 0 {
            groups.entry(fluid.group).or_default().push(k);
        }
    }
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut grouped: BTreeSet<u64> = BTreeSet::new();
    for members in groups.values_mut() {
        og_core::sort::sort_by_key(members, |&k| scene.fluids[k].0.to_bits());
        members.truncate(crate::fluid::GROUP_SIZE);
        if members.len() < 2 {
            continue;
        }
        grouped.extend(members.iter().map(|&k| scene.fluids[k].0.to_bits()));
        sets.push(members.clone());
    }
    for (k, (e, ..)) in scene.fluids.iter().enumerate() {
        if !grouped.contains(&e.to_bits()) {
            sets.push(vec![k]);
        }
    }
    let mut jobs: Vec<FluidSet<'_, FluidState>> = sets
        .iter()
        .map(|members| {
            let taken = members
                .iter()
                .map(|&k| {
                    let key = scene.fluids[k].0.to_bits();
                    (key, sim.fluids.remove(&key).expect("built above"))
                })
                .collect();
            (members.as_slice(), taken)
        })
        .collect();
    let work = jobs
        .iter()
        .map(|(members, taken)| {
            let particles: usize = taken.iter().map(|(_, s)| s.positions.len()).sum();
            let substeps = members
                .iter()
                .map(|&k| scene.fluids[k].2.substeps.clamp(1, 8))
                .max()
                .unwrap_or(1);
            particles * substeps as usize
        })
        .sum();
    let colliders = &sim.world.colliders;
    soft_each(Soft::Fluids, &mut jobs, work, grains.fluid_work, |job| {
        let (members, taken) = (job.0, &mut job.1);
        fluid_set(scene, members, taken, dt, g, colliders);
    });
    for (_, taken) in jobs {
        sim.fluids.extend(taken);
    }
    laps.lap(Phase::Fluids);
    fluids3(scene, settings, dt, sim);
    laps.lap(Phase::Fluids3d);
    Ok(())
}

/// A fluid set as it steps: its members (indices into the scene's fluids)
/// and their states, taken out of the simulation by entity.
type FluidSet<'a, S> = (&'a [usize], Vec<(u64, S)>);

/// One tick of a 2D fluid alone (`members` one index into `scene.fluids`)
/// or of a group's fluids together, against the solids around them.
fn fluid_set(
    scene: &Scene,
    members: &[usize],
    taken: &mut [(u64, FluidState)],
    dt: f32,
    g: Vec2,
    colliders: &rp::ColliderSet,
) {
    if let [k] = members {
        let (_, position, fluid) = &scene.fluids[*k];
        let centre = Vec2::new(position.x, position.y);
        let state = &mut taken[0].1;
        state.leave();
        let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in &state.positions {
            min = min.min(*p);
            max = max.max(*p);
        }
        let near = Near::around(
            colliders,
            Vec3::new(min.x, min.y, position.z),
            Vec3::new(max.x, max.y, position.z),
            fluid.radius * 2.0,
        );
        state.step(fluid, dt, g, centre, position.z, &near);
        return;
    }
    let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    let (mut z, mut reach) = (0.0f32, 0.0f32);
    for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
        let (_, position, fluid) = &scene.fluids[k];
        state.join(slot as u32);
        for p in &state.positions {
            min = min.min(*p);
            max = max.max(*p);
        }
        if slot == 0 {
            z = position.z;
        }
        reach = reach.max(fluid.radius * 2.0);
    }
    let near = Near::around(
        colliders,
        Vec3::new(min.x, min.y, z),
        Vec3::new(max.x, max.y, z),
        reach,
    );
    let mut list: Vec<crate::fluid::Member<'_>> = taken
        .iter_mut()
        .zip(members.iter())
        .map(|((_, state), &k)| {
            let (_, position, fluid) = &scene.fluids[k];
            crate::fluid::Member {
                fluid,
                state,
                centre: Vec2::new(position.x, position.y),
            }
        })
        .collect();
    crate::fluid::step_group(&mut list, dt, g, z, &near);
}

/// Steps the 3D fluids (the 3D profile only: the 2D one keeps none), each
/// alone or with the others of its group, as the 2D fluids are grouped:
/// each set from its own particles, side by side on the lanes as
/// [`soft`] steps the 2D ones.
fn fluids3(scene: &Scene, settings: &Physics, dt: f32, sim: &mut Sim) {
    use crate::fluid3d::{Fluid3State, Member, step};
    if DIM != 3 {
        sim.fluids3.clear();
        return;
    }
    let live: BTreeSet<u64> = og_core::sort::set(scene.fluids3.iter().map(|f| f.0.to_bits()));
    sim.fluids3.retain(|e, _| live.contains(e));
    // Each fluid's set: its group's first four (entity order), or itself.
    let mut sets: BTreeMap<(u32, u64), Vec<usize>> = BTreeMap::new();
    let mut groups: IdMap<u32, Vec<usize>> = IdMap::new();
    for (k, (e, position, fluid)) in scene.fluids3.iter().enumerate() {
        let key = e.to_bits();
        if fluid.simulation == SIMULATION_VISUAL {
            // The renderer's (SIMULATION_VISUAL): no particles in the
            // tick, in no group.
            let empty = Fluid3D {
                count: 0,
                emit_rate: 0.0,
                ..*fluid
            };
            if sim
                .fluids3
                .get(&key)
                .is_none_or(|s| !s.positions.is_empty())
            {
                sim.fluids3.insert(key, Fluid3State::new(&empty, *position));
            }
            continue;
        }
        let rebuild = sim
            .fluids3
            .get(&key)
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed);
        if rebuild {
            sim.fluids3.insert(key, Fluid3State::new(fluid, *position));
        }
        if fluid.group != 0 {
            groups.entry(fluid.group).or_default().push(k);
        } else {
            sets.insert((0, key), vec![k]);
        }
    }
    for (group, mut members) in groups {
        og_core::sort::sort_by_key(&mut members, |&k| scene.fluids3[k].0.to_bits());
        for (rank, &k) in members.iter().enumerate() {
            if rank >= crate::fluid::GROUP_SIZE {
                sets.insert((0, scene.fluids3[k].0.to_bits()), vec![k]);
            }
        }
        members.truncate(crate::fluid::GROUP_SIZE);
        if members.len() == 1 {
            sets.insert((0, scene.fluids3[members[0]].0.to_bits()), members);
        } else {
            sets.insert((group, 0), members);
        }
    }
    let mut jobs: Vec<FluidSet<'_, Fluid3State>> = sets
        .values()
        .map(|members| {
            let taken = members
                .iter()
                .map(|&k| {
                    let key = scene.fluids3[k].0.to_bits();
                    (key, sim.fluids3.remove(&key).expect("built above"))
                })
                .collect();
            (members.as_slice(), taken)
        })
        .collect();
    let substeps_of = |members: &[usize]| {
        members
            .iter()
            .map(|&k| scene.fluids3[k].2.substeps.clamp(1, 8))
            .max()
            .unwrap_or(1)
    };
    let work = jobs
        .iter()
        .map(|(members, taken)| {
            let particles: usize = taken.iter().map(|(_, s)| s.positions.len()).sum();
            particles * substeps_of(members) as usize
        })
        .sum();
    let per_task = crate::lanes::now().fluid_work;
    let colliders = &sim.world.colliders;
    let gravity = flat(settings.gravity);
    let soft = crate::lanes::Soft::Fluids3d;
    crate::lanes::soft_each(soft, &mut jobs, work, per_task, |job| {
        let (members, taken) = (job.0, &mut job.1);
        let grouped = members.len() > 1;
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let mut reach = 0.0f32;
        for (slot, ((_, state), &k)) in taken.iter_mut().zip(members).enumerate() {
            let (_, position, fluid) = &scene.fluids3[k];
            if grouped {
                state.join(slot as u32);
            } else {
                state.leave();
            }
            for p in &state.positions {
                min = min.min(*p);
                max = max.max(*p);
            }
            // The emitter's particles appear this tick: its cube too.
            if fluid.emit_rate > 0.0 {
                let at = *position + fluid.emit_offset;
                let r = Vec3::splat(fluid.emit_radius.max(0.0));
                min = min.min(at - r);
                max = max.max(at + r);
            }
            reach = reach.max(fluid.radius.max(0.02) * 2.0);
        }
        if min.x > max.x {
            return;
        }
        // A particle moves at most half the radius a substep.
        let substeps = substeps_of(members) as f32;
        let near = Near::around(colliders, min, max, reach * (1.0 + substeps * 0.25));
        let mut list: Vec<Member<'_>> = taken
            .iter_mut()
            .zip(members.iter())
            .map(|((_, state), &k)| {
                let (_, position, fluid) = &scene.fluids3[k];
                Member {
                    fluid,
                    state,
                    centre: *position,
                }
            })
            .collect();
        step(&mut list, dt, gravity, &near);
    });
    for (_, taken) in jobs {
        sim.fluids3.extend(taken);
    }
}

fn flat_keep(p: Vec3, z: f32, flat2: bool) -> Vec3 {
    if flat2 { Vec3::new(p.x, p.y, z) } else { p }
}

fn bounds(points: &[Vec3], slack: f32) -> (Vec3, Vec3) {
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    let grow = Vec3::splat(slack.max(0.0) * 0.25);
    (min - grow, max + grow)
}

/// The bodies water acts on, from the rapier world: every dynamic body (it
/// floats) and every controller (it splashes), with its colliders' box.
fn floaters(scene: &Scene, sim: &Sim, handles: &super::sim::Handles<'_>) -> Vec<Floater> {
    let mut list = Vec::new();
    for b in &scene.bodies {
        let moving = b.body.kind == BODY_DYNAMIC && b.control.is_none();
        if !(moving || b.control.is_some()) {
            continue;
        }
        // A body with og.Buoyancy floats by its own hull (`buoyancy`): the
        // surface only takes its splash.
        let floats = scene
            .floats
            .binary_search_by_key(&b.entity.to_bits(), |f| f.0.to_bits())
            .is_ok();
        let dynamic = moving && !floats;
        let Some(&h) = handles.get(&b.entity.to_bits()) else {
            continue;
        };
        let rb = &sim.world.bodies[h];
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for &c in rb.colliders() {
            let aabb = sim.world.colliders[c].compute_aabb();
            min = min.min(unv(aabb.mins));
            max = max.max(unv(aabb.maxs));
        }
        if min.x > max.x {
            continue;
        }
        if DIM == 2 {
            min.z = 0.0;
            max.z = 0.0;
        }
        let velocity = match b.control {
            Some(crate::gather::Control::Character(c)) => c.velocity,
            Some(crate::gather::Control::Platformer(p)) => p.velocity,
            None => unv(rb.linvel()),
        };
        list.push(Floater {
            entity: b.entity.to_bits(),
            min,
            max,
            centre: (min + max) * 0.5,
            velocity: flat(velocity),
            dynamic,
        });
    }
    list
}

/// Water before the step: each surface and line is splashed by and pushes
/// the bodies in it (buoyancy and drag on dynamic bodies), then its waves
/// travel one tick.
#[allow(clippy::too_many_arguments)]
pub(crate) fn water(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    // The wakes of every surface and sea still there (a sea's are og.Buoyancy's: `buoyancy`).
    let live: BTreeSet<u64> = og_core::sort::set(
        scene
            .surfaces
            .iter()
            .map(|s| s.0.to_bits())
            .chain(waters.iter().filter(|w| w.sea.is_some()).map(|w| w.entity)),
    );
    sim.wakes.retain(|e, _| live.contains(e));
    if scene.surfaces.is_empty() && scene.lines.is_empty() {
        sim.surfaces.clear();
        sim.lines.clear();
        return Ok(());
    }
    let list = floaters(scene, sim, handles);
    let g = -settings.gravity.y;
    let mut forces: Vec<WaterForce> = Vec::new();
    sim.surfaces.retain(|e, _| live.contains(e));
    for (e, position, surface) in &scene.surfaces {
        let key = e.to_bits();
        let rebuild = sim
            .surfaces
            .get(&key)
            .is_none_or(|s| s.resolution != surface.resolution || s.size != surface.size);
        if rebuild {
            sim.surfaces.insert(key, SurfaceState::new(surface));
        }
        let state = sim.surfaces.get_mut(&key).expect("inserted");
        let (f, crossings) = state.step(surface, dt, g, *position, &list);
        let (size, cell) = (state.size, state.cell());
        sim.wakes
            .entry(key)
            .or_default()
            .record(surface, size, cell, dt, tick, *position, &list);
        forces.extend(f);
        for c in crossings {
            out.splashes.push(Splash {
                water: key,
                entity: c.entity,
                point: c.point,
                speed: c.speed,
            });
        }
    }
    let live: BTreeSet<u64> = og_core::sort::set(scene.lines.iter().map(|s| s.0.to_bits()));
    sim.lines.retain(|e, _| live.contains(e));
    for (e, position, line) in &scene.lines {
        let key = e.to_bits();
        let rebuild = sim
            .lines
            .get(&key)
            .is_none_or(|s| s.columns != line.columns || s.width != line.width);
        if rebuild {
            sim.lines.insert(key, LineState::new(line));
        }
        let state = sim.lines.get_mut(&key).expect("inserted");
        let centre = Vec2::new(position.x, position.y);
        let (f, crossings) = state.step(line, dt, g, centre, &list);
        forces.extend(f);
        for c in crossings {
            out.splashes.push(Splash {
                water: key,
                entity: c.entity,
                point: c.point,
                speed: c.speed,
            });
        }
    }
    for f in forces {
        let Some(&h) = handles.get(&f.entity) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        let linvel = rb.linvel();
        rb.set_linvel(linvel * f.keep, true);
        rb.apply_impulse(v(flat(f.impulse)), true);
    }
    Ok(())
}

// ---- og.Buoyancy -------------------------------------------------------------

/// A body's hull for og.Buoyancy, in its own frame: the centres of the
/// cells of its colliders' box (`samples` a side, `lo` its low corner)
/// that lie in a solid collider, with their cell indices, each standing
/// for an equal share (`each`) of the colliders' volume; `cell` is a
/// cell's size.
struct Hull {
    points: Vec<(Vec3, [u32; 3])>,
    lo: Vec3,
    hi: Vec3,
    cell: Vec3,
    each: f32,
}

fn hull(colliders: &rp::ColliderSet, rb: &rp::RigidBody, samples: u32) -> Option<Hull> {
    let parts: Vec<&rp::Collider> = rb
        .colliders()
        .iter()
        .map(|&c| &colliders[c])
        .filter(|c| !c.is_sensor())
        .collect();
    let at = |c: &rp::Collider| *c.position_wrt_parent().unwrap_or(&rp::Pose::IDENTITY);
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for c in &parts {
        let b = c.shape().compute_aabb(&at(c));
        lo = lo.min(unv(b.mins));
        hi = hi.max(unv(b.maxs));
    }
    if parts.is_empty() || lo.x > hi.x {
        return None;
    }
    let n = samples.clamp(1, 12);
    let nz = if DIM == 3 { n } else { 1 };
    let cell = (hi - lo) / Vec3::new(n as f32, n as f32, nz as f32);
    let mut points = Vec::new();
    for k in 0..nz {
        for j in 0..n {
            for i in 0..n {
                let p = lo + cell * Vec3::new(i as f32 + 0.5, j as f32 + 0.5, k as f32 + 0.5);
                if parts.iter().any(|c| c.shape().contains_point(&at(c), v(p))) {
                    points.push((p, [i, j, k]));
                }
            }
        }
    }
    if points.is_empty() {
        let mid = n / 2;
        let p = lo + cell * Vec3::new(mid as f32 + 0.5, mid as f32 + 0.5, (nz / 2) as f32 + 0.5);
        points.push((p, [mid, mid, nz / 2]));
    }
    let volume: f32 = parts
        .iter()
        .map(|c| c.shape().mass_properties(1.0).mass())
        .sum();
    let volume = if volume > 0.0 {
        volume
    } else {
        cell.x * cell.y * cell.z.max(if DIM == 3 { 0.0 } else { 1.0 }) * points.len() as f32
    };
    Some(Hull {
        each: volume / points.len() as f32,
        points,
        lo,
        hi,
        cell,
    })
}

/// The water surface over world point (`x`, `z`): its height (world y)
/// and the water's velocity and acceleration there; the highest of the og.WaterSurface squares
/// and og.Water bodies over it (one entity with both: its rest level and
/// its sea together); `None` over no water. An og.WaterSurface lifts at its
/// rest level, as its own buoyancy does: its waves are the bodies'
/// splashes, and a body riding its own wake would pump them.
fn water_level(
    scene: &Scene,
    waters: &[crate::WaterBody],
    x: f32,
    z: f32,
    seconds: f64,
) -> Option<og_core::ocean::SurfacePoint> {
    use og_core::ocean::SurfacePoint;
    let mut level: Option<SurfacePoint> = None;
    let mut take = |y: f32, p: SurfacePoint| {
        if level.is_none_or(|l| y > l.height) {
            level = Some(SurfacePoint { height: y, ..p });
        }
    };
    let (x64, z64) = (f64::from(x), f64::from(z));
    for (e, position, surface) in &scene.surfaces {
        let half = surface.size * 0.5;
        if (x - position.x).abs() > half || (z - position.z).abs() > half {
            continue;
        }
        let key = e.to_bits();
        let sea = waters
            .iter()
            .find(|w| w.entity == key)
            .and_then(|w| w.surface(x64, z64, seconds))
            .unwrap_or_default();
        take(position.y + sea.height, sea);
    }
    for w in waters {
        if scene.surfaces.iter().any(|s| s.0.to_bits() == w.entity) {
            continue;
        }
        if let Some(p) = w.surface(x64, z64, seconds) {
            take(w.position.y + p.height, p);
        }
    }
    level
}

/// The footprint of a floating body's hull (its box `hull.lo` to `hull.hi`,
/// turned and placed by `at`) in each sea of `waters` it lies over:
/// the rectangle its own x and z axes span over the water's xz, its box,
/// and how far it reaches under the water at its centre.
fn hull_cuts(
    entity: u64,
    hull: &Hull,
    at: &rp::Pose,
    speed: f32,
    scene: &Scene,
    waters: &[crate::WaterBody],
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    let size = hull.hi - hull.lo;
    let centre = unv(at.transform_point(v(hull.lo + size * 0.5)));
    let turn = |a: Vec3| unv(at.rotation * v(a));
    let (ax, ay, az) = (turn(Vec3::X), turn(Vec3::Y), turn(Vec3::Z));
    let flat = |a: Vec3| og_core::glam::Vec2::new(a.x, a.z);
    let axis = flat(ax).try_normalize().unwrap_or(og_core::glam::Vec2::X);
    let side = og_core::glam::Vec2::new(-axis.y, axis.x);
    // The box's half extents along `axis` and across it.
    let half = size * 0.5;
    let reach = |d: og_core::glam::Vec2| {
        (flat(ax).dot(d) * half.x).abs()
            + (flat(ay).dot(d) * half.y).abs()
            + (flat(az).dot(d) * half.z).abs()
    };
    let foot = og_core::glam::Vec2::new(reach(axis), reach(side));
    let extent =
        og_core::glam::Vec2::new(reach(og_core::glam::Vec2::X), reach(og_core::glam::Vec2::Y));
    let low = centre.y - (ax.y.abs() * half.x + ay.y.abs() * half.y + az.y.abs() * half.z);
    let level =
        water_level(scene, waters, centre.x, centre.z, scene.seconds).map_or(low, |l| l.height);
    for w in waters.iter().filter(|w| w.sea.is_some()) {
        let Some(list) = cuts.get_mut(&w.entity) else {
            continue;
        };
        let local = og_core::glam::Vec2::new(centre.x - w.position.x, centre.z - w.position.z);
        if local.x.abs() > w.size.x * 0.5 + extent.x || local.y.abs() > w.size.y * 0.5 + extent.y {
            continue;
        }
        list.push(crate::water::WaterCut {
            entity,
            min: local - extent,
            max: local + extent,
            centre: local,
            half: foot,
            axis,
            depth: level - low,
            speed,
        });
    }
}

/// og.Buoyancy before the step (3D): each floating body's hull cells under
/// the water at their points push it up by the water they displace, at
/// those points (a torque that rights and pitches it), and slow it as far
/// as they are under; its `submerged` is written. Cells and bodies in a
/// fixed order, og-core's deterministic math: the same on every run. Each
/// sea (an og.Water with a wind) records the hulls in it for the renderer
/// ([`crate::water::Wakes::record_sea`]; never read by the tick).
#[allow(clippy::too_many_arguments)]
pub(crate) fn buoyancy(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) {
    let mut cuts: BTreeMap<u64, Vec<crate::water::WaterCut>> = waters
        .iter()
        .filter(|w| w.sea.is_some())
        .map(|w| (w.entity, Vec::new()))
        .collect();
    if DIM == 3 {
        floats(scene, settings, dt, waters, sim, handles, out, &mut cuts);
    }
    for (key, list) in cuts {
        sim.wakes.entry(key).or_default().record_sea(dt, tick, list);
    }
}

/// [`buoyancy`]'s floating bodies, each hull's footprint added to the
/// `cuts` of the seas it lies in.
#[allow(clippy::too_many_arguments)]
fn floats(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    waters: &[crate::WaterBody],
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
    cuts: &mut BTreeMap<u64, Vec<crate::water::WaterCut>>,
) {
    let g = -settings.gravity.y;
    for (e, b) in &scene.floats {
        let Some(&h) = handles.get(&e.to_bits()) else {
            continue;
        };
        let rb = &sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let Some(hull) = hull(&sim.world.colliders, rb, b.samples) else {
            continue;
        };
        let at = *rb.position();
        // A cell's half height in the world, its box turned with the body.
        let up = [Vec3::X, Vec3::Y, Vec3::Z].map(|a| unv(at.rotation * v(a)).y.abs());
        let half = 0.5 * (up[0] * hull.cell.x + up[1] * hull.cell.y + up[2] * hull.cell.z);
        // The water level is read once per column of cells along the body's
        // axis nearest the world's up, at the column's middle: a sea's
        // height costs some 300 sines a point.
        let column = if up[1] >= up[0] && up[1] >= up[2] {
            1
        } else if up[0] >= up[2] {
            0
        } else {
            2
        };
        let middle =
            (hull.lo + hull.cell * Vec3::splat(b.samples.clamp(1, 12) as f32) * 0.5)[column];
        // Each column's level and the water's velocity there (its
        // particles' orbit): a hull is slowed against the moving water, not
        // still water, so it is carried round with it as a float is and
        // rides through a crest. (Against the level's rise at a fixed point,
        // the wave's phase speed times its slope, steep crests flung hulls
        // clear of the water.)
        let mut levels: BTreeMap<[u32; 2], Option<og_core::ocean::SurfacePoint>> = BTreeMap::new();
        let mass = rb.mass();
        let share = 1.0 / hull.points.len() as f32;
        let mut under = 0.0;
        let mut pushes = Vec::with_capacity(hull.points.len());
        for (p, index) in &hull.points {
            let w = unv(at.transform_point(v(*p)));
            let key = match column {
                0 => [index[1], index[2]],
                1 => [index[0], index[2]],
                _ => [index[0], index[1]],
            };
            let level = *levels.entry(key).or_insert_with(|| {
                let mut q = *p;
                q[column] = middle;
                let c = unv(at.transform_point(v(q)));
                water_level(scene, waters, c.x, c.z, scene.seconds)
            });
            let Some(water) = level else {
                continue;
            };
            let level = water.height;
            let fraction = if half > 1e-6 {
                ((level - (w.y - half)) / (2.0 * half)).clamp(0.0, 1.0)
            } else if w.y < level {
                1.0
            } else {
                0.0
            };
            if fraction <= 0.0 {
                continue;
            }
            under += fraction * share;
            pushes.push((w, fraction, water));
        }
        let rb = &mut sim.world.bodies[h];
        for (w, fraction, water) in pushes {
            // Archimedes in moving water (Froude–Krylov): the pressure that
            // buoys a cell pushes it as hard as it accelerates the water it
            // displaces against gravity, ρV(g + a), so where a crest falls
            // away the lift falls with it and a floater sinks with the water
            // at its draft instead of being flung clear.
            let held = Vec3::Y * g + water.acceleration;
            let held = Vec3::new(held.x, held.y.max(0.0), held.z);
            let lift = held * (b.water_density * hull.each * fraction * dt);
            let flow = water.velocity;
            // The drag: its share of the mass losing `drag` of its speed
            // through the water a second, never more than all of it.
            let speed = unv(rb.velocity_at_point(v(w))) - flow;
            let slow = (b.drag * dt).min(1.0) * mass * share * fraction;
            rb.apply_impulse_at_point(v(flat(lift - speed * slow)), v(w), true);
        }
        if under > 0.0 {
            let keep = 1.0 - (b.angular_drag * under * dt).min(1.0);
            let spin = unangvel(rb.angvel());
            rb.set_angvel(angvel(spin * keep), true);
        }
        hull_cuts(
            e.to_bits(),
            &hull,
            &at,
            unv(rb.linvel()).y,
            scene,
            waters,
            cuts,
        );
        out.floats.push((
            *e,
            Buoyancy {
                submerged: under,
                ..*b
            },
        ));
    }
}
