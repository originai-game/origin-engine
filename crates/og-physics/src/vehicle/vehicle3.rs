//! Independent wheel dynamics. Contacts are cast against current collider poses
//! (including frame zero), then solved sequentially against both bodies.
use super::sim::Sim;
use super::{rp, unv, v};
use crate::gather::Scene;
use crate::schema::*;
use crate::vehicle_wheels::{Spec, specs};
use crate::{StepOut, WheelPose};
use og_core::Result;
use og_core::glam::{Quat, Vec2, Vec3};
use og_core::math::{atan, atan2, cos, exp, quat_from_axis_angle, sin, sin_cos};
use rapier3d::parry::query::{ShapeCastOptions, cast_shapes};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
struct Wheel {
    omega: f32,
    inertia: f32,
    spin_angle: f32,
    compression: f32,
    prev_compression: f32,
    length: f32,
    steering: f32,
    ground: Option<rp::ColliderHandle>,
    point: Vec3,
    normal: Vec3,
    load: f32,
    slip: f32,
    side_speed: f32,
    tyre_force: Vec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct VehicleState {
    chassis: rp::RigidBodyHandle,
    specs: Vec<Spec>,
    wheels: Vec<Wheel>,
}

impl VehicleState {
    pub(crate) fn shift(&mut self, s: &og_ecs::Shift) {
        for w in &mut self.wheels {
            w.point = s.moved(w.point);
        }
    }
}

fn collider_pose(w: &rp::PhysicsWorld, c: &rp::Collider) -> rp::Pose {
    c.parent()
        .zip(c.position_wrt_parent())
        .map_or(*c.position(), |(h, p)| *w.bodies[h].position() * *p)
}

fn hook<'a>(scene: &'a Scene, c: &rp::Collider) -> Option<&'a ContactHook> {
    scene
        .hooks
        .get(&(c.user_data as u64))
        .or_else(|| scene.hooks.get(&((c.user_data >> 64) as u64)))
}

fn ground_body(w: &rp::PhysicsWorld, wheel: &Wheel) -> Option<rp::RigidBodyHandle> {
    wheel
        .ground
        .and_then(|h| w.colliders.get(h))
        .and_then(|c| c.parent())
}

fn relative(
    w: &rp::PhysicsWorld,
    chassis: rp::RigidBodyHandle,
    wheel: &Wheel,
    surface: Vec3,
) -> Vec3 {
    let ground = ground_body(w, wheel).map_or(Vec3::ZERO, |h| {
        let body = &w.bodies[h];
        if body.body_type() == rp::RigidBodyType::KinematicPositionBased {
            let velocity = rp::RigidBodyPosition {
                position: *body.position(),
                next_position: *body.next_position(),
            }
            .interpolate_velocity(
                w.integration_parameters.inv_dt(),
                body.local_center_of_mass(),
            );
            unv(velocity.velocity_at_point(v(wheel.point), body.center_of_mass()))
        } else {
            unv(body.velocity_at_point(v(wheel.point)))
        }
    });
    unv(w.bodies[chassis].velocity_at_point(v(wheel.point))) - ground - surface
}

/// Anticipate the continuous integration so static tyre friction can hold a
/// slope rather than correcting gravity one substep late forever.
fn acceleration(w: &rp::PhysicsWorld, h: rp::RigidBodyHandle, at: Vec3) -> Vec3 {
    let rb = &w.bodies[h];
    if !rb.is_dynamic() || rb.is_sleeping() {
        return Vec3::ZERO;
    }
    let m = rb.mass_properties();
    let linear = (w.gravity * rb.gravity_scale() + rb.user_force() / rb.mass().max(1.0e-6))
        * (m.effective_inv_mass * rb.mass());
    let angular = m.effective_world_inv_inertia * rb.user_torque();
    unv(linear + angular.cross(v(at) - rb.center_of_mass()))
}

