//! Reflected vehicle configuration. Forces are SI; grip is a friction coefficient.
use og_core::glam::Vec3;
use og_core::{List, Reflect};

pub const DRIVE_REAR: u32 = 0;
pub const DRIVE_FRONT: u32 = 1;
pub const DRIVE_ALL: u32 = 2;
pub const VEHICLE_ARCADE: u32 = 0;
pub const VEHICLE_DRIVETRAIN: u32 = 1;

/// A dynamic chassis with independent wheels, forward +z in 3D and +x in 2D.
/// Suspension stiffness and damping are per chassis kg; drive and brake are
/// wheel torques in N·m. Game code draws wheels using vehicleWheels poses.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Vehicle", version = 4)]
pub struct Vehicle {
    #[reflect(min = -1, max = 1)]
    pub throttle: f32,
    #[reflect(min = -1, max = 1)]
    pub steer: f32,
    #[reflect(min = 0, max = 1)]
    pub brake: f32,
    #[reflect(name = "wheelBase", min = 0, unit = "m", default = 2.6, tunable)]
    pub wheel_base: f32,
    #[reflect(min = 0, unit = "m", default = 1.6, tunable)]
    pub track: f32,
    #[reflect(name = "wheelRadius", min = 0.01, unit = "m", default = 0.4, tunable)]
    pub wheel_radius: f32,
    #[reflect(name = "mountHeight", unit = "m", default = 0.2, tunable)]
    pub mount_height: f32,
    #[reflect(name = "suspensionRest", min = 0, unit = "m", default = 0.35, tunable)]
    pub suspension_rest: f32,
    #[reflect(
        name = "suspensionTravel",
        min = 0,
        unit = "m",
        default = 0.25,
        tunable
    )]
    pub suspension_travel: f32,
    #[reflect(name = "suspensionStiffness", min = 0, default = 30, tunable)]
    pub suspension_stiffness: f32,
    #[reflect(name = "compressionDamping", min = 0, tunable)]
    pub compression_damping: f32,
    #[reflect(name = "reboundDamping", min = 0, tunable)]
    pub rebound_damping: f32,
    #[reflect(name = "bumpStopStiffness", min = 0, tunable)]
    pub bump_stop_stiffness: f32,
    #[reflect(name = "antiRollStiffness", min = 0, tunable)]
    pub anti_roll_stiffness: f32,
    #[reflect(min = 0, default = 1, tunable)]
    pub grip: f32,
    /// Arcade torque per driven wheel, N·m.
    #[reflect(name = "engineForce", min = 0, default = 2500, tunable)]
    pub engine_force: f32,
    #[reflect(name = "maxSpeed", min = 0, unit = "mps", default = 40, tunable)]
    pub max_speed: f32,
    /// Brake torque per wheel before brakeBias, N·m.
    #[reflect(name = "brakeForce", min = 0, default = 2000, tunable)]
    pub brake_force: f32,
    #[reflect(
        name = "maxSteer",
        min = 0,
        max = 1.2,
        unit = "rad",
        default = 0.55,
        tunable
    )]
    pub max_steer: f32,
    #[reflect(variants = ["rear", "front", "all"])]
    pub drive: u32,
    #[reflect(name = "wheelMass", min = 0.01, unit = "kg", default = 20, tunable)]
    pub wheel_mass: f32,
    #[reflect(name = "rollInfluence", min = 0, max = 1, tunable)]
    pub roll_influence: f32,
    #[reflect(name = "wheelCast", variants = ["ray", "sphere", "cylinder"])]
    pub wheel_cast: u32,
    /// Zero disables bias (equal wheel torque); otherwise front axle share.
    #[reflect(name = "brakeBias", min = 0, max = 1, tunable)]
    pub brake_bias: f32,
    #[reflect(name = "aeroDrag", min = 0, tunable)]
    pub aero_drag: f32,
    #[reflect(name = "aeroDownforce", min = 0, tunable)]
    pub aero_downforce: f32,
    #[reflect(name = "rollingResistance", min = 0, tunable)]
    pub rolling_resistance: f32,
    #[reflect(min = 0, max = 1, tunable)]
    pub ackermann: f32,
    #[reflect(name = "steeringFade", min = 0, tunable)]
    pub steering_fade: f32,
    #[reflect(min = 0, max = 1, tunable)]
    pub abs: f32,
    #[reflect(min = 0, max = 1, tunable)]
    pub tcs: f32,
    #[reflect(unit = "mps")]
    pub speed: f32,
    pub contacts: u32,
    #[reflect(min = 0, max = 1)]
    pub slip: f32,
    #[reflect(variants = ["arcade", "drivetrain"])]
    pub model: u32,
    #[reflect(min = 0, max = 1)]
    pub handbrake: f32,
    #[reflect(name = "handbrakeGrip", min = 0, max = 1, default = 0.2, tunable)]
    pub handbrake_grip: f32,
    pub gears: List<f32, 8>,
    #[reflect(name = "reverseRatio", min = 0, default = 3.2, tunable)]
    pub reverse_ratio: f32,
    #[reflect(name = "finalDrive", min = 0, default = 3.6, tunable)]
    pub final_drive: f32,
    #[reflect(name = "idleRpm", min = 0, unit = "rpm", default = 900, tunable)]
    pub idle_rpm: f32,
    #[reflect(name = "maxRpm", min = 1, unit = "rpm", default = 6800, tunable)]
    pub max_rpm: f32,
    #[reflect(name = "torqueCurve")]
    pub torque_curve: List<f32, 8>,
    #[reflect(default = 1)]
    pub automatic: bool,
    #[reflect(name = "shiftUpRpm", min = 0, unit = "rpm", default = 6000, tunable)]
    pub shift_up_rpm: f32,
    #[reflect(name = "shiftDownRpm", min = 0, unit = "rpm", default = 2800, tunable)]
    pub shift_down_rpm: f32,
    #[reflect(name = "clutchTime", min = 0, unit = "s", default = 0.25, tunable)]
    pub clutch_time: f32,
    #[reflect(name = "engineInertia", min = 0, tunable)]
    pub engine_inertia: f32,
    #[reflect(name = "clutchStrength", min = 0, tunable)]
    pub clutch_strength: f32,
    #[reflect(name = "engineBraking", min = 0, tunable)]
    pub engine_braking: f32,
    #[reflect(variants = ["open", "lsd", "locked"])]
    pub differential: u32,
    #[reflect(name = "differentialStrength", min = 0, tunable)]
    pub differential_strength: f32,
    /// Zero disables axle bias; otherwise the front axle's drive share.
    #[reflect(name = "awdBias", min = 0, max = 1, tunable)]
    pub awd_bias: f32,
    #[reflect(min = -1, max = 8)]
    pub gear: i32,
    #[reflect(unit = "rpm")]
    pub rpm: f32,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
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
            compression_damping: 0.0,
            rebound_damping: 0.0,
            bump_stop_stiffness: 0.0,
            anti_roll_stiffness: 0.0,
            grip: 1.0,
            engine_force: 2500.0,
            max_speed: 40.0,
            brake_force: 2000.0,
            brake_bias: 0.0,
            max_steer: 0.55,
            drive: DRIVE_REAR,
            wheel_mass: 20.0,
            roll_influence: 0.0,
            wheel_cast: 0,
            aero_drag: 0.0,
            aero_downforce: 0.0,
            rolling_resistance: 0.0,
            ackermann: 0.0,
            steering_fade: 0.0,
            abs: 0.0,
            tcs: 0.0,
            speed: 0.0,
            contacts: 0,
            slip: 0.0,
            model: VEHICLE_ARCADE,
            handbrake: 0.0,
            handbrake_grip: 0.2,
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
            engine_inertia: 0.0,
            clutch_strength: 0.0,
            engine_braking: 0.0,
            differential: 0,
            differential_strength: 0.0,
            awd_bias: 0.0,
            gear: 0,
            rpm: 0.0,
        }
    }
}

