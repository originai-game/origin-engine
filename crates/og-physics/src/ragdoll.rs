//! Ragdolls, in both profiles: a capsule body per bone of the rig the host
//! registered for the entity (`PhysicsSystem::set_ragdoll_rig`), joints
//! with limits between a bone and its parent (ball joints with a cone in
//! 3D; in 2D hinges about z, the bones in the xy plane at their animated
//! z), following the animation's bone poses (`set_ragdoll_targets`, each
//! tick) kinematically while `weight` is 0 and simulated while it is above
//! 0. `blend` moves towards `weight` over `blendTime`; the host draws the
//! animation's pose blended towards the bodies' by it.

use super::sim::Sim;
use super::{pose, ragdoll_joint, rot, rp, unpose, v};
use crate::gather::Scene;
use crate::schema_ext::*;
use crate::{RagdollRig, StepOut};
use og_core::glam::{Quat, Vec3};
use og_core::{Error, ErrorKind, IdMap, Result};
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The ragdoll layer: bones are on bit 31 and do not collide with each other.
const BONE_GROUP: u32 = 1 << 31;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RagdollState {
    /// One body per rig bone, in rig order.
    bones: Vec<rp::RigidBodyHandle>,
    joints: Vec<rp::ImpulseJointHandle>,
    dynamic: bool,
    /// The bones' targets the last tick (their velocity when going limp).
    last: Vec<(Vec3, Quat)>,
    /// The rig's identity (bone count and lengths), so a new rig rebuilds.
    rig: Vec<u32>,
}

fn rig_key(rig: &RagdollRig) -> Vec<u32> {
    let mut key = vec![rig.bones.len() as u32];
    for b in &rig.bones {
        key.push(b.length.to_bits());
        key.push(b.parent.unwrap_or(u32::MAX));
    }
    key
}

fn remove(w: &mut rp::PhysicsWorld, s: &RagdollState) {
    for &h in &s.bones {
        w.remove_body(h);
    }
}

/// The capsule of a bone of `length` along its local `axis`: its pose in
/// the bone's frame and its half height and radius.
fn capsule(axis: Vec3, length: f32, thickness: f32) -> (rp::Pose, f32, f32) {
    let len = length.max(0.02);
    let radius = (len * thickness).clamp(0.01, len * 0.5);
    let half = (len * 0.5 - radius).max(0.001);
    let dir = axis.normalize_or(Vec3::Y);
    let p = rp::Pose::from_parts(v(dir * (len * 0.5)), rot(crate::arc(Vec3::Y, dir)));
    (p, half, radius)
}