/// J M^-1 J^T, with a single inverse inertia, including locked axes.
fn effective(rb: &rp::RigidBody, point: Vec3, axis: Vec3) -> f32 {
    if !rb.is_dynamic() {
        return 0.0;
    }
    let m = rb.mass_properties();
    let j = v(point - unv(rb.center_of_mass())).cross(v(axis));
    v(axis).dot(m.effective_inv_mass * v(axis)) + j.dot(m.effective_world_inv_inertia * j)
}

fn pair_mass(
    w: &rp::PhysicsWorld,
    chassis: rp::RigidBodyHandle,
    wheel: &Wheel,
    point: Vec3,
    axis: Vec3,
) -> f32 {
    effective(&w.bodies[chassis], point, axis)
        + ground_body(w, wheel).map_or(0.0, |h| effective(&w.bodies[h], wheel.point, axis))
}

fn impulse(
    w: &mut rp::PhysicsWorld,
    chassis: rp::RigidBodyHandle,
    wheel: &Wheel,
    at: Vec3,
    j: Vec3,
) {
    w.bodies[chassis].apply_impulse_at_point(v(j), v(at), false);
    if let Some(h) = ground_body(w, wheel)
        && w.bodies[h].is_dynamic()
    {
        w.bodies[h].apply_impulse_at_point(v(-j), v(wheel.point), j.length_squared() > 0.0);
    }
}

/// Cast the entire droop, testing hooks before selecting the closest contact.
fn cast(
    scene: &Scene,
    world: &rp::PhysicsWorld,
    chassis: rp::RigidBodyHandle,
    s: &Spec,
    wheel: &mut Wheel,
    cast: u32,
) {
    let rb = &world.bodies[chassis];
    let p = *rb.position();
    let hard = p.transform_point(v(s.mount));
    let down = p.rotation * v(Vec3::NEG_Y);
    let max = s.rest + s.travel;
    wheel.ground = None;
    wheel.length = max;
    wheel.load = 0.0;
    wheel.tyre_force = Vec3::ZERO;
    wheel.point = Vec3::ZERO;
    wheel.normal = Vec3::ZERO;
    wheel.side_speed = 0.0;
    let own = rb.colliders().first().map(|h| &world.colliders[*h]);
    let own_hook = own.and_then(|c| hook(scene, c));
    let shape = if cast == 2 {
        rp::SharedShape::cylinder(s.radius * 0.25, s.radius)
    } else {
        rp::SharedShape::ball(s.radius)
    };
    let orientation = p.rotation
        * super::rot(quat_from_axis_angle(Vec3::Y, wheel.steering))
        * super::rot(quat_from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_2));
    let start = rp::Pose::from_parts(hard, orientation);
    let mut closest = f32::MAX;
    for (handle, c) in world.colliders.iter() {
        if !c.is_enabled()
            || c.is_sensor()
            || c.parent() == Some(chassis)
            || own.is_some_and(|a| !a.collision_groups().test(c.collision_groups()))
        {
            continue;
        }
        let other_hook = hook(scene, c);
        let ignored = |h: Option<&ContactHook>, other: &rp::Collider| {
            h.is_some_and(|h| {
                h.ignore.as_slice().iter().any(|&e| {
                    e != 0 && (e == other.user_data as u64 || e == (other.user_data >> 64) as u64)
                })
            })
        };
        if ignored(own_hook, c) || own.is_some_and(|a| ignored(other_hook, a)) {
            continue;
        }
        let pos = collider_pose(world, c);
        let hit = if cast == 0 {
            c.shape()
                .cast_ray_and_get_normal(&pos, &rp::Ray::new(hard, down), max + s.radius, true)
                .map(|h| {
                    (
                        h.time_of_impact - s.radius,
                        hard + down * h.time_of_impact,
                        h.normal,
                    )
                })
        } else {
            cast_shapes(
                &pos,
                rp::Vector::ZERO,
                c.shape(),
                &start,
                down,
                shape.as_ref(),
                ShapeCastOptions {
                    max_time_of_impact: max,
                    compute_impact_geometry_on_penetration: true,
                    ..Default::default()
                },
            )
            .ok()
            .flatten()
            .map(|h| {
                (
                    h.time_of_impact,
                    pos.transform_point(h.witness1),
                    pos.rotation * h.normal1,
                )
            })
        };
        let Some((length, point, normal)) = hit else {
            continue;
        };
        let normal = unv(normal).normalize_or_zero();
        // A grazing wall is not a load-bearing suspension contact.
        if normal.dot(-unv(down)) <= 0.1 || length >= closest {
            continue;
        }
        let one_way = |h: Option<&ContactHook>, n: Vec3| {
            h.is_none_or(|h| {
                h.one_way == Vec3::ZERO
                    || n.dot(h.one_way.normalize_or_zero()) >= std::f32::consts::FRAC_1_SQRT_2
            })
        };
        if !one_way(other_hook, normal) || !one_way(own_hook, -normal) {
            continue;
        }
        closest = length;
        wheel.length = length.clamp((s.rest - s.travel).max(0.0), max);
        wheel.ground = Some(handle);
        wheel.point = unv(point);
        wheel.normal = normal;
    }
    wheel.prev_compression = wheel.compression;
    wheel.compression = s.rest - wheel.length;
    if wheel.ground.is_none() {
        wheel.slip = 0.0;
    }
}