/// An explicit wheel of its nearest Vehicle ancestor, in either profile.
/// Mount is chassis-local and never follows the visual wheel Transform.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.VehicleWheel")]
pub struct VehicleWheel {
    #[reflect(unit = "m")]
    pub mount: Vec3,
    #[reflect(min = 0.01, unit = "m", default = 0.4, tunable)]
    pub radius: f32,
    #[reflect(name = "suspensionRest", min = 0, unit = "m", default = 0.35, tunable)]
    pub suspension_rest: f32,
    #[reflect(
        name = "suspensionTravel",
        min = 0,
        unit = "m",
        default = 0.25,
        tunable
    )]
    pub suspension_travel: f32,
    #[reflect(name = "suspensionStiffness", min = 0, default = 30, tunable)]
    pub suspension_stiffness: f32,
    #[reflect(name = "compressionDamping", min = 0, tunable)]
    pub compression_damping: f32,
    #[reflect(name = "reboundDamping", min = 0, tunable)]
    pub rebound_damping: f32,
    #[reflect(min = 0, default = 1, tunable)]
    pub grip: f32,
    #[reflect(min = -1, max = 1, tunable)]
    pub steer: f32,
    pub drive: bool,
    #[reflect(default = 1)]
    pub brake: bool,
    pub handbrake: bool,
}

impl Default for VehicleWheel {
    fn default() -> Self {
        Self {
            mount: Vec3::ZERO,
            radius: 0.4,
            suspension_rest: 0.35,
            suspension_travel: 0.25,
            suspension_stiffness: 30.0,
            compression_damping: 0.0,
            rebound_damping: 0.0,
            grip: 1.0,
            steer: 0.0,
            drive: false,
            brake: true,
            handbrake: false,
        }
    }
}