fn build(
    w: &mut rp::PhysicsWorld,
    entity: u64,
    rig: &RagdollRig,
    r: &Ragdoll,
    targets: &[(Vec3, Quat)],
) -> Result<RagdollState> {
    if targets.len() != rig.bones.len() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "ragdoll of entity {}: {} targets for {} bones",
                Entity::from_bits(entity),
                targets.len(),
                rig.bones.len()
            ),
        ));
    }
    let total: f32 = rig.bones.iter().map(|b| b.length.max(0.02)).sum();
    let mut bones = Vec::with_capacity(rig.bones.len());
    for (b, &(p, q)) in rig.bones.iter().zip(targets) {
        let body = rp::RigidBodyBuilder::kinematic_position_based()
            .pose(pose(p, q))
            .linear_damping(r.damping.max(0.0))
            .angular_damping(r.damping.max(0.0) * 2.0)
            .can_sleep(false)
            .user_data(entity as u128);
        let h = w.bodies.insert(body);
        let (local, half, radius) = capsule(b.axis, b.length, r.thickness);
        let mass = r.mass.max(0.1) * b.length.max(0.02) / total.max(1e-6);
        let collider = rp::ColliderBuilder::capsule_y(half, radius)
            .position(local)
            .mass(mass)
            .friction(0.7)
            .collision_groups(rp::InteractionGroups::new(
                rp::Group::from_bits_retain(BONE_GROUP),
                rp::Group::from_bits_retain(!BONE_GROUP),
                rp::InteractionTestMode::And,
            ))
            .user_data(entity as u128);
        w.colliders.insert_with_parent(collider, h, &mut w.bodies);
        bones.push(h);
    }
    let limit = r.joint_limit.clamp(0.0, 3.1);
    let mut joints = Vec::new();
    for (i, b) in rig.bones.iter().enumerate() {
        let Some(parent) = b.parent else { continue };
        let parent = parent as usize;
        if parent >= bones.len() || parent == i {
            continue;
        }
        let (pp, pq) = targets[parent];
        let (cp, cq) = targets[i];
        let p_pose = pose(pp, pq);
        let c_pose = pose(cp, cq);
        // The joint sits at the child's origin, framed as the bones are now.
        let anchor1 = p_pose.inverse_transform_point(v(cp));
        let rel = (p_pose.inverse() * c_pose).rotation;
        let joint = ragdoll_joint(rp::Pose::from_parts(anchor1, rel), limit);
        joints.push(
            w.impulse_joints
                .insert(bones[parent], bones[i], joint, true),
        );
    }
    Ok(RagdollState {
        bones,
        joints,
        dynamic: false,
        last: targets.to_vec(),
        rig: rig_key(rig),
    })
}

/// The targets of a ragdoll with none given: the rig's rest pose at the entity.
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

/// What a ragdoll's bones are told before the step.
enum Bones {
    /// Follow the animation kinematically: each bone's next pose.
    Follow(Vec<rp::Pose>),
    /// Simulated: their damping.
    Damp(f32, f32),
}

/// One ragdoll's sync before the step, planned from its component, rig,
/// targets and state (read only, by ranges of ragdolls on the lanes), then
/// done to the rapier world in ragdoll order.
struct Plan {
    key: u64,
    /// No state yet, or one for another rig: (re)built at `target`.
    build: bool,
    target: Vec<(Vec3, Quat)>,
    /// The component with its blend moved.
    blended: Ragdoll,
    /// Going limp: each bone's velocity (from the animation) and its share
    /// of the hit.
    limp: Option<(Vec<rp::Vector>, rp::Vector)>,
    /// Recovered: the bones kinematic again.
    recover: bool,
    bones: Bones,
}

#[allow(clippy::too_many_arguments)]
fn plan(
    (e, position, rotation, r): &(Entity, Vec3, Quat, Ragdoll),
    dt: f32,
    states: &IdMap<u64, RagdollState>,
    rigs: &IdMap<u64, Arc<RagdollRig>>,
    targets: &IdMap<u64, Vec<(Vec3, Quat)>>,
) -> Option<Plan> {
    let key = e.to_bits();
    let rig = rigs.get(&key)?;
    let target = targets
        .get(&key)
        .cloned()
        .unwrap_or_else(|| rest(rig, *position, *rotation));
    let state = states.get(&key).filter(|s| s.rig == rig_key(rig));
    let build = state.is_none();
    // A ragdoll (re)built this tick starts kinematic, its last targets these.
    let (dynamic, last) = state.map_or((false, &target), |s| (s.dynamic, &s.last));
    let mut c = *r;
    let rate = if r.blend_time > 0.0 {
        dt / r.blend_time
    } else {
        1.0
    };
    let goal = r.weight.clamp(0.0, 1.0);
    c.blend = if c.blend < goal {
        (c.blend + rate).min(goal)
    } else {
        (c.blend - rate).max(goal)
    };
    let mut limp = None;
    let mut recover = false;
    let mut now_dynamic = dynamic;
    if goal > 0.0 && !dynamic {
        // Going limp: the bodies take the animation's velocity and the hit.
        now_dynamic = true;
        let n = rig.bones.len().max(1) as f32;
        let velocities = (0..rig.bones.len())
            .map(|i| v((target[i].0 - last[i].0) / dt.max(1e-6)))
            .collect();
        limp = Some((velocities, v(r.impulse / n)));
    } else if goal <= 0.0 && dynamic && c.blend <= 0.0 {
        // Recovered: the bodies follow the animation again.
        now_dynamic = false;
        recover = true;
    }
    let bones = if !now_dynamic {
        Bones::Follow(target.iter().map(|&(p, q)| pose(p, q)).collect())
    } else {
        Bones::Damp(r.damping.max(0.0), r.damping.max(0.0) * 2.0)
    };
    Some(Plan {
        key,
        build,
        target,
        blended: c,
        limp,
        recover,
        bones,
    })
}

