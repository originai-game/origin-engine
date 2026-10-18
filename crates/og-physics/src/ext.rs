//! Physics beyond rigid bodies, written once and compiled for each profile
//! (like `sim.rs`): joints, the character controller, Verlet ropes and
//! cloth, the 2D fluid and water, run by `sim::step` around the rigid-body
//! step. `platformer.rs` (2D), `vehicle2.rs` / `vehicle3.rs` and
//! `ragdoll3.rs` hold what only one profile has.

use super::sim::{BodyPoses, Sim};
use super::{DIM, flat, joint_frame, pose, rp, unpose, unv, v};
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
    pub fn around(w: &'a rp::PhysicsWorld, min: Vec3, max: Vec3, margin: f32) -> Near<'a> {
        let lo = flat(min) - Vec3::splat(margin);
        let hi = flat(max) + Vec3::splat(margin);
        let near = w
            .colliders
            .iter()
            .filter(|(_, c)| !c.is_sensor() && c.collision_groups().memberships.bits() != 0)
            .filter(|(_, c)| {
                let b = c.compute_aabb();
                let (bmin, bmax) = (unv(b.mins), unv(b.maxs));
                (0..DIM).all(|k| bmin[k] <= hi[k] && bmax[k] >= lo[k])
            })
            .map(|(h, _)| h)
            .collect();
        Near {
            colliders: &w.colliders,
            near,
        }
    }
}

