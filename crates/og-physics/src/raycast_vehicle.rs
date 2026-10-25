//! rapier3d 0.36.1's `DynamicRayCastVehicleController`
//! (`src/control/ray_cast_vehicle_controller.rs`, ported by Dimforge from
//! Bullet's btRaycastVehicle; Apache-2.0, Copyright 2020 Dimforge EURL),
//! carried here so og.Vehicle can set what rapier keeps private: each
//! wheel's roll influence (`og.Vehicle.rollInfluence`) and its skid info
//! (read back as `WheelPose.slip`). The algorithm is rapier's, with those
//! two fields public, `crate::` paths spelled through the public API, the
//! 2D branch and profiling dropped, and its passes regrouped as one pass
//! per wheel (place, cast, spring; then size the friction; then apply), the
//! arithmetic in upstream's order so a world's hash is the same. Upstream's
//! TODOs are done: the friction impulses' denominators take the bodies'
//! effective inverse mass (locked axes, a non-dynamic ground) as rapier's
//! solver does, and a dynamic ground body takes the opposite of the
//! suspension and friction impulses at the contact point (a raft carries
//! the car's weight and is driven backwards by its wheels).

use rapier3d::dynamics::{RigidBody, RigidBodyHandle, RigidBodySet};
use rapier3d::geometry::{ColliderHandle, ColliderSet, Ray};
use rapier3d::math::{Real, Vector, VectorExt, rotation_from_angle};
use rapier3d::pipeline::QueryPipeline;
use rapier3d::prelude::QueryPipelineMut;
use rapier3d::utils::{CrossProduct, DotProduct};
use serde::{Deserialize, Serialize};

/// rapier's `utils::inv`: 1 / `val`, or 0 for a value within 1e-20 of 0.
fn inv(val: Real) -> Real {
    if (-1.0e-20..=1.0e-20).contains(&val) {
        0.0
    } else {
        1.0 / val
    }
}

/// A character controller to simulate vehicles using ray-casting for the wheels.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicRayCastVehicleController {
    wheels: Vec<Wheel>,
    forward_ws: Vec<Vector>,
    axle: Vec<Vector>,
    /// The current forward speed of the vehicle.
    pub current_vehicle_speed: Real,

    /// Handle of the vehicle’s chassis.
    pub chassis: RigidBodyHandle,
    /// The chassis’ local _up_ direction (`0 = x, 1 = y, 2 = z`)
    pub index_up_axis: usize,
    /// The chassis’ local _forward_ direction (`0 = x, 1 = y, 2 = z`)
    pub index_forward_axis: usize,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
/// Parameters affecting the physical behavior of a wheel.
pub struct WheelTuning {
    /// The suspension stiffness.
    ///
    /// Increase this value if the suspension appears to not push the vehicle strong enough.
    pub suspension_stiffness: Real,
    /// The suspension’s damping when it is being compressed.
    pub suspension_compression: Real,
    /// The suspension’s damping when it is being released.
    ///
    /// Increase this value if the suspension appears to overshoot.
    pub suspension_damping: Real,
    /// The maximum distance the suspension can travel before and after its resting length.
    pub max_suspension_travel: Real,
    /// The multiplier of friction between a tire and the collider it's on top of.
    pub side_friction_stiffness: Real,
    /// Parameter controlling how much traction the tire has.
    ///
    /// The larger the value, the more instantaneous braking will happen (with the risk of
    /// causing the vehicle to flip if it’s too strong).
    pub friction_slip: Real,
    /// The maximum force applied by the suspension.
    pub max_suspension_force: Real,
}

impl Default for WheelTuning {
    fn default() -> Self {
        Self {
            suspension_stiffness: 5.88,
            suspension_compression: 0.83,
            suspension_damping: 0.88,
            max_suspension_travel: 5.0,
            side_friction_stiffness: 1.0,
            friction_slip: 10.5,
            max_suspension_force: 6000.0,
        }
    }
}

/// Objects used to initialize a wheel.
struct WheelDesc {
    /// The position of the wheel, relative to the chassis.
    pub chassis_connection_cs: Vector,
    /// The direction of the wheel’s suspension, relative to the chassis.
    ///
    /// The ray-casting will happen following this direction to detect the ground.
    pub direction_cs: Vector,
    /// The wheel’s axle axis, relative to the chassis.
    pub axle_cs: Vector,
    /// The rest length of the wheel’s suspension spring.
    pub suspension_rest_length: Real,
    /// The maximum distance the suspension can travel before and after its resting length.
    pub max_suspension_travel: Real,
    /// The wheel’s radius.
    pub radius: Real,

