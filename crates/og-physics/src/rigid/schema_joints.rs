//! The reflected types of joints: og.Joint and the event of one breaking.

use og_core::Reflect;
use og_core::glam::Vec3;
use serde::{Deserialize, Serialize};

/// `Joint.kind`: the two bodies move as one.
pub const JOINT_FIXED: u32 = 0;
/// `Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor.
pub const JOINT_REVOLUTE: u32 = 1;
/// `Joint.kind`: a slider along `axis`, with optional limits and a motor.
pub const JOINT_PRISMATIC: u32 = 2;
/// `Joint.kind`: a damped spring pulling the anchors to `restLength` apart.
pub const JOINT_SPRING: u32 = 3;
/// `Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it.
pub const JOINT_ROPE: u32 = 4;
/// `Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge).
pub const JOINT_BALL: u32 = 5;

/// `Joint.motor`: no motor.
pub const MOTOR_NONE: u32 = 0;
/// `Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s).
pub const MOTOR_VELOCITY: u32 = 1;
/// `Joint.motor`: spring towards `motorTarget` angle or position (a servo).
pub const MOTOR_POSITION: u32 = 2;

/// A joint between two bodies (or a body and a point of the world). The
/// component may sit on one of the bodies (`body` 0) or on an entity of its
/// own, so one body can have several joints. Anchors are in each body's
/// frame (rotated and moved with it, not scaled); without `other` the
/// second anchor is a world point. A joint whose bodies are not both
/// `og.RigidBody` entities does nothing until they are.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Joint", version = 2)]
pub struct Joint {
    /// JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5.
    #[reflect(min = 0, max = 5, label = "Kind")]
    pub kind: u32,
    /// The first body; 0 is this entity.
    pub body: u64,
    /// The second body; 0 joins the first one to the world at `otherAnchor`.
    pub other: u64,
    /// Where the joint sits on the first body, in its frame.
    #[reflect(unit = "m")]
    pub anchor: Vec3,
    /// Where it sits on the second body, in its frame (a world point without `other`).
    #[reflect(name = "otherAnchor", unit = "m")]
    pub other_anchor: Vec3,
    /// The hinge or slide axis in the first body's frame (a hinge in 2D turns about z).
    #[reflect(default = [1, 0, 0])]
    pub axis: Vec3,
    /// Whether `min`..`max` bound the hinge angle (rad) or the slide (m).
    pub limits: bool,
    /// The lower limit: radians for a hinge, metres for a slider.
    pub min: f32,
    /// The upper limit.
    pub max: f32,
    /// MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget).
    #[reflect(min = 0, max = 2)]
    pub motor: u32,
    /// The motor's target speed (rad/s, m/s) or angle or position (rad, m).
    #[reflect(name = "motorTarget", tunable)]
    pub motor_target: f32,
    /// A position motor's stiffness; a velocity motor's strength.
    #[reflect(name = "motorStiffness", min = 0, default = 50, tunable)]
    pub motor_stiffness: f32,
    /// A position motor's damping.
    #[reflect(name = "motorDamping", min = 0, default = 5, tunable)]
    pub motor_damping: f32,
    /// The most force (or torque) the motor applies; 0 is unbounded.
    #[reflect(name = "motorMaxForce", min = 0, tunable)]
    pub motor_max_force: f32,
    /// A spring's rest length.
    #[reflect(name = "restLength", min = 0, unit = "m", default = 1, tunable)]
    pub rest_length: f32,
    /// A spring's stiffness (N/m).
    #[reflect(min = 0, default = 40, tunable)]
    pub stiffness: f32,
    /// A spring's damping (N·s/m).
    #[reflect(min = 0, default = 2, tunable)]
    pub damping: f32,
    /// A rope's longest distance between the anchors.
    #[reflect(min = 0, unit = "m", default = 2, tunable)]
    pub length: f32,
    /// Whether the two bodies still collide with each other.
    pub collide: bool,
    /// Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes.
    pub value: f32,
    /// The force (N) the joint holds before it breaks, from the solver's
    /// impulse over a step; 0 never breaks.
    #[reflect(name = "breakForce", min = 0, unit = "N", tunable)]
    #[serde(skip)]
    pub break_force: f32,
    /// The torque (N·m) it holds before it breaks; 0 never breaks.
    #[reflect(name = "breakTorque", min = 0, unit = "N*m", tunable)]
    #[serde(skip)]
    pub break_torque: f32,
    /// Written: true once the joint broke (og.JointBroken); a broken joint
    /// holds nothing. Write false to mend it, rebuilt where its bodies are.
    #[serde(skip)]
    pub broken: bool,
}

impl Default for Joint {
    fn default() -> Self {
        Joint {
            kind: JOINT_FIXED,
            body: 0,
            other: 0,
            anchor: Vec3::ZERO,
            other_anchor: Vec3::ZERO,
            axis: Vec3::X,
            limits: false,
            min: 0.0,
            max: 0.0,
            motor: MOTOR_NONE,
            motor_target: 0.0,
            motor_stiffness: 50.0,
            motor_damping: 5.0,
            motor_max_force: 0.0,
            rest_length: 1.0,
            stiffness: 40.0,
            damping: 2.0,
            length: 2.0,
            collide: false,
            value: 0.0,
            break_force: 0.0,
            break_torque: 0.0,
            broken: false,
        }
    }
}

impl Joint {
    /// A joint of `kind` from this entity's body to `other` (0: the world).
    pub fn new(kind: u32, other: u64) -> Joint {
        Joint {
            kind,
            other,
            ..Joint::default()
        }
    }
}

/// A joint broke during the tick: the solver's force or torque on it passed
/// its breakForce or breakTorque. Sent in joint entity order; the joint's
/// `broken` is set.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.JointBroken")]
pub struct JointBroken {
    /// The og.Joint's entity.
    pub joint: u64,
    /// Its first body.
    pub body: u64,
    /// Its second body (0: the world).
    pub other: u64,
    /// The force it carried when it broke.
    #[reflect(unit = "N")]
    pub force: f32,
    /// The torque it carried when it broke.
    #[reflect(unit = "N*m")]
    pub torque: f32,
}
