//! The reflected types of vehicles: og.Vehicle and og.VehicleWheel.

use og_core::{List, Reflect};
use serde::{Deserialize, Serialize};

/// `Vehicle.drive`: the rear wheels drive.
pub const DRIVE_REAR: u32 = 0;
/// `Vehicle.drive`: the front wheels drive.
pub const DRIVE_FRONT: u32 = 1;
/// `Vehicle.drive`: every wheel drives.
pub const DRIVE_ALL: u32 = 2;
/// `Vehicle.model`: the arcade car: engineForce, faded out towards maxSpeed.
pub const VEHICLE_ARCADE: u32 = 0;
/// `Vehicle.model`: an engine with a torque curve driving through a gearbox.
pub const VEHICLE_DRIVETRAIN: u32 = 1;

/// A car on a dynamic chassis (the entity's og.RigidBody and og.Collider).
/// 3D: a raycast car with four wheels (suspension, grip, steering on the
/// front axle), or with its og.VehicleWheel children; its forward is the chassis's local +z (three.js
/// Object3D.lookAt aims +z). 2D: two
/// wheel bodies on sprung wheel joints (front at +x), driven by a motor.
/// The implicit wheels are not entities: og.physics.vehicleWheels gives their poses
/// for drawing. The defaults drive a road car of about 1,000 kg: give the
/// chassis that mass (og.RigidBody.mass 1000), since a car-sized box at
/// the collider's density 1 weighs a few kilograms and the forces, in
/// newtons, would throw it. The suspension and the grip scale with the
/// chassis's mass (Bullet's units); the engine and the brakes do not. The
/// wheel rays meet what the chassis's collider would (its og.Collider
/// layers and mask), so cars on layers that do not collide pass through
/// each other.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Vehicle", version = 2)]
pub struct Vehicle {
    /// The accelerator, -1 (reverse) to 1. Set it every tick.
    #[reflect(min = -1, max = 1)]
    pub throttle: f32,
    /// The steering, -1 (left) to 1 (right); 3D only.
    #[reflect(min = -1, max = 1)]
    pub steer: f32,
    /// The brake, 0 to 1.
    #[reflect(min = 0, max = 1)]
    pub brake: f32,
    /// The distance between the front and rear axles.
    #[reflect(name = "wheelBase", min = 0, unit = "m", default = 2.6, tunable)]
    pub wheel_base: f32,
    /// The distance between the left and right wheels (3D).
    #[reflect(min = 0, unit = "m", default = 1.6, tunable)]
    pub track: f32,
    /// The wheels' radius.
    #[reflect(name = "wheelRadius", min = 0.01, unit = "m", default = 0.4, tunable)]
    pub wheel_radius: f32,
    /// The height of the wheel mounts below the chassis centre.
    #[reflect(name = "mountHeight", unit = "m", default = 0.5, tunable)]
    pub mount_height: f32,
    /// The suspension's length at rest.
    #[reflect(name = "suspensionRest", min = 0, unit = "m", default = 0.35, tunable)]
    pub suspension_rest: f32,
    /// How far the suspension compresses or extends from rest.
    #[reflect(
        name = "suspensionTravel",
        min = 0,
        unit = "m",
        default = 0.25,
        tunable
    )]
    pub suspension_travel: f32,
    /// The suspension's stiffness.
    #[reflect(name = "suspensionStiffness", min = 0, default = 30, tunable)]
    pub suspension_stiffness: f32,
    /// The suspension's damping.
    #[reflect(name = "suspensionDamping", min = 0, default = 4, tunable)]
    pub suspension_damping: f32,
    /// The tyres' grip (3D: the friction slip, how much of the load a tyre holds before it slides; 2D: the wheels' friction).
    #[reflect(min = 0, default = 2, tunable)]
    pub grip: f32,
    /// The engine's force at full throttle (3D: per driven wheel, N; 2D: the motor's torque, N·m).
    #[reflect(name = "engineForce", min = 0, default = 2500, tunable)]
    pub engine_force: f32,
    /// The top speed: the engine's force fades out towards it (throttle against the motion brakes first).
    #[reflect(name = "maxSpeed", min = 0, unit = "m/s", default = 40, tunable)]
    pub max_speed: f32,
    /// The brakes' force at full brake (3D: per wheel, N, every wheel braking; 2D: the wheels' braking torque).
    #[reflect(name = "brakeForce", min = 0, default = 2000, tunable)]
    pub brake_force: f32,
    /// The front wheels' largest steering angle (3D).
    #[reflect(
        name = "maxSteer",
        min = 0,
        max = 1.2,
        unit = "rad",
        default = 0.55,
        tunable
    )]
    pub max_steer: f32,
    /// DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2.
    #[reflect(min = 0, max = 2)]
    pub drive: u32,
    /// The wheels' mass (2D: the wheel bodies; 3D ignores it).
    #[reflect(name = "wheelMass", min = 0.01, unit = "kg", default = 20, tunable)]
    pub wheel_mass: f32,
    /// How much the tyres' sideways grip rolls the chassis in a bend (3D): 0 pushes at the centre of mass's height, so the car corners flat and never tips over from steering (an arcade car); towards 1 at the road, the roll that tips a tall car over (rapier's own car uses 0.1, which rolls a 1,000 kg car at full lock above about 20 m/s). Raise it a little for body roll, with og.RigidBody.centerOfMass lowered to keep the car on its wheels.
    #[reflect(name = "rollInfluence", min = 0, max = 1, tunable)]
    pub roll_influence: f32,
    /// Written: the forward speed (m/s; negative reversing).
    #[reflect(unit = "m/s")]
    pub speed: f32,
    /// Written: how many wheels touch the ground.
    pub contacts: u32,
    /// Written: the largest slip of its wheels (og.physics.vehicleWheels' `slip`): 0 every tyre grips, towards 1 one slides (3D; 0 in 2D).
    #[reflect(min = 0, max = 1)]
    pub slip: f32,
    /// VEHICLE_ARCADE 0 (engineForce, faded out towards maxSpeed) or VEHICLE_DRIVETRAIN 1 (an engine with a torque curve through a gearbox).
    #[reflect(min = 0, max = 1)]
    #[serde(skip)]
    pub model: u32,
    /// The handbrake, 0 to 1: brakes the handbrake wheels (the rear pair, or the og.VehicleWheel children flagged `handbrake`) with brakeForce, takes their drive and loosens their grip.
    #[reflect(min = 0, max = 1)]
    #[serde(skip)]
    pub handbrake: f32,
    /// The handbrake wheels' grip at full handbrake, sideways and in all, as a share of their own.
    #[reflect(name = "handbrakeGrip", min = 0, max = 1, default = 0.2, tunable)]
    #[serde(skip)]
    pub handbrake_grip: f32,
    /// The handbrake wheels' sideways grip while the car drifts (the handbrake is on or one of them slides), as a share of their own: below 1 the tail stays out.
    #[reflect(name = "driftGrip", min = 0, max = 1, default = 1, tunable)]
    #[serde(skip)]
    pub drift_grip: f32,
    /// Drivetrain: the forward gears' ratios, first gear first (empty: 3.4, 2.2, 1.6, 1.25, 1.0, 0.82).
    #[serde(skip)]
    pub gears: List<f32, 8>,
    /// Drivetrain: the reverse gear's ratio.
    #[reflect(name = "reverseRatio", min = 0, default = 3.2, tunable)]
    #[serde(skip)]
    pub reverse_ratio: f32,
    /// Drivetrain: the final drive's ratio, after every gear.
    #[reflect(name = "finalDrive", min = 0, default = 3.6, tunable)]
    #[serde(skip)]
    pub final_drive: f32,
    /// Drivetrain: the engine's speed at rest, in revolutions per minute.
    #[reflect(name = "idleRpm", min = 0, default = 900, tunable)]
    #[serde(skip)]
    pub idle_rpm: f32,
    /// Drivetrain: the rev limit: the engine gives no torque at or above it.
    #[reflect(name = "maxRpm", min = 1, default = 6800, tunable)]
    #[serde(skip)]
    pub max_rpm: f32,
    /// Drivetrain: the engine's torque (N·m) at full throttle, at evenly spaced speeds from 0 to maxRpm, interpolated between them (empty: 140, 170, 195, 210, 215, 205, 185, 150).
    #[reflect(name = "torqueCurve")]
    #[serde(skip)]
    pub torque_curve: List<f32, 8>,
    /// Drivetrain: the gearbox shifts by itself (and into reverse when throttle is negative at a standstill); false leaves `gear` to the game.
    #[reflect(default = 1)]
    #[serde(skip)]
    pub automatic: bool,
    /// Drivetrain, automatic: the engine speed it shifts up at.
    #[reflect(name = "shiftUpRpm", min = 0, default = 6000, tunable)]
    #[serde(skip)]
    pub shift_up_rpm: f32,
    /// Drivetrain, automatic: the engine speed it shifts down at.
    #[reflect(name = "shiftDownRpm", min = 0, default = 2800, tunable)]
    #[serde(skip)]
    pub shift_down_rpm: f32,
    /// Drivetrain: how long a gear change takes, the engine driving nothing meanwhile.
    #[reflect(name = "clutchTime", min = 0, unit = "s", default = 0.25, tunable)]
    #[serde(skip)]
    pub clutch_time: f32,
    /// Drivetrain: the gear, -1 reverse, 0 neutral, 1 and up forward. Written by an automatic gearbox; set by the game for a manual one.
    #[reflect(min = -1, max = 8)]
    #[serde(skip)]
    pub gear: i32,
    /// Written, drivetrain: the engine's speed (rpm), for the engine sound and the tachometer.
    #[serde(skip)]
    pub rpm: f32,
}