/// Every og.Ragdoll before the step: built at the animation's pose, its
/// bones following it (weight 0) or simulated (weight above 0), its blend
/// moved towards the weight. Each ragdoll is planned alone (by
/// [`crate::Grains::ragdolls`] a task on the lanes from two of them), then
/// its bodies are built and told their targets in ragdoll order, bone by
/// bone, as one thread would.
#[allow(clippy::too_many_arguments)]
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
    let plans: Vec<Option<Plan>> = {
        let states = &sim.ragdolls;
        let mut parts =
            crate::lanes::map_ranges(scene.ragdolls.len(), crate::lanes::now().ragdolls, |r| {
                scene.ragdolls[r]
                    .iter()
                    .map(|ragdoll| plan(ragdoll, dt, states, rigs, targets))
                    .collect::<Vec<_>>()
            });
        if parts.len() == 1 {
            parts.pop().unwrap_or_default()
        } else {
            parts.into_iter().flatten().collect()
        }
    };
    for ((e, _, _, r), plan) in scene.ragdolls.iter().zip(plans) {
        let Some(plan) = plan else { continue };
        let key = plan.key;
        if plan.build {
            if let Some(s) = sim.ragdolls.remove(&key) {
                remove(&mut sim.world, &s);
            }
            let rig = &rigs[&key];
            let s = build(&mut sim.world, key, rig, r, &plan.target)?;
            sim.ragdolls.insert(key, s);
        }
        let state = sim.ragdolls.get_mut(&key).expect("inserted");
        if let Some((velocities, impulse)) = &plan.limp {
            state.dynamic = true;
            for (&h, velocity) in state.bones.iter().zip(velocities) {
                let rb = &mut sim.world.bodies[h];
                rb.set_body_type(rp::RigidBodyType::Dynamic, true);
                rb.set_linvel(*velocity, true);
                rb.apply_impulse(*impulse, true);
            }
        } else if plan.recover {
            state.dynamic = false;
            for &h in &state.bones {
                sim.world.bodies[h].set_body_type(rp::RigidBodyType::KinematicPositionBased, true);
            }
        }
        match &plan.bones {
            Bones::Follow(poses) => {
                for (&h, at) in state.bones.iter().zip(poses) {
                    sim.world.bodies[h].set_next_kinematic_position(*at);
                }
            }
            Bones::Damp(linear, angular) => {
                for &h in &state.bones {
                    let rb = &mut sim.world.bodies[h];
                    rb.set_linear_damping(*linear);
                    rb.set_angular_damping(*angular);
                }
            }
        }
        state.last = plan.target;
        out.ragdolls.push((*e, plan.blended));
    }
    Ok(())
}

/// The bones' poses after the step, in rig order (in 2D at the z their
/// targets last had).
pub(crate) fn pose_of(sim: &Sim, entity: u64) -> Option<Vec<(Vec3, Quat)>> {
    let s = sim.ragdolls.get(&entity)?;
    Some(
        s.bones
            .iter()
            .zip(&s.last)
            .filter_map(|(&h, last)| Some((sim.world.bodies.get(h)?, last.0)))
            .map(|(rb, at)| unpose(rb.position(), at))
            .collect(),
    )
}