    /// The suspension stiffness.
    ///
    /// Increase this value if the suspension appears to not push the vehicle strong enough.
    pub suspension_stiffness: Real,
    /// The suspension’s damping when it is being compressed.
    pub damping_compression: Real,
    /// The suspension’s damping when it is being released.
    ///
    /// Increase this value if the suspension appears to overshoot.
    pub damping_relaxation: Real,
    /// Parameter controlling how much traction the tire has.
    ///
    /// The larger the value, the more instantaneous braking will happen (with the risk of
    /// causing the vehicle to flip if it’s too strong).
    pub friction_slip: Real,
    /// The maximum force applied by the suspension.
    pub max_suspension_force: Real,
    /// The multiplier of friction between a tire and the collider it's on top of.
    pub side_friction_stiffness: Real,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
/// A wheel attached to a vehicle.
pub struct Wheel {
    raycast_info: RayCastInfo,

    center: Vector,
    wheel_direction_ws: Vector,
    wheel_axle_ws: Vector,

    /// The position of the wheel, relative to the chassis.
    pub chassis_connection_point_cs: Vector,
    /// The direction of the wheel’s suspension, relative to the chassis.
    ///
    /// The ray-casting will happen following this direction to detect the ground.
    pub direction_cs: Vector,
    /// The wheel’s axle axis, relative to the chassis.
    pub axle_cs: Vector,
    /// The rest length of the wheel’s suspension spring.
    pub suspension_rest_length: Real,
    /// The maximum distance the suspension can travel before and after its resting length.
    pub max_suspension_travel: Real,
    /// The wheel’s radius.
    pub radius: Real,
    /// The suspension stiffness.
    ///
    /// Increase this value if the suspension appears to not push the vehicle strong enough.
    pub suspension_stiffness: Real,
    /// The suspension’s damping when it is being compressed.
    pub damping_compression: Real,
    /// The suspension’s damping when it is being released.
    ///
    /// Increase this value if the suspension appears to overshoot.
    pub damping_relaxation: Real,
    /// Parameter controlling how much traction the tire has.
    ///
    /// The larger the value, the more instantaneous braking will happen (with the risk of
    /// causing the vehicle to flip if it’s too strong).
    pub friction_slip: Real,
    /// The multiplier of friction between a tire and the collider it's on top of.
    pub side_friction_stiffness: Real,
    /// The wheel’s current rotation on its axle.
    pub rotation: Real,
    delta_rotation: Real,
    /// How much of a side impulse rolls the chassis: 0 applies it at the
    /// centre of mass's height (no roll), 1 at the contact point.
    pub roll_influence: Real,
    /// The maximum force applied by the suspension.
    pub max_suspension_force: Real,

    /// The forward impulses applied by the wheel on the chassis.
    pub forward_impulse: Real,
    /// The side impulses applied by the wheel on the chassis.
    pub side_impulse: Real,

    /// The steering angle for this wheel.
    pub steering: Real,
    /// The forward force applied by this wheel on the chassis.
    pub engine_force: Real,
    /// The maximum amount of braking impulse applied to slow down the vehicle.
    pub brake: Real,

    clipped_inv_contact_dot_suspension: Real,
    suspension_relative_velocity: Real,
    /// The force applied by the suspension.
    pub wheel_suspension_force: Real,
    /// How much of its impulse the tyre held at the last update: 1
    /// gripping, below 1 sliding (the share it could hold).
    pub skid_info: Real,
}

impl Wheel {
    fn new(info: WheelDesc) -> Self {
        Self {
            raycast_info: RayCastInfo::default(),
            suspension_rest_length: info.suspension_rest_length,
            max_suspension_travel: info.max_suspension_travel,
            radius: info.radius,
            suspension_stiffness: info.suspension_stiffness,
            damping_compression: info.damping_compression,
            damping_relaxation: info.damping_relaxation,
            chassis_connection_point_cs: info.chassis_connection_cs,
            direction_cs: info.direction_cs,
            axle_cs: info.axle_cs,
            wheel_direction_ws: info.direction_cs,
            wheel_axle_ws: info.axle_cs,
            center: Vector::ZERO,
            friction_slip: info.friction_slip,
            steering: 0.0,
            engine_force: 0.0,
            rotation: 0.0,
            delta_rotation: 0.0,
            brake: 0.0,
            roll_influence: 0.1,
            clipped_inv_contact_dot_suspension: 0.0,
            suspension_relative_velocity: 0.0,
            wheel_suspension_force: 0.0,
            max_suspension_force: info.max_suspension_force,
            skid_info: 0.0,
            side_impulse: 0.0,
            forward_impulse: 0.0,
            side_friction_stiffness: info.side_friction_stiffness,
        }
    }

