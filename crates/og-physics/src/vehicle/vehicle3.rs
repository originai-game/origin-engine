//! The 3D car (the 3D profile only): rapier's raycast vehicle
//! (`DynamicRayCastVehicleController`, after Bullet's btRaycastVehicle) on
//! the entity's dynamic chassis, four wheels at the corners of `wheelBase`
//! by `track`, forward along the chassis's local +z (three.js
//! Object3D.lookAt aims +z), steering on the front axle. Updated before the
//! rigid-body step (it applies the suspension, drive and grip impulses).

use super::sim::Sim;
use super::{rp, unv, v};
use crate::gather::{Scene, WheelChild};
use crate::schema::*;
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

impl VehicleState {
    /// Moves the controller's world-space wheel points by `s.d`.
    pub(crate) fn shift(&mut self, s: &og_ecs::Shift) {
        self.controller.shift(&|p| super::v(s.moved(super::unv(p))));
    }
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

/// One wheel as the controller is built and driven: the four implicit
/// wheels from og.Vehicle's fields, or an og.VehicleWheel child's own.
#[derive(Clone, Copy)]
struct Spec {
    /// The og.VehicleWheel entity; 0 for an implicit wheel.
    entity: u64,
    mount: Vec3,
    radius: f32,
    rest: f32,
    travel: f32,
    stiffness: f32,
    damping: f32,
    grip: f32,
    /// The share of the steering it takes.
    steer: f32,
    drive: bool,
    brake: bool,
    handbrake: bool,
}

/// The wheels: front left, front right, rear left, rear right in chassis
/// space, or the og.VehicleWheel children in entity order.
fn specs(veh: &Vehicle, children: &[WheelChild]) -> Vec<Spec> {
    if !children.is_empty() {
        return children
            .iter()
            .map(|c| {
                let w = &c.wheel;
                Spec {
                    entity: c.entity.to_bits(),
                    mount: c.mount,
                    radius: w.radius,
                    rest: w.suspension_rest,
                    travel: w.suspension_travel,
                    stiffness: w.suspension_stiffness,
                    damping: w.suspension_damping,
                    grip: w.grip,
                    steer: w.steer.clamp(-1.0, 1.0),
                    drive: w.drive,
                    brake: w.brake,
                    handbrake: w.handbrake,
                }
            })
            .collect();
    }
    let (z, x, y) = (veh.wheel_base * 0.5, veh.track * 0.5, -veh.mount_height);
    [
        Vec3::new(x, y, z),
        Vec3::new(-x, y, z),
        Vec3::new(x, y, -z),
        Vec3::new(-x, y, -z),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, mount)| {
        let front = i < 2;
        Spec {
            entity: 0,
            mount,
            radius: veh.wheel_radius,
            rest: veh.suspension_rest,
            travel: veh.suspension_travel,
            stiffness: veh.suspension_stiffness,
            damping: veh.suspension_damping,
            grip: veh.grip,
            steer: if front { 1.0 } else { 0.0 },
            drive: match veh.drive {
                DRIVE_FRONT => front,
                DRIVE_ALL => true,
                _ => !front,
            },
            brake: true,
            handbrake: !front,
        }
    })
    .collect()
}

/// What building bakes in of the og.VehicleWheel children (their entity,
/// mount, radius and rest); empty for the implicit wheels.
fn baked(specs: &[Spec]) -> Vec<(u64, [u32; 5])> {
    specs
        .iter()
        .filter(|s| s.entity != 0)
        .map(|s| {
            (
                s.entity,
                [
                    s.mount.x.to_bits(),
                    s.mount.y.to_bits(),
                    s.mount.z.to_bits(),
                    s.radius.to_bits(),
                    s.rest.to_bits(),
                ],
            )
        })
        .collect()
}