impl Solids for Near<'_> {
    fn push_out(&self, p: Vec3, radius: f32) -> Option<(Vec3, Vec3)> {
        let mut at = p;
        let mut hit = None;
        for &h in &self.near {
            let c = &self.colliders[h];
            let proj = c.shape().project_point(c.position(), v(at), false);
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
    handles: &BTreeMap<u64, rp::RigidBodyHandle>,
) -> Result<()> {
    let live: BTreeSet<u64> = scene.joints.iter().map(|(e, _)| e.to_bits()).collect();
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

/// What a controller's move did.
pub(crate) struct Moved {
    pub translation: Vec3,
    pub grounded: bool,
    pub sliding: bool,
    /// What it hit on the way: the collider and its surface normal (world).
    pub hits: Vec<(rp::ColliderHandle, Vec3)>,
}

/// The settings rapier's character controller moves a shape with.
pub(crate) struct MoveDesc {
    pub skin: f32,
    pub max_slope: f32,
    pub step_height: f32,
    pub snap: f32,
}

/// Moves `shape` from `at` by `desired` against the world, sliding.
/// `filter` decides which colliders block it.
pub(crate) fn move_shape(
    w: &rp::PhysicsWorld,
    filter: rp::QueryFilter<'_>,
    shape: &dyn rp::Shape,
    at: &rp::Pose,
    desired: Vec3,
    d: &MoveDesc,
    dt: f32,
) -> Moved {
    let slope = d.max_slope.clamp(0.0, 89.0).to_radians();
    // A step is taken only once the shape's bottom clears the edge: the
    // width it needs on top is its horizontal half extent.
    let reach = {
        let b = shape.compute_aabb(&rp::Pose::IDENTITY);
        let half = (unv(b.maxs) - unv(b.mins)) * 0.5;
        half.x.max(if DIM == 3 { half.z } else { 0.0 })
    };
    let kcc = super::ctl::KinematicCharacterController {
        up: v(Vec3::Y),
        offset: super::ctl::CharacterLength::Absolute(d.skin.max(0.001)),
        slide: true,
        autostep: (d.step_height > 0.0).then_some(super::ctl::CharacterAutostep {
            max_height: super::ctl::CharacterLength::Absolute(d.step_height),
            min_width: super::ctl::CharacterLength::Absolute(reach + 0.05),
            include_dynamic_bodies: false,
        }),
        max_slope_climb_angle: slope,
        min_slope_slide_angle: slope,
        snap_to_ground: (d.snap > 0.0).then_some(super::ctl::CharacterLength::Absolute(d.snap)),
        normal_nudge_factor: 1.0e-4,
    };
    let qp = w.query_pipeline_with_filter(filter);
    let mut hits = Vec::new();
    let m = kcc.move_shape(dt, &qp, shape, at, v(flat(desired)), |c| {
        hits.push((c.handle, unv(c.hit.normal1)));
    });
    Moved {
        translation: unv(m.translation),
        grounded: m.grounded,
        sliding: m.is_sliding_down_slope,
        hits,
    }
}

/// The collider under a shape at `at` within `reach` (its entity and the
/// surface normal), ignoring `own`.
pub(crate) fn ground_under(
    w: &rp::PhysicsWorld,
    filter: rp::QueryFilter<'_>,
    shape: &dyn rp::Shape,
    at: &rp::Pose,
    reach: f32,
) -> Option<(u64, Vec3)> {
    let qp = w.query_pipeline_with_filter(filter);
    let options = super::ShapeCastOptions::with_max_time_of_impact(reach);
    qp.cast_shape(at, v(-Vec3::Y), shape, options)
        .map(|(h, hit)| (entity_of(&w.colliders[h]), unv(hit.normal1)))
}

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

/// Pushes the dynamic bodies a controller walked into: each takes the
/// controller's speed into it along the contact normal.
pub(crate) fn push(w: &mut rp::PhysicsWorld, hits: &[(rp::ColliderHandle, Vec3)], velocity: Vec3) {
    for &(h, normal) in hits {
        let Some(parent) = w.colliders.get(h).and_then(|c| c.parent()) else {
            continue;
        };
        let rb = &mut w.bodies[parent];
        if !rb.is_dynamic() {
            continue;
        }
        let into = -normal;
        let speed = velocity.dot(into) - unv(rb.linvel()).dot(into);
        if speed > 0.0 {
            let impulse = into * (speed * rb.mass() * 0.8);
            rb.apply_impulse(v(flat(impulse)), true);
        }
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
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(handle)
            .exclude_sensors();
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
        let start = rp::Pose::from_parts(v(foot + ride), at.rotation);
        let moved = move_shape(
            &sim.world,
            filter,
            &*shape,
            &start,
            desired,
            &MoveDesc {
                skin: c.skin,
                max_slope: c.max_slope,
                step_height: c.step_height,
                snap: if jumped || vertical > 0.0 {
                    0.0
                } else {
                    c.snap_distance
                },
            },
            dt,
        );
        let end = foot + ride + moved.translation;
        let was_grounded = c.grounded;
        let grounded =
            moved.grounded && !(jumped || (vertical > 0.0 && moved.translation.y > 1e-5));
        if moved.translation.y < desired.y - 1e-4 && vertical > 0.0 {
            vertical = 0.0; // a ceiling
        }
        if grounded && vertical < 0.0 {
            vertical = 0.0;
        }
        let under = rp::Pose::from_parts(v(end), at.rotation);
        let ground = if grounded {
            ground_under(&sim.world, filter, &*shape, &under, c.skin * 2.0 + 0.02)
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
            push(&mut sim.world, &moved.hits, desired / dt);
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
pub(crate) fn soft(
    scene: &Scene,
    settings: &Physics,
    dt: f32,
    tick: u64,
    sim: &mut Sim,
    out: &mut StepOut,
) -> Result<()> {
    let flat2 = DIM == 2;
    let gravity = flat(settings.gravity);
    // Ropes.
    let live: BTreeSet<u64> = scene.ropes.iter().map(|r| r.0.to_bits()).collect();
    sim.ropes.retain(|e, _| live.contains(e));
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
        let state = sim.ropes.get_mut(&key).expect("inserted");
        let (min, max) = bounds(&state.points, rope.length);
        let near = Near::around(&sim.world, min, max, rope.radius * 2.0 + 0.1);
        state.step(rope, dt, gravity, start, end, flat2, &near);
    }
    // Cloth.
    let live: BTreeSet<u64> = scene.cloths.iter().map(|c| c.0.to_bits()).collect();
    sim.cloths.retain(|e, _| live.contains(e));
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
        let state = sim.cloths.get_mut(&key).expect("inserted");
        let size = cloth.spacing * (cloth.columns.max(cloth.rows)) as f32;
        let (min, max) = bounds(&state.points, size);
        let near = Near::around(&sim.world, min, max, cloth.radius * 2.0 + 0.1);
        state.step(cloth, dt, tick, gravity, *position, rotation, flat2, &near);
    }
    // Fluids.
    let live: BTreeSet<u64> = scene.fluids.iter().map(|f| f.0.to_bits()).collect();
    sim.fluids.retain(|e, _| live.contains(e));
    for (e, position, fluid) in &scene.fluids {
        let key = e.to_bits();
        let centre = Vec2::new(position.x, position.y);
        let rebuild = sim
            .fluids
            .get(&key)
            .is_none_or(|s| s.count != fluid.count || s.seed != fluid.seed);
        if rebuild {
            sim.fluids.insert(key, FluidState::new(fluid, centre));
        }
        let state = sim.fluids.get_mut(&key).expect("inserted");
        let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in &state.positions {
            min = min.min(*p);
            max = max.max(*p);
        }
        let near = Near::around(
            &sim.world,
            Vec3::new(min.x, min.y, position.z),
            Vec3::new(max.x, max.y, position.z),
            fluid.radius * 2.0,
        );
        let g = Vec2::new(settings.gravity.x, settings.gravity.y);
        state.step(fluid, dt, g, centre, position.z, &near);
    }
    Ok(())
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
fn floaters(
    scene: &Scene,
    sim: &Sim,
    handles: &BTreeMap<u64, rp::RigidBodyHandle>,
) -> Vec<Floater> {
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
    handles: &BTreeMap<u64, rp::RigidBodyHandle>,
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
    let live: BTreeSet<u64> = scene.surfaces.iter().map(|s| s.0.to_bits()).collect();
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
    let live: BTreeSet<u64> = scene.lines.iter().map(|s| s.0.to_bits()).collect();
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
