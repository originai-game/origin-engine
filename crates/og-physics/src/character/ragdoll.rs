//! Rest-frame ragdolls. Bodies live at bone origins; collider mass centres
//! are offset along the bone. Activation therefore transfers COM velocity,
//! not origin velocity. A rig is explicit and ordered, never name-inferred.

use super::sim::Sim;
use super::{pose, ragdoll_joint, ragdoll_motor, rot, rp, unpose, unv, v};
use crate::gather::Scene;
use crate::schema::*;
use crate::{RagdollRig, StepOut};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RagdollState {
    pub(crate) bones: Vec<rp::RigidBodyHandle>,
    joints: Vec<(u32, rp::ImpulseJointHandle)>,
    motors: Vec<(u32, rp::ImpulseJointHandle)>,
    exclusions: Vec<rp::ImpulseJointHandle>,
    pub(crate) dynamic: bool,
    stepped: bool,
    last: Vec<(Vec3, Quat)>,
    rig: Vec<u32>,
    tuned: Vec<u32>,
    owner: Option<rp::RigidBodyHandle>,
}

impl RagdollState {
    pub(crate) fn shift(&mut self, s: &og_ecs::Shift) {
        for (p, _) in &mut self.last {
            *p = s.moved(*p);
        }
    }
}

fn rig_key(rig: &RagdollRig) -> Vec<u32> {
    let mut key = vec![rig.bones.len() as u32];
    for b in &rig.bones {
        key.extend([b.parent.unwrap_or(u32::MAX), b.shape, b.joint_kind]);
        key.extend(b.rest.0.to_array().into_iter().map(f32::to_bits));
        key.extend(b.rest.1.to_array().into_iter().map(f32::to_bits));
        key.extend(b.axis.to_array().into_iter().map(f32::to_bits));
        key.extend(b.hinge_axis.to_array().into_iter().map(f32::to_bits));
        key.extend([b.length, b.mass, b.swing, b.twist_min, b.twist_max].map(f32::to_bits));
    }
    key
}

fn remove(w: &mut rp::PhysicsWorld, state: &RagdollState) {
    for &h in &state.bones {
        w.remove_body(h);
    }
}

fn exclude_owner(
    w: &mut rp::PhysicsWorld,
    bones: &[rp::RigidBodyHandle],
    owner: Option<rp::RigidBodyHandle>,
) -> Vec<rp::ImpulseJointHandle> {
    let Some(owner) = owner else {
        return Vec::new();
    };
    bones
        .iter()
        .map(|&h| {
            // A free joint constrains no motion but excludes this pair's contacts.
            w.impulse_joints.insert(
                owner,
                h,
                rp::GenericJointBuilder::new(rp::JointAxesMask::empty())
                    .contacts_enabled(false)
                    .build(),
                false,
            )
        })
        .collect()
}

fn shape(b: &crate::RagdollBone, thickness: f32) -> (rp::SharedShape, rp::Pose) {
    let len = b.length;
    let radius = (len * thickness.max(0.0)).min(len * 0.5);
    let shape = super::ragdoll_shape(b.shape, len, radius);
    let dir = b.axis.normalize_or(Vec3::Y);
    (
        shape,
        rp::Pose::from_parts(v(dir * (len * 0.5)), rot(crate::arc(Vec3::Y, dir))),
    )
}

fn joint(rig: &RagdollRig, i: usize, limit: f32) -> rp::GenericJoint {
    let b = &rig.bones[i];
    let parent = &rig.bones[b.parent.expect("joint has parent") as usize];
    let pp = pose(parent.rest.0, parent.rest.1);
    let cp = pose(b.rest.0, b.rest.1);
    // Both frames name the same rest-world basis, with X along the
    // supplied bone/hinge axis. Rest rotation is zero joint error.
    let axis = if b.joint_kind == 1 {
        b.hinge_axis
    } else {
        b.axis
    };
    let basis = b.rest.1 * crate::arc(Vec3::X, axis.normalize_or(Vec3::Y));
    let frame = pose(b.rest.0, basis);
    ragdoll_joint(pp.inverse() * frame, cp.inverse() * frame, b, limit)
}

