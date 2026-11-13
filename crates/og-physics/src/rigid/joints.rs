//! og.Joint: built, synced, broken and read back around the rigid-body step.

use super::*;

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
    Some((super::super::sim::EXT_JOINTS, bytes))
}

/// Puts [`joint_section`]'s values back on the restored joints.
pub(crate) fn restore_joint_section(
    joints: &mut IdMap<u64, TrackedJoint>,
    sections: &[(u16, &[u8])],
) -> Result<()> {
    use bincode::Options;
    for &(tag, bytes) in sections {
        if tag != super::super::sim::EXT_JOINTS {
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
                JOINT_BALL => super::super::ball_axes(),
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
    handles: &super::super::sim::Handles<'_>,
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
        let (linear, angular) = super::super::joint_load(joint, &sim.world.bodies);
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
            JOINT_REVOLUTE => super::super::hinge_angle(&f1, &f2),
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
