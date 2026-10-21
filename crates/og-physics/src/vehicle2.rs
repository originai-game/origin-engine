//! The 2D car (the 2D profile only): two wheel bodies (circles of
//! `wheelRadius`) under the entity's dynamic chassis at ±`wheelBase` / 2
//! along its local x (front at +x), each on a wheel joint: free to spin,
//! sliding along the chassis's local y within `suspensionTravel` on a
//! position motor (the spring), its spin driven by a velocity motor (the
//! engine) or held (the brake). The wheels are bodies of the rapier world,
//! not entities; they collide as the chassis's entity.

use super::sim::Sim;
use super::{rp, unrot, unv, v};
use crate::gather::Scene;
use crate::schema_ext::*;
use crate::{StepOut, WheelPose};
use og_core::Result;
use og_core::glam::Vec3;
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct VehicleState {
    chassis: rp::RigidBodyHandle,
    layout: [u32; 6],
    /// Front, rear.
    wheels: [rp::RigidBodyHandle; 2],
    joints: [rp::ImpulseJointHandle; 2],
}

fn layout(v: &Vehicle) -> [u32; 6] {
    [
        v.wheel_base.to_bits(),
        v.wheel_radius.to_bits(),
        v.mount_height.to_bits(),
        v.suspension_rest.to_bits(),
        v.wheel_mass.to_bits(),
        v.grip.to_bits(),
    ]
}

fn remove(w: &mut rp::PhysicsWorld, s: &VehicleState) {
    for h in s.wheels {
        w.remove_body(h);
    }
}

fn mounts(veh: &Vehicle) -> [Vec3; 2] {
    let x = veh.wheel_base * 0.5;
    let y = -veh.mount_height - veh.suspension_rest;
    [Vec3::new(x, y, 0.0), Vec3::new(-x, y, 0.0)]
}

fn build(w: &mut rp::PhysicsWorld, chassis: rp::RigidBodyHandle, veh: &Vehicle) -> VehicleState {
    let (pose, groups, entity) = {
        let rb = &w.bodies[chassis];
        let groups = rb
            .colliders()
            .first()
            .map(|&c| w.colliders[c].collision_groups())
            .unwrap_or_default();
        (*rb.position(), groups, rb.user_data)
    };
    let mut wheels = [chassis; 2];
    let mut joints = [rp::ImpulseJointHandle::invalid(); 2];
    for (i, m) in mounts(veh).into_iter().enumerate() {
        let at = pose.transform_point(v(m));
        let body = rp::RigidBodyBuilder::dynamic()
            .translation(at)
            .rotation(pose.rotation.angle())
            .can_sleep(false)
            .user_data(entity);
        let h = w.bodies.insert(body);
        let collider = rp::ColliderBuilder::ball(veh.wheel_radius.max(0.01))
            .friction(veh.grip.max(0.0))
            .mass(veh.wheel_mass.max(0.01))
            .collision_groups(groups)
            .user_data(entity);
        w.colliders.insert_with_parent(collider, h, &mut w.bodies);
        let frame1 = rp::Pose::from_parts(v(m), rp::Rotation::IDENTITY);
        let joint = rp::GenericJointBuilder::new(rp::JointAxesMask::LIN_X)
            .local_frame1(frame1)
            .local_frame2(rp::Pose::IDENTITY)
            .contacts_enabled(false)
            .limits(
                rp::JointAxis::LinY,
                [
                    -veh.suspension_travel.max(0.0),
                    veh.suspension_travel.max(0.0),
                ],
            )
            .build();
        joints[i] = w.impulse_joints.insert(chassis, h, joint, true);
        wheels[i] = h;
    }
    VehicleState {
        chassis,
        layout: layout(veh),
        wheels,
        joints,
    }
}