impl Default for Vehicle {
    fn default() -> Self {
        Vehicle {
            throttle: 0.0,
            steer: 0.0,
            brake: 0.0,
            wheel_base: 2.6,
            track: 1.6,
            wheel_radius: 0.4,
            mount_height: 0.2,
            suspension_rest: 0.35,
            suspension_travel: 0.25,
            suspension_stiffness: 30.0,
            suspension_damping: 4.0,
            grip: 2.0,
            engine_force: 2500.0,
            max_speed: 40.0,
            brake_force: 2000.0,
            max_steer: 0.55,
            drive: DRIVE_REAR,
            wheel_mass: 20.0,
            roll_influence: 0.0,
            speed: 0.0,
            contacts: 0,
            slip: 0.0,
            model: VEHICLE_ARCADE,
            handbrake: 0.0,
            handbrake_grip: 0.2,
            drift_grip: 1.0,
            gears: List::new(),
            reverse_ratio: 3.2,
            final_drive: 3.6,
            idle_rpm: 900.0,
            max_rpm: 6800.0,
            torque_curve: List::new(),
            automatic: true,
            shift_up_rpm: 6000.0,
            shift_down_rpm: 2800.0,
            clutch_time: 0.25,
            gear: 0,
            rpm: 0.0,
        }
    }
}