fn build(
    w: &mut rp::PhysicsWorld,
    entity: u64,
    rig: &RagdollRig,
    r: &Ragdoll,
    targets: &[(Vec3, Quat)],
    owner: Option<rp::RigidBodyHandle>,
) -> RagdollState {
    let mut bones = Vec::with_capacity(rig.bones.len());
    for &(p, q) in targets {
        let h = w.bodies.insert(
            rp::RigidBodyBuilder::kinematic_position_based()
                .pose(pose(p, q))
                .can_sleep(false)
                .user_data(entity as u128),
        );
        bones.push(h);
    }
    let exclusions = exclude_owner(w, &bones, owner);
    let mut motors = Vec::new();
    let joints = rig
        .bones
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            let p = b.parent? as usize;
            let constraint = joint(rig, i, r.joint_limit);
            // Rapier does not implement coupled angular motors. Keep the
            // cone limit and the independent PD axes in separate joints,
            // sharing the same rest frames, rather than silently losing swing drive.
            let motor = rp::GenericJointBuilder::new(rp::JointAxesMask::empty())
                .local_frame1(constraint.local_frame1)
                .local_frame2(constraint.local_frame2)
                .contacts_enabled(false)
                .build();
            motors.push((
                i as u32,
                w.impulse_joints.insert(bones[p], bones[i], motor, false),
            ));
            Some((
                i as u32,
                w.impulse_joints
                    .insert(bones[p], bones[i], constraint, false),
            ))
        })
        .collect();
    let mut state = RagdollState {
        bones,
        joints,
        motors,
        exclusions,
        dynamic: false,
        stepped: false,
        last: targets.to_vec(),
        rig: rig_key(rig),
        tuned: Vec::new(),
        owner,
    };
    tune(w, &mut state, rig, r);
    state
}

/// Tunables update existing bodies/colliders/joints without losing momentum.
fn tune(w: &mut rp::PhysicsWorld, s: &mut RagdollState, rig: &RagdollRig, r: &Ragdoll) {
    let key = vec![
        r.mass.to_bits(),
        r.thickness.to_bits(),
        r.joint_limit.to_bits(),
        r.damping.to_bits(),
        r.layers,
        r.mask,
    ];
    if key == s.tuned {
        return;
    }
    s.tuned = key;
    let total: f32 = rig
        .bones
        .iter()
        .filter(|b| b.mass == 0.0)
        .map(|b| b.length)
        .sum();
    let groups = rp::InteractionGroups::new(
        rp::Group::from_bits_retain(r.layers),
        rp::Group::from_bits_retain(r.mask),
        rp::InteractionTestMode::And,
    );
    for (&h, b) in s.bones.iter().zip(&rig.bones) {
        let mass = if b.mass > 0.0 {
            b.mass
        } else {
            r.mass.max(0.0) * b.length / total.max(f32::MIN_POSITIVE)
        };
        let (shape, local) = shape(b, r.thickness);
        let rb = &mut w.bodies[h];
        rb.set_linear_damping(r.damping.max(0.0));
        rb.set_angular_damping(r.damping.max(0.0));
        if let Some(&ch) = rb.colliders().first() {
            let c = &mut w.colliders[ch];
            c.set_shape(shape);
            c.set_position_wrt_parent(local);
            c.set_mass(mass);
            c.set_collision_groups(groups);
        } else {
            w.colliders.insert_with_parent(
                rp::ColliderBuilder::new(shape)
                    .position(local)
                    .mass(mass)
                    .friction(0.7)
                    .collision_groups(groups)
                    .user_data(w.bodies[h].user_data),
                h,
                &mut w.bodies,
            );
        }
        w.bodies[h].recompute_mass_properties_from_colliders(&w.colliders);
    }
    for &(i, h) in &s.joints {
        if let Some(j) = w.impulse_joints.get_mut(h, false) {
            j.data = joint(rig, i as usize, r.joint_limit);
        }
    }
}

fn rest(rig: &RagdollRig, position: Vec3, rotation: Quat) -> Vec<(Vec3, Quat)> {
    rig.bones
        .iter()
        .map(|b| {
            (
                position + rotation * b.rest.0,
                (rotation * b.rest.1).normalize(),
            )
        })
        .collect()
}

