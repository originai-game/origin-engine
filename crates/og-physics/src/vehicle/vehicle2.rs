//! Physical 2D wheels on force-based suspension, using the shared drivetrain.
use super::sim::Sim;
use super::{rp, unrot, unv, v};
use crate::gather::Scene;
use crate::schema::*;
use crate::vehicle_wheels::{Spec, specs};
use crate::{StepOut, WheelPose};
use og_core::{Result, glam::Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct VehicleState {
    chassis: rp::RigidBodyHandle,
    specs: Vec<Spec>,
    wheels: Vec<rp::RigidBodyHandle>,
    joints: Vec<rp::ImpulseJointHandle>,
    differential_joints: Vec<rp::ImpulseJointHandle>,
    /// Last step's chassis pose: distinguish a teleport from integration.
    pose: rp::Pose,
    scene_pose: rp::Pose,
    z: f32,
}

impl VehicleState {
    pub(crate) fn shift(&mut self, s: &og_ecs::Shift) {
        self.pose.translation = v(s.moved(unv(self.pose.translation)));
        self.scene_pose.translation = v(s.moved(unv(self.scene_pose.translation)));
        self.z = s.moved(Vec3::new(0.0, 0.0, self.z)).z;
    }
}

fn add_wheel(
    world: &mut rp::PhysicsWorld,
    chassis: rp::RigidBodyHandle,
    s: &Spec,
    mass: f32,
) -> (rp::RigidBodyHandle, rp::ImpulseJointHandle) {
    let rb = &world.bodies[chassis];
    let pose = *rb.position();
    let entity = rb.user_data;
    let groups = rb
        .colliders()
        .first()
        .map(|h| world.colliders[*h].collision_groups())
        .unwrap_or_default();
    let mount = s.mount - Vec3::Y * s.rest;
    let wheel = world.bodies.insert(
        rp::RigidBodyBuilder::dynamic()
            .translation(pose.transform_point(v(mount)))
            .rotation(pose.rotation.angle())
            .user_data(entity),
    );
    world.colliders.insert_with_parent(
        rp::ColliderBuilder::ball(s.radius)
            .mass(mass.max(0.01))
            .friction(s.grip)
            .friction_combine_rule(rp::CoefficientCombineRule::Multiply)
            .active_hooks(
                rp::ActiveHooks::FILTER_CONTACT_PAIRS | rp::ActiveHooks::MODIFY_SOLVER_CONTACTS,
            )
            .collision_groups(groups)
            .user_data(entity),
        wheel,
        &mut world.bodies,
    );
    let joint = rp::GenericJointBuilder::new(rp::JointAxesMask::LIN_X)
        .local_anchor1(v(mount))
        .local_anchor2(rp::Vector::ZERO)
        .contacts_enabled(false)
        .motor_model(rp::JointAxis::LinY, rp::MotorModel::ForceBased)
        .limits(rp::JointAxis::LinY, [-s.travel, s.travel])
        .build();
    let joint = world.impulse_joints.insert(chassis, wheel, joint, true);
    (wheel, joint)
}

pub(crate) fn vehicles(
    scene: &Scene,
    dt: f32,
    sim: &mut Sim,
    handles: &super::sim::Handles<'_>,
    _: &mut StepOut,
) -> Result<()> {
    let gone: Vec<_> = sim
        .vehicles
        .keys()
        .filter(|e| {
            !scene.vehicles.iter().any(|v| v.0.to_bits() == **e)
                || handles
                    .get(e)
                    .is_none_or(|h| !sim.world.bodies[*h].is_dynamic())
        })
        .copied()
        .collect();
    for key in gone {
        if let Some(s) = sim.vehicles.remove(&key) {
            for h in s.wheels {
                sim.world.remove_body(h);
            }
        }
        sim.ext.vehicles.remove(&key);
    }
    for (e, veh) in &scene.vehicles {
        let key = e.to_bits();
        let Some(&chassis) = handles.get(&key) else {
            continue;
        };
        if !sim.world.bodies[chassis].is_dynamic() {
            continue;
        }
        let z = scene
            .bodies
            .iter()
            .find(|b| b.entity == *e)
            .map_or(0.0, |b| b.position.z);
        let mut state = sim.vehicles.remove(&key).unwrap_or_else(|| VehicleState {
            chassis,
            specs: Vec::new(),
            wheels: Vec::new(),
            joints: Vec::new(),
            differential_joints: Vec::new(),
            pose: *sim.world.bodies[chassis].position(),
            scene_pose: *sim.world.bodies[chassis].position(),
            z,
        });
        if chassis != state.chassis {
            for h in state.wheels.drain(..) {
                sim.world.remove_body(h);
            }
            state.specs.clear();
            state.joints.clear();
            state.chassis = chassis;
            state.pose = *sim.world.bodies[chassis].position();
        }
        let layout = specs(
            veh,
            scene.wheels.get(&key).map_or(&[], Vec::as_slice),
            &state.specs,
            true,
        );
        let pose = *sim.world.bodies[chassis].position();
        let scene_pose = scene
            .bodies
            .iter()
            .find(|b| b.entity == *e)
            .map_or(pose, |b| super::pose(b.position, b.quaternion));
        if scene_pose != state.scene_pose && pose != state.pose {
            let delta = pose * state.pose.inverse();
            for &h in &state.wheels {
                let rb = &mut sim.world.bodies[h];
                rb.set_position(delta * *rb.position(), true);
                rb.set_linvel(delta.rotation * rb.linvel(), false);
            }
        }
        state.scene_pose = scene_pose;
        state.z = z;
        let mut bodies = Vec::new();
        let mut joints = Vec::new();
        for (i, s) in layout.iter().enumerate() {
            let index = if s.entity == 0 {
                (i < state.specs.len()).then_some(i)
            } else {
                state.specs.iter().position(|p| p.entity == s.entity)
            };
            let (body, joint) = index
                .map(|i| (state.wheels[i], state.joints[i]))
                .filter(|(h, j)| {
                    sim.world.bodies.get(*h).is_some() && sim.world.impulse_joints.get(*j).is_some()
                })
                .unwrap_or_else(|| add_wheel(&mut sim.world, chassis, s, veh.wheel_mass));
            if let Some(old) = index.and_then(|i| state.specs.get(i)) {
                let delta = s.mount - old.mount - Vec3::Y * (s.rest - old.rest);
                if delta != Vec3::ZERO {
                    let rb = &mut sim.world.bodies[body];
                    rb.set_translation(rb.translation() + pose.rotation * v(delta), true);
                }
            }
            let rb = &mut sim.world.bodies[body];
            let c = &mut sim.world.colliders[rb.colliders()[0]];
            let hand = if s.handbrake {
                1.0 + (veh.handbrake_grip - 1.0) * veh.handbrake.clamp(0.0, 1.0)
            } else {
                1.0
            };
            let grip = s.grip * hand.max(0.0);
            let shape_changed = c.shape().as_ball().is_none_or(|b| b.radius != s.radius);
            let mass_changed = c.mass() != veh.wheel_mass.max(0.01);
            if shape_changed {
                c.set_shape(rp::SharedShape::ball(s.radius));
            }
            if mass_changed {
                c.set_mass(veh.wheel_mass.max(0.01));
            }
            if c.friction() != grip
                || shape_changed
                || mass_changed
                || index.is_none_or(|i| state.specs[i] != *s)
            {
                c.set_friction(grip);
                rb.recompute_mass_properties_from_colliders(&sim.world.colliders);
                rb.wake_up(true);
                sim.world.bodies[chassis].wake_up(true);
            }
            bodies.push(body);
            joints.push(joint);
        }
        for &h in &state.wheels {
            if !bodies.contains(&h) {
                sim.world.remove_body(h);
            }
        }
        state.wheels = bodies;
        state.joints = joints;
        state.specs = layout;
        let driven: Vec<_> = state
            .specs
            .iter()
            .zip(&state.wheels)
            .filter(|(s, _)| s.drive && veh.differential == 2)
            .map(|(_, h)| *h)
            .collect();
        let pairs: Vec<_> = driven
            .first()
            .into_iter()
            .flat_map(|first| driven.iter().skip(1).map(move |h| (*first, *h)))
            .collect();
        if state.differential_joints.len() != pairs.len()
            || state
                .differential_joints
                .iter()
                .zip(&pairs)
                .any(|(j, &(a, b))| {
                    sim.world
                        .impulse_joints
                        .get(*j)
                        .is_none_or(|j| j.body1() != a || j.body2() != b)
                })
        {
            for j in state.differential_joints.drain(..) {
                sim.world.impulse_joints.remove(j, true);
            }
            for (a, b) in pairs {
                let relative =
                    sim.world.bodies[a].position().inverse() * *sim.world.bodies[b].position();
                let joint = rp::GenericJointBuilder::new(rp::JointAxesMask::ANG_X)
                    .local_frame1(relative)
                    .build();
                state
                    .differential_joints
                    .push(sim.world.impulse_joints.insert(a, b, joint, true));
            }
        }
        if dt == 0.0 {
            sim.vehicles.insert(key, state);
            continue;
        }
        let rb = &mut sim.world.bodies[chassis];
        let active = veh.throttle != 0.0 || veh.brake != 0.0 || veh.handbrake != 0.0;
        if active {
            rb.wake_up(true);
        }
        if rb.is_sleeping() {
            sim.vehicles.insert(key, state);
            continue;
        }
        let mass = rb.mass();
        let forward = unv(*rb.rotation() * v(Vec3::X));
        let speed = unv(rb.linvel()).dot(forward);
        let velocity = unv(rb.linvel());
        let up = unv(*rb.rotation() * v(Vec3::Y));
        rb.apply_impulse(
            v((-velocity * velocity.length() * veh.aero_drag.max(0.0)
                - up * speed * speed * veh.aero_downforce.max(0.0))
                * dt),
            false,
        );
        let poses = wheels_for(&sim.world, &state);
        let wheel_info: Vec<_> = state
            .specs
            .iter()
            .zip(&state.wheels)
            .enumerate()
            .map(|(i, (s, &h))| {
                let wheel = &sim.world.bodies[h];
                let omega = -wheel.angvel();
                s.drive(veh, omega, poses[i].slip, true)
            })
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
        let spins = crate::drivetrain::step(veh, gearbox, speed, &wheel_info, dt);
        for (i, s) in state.specs.iter().enumerate() {
            let h = state.wheels[i];
            if active {
                sim.world.bodies[h].wake_up(true);
            }
            let wheel_pose = *sim.world.bodies[h].position();
            let anchor = s.mount - Vec3::Y * s.rest;
            let relative = unv(sim.world.bodies[h].linvel()
                - sim.world.bodies[chassis].velocity_at_point(wheel_pose.translation))
            .dot(up);
            let damping = if relative > 0.0 {
                s.compression_damping
            } else {
                s.rebound_damping
            };
            let compression = unv(wheel_pose.translation
                - sim.world.bodies[chassis]
                    .position()
                    .transform_point(v(anchor)))
            .dot(up);
            let bump = (compression - s.travel * 0.8).max(0.0);
            let bump_k = veh.bump_stop_stiffness.max(0.0);
            let stiffness = s.stiffness * mass + 2.0 * bump_k * bump;
            let force = s.stiffness * mass * compression + bump_k * bump * bump;
            let target = if stiffness > 0.0 {
                compression - force / stiffness
            } else {
                0.0
            };
            let j = sim
                .world
                .impulse_joints
                .get_mut(state.joints[i], false)
                .expect("live joint");
            j.data.set_local_anchor1(v(anchor));
            j.data
                .set_motor_model(rp::JointAxis::LinY, rp::MotorModel::ForceBased);
            j.data
                .set_motor_position(rp::JointAxis::LinY, target, stiffness, damping * mass);
            j.data
                .set_limits(rp::JointAxis::LinY, [-s.travel, s.travel]);
            let drive = (spins[i].omega - wheel_info[i].omega) * wheel_info[i].inertia;
            // Cancel opposing axle torques analytically before contact solving.
            // Otherwise the finite joint/contact sweeps see a drive-speed kick
            // each substep even when the brakes can statically hold it.
            let cancelled = if drive * spins[i].omega > 0.0 {
                drive.clamp(-spins[i].brake, spins[i].brake)
            } else {
                0.0
            };
            // Rapier solves the remaining brake budget with tyre contact.
            j.data
                .set_motor_model(rp::JointAxis::AngX, rp::MotorModel::ForceBased);
            j.data
                // Infinite damping gives zero constraint softness, not an
                // infinite brake torque: max_force below caps the impulse.
                .set_motor_velocity(rp::JointAxis::AngX, 0.0, f32::INFINITY);
            j.data
                .set_motor_max_force(rp::JointAxis::AngX, (spins[i].brake - cancelled.abs()) / dt);
            let rb = &mut sim.world.bodies[h];
            // Equal and opposite axle torque, with no velocity target and no ×10.
            let angular_impulse = -(drive - cancelled);
            rb.apply_torque_impulse(angular_impulse, false);
            sim.world.bodies[chassis].apply_torque_impulse(-angular_impulse, false);
        }
        state.pose = pose;
        sim.vehicles.insert(key, state);
    }
    Ok(())
}

pub(crate) fn readback(scene: &Scene, sim: &mut Sim, out: &mut StepOut) {
    for (e, v) in &scene.vehicles {
        let Some(s) = sim.vehicles.get_mut(&e.to_bits()) else {
            continue;
        };
        let rb = &sim.world.bodies[s.chassis];
        s.pose = *rb.position();
        let speed = unv(rb.linvel()).dot(unv(*rb.rotation() * super::v(Vec3::X)));
        let poses = wheels(sim, e.to_bits()).unwrap_or_default();
        let g = sim
            .ext
            .vehicles
            .get(&e.to_bits())
            .map(|x| x.gearbox)
            .unwrap_or_default();
        out.vehicles.push((
            *e,
            Vehicle {
                speed,
                contacts: poses.iter().filter(|w| w.contact).count() as u32,
                slip: poses.iter().map(|w| w.slip).fold(0.0, f32::max),
                gear: g.gear,
                rpm: g.rpm,
                ..*v
            },
        ));
    }
}

pub(crate) fn wheels(sim: &Sim, entity: u64) -> Option<Vec<WheelPose>> {
    let state = sim.vehicles.get(&entity)?;
    Some(wheels_for(&sim.world, state))
}

fn wheels_for(world: &rp::PhysicsWorld, state: &VehicleState) -> Vec<WheelPose> {
    let chassis = &world.bodies[state.chassis];
    state
        .specs
        .iter()
        .zip(&state.wheels)
        .map(|(s, &h)| {
            let rb = &world.bodies[h];
            let collider = rb.colliders()[0];
            let mut point = Vec3::ZERO;
            let mut normal = Vec3::ZERO;
            let mut ground_entity = 0;
            let mut ground_velocity = Vec3::ZERO;
            let mut load = 0.0;
            for pair in world.narrow_phase.contact_pairs_with(collider) {
                if !pair.has_any_active_contact() {
                    continue;
                }
                let other = if pair.collider1 == collider {
                    pair.collider2
                } else {
                    pair.collider1
                };
                let c = &world.colliders[other];
                if let Some((_, contact)) = pair.find_deepest_contact() {
                    let c1 = &world.colliders[pair.collider1];
                    point = unv(c1.position().transform_point(contact.local_p1));
                    normal = (unv(rb.translation()) - point).normalize_or_zero();
                    ground_entity = c.user_data as u64;
                    ground_velocity = c.parent().map_or(Vec3::ZERO, |h| {
                        unv(world.bodies[h].velocity_at_point(v(point)))
                    });
                    // Solver tangent velocity already includes both hooks,
                    // in collider-pair order, on the actual contact manifold.
                    let sign = if pair.collider1 == collider {
                        -1.0
                    } else {
                        1.0
                    };
                    if let Some(contact) = pair
                        .manifolds()
                        .iter()
                        .flat_map(|m| &m.data.solver_contacts)
                        .next()
                    {
                        ground_velocity += unv(contact.tangent_velocity) * sign;
                    }
                }
                load += pair.total_impulse_magnitude() / world.integration_parameters.dt;
            }
            let forward = unv(*chassis.rotation() * v(Vec3::X));
            let velocity = unv(rb.linvel()) - ground_velocity;
            let speed = velocity.dot(forward);
            let omega = -rb.angvel();
            let slip = (omega * s.radius - speed).abs()
                / speed.abs().max((omega * s.radius).abs()).max(0.5);
            let hard = chassis.position().transform_point(v(s.mount));
            let length = unv(hard - rb.translation()).dot(unv(*chassis.rotation() * v(Vec3::Y)));
            let mut position = unv(rb.translation());
            position.z = state.z;
            WheelPose {
                position,
                quaternion: unrot(*rb.rotation()),
                contact: load > 0.0,
                slip: if load > 0.0 { slip / (1.0 + slip) } else { 0.0 },
                side_speed: 0.0,
                entity: s.entity,
                suspension_compression: s.rest - length,
                load,
                contact_point: if load > 0.0 {
                    point + Vec3::Z * state.z
                } else {
                    Vec3::ZERO
                },
                contact_normal: if load > 0.0 { normal } else { Vec3::ZERO },
                ground_entity: if load > 0.0 { ground_entity } else { 0 },
                angular_velocity: omega,
            }
        })
        .collect()
}

pub(crate) fn debug_lines(sim: &Sim, out: &mut Vec<super::debug_lines::DebugLine>) {
    for (key, state) in &sim.vehicles {
        let Some(poses) = wheels(sim, *key) else {
            continue;
        };
        for (s, w) in state.specs.iter().zip(poses) {
            let hard = unv(sim.world.bodies[state.chassis]
                .position()
                .transform_point(v(s.mount)));
            // Match Rapier's 2D overlay plane, independent of visual z.
            let center = w.position.truncate().extend(0.0);
            let point = w.contact_point.truncate().extend(0.0);
            out.push((hard, center, [0.2, 1.0, 0.3, 1.0]));
            if w.contact {
                out.push((
                    point,
                    point
                        + w.contact_normal * w.load
                            / sim.world.bodies[state.chassis].mass().max(0.01)
                            / 9.81,
                    [0.2, 0.5, 1.0, 1.0],
                ));
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
        use super::sim::{body_record, collider_records, joint_fields, owner};
        use og_core::block_form::{Field, Record};
        out.push(
            Record::new(entity, "vehicle")
                .field("chassis", owner(world.bodies[self.chassis].user_data))
                .field("pose", Field::of(&self.pose))
                .field("scene_pose", Field::of(&self.scene_pose))
                .field("z", Field::of(&self.z)),
        );
        for (i, j) in self.differential_joints.iter().enumerate() {
            if let Some(j) = world.impulse_joints.get(*j) {
                let mut record = Record::new(entity, "differential joint").key(Field::of(&i));
                record.fields = joint_fields(j, &world.bodies);
                out.push(record);
            }
        }
        for (i, ((s, &h), &j)) in self
            .specs
            .iter()
            .zip(&self.wheels)
            .zip(&self.joints)
            .enumerate()
        {
            out.push(
                Record::new(entity, "wheel spec")
                    .key(Field::of(&i))
                    .members(s, &[]),
            );
            out.push(body_record(
                entity,
                "wheel",
                Field::of(&i),
                h,
                world,
                Vec3::ZERO,
            ));
            collider_records(
                out,
                entity,
                "wheel collider",
                world.bodies[h].colliders(),
                world,
                &|k| Field::of(&(i, k)),
            );
            let mut r = Record::new(entity, "wheel joint").key(Field::of(&i));
            if let Some(j) = world.impulse_joints.get(j) {
                r.fields = joint_fields(j, &world.bodies);
            }
            out.push(r);
        }
    }
}