    /// Information about suspension and the ground obtained from the ray-casting
    /// for this wheel.
    pub fn raycast_info(&self) -> &RayCastInfo {
        &self.raycast_info
    }
}

/// Information about suspension and the ground obtained from the ray-casting
/// to simulate a wheel’s suspension.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Default)]
pub struct RayCastInfo {
    /// The (world-space) contact normal between the wheel and the floor.
    pub contact_normal_ws: Vector,
    /// The (world-space) point hit by the wheel’s ray-cast.
    pub contact_point_ws: Vector,
    /// The suspension length for the wheel.
    pub suspension_length: Real,
    /// The (world-space) starting point of the ray-cast.
    pub hard_point_ws: Vector,
    /// Is the wheel in contact with the ground?
    pub is_in_contact: bool,
    /// The collider hit by the ray-cast.
    pub ground_object: Option<ColliderHandle>,
}

impl DynamicRayCastVehicleController {
    /// Creates a new vehicle represented by the given rigid-body.
    ///
    /// Wheels have to be attached afterwards calling [`Self::add_wheel`].
    pub fn new(chassis: RigidBodyHandle) -> Self {
        Self {
            wheels: vec![],
            forward_ws: vec![],
            axle: vec![],
            current_vehicle_speed: 0.0,
            chassis,
            index_up_axis: 1,
            index_forward_axis: 0,
        }
    }

    //
    // basically most of the code is general for 2 or 4 wheel vehicles, but some of it needs to be reviewed
    //
    /// Adds a wheel to this vehicle.
    pub fn add_wheel(
        &mut self,
        chassis_connection_cs: Vector,
        direction_cs: Vector,
        axle_cs: Vector,
        suspension_rest_length: Real,
        radius: Real,
        tuning: &WheelTuning,
    ) -> &mut Wheel {
        let ci = WheelDesc {
            chassis_connection_cs,
            direction_cs,
            axle_cs,
            suspension_rest_length,
            radius,
            suspension_stiffness: tuning.suspension_stiffness,
            damping_compression: tuning.suspension_compression,
            damping_relaxation: tuning.suspension_damping,
            friction_slip: tuning.friction_slip,
            max_suspension_travel: tuning.max_suspension_travel,
            max_suspension_force: tuning.max_suspension_force,
            side_friction_stiffness: tuning.side_friction_stiffness,
        };

        let wheel_id = self.wheels.len();
        self.wheels.push(Wheel::new(ci));

        &mut self.wheels[wheel_id]
    }

    fn update_wheel_transform(&mut self, chassis: &RigidBody, wheel_index: usize) {
        self.update_wheel_transforms_ws(chassis, wheel_index);
        let wheel = &mut self.wheels[wheel_index];

        let steering_orn = rotation_from_angle(-wheel.wheel_direction_ws * wheel.steering);
        wheel.wheel_axle_ws = steering_orn * (chassis.position().rotation * wheel.axle_cs);
        wheel.center = wheel.raycast_info.hard_point_ws
            + wheel.wheel_direction_ws * wheel.raycast_info.suspension_length;
    }

    fn update_wheel_transforms_ws(&mut self, chassis: &RigidBody, wheel_id: usize) {
        let wheel = &mut self.wheels[wheel_id];
        wheel.raycast_info.is_in_contact = false;

        let chassis_transform = chassis.position();

        wheel.raycast_info.hard_point_ws = chassis_transform * wheel.chassis_connection_point_cs;
        wheel.wheel_direction_ws = chassis_transform.rotation * wheel.direction_cs;
        wheel.wheel_axle_ws = chassis_transform.rotation * wheel.axle_cs;
    }