pub(crate) fn ragdolls(
    scene: &Scene,
    dt: f32,
    sim: &mut Sim,
    rigs: &IdMap<u64, Arc<RagdollRig>>,
    targets: &IdMap<u64, Vec<(Vec3, Quat)>>,
    out: &mut StepOut,
) -> Result<()> {
    let live: BTreeSet<u64> = og_core::sort::set(scene.ragdolls.iter().map(|r| r.0.to_bits()));
    let gone: Vec<u64> = sim
        .ragdolls
        .keys()
        .filter(|e| !live.contains(e))
        .copied()
        .collect();
    for e in gone {
        if let Some(s) = sim.ragdolls.remove(&e) {
            remove(&mut sim.world, &s);
        }
    }
    for &(e, position, rotation, mut r) in &scene.ragdolls {
        let key = e.to_bits();
        let Some(rig) = rigs.get(&key) else { continue };
        let target = targets
            .get(&key)
            .cloned()
            .unwrap_or_else(|| rest(rig, position, rotation));
        if target.len() != rig.bones.len()
            || target.iter().any(|(p, q)| !p.is_finite() || !q.is_finite())
        {
            return Err(Error::new(
                ErrorKind::Invalid,
                "ragdoll targets must be finite and match the rig",
            ));
        }
        let rig_key = rig_key(rig);
        // `sim.bodies` is completed-step tracking: newly inserted owners
        // are not there yet. Read live bodies, excluding this rig's bones.
        let owner = sim
            .world
            .bodies
            .iter()
            .find(|(h, b)| {
                b.user_data == key as u128
                    && sim.ragdolls.get(&key).is_none_or(|s| !s.bones.contains(h))
            })
            .map(|(h, _)| h);
        if sim.ragdolls.get(&key).is_none_or(|s| s.rig != rig_key) {
            if let Some(s) = sim.ragdolls.remove(&key) {
                remove(&mut sim.world, &s);
            }
            let s = build(&mut sim.world, key, rig, &r, &target, owner);
            sim.ragdolls.insert(key, s);
        }
        let state = sim.ragdolls.get_mut(&key).expect("built");
        if state.owner != owner {
            for h in state.exclusions.drain(..) {
                sim.world.impulse_joints.remove(h, false);
            }
            state.exclusions = exclude_owner(&mut sim.world, &state.bones, owner);
            state.owner = owner;
        }
        tune(&mut sim.world, state, rig, &r);
        let rate = if r.blend_time > 0.0 {
            dt / r.blend_time
        } else {
            1.0
        };
        let goal = r.weight.clamp(0.0, 1.0);
        r.blend += (goal - r.blend).clamp(-rate, rate);
        let active = goal > 0.0 || r.blend > 0.0;
        let total_mass: f32 = state
            .bones
            .iter()
            .enumerate()
            .filter(|(i, h)| {
                r.bone_mask.as_slice().contains(&(*i as u32))
                    && (!state.stepped || !sim.world.bodies[**h].is_dynamic())
            })
            .map(|(_, &h)| sim.world.bodies[h].mass())
            .sum();
        state.dynamic = false;
        for (i, &h) in state.bones.iter().enumerate() {
            let dynamic = active && r.bone_mask.as_slice().contains(&(i as u32));
            let rb = &mut sim.world.bodies[h];
            if dynamic {
                state.dynamic = true;
                if !rb.is_dynamic() || !state.stepped {
                    let offset = unv(rb.local_center_of_mass());
                    let (p, q) = target[i];
                    let (p0, q0) = state.last[i];
                    let dq = (q * q0.conjugate()).normalize();
                    let dq = if dq.w < 0.0 { -dq } else { dq };
                    // Query initialization populates the live world at dt=0
                    // without consuming activation motion or impulses.
                    let inv_dt = if dt > 0.0 { 1.0 / dt } else { 0.0 };
                    let axis = Vec3::new(dq.x, dq.y, dq.z);
                    let len = axis.length();
                    let angular = if len > 0.0 {
                        axis * (2.0 * og_core::math::atan2(len, dq.w) * inv_dt / len)
                    } else {
                        Vec3::ZERO
                    };
                    let linear = (p + q * offset - p0 - q0 * offset) * inv_dt;
                    rb.set_body_type(rp::RigidBodyType::Dynamic, true);
                    rb.set_position(pose(p, q), true);
                    rb.set_linvel(v(linear), true);
                    super::ragdoll_angvel(rb, angular);
                    if dt > 0.0 {
                        rb.apply_impulse(
                            v(r.impulse * (rb.mass() / total_mass.max(f32::MIN_POSITIVE))),
                            true,
                        );
                    }
                }
            } else {
                if rb.is_dynamic() {
                    rb.set_body_type(rp::RigidBodyType::KinematicPositionBased, true);
                }
                rb.set_next_kinematic_position(pose(target[i].0, target[i].1));
            }
        }
        for &(i, h) in &state.motors {
            let i = i as usize;
            let parent = rig.bones[i].parent.expect("joint parent") as usize;
            if let Some(j) = sim.world.impulse_joints.get_mut(h, false) {
                let parent_at = pose(target[parent].0, target[parent].1);
                let child_at = pose(target[i].0, target[i].1);
                let relative =
                    (parent_at * j.data.local_frame1).inverse() * (child_at * j.data.local_frame2);
                ragdoll_motor(&mut j.data, relative, r.motor_stiffness, r.motor_damping);
            }
        }
        state.stepped |= dt > 0.0;
        state.last = target;
        r.impulse = Vec3::ZERO;
        out.ragdolls.push((e, r));
    }
    Ok(())
}