pub(crate) fn vehicles(
    scene: &Scene,
    dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    _: &mut StepOut,
) -> Result<()> {
    if dt == 0.0 {
        return Ok(());
    }
    sim.vehicles.retain(|e, _| {
        scene.vehicles.iter().any(|v| v.0.to_bits() == *e)
            && handles
                .get(e)
                .is_some_and(|h| sim.world.bodies[*h].is_dynamic())
    });
    sim.ext.vehicles.retain(|e, _| sim.vehicles.contains_key(e));
    for (entity, veh) in &scene.vehicles {
        let key = entity.to_bits();
        let Some(&chassis) = handles.get(&key) else {
            continue;
        };
        if !sim.world.bodies[chassis].is_dynamic() {
            continue;
        }
        let state = sim.vehicles.entry(key).or_insert_with(|| VehicleState {
            chassis,
            specs: Vec::new(),
            wheels: Vec::new(),
        });
        if state.chassis != chassis {
            state.chassis = chassis;
            state.wheels.clear();
            state.specs.clear();
        }
        let specs = specs(
            veh,
            scene.wheels.get(&key).map_or(&[], Vec::as_slice),
            &state.specs,
            false,
        );
        // Reconcile by identity, preserving wheel spin through all geometry edits.
        let wheels = specs
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let old = if s.entity == 0 {
                    Some(i)
                } else {
                    state.specs.iter().position(|a| a.entity == s.entity)
                };
                old.and_then(|i| state.wheels.get(i))
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        state.specs = specs;
        state.wheels = wheels;
        let rb = &mut sim.world.bodies[chassis];
        if veh.throttle != 0.0 || veh.brake != 0.0 || veh.handbrake != 0.0 || veh.steer != 0.0 {
            rb.wake_up(true);
        }
        if rb.is_sleeping() {
            continue;
        }
        let up = unv(rb.rotation() * v(Vec3::Y));
        let speed = unv(rb.linvel()).dot(unv(rb.rotation() * v(Vec3::Z)));
        let velocity = unv(rb.linvel());
        let aero = -velocity * velocity.length() * veh.aero_drag.max(0.0)
            - up * speed * speed * veh.aero_downforce.max(0.0);
        rb.apply_impulse(v(aero * dt), false);
        let mass = rb.mass();
        let base_steer = -veh.steer.clamp(-1.0, 1.0) * veh.max_steer
            / (1.0 + veh.steering_fade.max(0.0) * speed.abs());
        for (s, wheel) in state.specs.iter().zip(&mut state.wheels) {
            let angle = base_steer * s.steer;
            let ackermann = if angle.abs() > 1.0e-5 && veh.wheel_base > 0.0 {
                let (sine, cosine) = sin_cos(angle);
                atan(veh.wheel_base / (veh.wheel_base * cosine / sine - s.mount.x))
            } else {
                angle
            };
            wheel.steering = angle + (ackermann - angle) * veh.ackermann.clamp(0.0, 1.0);
            wheel.inertia = 0.5 * veh.wheel_mass.max(0.01) * s.radius * s.radius;
            cast(scene, &sim.world, chassis, s, wheel, veh.wheel_cast);
        }
        // Pair opposite mounts on each axle once, independent of child entity order.
        let mut anti_roll = vec![0.0; state.wheels.len()];
        for (i, a) in state.specs.iter().enumerate() {
            if a.mount.x <= 0.0 {
                continue;
            }
            if let Some(j) = state
                .specs
                .iter()
                .position(|b| b.mount.x < 0.0 && (a.mount.z - b.mount.z).abs() < 1.0e-4)
            {
                let force = veh.anti_roll_stiffness.max(0.0)
                    * (state.wheels[i].compression - state.wheels[j].compression);
                anti_roll[i] += force;
                anti_roll[j] -= force;
            }
        }
        for ((s, wheel), arb) in state.specs.iter().zip(&mut state.wheels).zip(anti_roll) {
            if wheel.ground.is_none() {
                continue;
            }
            let relative = relative(&sim.world, chassis, wheel, Vec3::ZERO).dot(wheel.normal);
            let damping = mass
                * if relative < 0.0 {
                    s.compression_damping
                } else {
                    s.rebound_damping
                };
            let bump = (wheel.compression - s.travel * 0.8).max(0.0);
            let k = s.stiffness * mass;
            let inv_mass = pair_mass(&sim.world, chassis, wheel, wheel.point, wheel.normal);
            // Backward Euler spring/damper impulse: stable on light dynamic ground.
            let force = (k * wheel.compression - (damping + k * dt) * relative
                + veh.bump_stop_stiffness.max(0.0) * bump * bump
                + arb)
                / (1.0 + (damping + k * dt) * dt * inv_mass);
            wheel.load = force.max(0.0);
            impulse(
                &mut sim.world,
                chassis,
                wheel,
                wheel.point,
                wheel.normal * wheel.load * dt,
            );
        }
        let drive_wheels: Vec<_> = state
            .specs
            .iter()
            .zip(&state.wheels)
            .map(|(s, w)| s.drive(veh, w.omega, w.slip, false))
            .collect();
        let ext = sim.ext.vehicles.entry(key).or_default();
        ext.wheels = state
            .specs
            .iter()
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
            .collect();
        let gearbox = &mut ext.gearbox;
        let mut speeds = crate::drivetrain::step(veh, gearbox, speed, &drive_wheels, dt);
        for (wheel, spin) in state.wheels.iter_mut().zip(&speeds) {
            wheel.omega = spin.omega
                - spin
                    .omega
                    .clamp(-spin.brake / wheel.inertia, spin.brake / wheel.inertia);
        }
        // Sequential projected impulses share the full chassis/ground mass matrix
        // across all wheels. Each iteration sees the previous wheel's impulse.
        let mut accumulated = vec![Vec2::ZERO; state.wheels.len()];
        for _ in 0..16 {
            for (i, (s, wheel)) in state.specs.iter().zip(&mut state.wheels).enumerate() {
                let Some(ground) = wheel.ground.and_then(|h| sim.world.colliders.get(h)) else {
                    continue;
                };
                let own = sim.world.bodies[chassis]
                    .colliders()
                    .first()
                    .map(|h| &sim.world.colliders[*h]);
                let h1 = own.and_then(|c| hook(scene, c));
                let h2 = hook(scene, ground);
                let surface = h2.map_or(Vec3::ZERO, |h| h.surface_velocity)
                    - h1.map_or(Vec3::ZERO, |h| h.surface_velocity);
                let friction = match (
                    h1.filter(|h| h.friction >= 0.0),
                    h2.filter(|h| h.friction >= 0.0),
                ) {
                    (Some(a), Some(b)) => (a.friction + b.friction) * 0.5,
                    (Some(h), _) | (_, Some(h)) => h.friction,
                    _ => ground.friction(),
                };
                let rb = &sim.world.bodies[chassis];
                let forward = unv(rb.rotation()
                    * super::rot(quat_from_axis_angle(Vec3::Y, wheel.steering))
                    * v(Vec3::Z));
                let longitudinal =
                    (forward - wheel.normal * forward.dot(wheel.normal)).normalize_or_zero();
                let lateral = wheel.normal.cross(longitudinal).normalize_or_zero();
                let velocity = relative(&sim.world, chassis, wheel, surface)
                    + (acceleration(&sim.world, chassis, wheel.point)
                        - ground_body(&sim.world, wheel)
                            .map_or(Vec3::ZERO, |h| acceleration(&sim.world, h, wheel.point)))
                        * dt;
                let vl = velocity.dot(longitudinal);
                let vs = velocity.dot(lateral);
                let rolling = wheel.omega * s.radius;
                let regular = vl.abs().max(rolling.abs()).max(0.5);
                let kappa = (rolling - vl) / regular;
                let alpha = atan2(vs, vl.abs().max(0.5));
                let slip = Vec2::new(kappa, alpha).length();
                wheel.slip = (slip / (1.0 + slip)).clamp(0.0, 1.0);
                wheel.side_speed = vs.abs();
                let hand = if s.handbrake {
                    1.0 + (veh.handbrake_grip - 1.0) * veh.handbrake.clamp(0.0, 1.0)
                } else {
                    1.0
                };
                // Brush curve: peak at moderate slip, sliding approaches 75%.
                // At low speed use static friction, continuous at 0.5 m/s.
                let sliding = 0.75 + 0.25 * exp(-((slip - 0.12).max(0.0)) * 3.0);
                let blend = (regular - 0.5).clamp(0.0, 1.0);
                let capacity = s.grip * friction.max(0.0) * hand.max(0.0) * wheel.load * dt;
                let limit = capacity * (1.0 + (sliding - 1.0) * blend);
                let side_point = wheel.point
                    - up * (up.dot(wheel.point - unv(rb.center_of_mass()))
                        * (1.0 - veh.roll_influence.clamp(0.0, 1.0)));
                let side_velocity = vs
                    + unv(rb.angvel())
                        .cross(side_point - wheel.point)
                        .dot(lateral);
                let kl = pair_mass(&sim.world, chassis, wheel, wheel.point, longitudinal);
                let ks = pair_mass(&sim.world, chassis, wheel, side_point, lateral);
                let old = accumulated[i];
                // Implicit brush stiffness tends continuously to static friction
                // below 0.5 m/s; its peak/slide envelope is the friction circle.
                let softness = blend * regular / (16.5 * capacity).max(1.0e-6);
                let spin = &speeds[i];
                let locked_j = old.x - vl / kl.max(1.0e-6);
                let required_brake = locked_j * s.radius - spin.omega * wheel.inertia;
                let long_delta = if required_brake.abs() <= spin.brake {
                    locked_j - old.x
                } else {
                    let brake = required_brake.clamp(-spin.brake, spin.brake);
                    let rolling =
                        (spin.omega + (brake - old.x * s.radius) / wheel.inertia) * s.radius;
                    (rolling - vl - softness * old.x)
                        / (kl + s.radius * s.radius / wheel.inertia + softness).max(1.0e-6)
                };
                let delta = Vec2::new(
                    long_delta,
                    (-side_velocity - softness * old.y) / (ks + softness).max(1.0e-6),
                );
                let next = (old + delta).clamp_length_max(limit);
                let applied = next - old;
                accumulated[i] = next;
                impulse(
                    &mut sim.world,
                    chassis,
                    wheel,
                    wheel.point,
                    longitudinal * applied.x,
                );
                impulse(
                    &mut sim.world,
                    chassis,
                    wheel,
                    side_point,
                    lateral * applied.y,
                );
                let brake =
                    (next.x * s.radius - spin.omega * wheel.inertia).clamp(-spin.brake, spin.brake);
                wheel.omega = spin.omega + (brake - next.x * s.radius) / wheel.inertia;
                wheel.tyre_force = (longitudinal * next.x + lateral * next.y) / dt;
            }
            if veh.differential == 2 {
                let mut angular: Vec<_> = state.wheels.iter().map(|w| w.omega).collect();
                crate::drivetrain::differential(veh, &drive_wheels, &mut angular, dt);
                for ((wheel, spin), next) in state.wheels.iter_mut().zip(&mut speeds).zip(angular) {
                    // Carry the angular constraint impulse into the next tyre
                    // iteration; never reapply LSD's full-dt impulse per sweep.
                    spin.omega += next - wheel.omega;
                    wheel.omega = next;
                }
            }
        }
        for wheel in &mut state.wheels {
            wheel.spin_angle =
                (wheel.spin_angle + wheel.omega * dt).rem_euclid(std::f32::consts::TAU);
        }
    }
    Ok(())
}