fn tuning(s: &Spec) -> super::ctl::WheelTuning {
    super::ctl::WheelTuning {
        suspension_stiffness: s.stiffness.max(0.0),
        suspension_compression: s.damping.max(0.0) * 0.83,
        suspension_damping: s.damping.max(0.0),
        max_suspension_travel: s.travel.max(0.0),
        side_friction_stiffness: 1.0,
        friction_slip: s.grip.max(0.0) * 5.0,
        max_suspension_force: 1.0e5,
    }
}

fn build(
    chassis: rp::RigidBodyHandle,
    specs: &[Spec],
) -> super::ctl::DynamicRayCastVehicleController {
    let mut c = super::ctl::DynamicRayCastVehicleController::new(chassis);
    c.index_up_axis = 1;
    c.index_forward_axis = 2;
    for s in specs {
        c.add_wheel(
            v(s.mount),
            v(-Vec3::Y),
            v(-Vec3::X),
            s.rest.max(0.0),
            s.radius.max(0.01),
            &tuning(s),
        );
    }
    c
}

/// Every og.Vehicle before the step: built or rebuilt with its chassis,
/// its inputs and tunables applied (through its drivetrain for
/// VEHICLE_DRIVETRAIN), then rapier's update (raycasts and impulses).
pub(crate) fn vehicles(
    scene: &Scene,
    dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    out: &mut StepOut,
) -> Result<()> {
    let live: BTreeSet<u64> = og_core::sort::set(scene.vehicles.iter().map(|v| v.0.to_bits()));
    sim.vehicles.retain(|e, _| live.contains(e));
    sim.ext.vehicles.retain(|e, _| live.contains(e));
    for (e, veh) in &scene.vehicles {
        let key = e.to_bits();
        let Some(&chassis) = handles.get(&key) else {
            sim.vehicles.remove(&key);
            sim.ext.vehicles.remove(&key);
            continue;
        };
        if !sim.world.bodies[chassis].is_dynamic() {
            continue;
        }
        let children = scene.wheels.get(&key).map_or(&[][..], Vec::as_slice);
        let specs = specs(veh, children);
        let baked = baked(&specs);
        let drivetrain = veh.model == VEHICLE_DRIVETRAIN;
        let built_with = sim
            .ext
            .vehicles
            .get(&key)
            .map_or(&[][..], |x| x.wheels.as_slice());
        let rebuild = sim.vehicles.get(&key).is_none_or(|s| {
            s.chassis != chassis
                || built_with != baked.as_slice()
                || (baked.is_empty() && layout(&s.layout) != layout(veh))
        });
        if rebuild {
            sim.vehicles.insert(
                key,
                VehicleState {
                    chassis,
                    layout: *veh,
                    controller: build(chassis, &specs),
                },
            );
        }
        // What it keeps beyond the controller, only when it has any.
        if drivetrain || !baked.is_empty() {
            let x = sim.ext.vehicles.entry(key).or_default();
            x.wheels = baked;
            if !drivetrain {
                x.gearbox = Default::default();
            }
        } else {
            sim.ext.vehicles.remove(&key);
        }
        let state = sim.vehicles.get_mut(&key).expect("inserted");
        state.layout = *veh;
        let steer = -veh.steer.clamp(-1.0, 1.0) * veh.max_steer;
        let speed_now = {
            let rb = &sim.world.bodies[chassis];
            unv(rb.linvel()).dot(unv(rb.position().rotation * v(Vec3::Z)))
        };
        let throttle = veh.throttle.clamp(-1.0, 1.0);
        let driven = specs.iter().filter(|s| s.drive).count().max(1) as f32;
        // The push at each driven wheel: the drivetrain's torque split
        // across them as a force at each one's radius, or the arcade
        // engine's force; and the share of the brake throttle adds.
        let (torque, force, extra_brake) = if drivetrain {
            let radius = specs
                .iter()
                .filter(|s| s.drive)
                .map(|s| s.radius.max(0.01))
                .sum::<f32>()
                / driven;
            let x = sim.ext.vehicles.get_mut(&key).expect("kept above");
            let d = crate::drivetrain::step(veh, &mut x.gearbox, speed_now, radius, dt);
            (Some(d.torque), 0.0, d.brake)
        } else {
            // The engine fades out towards maxSpeed; throttle against the
            // motion brakes first (arcade reversing).
            let against = throttle * speed_now < 0.0 && speed_now.abs() > 1.0;
            let fade = (1.0 - speed_now.abs() / veh.max_speed.max(0.01)).clamp(0.0, 1.0);
            let force = if against {
                0.0
            } else {
                throttle * veh.engine_force * fade
            };
            (None, force, if against { throttle.abs() } else { 0.0 })
        };
        let force_at = |s: &Spec| match torque {
            Some(t) => t / driven / s.radius.max(0.01),
            None => force,
        };
        let gearbox = sim.ext.vehicles.get(&key).map(|x| x.gearbox);
        let state = sim.vehicles.get_mut(&key).expect("inserted");
        // rapier's brake is the most impulse a wheel's rolling friction
        // takes in one update: the force over the tick.
        let brake_force = veh.brake_force.max(0.0) * dt;
        let brake = (veh.brake.clamp(0.0, 1.0) + extra_brake).min(1.0) * brake_force;
        let handbrake = veh.handbrake.clamp(0.0, 1.0);
        let hand_grip = 1.0 + (veh.handbrake_grip.clamp(0.0, 1.0) - 1.0) * handbrake;
        // Drifting: the handbrake is on, or a handbrake wheel slid last step.
        let drifting = handbrake > 0.0
            || state
                .controller
                .wheels()
                .iter()
                .zip(&specs)
                .any(|(w, s)| s.handbrake && slip_of(w) > 0.2);
        let side = if drifting {
            veh.drift_grip.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let roll = if veh.roll_influence.is_finite() {
            veh.roll_influence.clamp(0.0, 1.0)
        } else {
            0.1
        };
        for (w, s) in state.controller.wheels_mut().iter_mut().zip(&specs) {
            let t = tuning(s);
            let held = s.handbrake && handbrake > 0.0;
            w.suspension_stiffness = t.suspension_stiffness;
            w.damping_compression = t.suspension_compression;
            w.damping_relaxation = t.suspension_damping;
            w.max_suspension_travel = t.max_suspension_travel;
            w.friction_slip = if s.handbrake {
                t.friction_slip * hand_grip
            } else {
                t.friction_slip
            };
            // A held handbrake loosens the tyre sideways too (handbrakeGrip),
            // drift grip on top: what lets the tail step out in a bend.
            w.side_friction_stiffness = if s.handbrake { side * hand_grip } else { 1.0 };
            w.roll_influence = roll;
            // A wheel that does not steer keeps +0, as before wheels had shares.
            w.steering = if s.steer == 0.0 { 0.0 } else { steer * s.steer };
            w.engine_force = if s.drive && !held { force_at(s) } else { 0.0 };
            w.brake = if s.brake { brake } else { 0.0 }
                + if s.handbrake {
                    handbrake * brake_force
                } else {
                    0.0
                };
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
        let (gear, rpm) = gearbox.map_or((veh.gear, veh.rpm), |g| (g.gear, g.rpm));
        out.vehicles.push((
            Entity::from_bits(key),
            Vehicle {
                speed,
                contacts,
                slip,
                gear: if drivetrain { gear } else { veh.gear },
                rpm: if drivetrain { rpm } else { veh.rpm },
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
    let ext = sim.ext.vehicles.get(&entity);
    let rb = sim.world.bodies.get(state.chassis)?;
    let p = *rb.position();
    let q = p.rotation;
    let chassis_q = Quat::from_xyzw(q.x, q.y, q.z, q.w);
    Some(
        state
            .controller
            .wheels()
            .iter()
            .enumerate()
            .map(|(i, w)| {
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
                    entity: ext.and_then(|x| x.wheels.get(i)).map_or(0, |w| w.0),
                }
            })
            .collect(),
    )
}