pub(crate) fn impulse(
    sim: &mut Sim,
    entity: u64,
    bone: u32,
    impulse: Vec3,
    point: Vec3,
) -> Result<()> {
    let rb = sim
        .ragdolls
        .get(&entity)
        .and_then(|s| s.bones.get(bone as usize))
        .and_then(|&h| sim.world.bodies.get_mut(h))
        .filter(|rb| rb.is_dynamic())
        .ok_or_else(|| Error::new(ErrorKind::Invalid, "ragdoll bone is absent or not dynamic"))?;
    rb.apply_impulse_at_point(v(impulse), v(point), true);
    Ok(())
}

pub(crate) fn readout(
    scene: &Scene,
    sim: &Sim,
    rigs: &IdMap<u64, Arc<RagdollRig>>,
    out: &mut StepOut,
) {
    for &(e, _, _, r) in &scene.ragdolls {
        if !r.drive_transform || !r.bone_mask.as_slice().contains(&0) {
            continue;
        }
        let Some(state) = sim.ragdolls.get(&e.to_bits()).filter(|s| s.dynamic) else {
            continue;
        };
        let Some(rig) = rigs.get(&e.to_bits()) else {
            continue;
        };
        let Some(&h) = state.bones.first() else {
            continue;
        };
        let Some(b) = rig.bones.first() else { continue };
        let (p, q) = unpose(sim.world.bodies[h].position(), state.last[0].0);
        let rotation = (q * b.rest.1.conjugate()).normalize();
        out.ragdoll_transforms
            .push((e, p - rotation * b.rest.0, rotation));
    }
}

pub(crate) fn pose_of(sim: &Sim, entity: u64) -> Option<Vec<(Vec3, Quat)>> {
    let s = sim.ragdolls.get(&entity)?;
    s.bones
        .iter()
        .zip(&s.last)
        .map(|(&h, last)| Some(unpose(sim.world.bodies.get(h)?.position(), last.0)))
        .collect()
}

impl RagdollState {
    pub(crate) fn form(
        &self,
        entity: u64,
        w: &rp::PhysicsWorld,
        out: &mut Vec<og_core::block_form::Record>,
    ) {
        use super::sim::{body_record, collider_records, joint_fields};
        use og_core::block_form::{Field, Record};
        out.push(
            Record::new(entity, "ragdoll")
                .field("dynamic", Field::of(&self.dynamic))
                .field("stepped", Field::of(&self.stepped))
                .field("last", Field::of(&self.last))
                .field("rig", Field::of(&self.rig)),
        );
        for (i, &h) in self.bones.iter().enumerate() {
            out.push(body_record(entity, "bone", Field::of(&i), h, w, Vec3::ZERO));
            if let Some(rb) = w.bodies.get(h) {
                collider_records(out, entity, "bone collider", rb.colliders(), w, &|k| {
                    Field::of(&(i, k))
                });
            }
        }
        for (kind, joints) in [("bone joint", &self.joints), ("bone motor", &self.motors)] {
            for &(i, h) in joints {
                if let Some(j) = w.impulse_joints.get(h) {
                    let mut r = Record::new(entity, kind).key(Field::of(&i));
                    r.fields = joint_fields(j, &w.bodies);
                    out.push(r);
                }
            }
        }
    }
}