pub(crate) fn readback(scene: &Scene, sim: &mut Sim, out: &mut StepOut) {
    for (e, v) in &scene.vehicles {
        let Some(s) = sim.vehicles.get_mut(&e.to_bits()) else {
            continue;
        };
        for (spec, wheel) in s.specs.iter().zip(&mut s.wheels) {
            let load = wheel.load;
            let force = wheel.tyre_force;
            let previous = wheel.prev_compression;
            cast(scene, &sim.world, s.chassis, spec, wheel, v.wheel_cast);
            wheel.prev_compression = previous;
            if wheel.ground.is_some() {
                wheel.load = load;
                wheel.tyre_force = force;
                let rb = &sim.world.bodies[s.chassis];
                let forward = unv(rb.rotation()
                    * super::rot(quat_from_axis_angle(Vec3::Y, wheel.steering))
                    * super::v(Vec3::Z));
                let longitudinal =
                    (forward - wheel.normal * forward.dot(wheel.normal)).normalize_or_zero();
                let own_surface = rb
                    .colliders()
                    .first()
                    .and_then(|h| hook(scene, &sim.world.colliders[*h]))
                    .map_or(Vec3::ZERO, |h| h.surface_velocity);
                let surface = wheel
                    .ground
                    .and_then(|h| sim.world.colliders.get(h))
                    .and_then(|c| hook(scene, c))
                    .map_or(Vec3::ZERO, |h| h.surface_velocity)
                    - own_surface;
                let velocity = relative(&sim.world, s.chassis, wheel, surface);
                let vl = velocity.dot(longitudinal);
                let vs = velocity.dot(wheel.normal.cross(longitudinal));
                let rolling = wheel.omega * spec.radius;
                let slip = Vec2::new(
                    (rolling - vl) / vl.abs().max(rolling.abs()).max(0.5),
                    atan2(vs, vl.abs().max(0.5)),
                )
                .length();
                wheel.slip = slip / (1.0 + slip);
                wheel.side_speed = vs.abs();
            }
        }
        let rb = &sim.world.bodies[s.chassis];
        let g = sim
            .ext
            .vehicles
            .get(&e.to_bits())
            .map(|x| x.gearbox)
            .unwrap_or_default();
        out.vehicles.push((
            *e,
            Vehicle {
                speed: unv(rb.linvel()).dot(unv(rb.rotation() * super::v(Vec3::Z))),
                contacts: s.wheels.iter().filter(|w| w.ground.is_some()).count() as u32,
                slip: s.wheels.iter().map(|w| w.slip).fold(0.0, f32::max),
                gear: g.gear,
                rpm: g.rpm,
                ..*v
            },
        ));
    }
}

