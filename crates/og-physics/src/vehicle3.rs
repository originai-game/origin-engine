//! The 3D car (the 3D profile only): rapier's raycast vehicle
//! (`DynamicRayCastVehicleController`, after Bullet's btRaycastVehicle) on
//! the entity's dynamic chassis, four wheels at the corners of `wheelBase`
//! by `track`, forward along the chassis's local +z (three.js
//! Object3D.lookAt aims +z), steering on the front axle. Updated before the
//! rigid-body step (it applies the suspension, drive and grip impulses).

use super::sim::Sim;
use super::{rp, unv, v};
use crate::gather::Scene;
use crate::schema_ext::*;
use crate::{StepOut, WheelPose};
use og_core::Result;
use og_core::glam::{Quat, Vec3};
use og_core::math::quat_from_axis_angle;
use og_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct VehicleState {
    chassis: rp::RigidBodyHandle,
    layout: Vehicle,
    controller: super::ctl::DynamicRayCastVehicleController,
}

/// What rebuilds a vehicle: its geometry (inputs, tunables and outputs do not).
fn layout(v: &Vehicle) -> [u32; 5] {
    [
        v.wheel_base.to_bits(),
        v.track.to_bits(),
        v.wheel_radius.to_bits(),
        v.mount_height.to_bits(),
        v.suspension_rest.to_bits(),
    ]
}

fn tuning(veh: &Vehicle) -> super::ctl::WheelTuning {
    super::ctl::WheelTuning {
        suspension_stiffness: veh.suspension_stiffness.max(0.0),
        suspension_compression: veh.suspension_damping.max(0.0) * 0.83,
        suspension_damping: veh.suspension_damping.max(0.0),
        max_suspension_travel: veh.suspension_travel.max(0.0),
        side_friction_stiffness: 1.0,
        friction_slip: veh.grip.max(0.0) * 5.0,
        max_suspension_force: 1.0e5,
    }
}

/// The wheels in chassis space: front left, front right, rear left, rear right.
fn mounts(veh: &Vehicle) -> [Vec3; 4] {
    let (z, x, y) = (veh.wheel_base * 0.5, veh.track * 0.5, -veh.mount_height);
    [
        Vec3::new(x, y, z),
        Vec3::new(-x, y, z),
        Vec3::new(x, y, -z),
        Vec3::new(-x, y, -z),
    ]
}

fn build(
    chassis: rp::RigidBodyHandle,
    veh: &Vehicle,
) -> super::ctl::DynamicRayCastVehicleController {
    let mut c = super::ctl::DynamicRayCastVehicleController::new(chassis);
    c.index_up_axis = 1;
    c.index_forward_axis = 2;
    let t = tuning(veh);
    for m in mounts(veh) {
        c.add_wheel(
            v(m),
            v(-Vec3::Y),
            v(-Vec3::X),
            veh.suspension_rest.max(0.0),
            veh.wheel_radius.max(0.01),
            &t,
        );
    }
    c
}