pub(crate) fn vehicles(
    scene: &Scene,
    _dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    let live: BTreeSet<u64> = scene.vehicles.iter().map(|v| v.0.to_bits()).collect();
    let gone: Vec<u64> = sim
        .vehicles
        .keys()
        .filter(|e| !live.contains(e))
        .copied()
        .collect();
    for e in gone {
        if let Some(s) = sim.vehicles.remove(&e) {
            remove(&mut sim.world, &s);
        }
    }
    for (e, veh) in &scene.vehicles {
        let key = e.to_bits();
        let chassis = handles
            .get(&key)
            .copied()
            .filter(|&h| sim.world.bodies[h].is_dynamic());
        let Some(chassis) = chassis else {
            if let Some(s) = sim.vehicles.remove(&key) {
                remove(&mut sim.world, &s);
            }
            continue;
        };
        let stale = sim.vehicles.get(&key).is_none_or(|s| {
            s.chassis != chassis
                || s.layout != layout(veh)
                || sim.world.impulse_joints.get(s.joints[0]).is_none()
        });
        if stale {
            if let Some(s) = sim.vehicles.remove(&key) {
                remove(&mut sim.world, &s);
            }
            let s = build(&mut sim.world, chassis, veh);
            sim.vehicles.insert(key, s);
        }
        let state = sim.vehicles.get(&key).expect("inserted").clone();
        let radius = veh.wheel_radius.max(0.01);
        // Bullet's units, as the 3D car: the spring's force is stiffness ×
        // compression × the chassis's mass.
        let chassis_mass = sim.world.bodies[chassis].mass().max(0.01);
        let brake = veh.brake.clamp(0.0, 1.0);
        for (i, &j) in state.joints.iter().enumerate() {
            let front = i == 0;
            let driven = match veh.drive {
                DRIVE_FRONT => front,
                DRIVE_ALL => true,
                _ => !front,
            };
            let Some(joint) = sim.world.impulse_joints.get_mut(j, true) else {
                continue;
            };
            let d = &mut joint.data;
            d.set_motor_position(
                rp::JointAxis::LinY,
                0.0,
                veh.suspension_stiffness.max(0.0) * chassis_mass,
                veh.suspension_damping.max(0.0) * chassis_mass,
            );
            d.set_limits(
                rp::JointAxis::LinY,
                [
                    -veh.suspension_travel.max(0.0),
                    veh.suspension_travel.max(0.0),
                ],
            );
            if brake > 0.0 {
                d.set_motor_velocity(rp::JointAxis::AngX, 0.0, 10.0);
                d.set_motor_max_force(rp::JointAxis::AngX, brake * veh.brake_force.max(0.0) * 10.0);
            } else if driven && veh.throttle != 0.0 {
                // Rolling towards +x turns the wheel clockwise (negative about z).
                let target = -veh.throttle.clamp(-1.0, 1.0) * veh.max_speed / radius;
                d.set_motor_velocity(rp::JointAxis::AngX, target, 50.0);
                d.set_motor_max_force(rp::JointAxis::AngX, veh.engine_force.max(0.0));
            } else {
                d.set_motor_velocity(rp::JointAxis::AngX, 0.0, 0.0);
                d.set_motor_max_force(rp::JointAxis::AngX, 0.0);
            }
        }
        let rb = &sim.world.bodies[chassis];
        let forward = unv(rb.position().rotation * v(Vec3::X));
        let speed = unv(rb.linvel()).dot(forward);
        let mut contacts = 0;
        for &wheel in &state.wheels {
            let Some(&c) = sim.world.bodies[wheel].colliders().first() else {
                continue;
            };
            if sim
                .world
                .narrow_phase
                .contact_pairs_with(c)
                .any(|p| p.has_any_active_contact())
            {
                contacts += 1;
            }
        }
        out.vehicles.push((
            Entity::from_bits(key),
            Vehicle {
                speed,
                contacts,
                slip: 0.0,
                ..*veh
            },
        ));
    }
    Ok(())
}

pub(crate) fn wheels(sim: &Sim, entity: u64) -> Option<Vec<WheelPose>> {
    let state = sim.vehicles.get(&entity)?;
    Some(
        state
            .wheels
            .iter()
            .filter_map(|&h| sim.world.bodies.get(h))
            .map(|rb| {
                let c = rb.colliders().first().copied();
                let contact = c.is_some_and(|c| {
                    sim.world
                        .narrow_phase
                        .contact_pairs_with(c)
                        .any(|p| p.has_any_active_contact())
                });
                WheelPose {
                    position: unv(rb.translation()),
                    quaternion: unrot(*rb.rotation()),
                    contact,
                    slip: 0.0,
                    side_speed: 0.0,
                }
            })
            .collect(),
    )
}