pub(crate) fn wheels(sim: &Sim, entity: u64) -> Option<Vec<WheelPose>> {
    let state = sim.vehicles.get(&entity)?;
    let rb = sim.world.bodies.get(state.chassis)?;
    let q = rb.rotation();
    let q = Quat::from_xyzw(q.x, q.y, q.z, q.w);
    Some(
        state
            .specs
            .iter()
            .zip(&state.wheels)
            .map(|(s, w)| WheelPose {
                position: unv(rb.position().transform_point(v(s.mount))) - q * Vec3::Y * w.length,
                quaternion: (q
                    * quat_from_axis_angle(Vec3::Y, w.steering)
                    * quat_from_axis_angle(Vec3::X, w.spin_angle))
                .normalize(),
                contact: w.ground.is_some(),
                slip: w.slip,
                side_speed: w.side_speed,
                entity: s.entity,
                suspension_compression: w.compression,
                load: w.load,
                contact_point: w.point,
                contact_normal: w.normal,
                ground_entity: w
                    .ground
                    .and_then(|h| sim.world.colliders.get(h))
                    .map_or(0, |c| c.user_data as u64),
                angular_velocity: w.omega,
            })
            .collect(),
    )
}

pub(crate) fn debug_lines(sim: &Sim, out: &mut Vec<super::debug_lines::DebugLine>) {
    for (key, state) in &sim.vehicles {
        let Some(poses) = wheels(sim, *key) else {
            continue;
        };
        let rb = &sim.world.bodies[state.chassis];
        for ((s, w), pose) in state.specs.iter().zip(&state.wheels).zip(poses) {
            let hard = unv(rb.position().transform_point(v(s.mount)));
            let down = unv(rb.rotation() * v(Vec3::NEG_Y));
            let cap = unv(rb.rotation() * v(Vec3::X)) * s.radius * 0.2;
            let compressed = hard + down * (s.rest - s.travel).max(0.0);
            let extended = hard + down * (s.rest + s.travel);
            let travel = [0.1, 0.8, 0.9, 1.0];
            out.push((compressed, extended, travel));
            out.push((compressed - cap, compressed + cap, travel));
            out.push((extended - cap, extended + cap, travel));
            out.push((hard, pose.position, [0.2, 1.0, 0.3, 1.0]));
            for i in 0..16 {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let next = (i + 1) as f32 * std::f32::consts::TAU / 16.0;
                for side in [-1.0, 1.0] {
                    let p = |a: f32| {
                        pose.position
                            + pose.quaternion
                                * Vec3::new(
                                    side * s.radius * 0.25,
                                    cos(a) * s.radius,
                                    sin(a) * s.radius,
                                )
                    };
                    out.push((p(angle), p(next), [0.9, 0.7, 0.1, 1.0]));
                }
            }
            if pose.contact {
                let scale = 1.0 / rb.mass().max(0.01) / 9.81;
                for (force, color) in [
                    (w.tyre_force, [1.0, 0.2, 0.1, 1.0]),
                    (w.normal * w.load, [0.2, 0.5, 1.0, 1.0]),
                ] {
                    let delta = force * scale;
                    let length = delta.length();
                    if length <= f32::EPSILON {
                        continue;
                    }
                    let direction = delta / length;
                    let side = direction.any_orthonormal_vector();
                    let up = direction.cross(side);
                    let head = (length * 0.3).min(s.radius * 0.25);
                    let tip = w.point + delta;
                    out.push((w.point, tip, color));
                    for wing in [side, -side, up, -up] {
                        out.push((tip, tip - direction * head + wing * head * 0.5, color));
                    }
                }
            }
        }
    }
}

impl VehicleState {
    pub(crate) fn form(
        &self,
        entity: u64,
        world: &rp::PhysicsWorld,
        out: &mut Vec<og_core::block_form::Record>,
    ) {
        use og_core::block_form::{Field, Record};
        out.push(Record::new(entity, "vehicle").field(
            "chassis",
            super::sim::owner(world.bodies[self.chassis].user_data),
        ));
        for (i, (s, w)) in self.specs.iter().zip(&self.wheels).enumerate() {
            let ground = w
                .ground
                .and_then(|h| world.colliders.get(h))
                .map_or(Field::Entity(0), |c| super::sim::owner(c.user_data));
            let mut record = Record::new(entity, "wheel")
                .key(Field::of(&i))
                .members(s, &[])
                .members(w, &[]);
            record.fields.retain(|(name, _)| name != "ground");
            out.push(record.field("ground", ground));
        }
    }
}