/// Every og.Vehicle before the step: built or rebuilt with its chassis,
/// its inputs and tunables applied, then rapier's update (raycasts and
/// impulses).
pub(crate) fn vehicles(
    scene: &Scene,
    dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    let live: BTreeSet<u64> = scene.vehicles.iter().map(|v| v.0.to_bits()).collect();
    sim.vehicles.retain(|e, _| live.contains(e));
    for (e, veh) in &scene.vehicles {
        let key = e.to_bits();
        let Some(&chassis) = handles.get(&key) else {
            sim.vehicles.remove(&key);
            continue;
        };
        if !sim.world.bodies[chassis].is_dynamic() {
            continue;
        }
        let rebuild = sim
            .vehicles
            .get(&key)
            .is_none_or(|s| s.chassis != chassis || layout(&s.layout) != layout(veh));
        if rebuild {
            sim.vehicles.insert(
                key,
                VehicleState {
                    chassis,
                    layout: *veh,
                    controller: build(chassis, veh),
                },
            );
        }
        let state = sim.vehicles.get_mut(&key).expect("inserted");
        state.layout = *veh;
        let t = tuning(veh);
        let steer = -veh.steer.clamp(-1.0, 1.0) * veh.max_steer;
        // The engine fades out towards maxSpeed; throttle against the
        // motion brakes first (arcade reversing).
        let speed_now = {
            let rb = &sim.world.bodies[chassis];
            unv(rb.linvel()).dot(unv(rb.position().rotation * v(Vec3::Z)))
        };
        let throttle = veh.throttle.clamp(-1.0, 1.0);
        let against = throttle * speed_now < 0.0 && speed_now.abs() > 1.0;
        let fade = (1.0 - speed_now.abs() / veh.max_speed.max(0.01)).clamp(0.0, 1.0);
        let force = if against {
            0.0
        } else {
            throttle * veh.engine_force * fade
        };
        // rapier's brake is the most impulse a wheel's rolling friction
        // takes in one update: the force over the tick.
        let brake = (veh.brake.clamp(0.0, 1.0) + if against { throttle.abs() } else { 0.0 })
            .min(1.0)
            * veh.brake_force.max(0.0)
            * dt;
        let roll = if veh.roll_influence.is_finite() {
            veh.roll_influence.clamp(0.0, 1.0)
        } else {
            0.1
        };
        for (i, w) in state.controller.wheels_mut().iter_mut().enumerate() {
            let front = i < 2;
            w.suspension_stiffness = t.suspension_stiffness;
            w.damping_compression = t.suspension_compression;
            w.damping_relaxation = t.suspension_damping;
            w.max_suspension_travel = t.max_suspension_travel;
            w.friction_slip = t.friction_slip;
            w.roll_influence = roll;
            w.steering = if front { steer } else { 0.0 };
            let driven = match veh.drive {
                DRIVE_FRONT => front,
                DRIVE_ALL => true,
                _ => !front,
            };
            w.engine_force = if driven { force } else { 0.0 };
            w.brake = brake;
        }
        let w = &mut sim.world;
        // The rays meet what the chassis's collider would: its layers
        // against the other's mask and the other's layers against its own.
        let groups = w.bodies[chassis]
            .colliders()
            .first()
            .map(|&c| w.colliders[c].collision_groups())
            .unwrap_or_default();
        let filter = rp::QueryFilter::new()
            .exclude_rigid_body(chassis)
            .exclude_sensors()
            .groups(groups);
        let queries = w.broad_phase.as_query_pipeline_mut(
            w.narrow_phase.query_dispatcher(),
            &mut w.bodies,
            &mut w.colliders,
            filter,
        );
        state.controller.update_vehicle(dt, queries);
        let rb = &sim.world.bodies[chassis];
        let forward = unv(rb.position().rotation * v(Vec3::Z));
        let speed = unv(rb.linvel()).dot(forward);
        let wheels = state.controller.wheels();
        let contacts = wheels
            .iter()
            .filter(|w| w.raycast_info().is_in_contact)
            .count() as u32;
        let slip = wheels.iter().map(slip_of).fold(0.0, f32::max);
        out.vehicles.push((
            Entity::from_bits(key),
            Vehicle {
                speed,
                contacts,
                slip,
                ..*veh
            },
        ));
    }
    Ok(())
}

/// How much a wheel slides: 1 − its skid info on the ground (the share of
/// its impulse the tyre held), 0 in the air.
fn slip_of(w: &super::ctl::Wheel) -> f32 {
    if w.raycast_info().is_in_contact {
        (1.0 - w.skid_info).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The wheels' poses for drawing, from the chassis as it is now.
pub(crate) fn wheels(sim: &Sim, entity: u64) -> Option<Vec<WheelPose>> {
    let state = sim.vehicles.get(&entity)?;
    let rb = sim.world.bodies.get(state.chassis)?;
    let p = *rb.position();
    let q = p.rotation;
    let chassis_q = Quat::from_xyzw(q.x, q.y, q.z, q.w);
    Some(
        state
            .controller
            .wheels()
            .iter()
            .map(|w| {
                let hard = unv(p.transform_point(w.chassis_connection_point_cs));
                let down = chassis_q * Vec3::NEG_Y;
                let centre = hard + down * w.raycast_info().suspension_length;
                let steer = quat_from_axis_angle(Vec3::Y, w.steering);
                let spin = quat_from_axis_angle(Vec3::X, w.rotation);
                let info = w.raycast_info();
                let side_speed = if info.is_in_contact {
                    let at = info.contact_point_ws;
                    let ground = info
                        .ground_object
                        .and_then(|c| sim.world.colliders.get(c))
                        .and_then(|c| c.parent())
                        .and_then(|b| sim.world.bodies.get(b))
                        .map_or(Vec3::ZERO, |b| unv(b.velocity_at_point(at)));
                    let axle = chassis_q * steer * Vec3::X;
                    (unv(rb.velocity_at_point(at)) - ground).dot(axle).abs()
                } else {
                    0.0
                };
                WheelPose {
                    position: centre,
                    quaternion: (chassis_q * steer * spin).normalize(),
                    contact: info.is_in_contact,
                    slip: slip_of(w),
                    side_speed,
                }
            })
            .collect(),
    )
}