/// A wheel of the og.Vehicle it is a child of (down its og.Parent chain),
/// at its og.Transform relative to the chassis. A vehicle with
/// og.VehicleWheel children (up to 16, in entity order) has those wheels in
/// place of its four implicit ones; 3D only.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.VehicleWheel")]
pub struct VehicleWheel {
    /// The wheel's radius.
    #[reflect(min = 0.01, unit = "m", default = 0.4, tunable)]
    pub radius: f32,
    /// The suspension's length at rest, down from the wheel's position.
    #[reflect(name = "suspensionRest", min = 0, unit = "m", default = 0.35, tunable)]
    pub suspension_rest: f32,
    /// How far the suspension compresses or extends from rest.
    #[reflect(
        name = "suspensionTravel",
        min = 0,
        unit = "m",
        default = 0.25,
        tunable
    )]
    pub suspension_travel: f32,
    /// The suspension's stiffness.
    #[reflect(name = "suspensionStiffness", min = 0, default = 30, tunable)]
    pub suspension_stiffness: f32,
    /// The suspension's damping.
    #[reflect(name = "suspensionDamping", min = 0, default = 4, tunable)]
    pub suspension_damping: f32,
    /// The tyre's grip, as og.Vehicle's `grip`.
    #[reflect(min = 0, default = 2, tunable)]
    pub grip: f32,
    /// How much of the vehicle's steering it takes: 1 a front wheel, 0 none, negative steers the other way (rear-wheel steering).
    #[reflect(min = -1, max = 1, tunable)]
    pub steer: f32,
    /// Whether the engine drives it.
    pub drive: bool,
    /// Whether the brake brakes it.
    #[reflect(default = 1)]
    pub brake: bool,
    /// Whether the handbrake brakes it (and drift grip applies to it).
    pub handbrake: bool,
}

impl Default for VehicleWheel {
    fn default() -> Self {
        VehicleWheel {
            radius: 0.4,
            suspension_rest: 0.35,
            suspension_travel: 0.25,
            suspension_stiffness: 30.0,
            suspension_damping: 4.0,
            grip: 2.0,
            steer: 0.0,
            drive: false,
            brake: true,
            handbrake: false,
        }
    }
}