    fn ray_cast(&mut self, queries: &QueryPipeline, chassis: &RigidBody, wheel_id: usize) {
        let wheel = &mut self.wheels[wheel_id];
        let raylen = wheel.suspension_rest_length + wheel.radius;
        let rayvector = wheel.wheel_direction_ws * raylen;
        let source = wheel.raycast_info.hard_point_ws;
        wheel.raycast_info.contact_point_ws = source + rayvector;
        let ray = Ray::new(source, rayvector);
        let hit = queries.cast_ray_and_get_normal(&ray, 1.0, true);

        wheel.raycast_info.ground_object = None;

        if let Some((collider_hit, mut hit)) = hit {
            if hit.time_of_impact == 0.0 {
                let collider = &queries.colliders[collider_hit];
                let up_ray = Ray::new(source + rayvector, -rayvector);
                if let Some(hit2) = collider.shape().cast_ray_and_get_normal(
                    collider.position(),
                    &up_ray,
                    1.0,
                    false,
                ) {
                    hit.normal = -hit2.normal;
                }

                if hit.normal == Vector::ZERO {
                    // If the hit is still not defined, set the normal.
                    hit.normal = -wheel.wheel_direction_ws;
                }
            }

            wheel.raycast_info.contact_normal_ws = hit.normal;
            wheel.raycast_info.is_in_contact = true;
            wheel.raycast_info.ground_object = Some(collider_hit);

            let hit_distance = hit.time_of_impact * raylen;
            wheel.raycast_info.suspension_length = hit_distance - wheel.radius;

            // clamp on max suspension travel
            let min_suspension_length = wheel.suspension_rest_length - wheel.max_suspension_travel;
            let max_suspension_length = wheel.suspension_rest_length + wheel.max_suspension_travel;
            wheel.raycast_info.suspension_length = wheel
                .raycast_info
                .suspension_length
                .clamp(min_suspension_length, max_suspension_length);
            wheel.raycast_info.contact_point_ws = ray.point_at(hit.time_of_impact);

            let denominator = wheel
                .raycast_info
                .contact_normal_ws
                .dot(wheel.wheel_direction_ws);
            let chassis_velocity_at_contact_point =
                chassis.velocity_at_point(wheel.raycast_info.contact_point_ws);
            let proj_vel = wheel
                .raycast_info
                .contact_normal_ws
                .dot(chassis_velocity_at_contact_point);

            if denominator >= -0.1 {
                wheel.suspension_relative_velocity = 0.0;
                wheel.clipped_inv_contact_dot_suspension = 1.0 / 0.1;
            } else {
                let inv = -1.0 / denominator;
                wheel.suspension_relative_velocity = proj_vel * inv;
                wheel.clipped_inv_contact_dot_suspension = inv;
            }
        } else {
            // No contact, put wheel info as in rest position
            wheel.raycast_info.suspension_length = wheel.suspension_rest_length;
            wheel.suspension_relative_velocity = 0.0;
            wheel.raycast_info.contact_normal_ws = -wheel.wheel_direction_ws;
            wheel.clipped_inv_contact_dot_suspension = 1.0;
        }
    }

