//! Physics beyond rigid bodies, written once and compiled for each profile
//! (like `sim.rs`): joints, the character controller, Verlet ropes and
//! cloth, the 2D fluid and water, run by `sim::step` around the rigid-body
//! step. `mover.rs` is the controllers' move against the world (both
//! profiles); `platformer.rs` (2D), `vehicle2.rs` / `vehicle3.rs` and
//! `ragdoll.rs` (both profiles, through each one's joint) hold what the profiles do differently.

use super::mover::Mover;
use super::sim::{BodyPoses, Sim};
use super::{DIM, flat, heightfield_surface, joint_frame, pose, rp, surface_only, unpose, unv, v};
use crate::fluid::FluidState;
use crate::gather::Scene;
use crate::schema::*;
use crate::schema_ext::*;
use crate::verlet::{ClothState, RopeState, Solids};
use crate::water::{Floater, LineState, SurfaceState, WaterForce};
use crate::{ControlOut, StepOut};
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::{Error, ErrorKind, Result};
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
            let snap = if jumped || vertical > 0.0 {
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
pub(crate) fn soft(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    sim: &mut Sim,
    out: &mut StepOut,
    laps: &mut crate::stats::Laps,
) -> Result<()> {
    use crate::lanes::soft_grain;
    use crate::stats::Phase;
    let site = og_lanes::Site::Physics;
    let grains = crate::lanes::now();
    let flat2 = DIM == 2;
    let gravity = flat(settings.gravity);
    // Ropes.
    let live: BTreeSet<u64> = og_core::sort::set(scene.ropes.iter().map(|r| r.0.to_bits()));
    sim.ropes.retain(|e, _| live.contains(e));
    let mut ropes: BTreeMap<u64, (&Rope, Vec3, Option<Vec3>)> = BTreeMap::new();
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
    let grain = soft_grain(jobs.len(), work, grains.rope_work);
    og_lanes::for_each_mut(site, &mut jobs, grain, |job| {
        let (rope, start, end) = *job.1;
        let state = &mut *job.0;
        let (min, max) = bounds(&state.points, rope.length);
        let near = Near::around(colliders, min, max, rope.radius * 2.0 + 0.1);
        state.step(rope, dt, gravity, start, end, flat2, &near);
    });
    // Cloth.
    let live: BTreeSet<u64> = og_core::sort::set(scene.cloths.iter().map(|c| c.0.to_bits()));
    sim.cloths.retain(|e, _| live.contains(e));
    let mut cloths: BTreeMap<u64, (&Cloth, Vec3, Quat)> = BTreeMap::new();
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
        cloths.insert(key, (cloth, *position, rotation));
    }
    let colliders = &sim.world.colliders;
    let mut jobs: Vec<_> = sim
        .cloths
        .iter_mut()
        .map(|(k, s)| (s, &cloths[k]))
        .collect();
    let work = jobs
        .iter()
        .map(|(s, (cloth, ..))| {
            s.points.len()
                * cloth.iterations.clamp(1, 64) as usize
                * cloth.substeps.clamp(1, 16) as usize
        })
        .sum();
    let grain = soft_grain(jobs.len(), work, grains.cloth_work);
    og_lanes::for_each_mut(site, &mut jobs, grain, |job| {
        let (cloth, position, rotation) = *job.1;
        let state = &mut *job.0;
        let size = cloth.spacing * (cloth.columns.max(cloth.rows)) as f32;
        let (min, max) = bounds(&state.points, size);
        let near = Near::around(colliders, min, max, cloth.radius * 2.0 + 0.1);
        state.step(cloth, dt, tick, gravity, position, rotation, flat2, &near);
    });
    laps.lap(Phase::Soft);
    // Fluids: each alone, or with the others of its group (the first four
    // in entity order); each set steps from its own particles.
    let live: BTreeSet<u64> = og_core::sort::set(scene.fluids.iter().map(|f| f.0.to_bits()));
    sim.fluids.retain(|e, _| live.contains(e));
    let g = Vec2::new(settings.gravity.x, settings.gravity.y);
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
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
    let grain = soft_grain(jobs.len(), work, grains.fluid_work);
    let colliders = &sim.world.colliders;
    og_lanes::for_each_mut(site, &mut jobs, grain, |job| {
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
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (k, (e, position, fluid)) in scene.fluids3.iter().enumerate() {
        let key = e.to_bits();
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
    let grain = crate::lanes::soft_grain(jobs.len(), work, crate::lanes::now().fluid_work);
    let colliders = &sim.world.colliders;
    let gravity = flat(settings.gravity);
    og_lanes::for_each_mut(og_lanes::Site::Physics, &mut jobs, grain, |job| {
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
        let dynamic = b.body.kind == BODY_DYNAMIC && b.control.is_none();
        if !(dynamic || b.control.is_some()) {
            continue;
        }
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
pub(crate) fn water(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    if scene.surfaces.is_empty() && scene.lines.is_empty() {
        sim.surfaces.clear();
        sim.lines.clear();
        return Ok(());
    }
    let list = floaters(scene, sim, handles);
    let g = -settings.gravity.y;
    let mut forces: Vec<WaterForce> = Vec::new();
    let live: BTreeSet<u64> = og_core::sort::set(scene.surfaces.iter().map(|s| s.0.to_bits()));
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