    /// Updates the vehicle’s velocity based on its suspension, engine force, and brake.
    pub fn update_vehicle(&mut self, dt: Real, queries: QueryPipelineMut) {
        let chassis = &queries.bodies[self.chassis];

        self.current_vehicle_speed = chassis.linvel().length();

        let forward_w = chassis.position().rotation * Vector::ith(self.index_forward_axis, 1.0);

        if forward_w.dot(chassis.linvel()) < 0.0 {
            self.current_vehicle_speed *= -1.0;
        }

        // One pass per wheel over the chassis as the last step left it:
        // place the wheel, cast its ray, size its spring. Nothing here
        // moves a body, so the wheels do not see each other's impulses.
        let chassis_mass = chassis.mass();
        for wheel_id in 0..self.wheels.len() {
            self.update_wheel_transform(chassis, wheel_id);
            self.ray_cast(&queries.as_ref(), chassis, wheel_id);
            self.wheels[wheel_id].update_suspension(chassis_mass);
        }

        for wheel in &mut self.wheels {
            let chassis = queries.bodies.get_mut(self.chassis).unwrap();
            if wheel.engine_force > 0.0 {
                chassis.wake_up(true);
            }

            // Apply the suspension force: up on the chassis, down on a
            // dynamic ground body (a raft carries the car's weight).
            let suspension_force = wheel.wheel_suspension_force.min(wheel.max_suspension_force);
            let impulse = wheel.raycast_info.contact_normal_ws * suspension_force * dt;
            let contact_point = wheel.raycast_info.contact_point_ws;
            chassis.apply_impulse_at_point(impulse, contact_point, false);
            if let Some(ground) = dynamic_ground(queries.bodies, queries.colliders, wheel) {
                ground.apply_impulse_at_point(-impulse, contact_point, true);
            }
        }

        self.update_friction(queries.bodies, queries.colliders, dt);

        let chassis = queries.bodies.get_mut(self.chassis).unwrap();

        for wheel in &mut self.wheels {
            let vel = chassis.velocity_at_point(wheel.raycast_info.hard_point_ws);

            if wheel.raycast_info.is_in_contact {
                // Use the same per-wheel forward direction as `update_friction` (rolling
                // direction on the contact plane) instead of the chassis' forward axis:
                // the latter yields a zero or sign-flipped rotation whenever it doesn't
                // match the wheel's actual rolling direction.
                let fwd = wheel
                    .raycast_info
                    .contact_normal_ws
                    .cross(wheel.wheel_axle_ws)
                    .normalize_or_zero();

                let proj2 = fwd.dot(vel);

                wheel.delta_rotation = (proj2 * dt) / (wheel.radius);
                wheel.rotation += wheel.delta_rotation;
            } else {
                wheel.rotation += wheel.delta_rotation;
            }

            wheel.delta_rotation *= 0.99; //damping of rotation when not in contact
        }
    }

    /// Reference to all the wheels attached to this vehicle.
    pub fn wheels(&self) -> &[Wheel] {
        &self.wheels
    }

    /// Mutable reference to all the wheels attached to this vehicle.
    pub fn wheels_mut(&mut self) -> &mut [Wheel] {
        &mut self.wheels
    }

    /// Sizes every wheel's side and forward impulses against the bodies'
    /// velocities as the suspension left them, then applies them all.
    fn update_friction(&mut self, bodies: &mut RigidBodySet, colliders: &ColliderSet, dt: Real) {
        let num_wheels = self.wheels.len();

        if num_wheels == 0 {
            return;
        }

        self.forward_ws.resize(num_wheels, Default::default());
        self.axle.resize(num_wheels, Default::default());

        // The rolling friction (coasting and braking) is shared among the
        // wheels on the ground, so they are counted first.
        let num_wheels_on_ground = self
            .wheels
            .iter()
            .filter(|w| w.raycast_info.ground_object.is_some())
            .count();

        // One pass per wheel, sizing its impulses: nothing is applied until
        // every wheel has been sized against the same velocities.
        let chassis = &bodies[self.chassis];
        for (i, wheel) in self.wheels.iter_mut().enumerate() {
            wheel.side_impulse = 0.0;
            wheel.forward_impulse = 0.0;
            wheel.skid_info = 1.0;

            let Some(ground_object) = wheel.raycast_info.ground_object else {
                continue;
            };
            let ground = colliders[ground_object].parent().map(|h| &bodies[h]);
            let contact_point = wheel.raycast_info.contact_point_ws;

            // The axle and the rolling direction on the contact plane.
            let surf_normal_ws = wheel.raycast_info.contact_normal_ws;
            let axle = wheel.wheel_axle_ws;
            let axle = (axle - surf_normal_ws * axle.dot(surf_normal_ws)).normalize_or_zero();
            self.axle[i] = axle;
            self.forward_ws[i] = surf_normal_ws.cross(axle).normalize_or_zero();

            // Sideways: take a share of the slide relative to the ground.
            wheel.side_impulse = side_impulse(
                chassis,
                ground.filter(|b| b.is_dynamic()),
                contact_point,
                axle,
            ) * wheel.side_friction_stiffness;

            // Forward: the engine's push, or else the rolling friction
            // (towards no relative velocity, up to the brake's impulse).
            wheel.forward_impulse = if wheel.engine_force != 0.0 {
                wheel.engine_force * dt
            } else {
                WheelContactPoint::new(
                    chassis,
                    ground,
                    contact_point,
                    self.forward_ws[i],
                    wheel.brake,
                )
                .calc_rolling_friction(num_wheels_on_ground)
            };

            // The friction circle: past what the suspension's load holds,
            // the tyre slides and both impulses scale down to the edge.
            let max_imp = wheel.wheel_suspension_force * dt * wheel.friction_slip;
            let x = wheel.forward_impulse * FORWARD_GRIP_SHARE;
            let y = wheel.side_impulse;
            let impulse_squared = x * x + y * y;

            if impulse_squared > max_imp * max_imp {
                wheel.skid_info = max_imp * inv(impulse_squared.sqrt());
                if wheel.skid_info < 1.0 {
                    wheel.forward_impulse *= wheel.skid_info;
                    wheel.side_impulse *= wheel.skid_info;
                }
            }
        }

        // Apply the impulses: the chassis takes them (the side one raised
        // towards its centre of mass by 1 − roll influence), a dynamic
        // ground body the opposite of both at the contact point. A fixed or
        // kinematic ground takes nothing.
        for (wheel_id, wheel) in self.wheels.iter().enumerate() {
            let contact_point = wheel.raycast_info.contact_point_ws;
            let forward_impulse = self.forward_ws[wheel_id] * wheel.forward_impulse;
            let side_impulse = self.axle[wheel_id] * wheel.side_impulse;
            let chassis = bodies.get_mut(self.chassis).unwrap();

            if wheel.forward_impulse != 0.0 {
                chassis.apply_impulse_at_point(forward_impulse, contact_point, false);
            }
            if wheel.side_impulse != 0.0 {
                let v_chassis_world_up =
                    chassis.position().rotation * Vector::ith(self.index_up_axis, 1.0);
                let impulse_point = contact_point
                    - v_chassis_world_up
                        * (v_chassis_world_up.dot(contact_point - chassis.center_of_mass())
                            * (1.0 - wheel.roll_influence));

                chassis.apply_impulse_at_point(side_impulse, impulse_point, false);
            }

            if let Some(ground) = dynamic_ground(bodies, colliders, wheel) {
                ground.apply_impulse_at_point(
                    -(forward_impulse + side_impulse),
                    contact_point,
                    true,
                );
            }
        }
    }
}

/// The body under `wheel` when it is dynamic: the one that takes the
/// opposite of the wheel's impulses. Ground without a body, or a fixed or
/// kinematic one, takes none.
fn dynamic_ground<'a>(
    bodies: &'a mut RigidBodySet,
    colliders: &ColliderSet,
    wheel: &Wheel,
) -> Option<&'a mut RigidBody> {
    let body = colliders[wheel.raycast_info.ground_object?].parent()?;
    bodies.get_mut(body).filter(|b| b.is_dynamic())
}

impl Wheel {
    /// The spring and damper's force from the ray's suspension length and
    /// the chassis's speed along it, in Bullet's units (× the chassis's
    /// mass); 0 off the ground.
    fn update_suspension(&mut self, chassis_mass: Real) {
        if !self.raycast_info.is_in_contact {
            self.wheel_suspension_force = 0.0;
            return;
        }
        let length_diff = self.suspension_rest_length - self.raycast_info.suspension_length;
        let spring =
            self.suspension_stiffness * length_diff * self.clipped_inv_contact_dot_suspension;
        let projected_rel_vel = self.suspension_relative_velocity;
        let susp_damping = if projected_rel_vel < 0.0 {
            self.damping_compression
        } else {
            self.damping_relaxation
        };
        let force = spring - susp_damping * projected_rel_vel;
        self.wheel_suspension_force = (force * chassis_mass).max(0.0);
    }
}

/// The share of a tyre's sideways velocity relative to the ground that one
/// update's side impulse removes: Bullet's `contactDamping`, a literal in
/// upstream's `resolveSingleBilateral`. 1 would stop a sideways slide in one
/// update; 0.2 lets it decay over about five, the friction circle capping
/// the impulse as the load allows.
const SIDE_FRICTION_DAMPING: Real = 0.2;

/// How much the forward impulse weighs against the side impulse on the
/// friction circle (Bullet's `fwdFactor`): half, so a tyre under full drive
/// or brake still holds most of its cornering grip.
const FORWARD_GRIP_SHARE: Real = 0.5;

struct WheelContactPoint<'a> {
    body0: &'a RigidBody,
    body1: Option<&'a RigidBody>,
    friction_position_world: Vector,
    friction_direction_world: Vector,
    jac_diag_ab_inv: Real,
    max_impulse: Real,
}

impl<'a> WheelContactPoint<'a> {
    pub fn new(
        body0: &'a RigidBody,
        body1: Option<&'a RigidBody>,
        friction_position_world: Vector,
        friction_direction_world: Vector,
        max_impulse: Real,
    ) -> Self {
        fn impulse_denominator(body: &RigidBody, pos: Vector, n: Vector) -> Real {
            let dpt = pos - body.center_of_mass();
            let gcross = dpt.gcross(n);
            let v = (body.mass_properties().effective_world_inv_inertia * gcross).gcross(dpt);
            inv_mass_along(body, n) + n.dot(v)
        }
        let denom0 = impulse_denominator(body0, friction_position_world, friction_direction_world);
        let denom1 = body1
            .map(|body1| {
                impulse_denominator(body1, friction_position_world, friction_direction_world)
            })
            .unwrap_or(0.0);
        let relaxation = 1.0;
        let jac_diag_ab_inv = relaxation / (denom0 + denom1);

        Self {
            body0,
            body1,
            friction_position_world,
            friction_direction_world,
            jac_diag_ab_inv,
            max_impulse,
        }
    }

    pub fn calc_rolling_friction(&self, num_wheels_on_ground: usize) -> Real {
        let contact_pos_world = self.friction_position_world;
        let max_impulse = self.max_impulse;

        let vel1 = self.body0.velocity_at_point(contact_pos_world);
        let vel2 = self
            .body1
            .map(|b| b.velocity_at_point(contact_pos_world))
            .unwrap_or_default();
        let vel = vel1 - vel2;
        let vrel = self.friction_direction_world.dot(vel);

        // calculate friction that moves us to zero relative velocity
        (-vrel * self.jac_diag_ab_inv / (num_wheels_on_ground as Real))
            .clamp(-max_impulse, max_impulse)
    }
}

/// `body`'s inverse mass along the unit direction `n` as rapier's solver
/// takes it, `n · (effective_inv_mass ∘ n)`: a locked translation axis and
/// a body that is not dynamic contribute nothing (an additional mass is
/// already in the local mass properties). Where the effective inverse mass
/// is the same on every axis it is that scalar exactly, `n` being unit, so
/// a free chassis is not moved by the rounding of |n|².
fn inv_mass_along(body: &RigidBody, n: Vector) -> Real {
    let mprops = body.mass_properties();
    let im = mprops.effective_inv_mass;
    if im == Vector::splat(mprops.local_mprops.inv_mass) {
        mprops.local_mprops.inv_mass
    } else {
        n.dot(im * n)
    }
}

/// The side impulse along `normal` at `pt` that removes
/// [`SIDE_FRICTION_DAMPING`] of the chassis's velocity relative to `ground`
/// there: a dynamic ground body shares the impulse (its inverse mass and
/// inertia in the denominator); `None` is ground that does not give.
fn side_impulse(
    chassis: &RigidBody,
    ground: Option<&RigidBody>,
    pt: Vector,
    normal: Vector,
) -> Real {
    let vel1 = chassis.velocity_at_point(pt);
    let dpt1 = pt - chassis.center_of_mass();
    let aj = dpt1.gcross(normal);
    let iaj = chassis.mass_properties().effective_world_inv_inertia * aj;
    let im1 = inv_mass_along(chassis, normal);

    let (vel2, im2, ibj_sq) = ground.map_or((Vector::ZERO, 0.0, 0.0), |ground| {
        let dpt2 = pt - ground.center_of_mass();
        let bj = dpt2.gcross(-normal);
        let ibj = ground.mass_properties().effective_world_inv_inertia * bj;
        let im2 = inv_mass_along(ground, normal);
        (ground.velocity_at_point(pt), im2, ibj.gdot(ibj))
    });

    let dvel = vel1 - vel2;
    let jac_diag_ab = im1 + im2 + iaj.gdot(iaj) + ibj_sq;
    let jac_diag_ab_inv = inv(jac_diag_ab);
    let rel_vel = normal.dot(dvel);

    -SIDE_FRICTION_DAMPING * rel_vel * jac_diag_ab_inv
}
