//! The reflected types of physics beyond rigid bodies (the E5 round):
//! joints, the character and platformer controllers, vehicles, Verlet
//! ropes and cloth, the 2D particle fluid, height-field and 2D water, and
//! ragdolls. As in `schema.rs`, field order, names and storage types are the
//! layout the IDL declares, enumerations are `u32` codes with `pub const`s,
//! and the doc comments are the user-facing documentation. Fields a step
//! writes are marked "written".

use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

// ---- joints ---------------------------------------------------------------

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
#[reflect(name = "og.Joint")]
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

// ---- controllers ----------------------------------------------------------

/// A kinematic character (3D, or top-down 2D): the entity's capsule (or any
/// og.Collider) moved by `move` each tick, sliding along walls, climbing
/// slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the
/// ground, carried by the moving platform it stands on and falling by
/// gravity. It keeps `skin` off every surface, standing that far over the
/// ground; a step is climbed when the ground beyond it is at most
/// `stepHeight` over the ground it stands on. What blocks it is every solid
/// collider its og.Collider's `layers` and `mask` interact with (each one's
/// layers meeting the other's mask), as for bodies. The entity needs an
/// og.Transform and an og.Collider; the step makes it a kinematic body and
/// pushes dynamic bodies it walks into.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.CharacterController")]
pub struct CharacterController {
    /// The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0). Set it every tick.
    #[reflect(name = "move", unit = "m/s")]
    pub movement: Vec3,
    /// Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time).
    pub jump: bool,
    /// The upward speed of a jump.
    #[reflect(name = "jumpSpeed", min = 0, unit = "m/s", default = 6, tunable)]
    pub jump_speed: f32,
    /// Multiplies og.Physics.gravity; 0 for top-down movement in any direction.
    #[reflect(name = "gravityScale", min = 0, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// The steepest slope it walks up.
    #[reflect(
        name = "maxSlope",
        min = 0,
        max = 89,
        unit = "deg",
        default = 45,
        tunable
    )]
    pub max_slope: f32,
    /// The highest step it climbs without jumping (0: none).
    #[reflect(name = "stepHeight", min = 0, unit = "m", default = 0.3, tunable)]
    pub step_height: f32,
    /// How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge.
    #[reflect(name = "snapDistance", min = 0, unit = "m", default = 0.2, tunable)]
    pub snap_distance: f32,
    /// The gap kept between it and every surface.
    #[reflect(min = 0.001, unit = "m", default = 0.02)]
    pub skin: f32,
    /// After leaving an edge, a jump still starts within this time.
    #[reflect(name = "coyoteTime", min = 0, unit = "s", default = 0.1, tunable)]
    pub coyote_time: f32,
    /// The fastest fall.
    #[reflect(name = "maxFallSpeed", min = 0, unit = "m/s", default = 40, tunable)]
    pub max_fall_speed: f32,
    /// Push dynamic bodies it walks into.
    #[reflect(name = "pushBodies", default = 1)]
    pub push_bodies: bool,
    /// Written: standing on walkable ground after the step.
    pub grounded: bool,
    /// Written: sliding down a slope steeper than maxSlope.
    pub sliding: bool,
    /// Written: the velocity it moved at this tick (the platform's carry included).
    #[reflect(unit = "m/s")]
    pub velocity: Vec3,
    /// Written: the ground's normal (zero in the air).
    #[reflect(name = "groundNormal")]
    pub ground_normal: Vec3,
    /// Written: seconds left in which a jump may still start (coyote time).
    pub coyote: f32,
    /// Written: `jump` as the last step saw it (a jump starts on its rising edge).
    #[reflect(name = "jumpWasHeld")]
    pub jump_was_held: bool,
    /// Written: the entity it stands on (0 in the air); a moving one carries it.
    pub ground: u64,
}

impl Default for CharacterController {
    fn default() -> Self {
        CharacterController {
            movement: Vec3::ZERO,
            jump: false,
            jump_speed: 6.0,
            gravity_scale: 1.0,
            max_slope: 45.0,
            step_height: 0.3,
            snap_distance: 0.2,
            skin: 0.02,
            coyote_time: 0.1,
            max_fall_speed: 40.0,
            push_bodies: true,
            grounded: false,
            sliding: false,
            velocity: Vec3::ZERO,
            ground_normal: Vec3::ZERO,
            coyote: 0.0,
            jump_was_held: false,
            ground: 0,
        }
    }
}

/// A 2D platformer character (the 2D profile): run, jump and fall with
/// designer units (jump height and time to apex, not gravity), coyote time,
/// jump buffering, variable jump height (releasing jump while rising cuts
/// the jump), apex hang, air jumps, wall slide and wall jump, one-way
/// platforms (colliders on `oneWayLayers` are solid from above only; hold
/// `down` to drop through), moving platforms (it rides what it stands on)
/// and corner correction (a jump that clips a ceiling corner by less than
/// `cornerCorrection` slides past it). It keeps `skin` off every surface;
/// what blocks it is every solid collider its og.Collider's `layers` and
/// `mask` interact with (each one's layers meeting the other's mask), as
/// for bodies. The entity needs an og.Transform and an og.Collider (a box
/// or capsule); it becomes a kinematic body.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.PlatformerController")]
pub struct PlatformerController {
    /// The run input, -1 (left) to 1 (right). Set it every tick.
    #[reflect(name = "moveX", min = -1, max = 1)]
    pub move_x: f32,
    /// Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump.
    pub jump: bool,
    /// Held to drop through a one-way platform.
    pub down: bool,
    /// A velocity change to take on the next step (a knock-back, a spring, a
    /// recoil): added to `velocity`, then set back to zero. One with an
    /// upward y leaves the ground; the run's acceleration and gravity act
    /// on the result from then on.
    #[reflect(unit = "m/s")]
    pub impulse: Vec2,
    /// The top running speed.
    #[reflect(name = "maxSpeed", min = 0, unit = "m/s", default = 8, tunable)]
    pub max_speed: f32,
    /// How fast it reaches maxSpeed on the ground.
    #[reflect(min = 0, unit = "m/s2", default = 70, tunable)]
    pub acceleration: f32,
    /// How fast it stops on the ground without input.
    #[reflect(min = 0, unit = "m/s2", default = 80, tunable)]
    pub deceleration: f32,
    /// Acceleration in the air.
    #[reflect(
        name = "airAcceleration",
        min = 0,
        unit = "m/s2",
        default = 45,
        tunable
    )]
    pub air_acceleration: f32,
    /// Deceleration in the air without input.
    #[reflect(
        name = "airDeceleration",
        min = 0,
        unit = "m/s2",
        default = 25,
        tunable
    )]
    pub air_deceleration: f32,
    /// How high a full jump goes.
    #[reflect(name = "jumpHeight", min = 0, unit = "m", default = 3.2, tunable)]
    pub jump_height: f32,
    /// How long a full jump takes to reach its height (sets gravity and the jump speed).
    #[reflect(name = "timeToApex", min = 0.05, unit = "s", default = 0.4, tunable)]
    pub time_to_apex: f32,
    /// Gravity multiplier while falling (snappier landings above 1).
    #[reflect(name = "fallGravity", min = 0, default = 1.8, tunable)]
    pub fall_gravity: f32,
    /// Gravity multiplier while rising with jump released (variable jump height; 1 disables it).
    #[reflect(name = "jumpCut", min = 1, default = 2.5, tunable)]
    pub jump_cut: f32,
    /// Gravity multiplier near the apex while jump is held (hang time; 1 disables it).
    #[reflect(name = "apexGravity", min = 0, max = 1, default = 0.6, tunable)]
    pub apex_gravity: f32,
    /// The vertical speed below which the apex multiplier applies.
    #[reflect(name = "apexThreshold", min = 0, unit = "m/s", default = 1.5, tunable)]
    pub apex_threshold: f32,
    /// The fastest fall.
    #[reflect(name = "maxFallSpeed", min = 0, unit = "m/s", default = 20, tunable)]
    pub max_fall_speed: f32,
    /// After running off an edge, a jump still starts within this time.
    #[reflect(name = "coyoteTime", min = 0, unit = "s", default = 0.1, tunable)]
    pub coyote_time: f32,
    /// A jump pressed this long before landing starts on landing.
    #[reflect(name = "jumpBuffer", min = 0, unit = "s", default = 0.12, tunable)]
    pub jump_buffer: f32,
    /// Extra jumps in the air (1: a double jump).
    #[reflect(name = "airJumps", min = 0, max = 8)]
    pub air_jumps: u32,
    /// Slide down walls it pushes against (and jump off them).
    #[reflect(name = "wallJumps", default = 1)]
    pub wall_jumps: bool,
    /// The fastest slide down a wall.
    #[reflect(name = "wallSlideSpeed", min = 0, unit = "m/s", default = 3, tunable)]
    pub wall_slide_speed: f32,
    /// A wall jump's velocity: x away from the wall, y up.
    #[reflect(name = "wallJump", unit = "m/s", default = [8, 12], tunable)]
    pub wall_jump: Vec2,
    /// After a wall jump, the run input is ignored for this long (so it leaves the wall).
    #[reflect(name = "wallJumpLock", min = 0, unit = "s", default = 0.14, tunable)]
    pub wall_jump_lock: f32,
    /// The collision layers that are one-way platforms (solid from above), as bits.
    #[reflect(name = "oneWayLayers")]
    pub one_way_layers: u32,
    /// How far a rising jump is nudged sideways past a ceiling corner (0: never).
    #[reflect(
        name = "cornerCorrection",
        min = 0,
        unit = "m",
        default = 0.15,
        tunable
    )]
    pub corner_correction: f32,
    /// The steepest slope it runs up.
    #[reflect(
        name = "maxSlope",
        min = 0,
        max = 89,
        unit = "deg",
        default = 50,
        tunable
    )]
    pub max_slope: f32,
    /// How far below it the ground still holds it (slopes, small drops).
    #[reflect(name = "snapDistance", min = 0, unit = "m", default = 0.1, tunable)]
    pub snap_distance: f32,
    /// The gap kept between it and every surface.
    #[reflect(min = 0.001, unit = "m", default = 0.01)]
    pub skin: f32,
    /// Written: standing on ground after the step.
    pub grounded: bool,
    /// Written: -1 sliding on a wall to its left, 1 to its right, 0 none.
    #[reflect(name = "onWall", min = -1, max = 1)]
    pub on_wall: i32,
    /// Written: -1 facing left, 1 right (the last run direction).
    #[reflect(min = -1, max = 1, default = 1)]
    pub facing: i32,
    /// Written: the velocity after the step (the platform's carry excluded).
    #[reflect(unit = "m/s")]
    pub velocity: Vec3,
    /// Written: rising from a jump (until the apex or a release cut).
    pub jumping: bool,
    /// Written: air jumps left before landing.
    #[reflect(name = "airJumpsLeft")]
    pub air_jumps_left: u32,
    /// Written: seconds of coyote time left.
    pub coyote: f32,
    /// Written: seconds a buffered jump press stays valid.
    pub buffer: f32,
    /// Written: seconds the run input stays locked after a wall jump.
    #[reflect(name = "wallLock")]
    pub wall_lock: f32,
    /// Written: seconds one-way platforms stay passable after dropping through.
    #[reflect(name = "dropTime")]
    pub drop_time: f32,
    /// Written: `jump` as the last step saw it.
    #[reflect(name = "jumpWasHeld")]
    pub jump_was_held: bool,
    /// Written: the entity it stands on (0 in the air); a moving one carries it.
    pub ground: u64,
}

impl Default for PlatformerController {
    fn default() -> Self {
        PlatformerController {
            move_x: 0.0,
            jump: false,
            down: false,
            impulse: Vec2::ZERO,
            max_speed: 8.0,
            acceleration: 70.0,
            deceleration: 80.0,
            air_acceleration: 45.0,
            air_deceleration: 25.0,
            jump_height: 3.2,
            time_to_apex: 0.4,
            fall_gravity: 1.8,
            jump_cut: 2.5,
            apex_gravity: 0.6,
            apex_threshold: 1.5,
            max_fall_speed: 20.0,
            coyote_time: 0.1,
            jump_buffer: 0.12,
            air_jumps: 0,
            wall_jumps: true,
            wall_slide_speed: 3.0,
            wall_jump: Vec2::new(8.0, 12.0),
            wall_jump_lock: 0.14,
            one_way_layers: 0,
            corner_correction: 0.15,
            max_slope: 50.0,
            snap_distance: 0.1,
            skin: 0.01,
            grounded: false,
            on_wall: 0,
            facing: 1,
            velocity: Vec3::ZERO,
            jumping: false,
            air_jumps_left: 0,
            coyote: 0.0,
            buffer: 0.0,
            wall_lock: 0.0,
            drop_time: 0.0,
            jump_was_held: false,
            ground: 0,
        }
    }
}

/// `CharacterEvent.kind`: a jump from the ground (or in coyote time).
pub const CHARACTER_JUMPED: u32 = 1;
/// `CharacterEvent.kind`: touched the ground after being in the air.
pub const CHARACTER_LANDED: u32 = 2;
/// `CharacterEvent.kind`: a jump off a wall.
pub const CHARACTER_WALL_JUMPED: u32 = 3;
/// `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps).
pub const CHARACTER_AIR_JUMPED: u32 = 4;
/// `CharacterEvent.kind`: left the ground without jumping (ran off an edge).
pub const CHARACTER_LEFT_GROUND: u32 = 5;
/// `CharacterEvent.kind`: hit a ceiling while rising.
pub const CHARACTER_BUMPED: u32 = 6;

/// Something a CharacterController or PlatformerController did this step:
/// the hooks for dust, squash, sounds and camera kicks. Read the next tick.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.CharacterEvent")]
pub struct CharacterEvent {
    /// The character.
    pub entity: u64,
    /// CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6.
    #[reflect(min = 1, max = 6)]
    pub kind: u32,
    /// The vertical speed at the moment (a landing's impact speed is negative).
    #[reflect(unit = "m/s")]
    pub speed: f32,
}

// ---- vehicles -------------------------------------------------------------

/// `Vehicle.drive`: the rear wheels drive.
pub const DRIVE_REAR: u32 = 0;
/// `Vehicle.drive`: the front wheels drive.
pub const DRIVE_FRONT: u32 = 1;
/// `Vehicle.drive`: every wheel drives.
pub const DRIVE_ALL: u32 = 2;

/// A car on a dynamic chassis (the entity's og.RigidBody and og.Collider).
/// 3D: a raycast car with four wheels (suspension, grip, steering on the
/// front axle); its forward is the chassis's local +z (three.js
/// Object3D.lookAt aims +z). 2D: two
/// wheel bodies on sprung wheel joints (front at +x), driven by a motor.
/// The wheels are not entities: og.physics.vehicleWheels gives their poses
/// for drawing.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Vehicle")]
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
    #[reflect(name = "mountHeight", unit = "m", default = 0.2, tunable)]
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
    /// The tyres' grip (3D: the friction slip; 2D: the wheels' friction).
    #[reflect(min = 0, default = 2, tunable)]
    pub grip: f32,
    /// The engine's force at full throttle (3D: per driven wheel, N; 2D: the motor's torque).
    #[reflect(name = "engineForce", min = 0, default = 80, tunable)]
    pub engine_force: f32,
    /// The top speed: the engine's force fades out towards it (throttle against the motion brakes first).
    #[reflect(name = "maxSpeed", min = 0, unit = "m/s", default = 20, tunable)]
    pub max_speed: f32,
    /// The brakes' force at full brake.
    #[reflect(name = "brakeForce", min = 0, default = 30, tunable)]
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
    #[reflect(name = "wheelMass", min = 0.01, unit = "kg", default = 1, tunable)]
    pub wheel_mass: f32,
    /// Written: the forward speed (m/s; negative reversing).
    #[reflect(unit = "m/s")]
    pub speed: f32,
    /// Written: how many wheels touch the ground.
    pub contacts: u32,
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
            engine_force: 80.0,
            max_speed: 20.0,
            brake_force: 30.0,
            max_steer: 0.55,
            drive: DRIVE_REAR,
            wheel_mass: 1.0,
            speed: 0.0,
            contacts: 0,
        }
    }
}

// ---- Verlet ropes and cloth -------------------------------------------------

/// A Verlet rope: `segments` links from the entity's world position (its
/// og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung
/// by gravity and wind, colliding with every solid collider at `radius`.
/// It moves nothing (one-way: a rope joint ties bodies together, og.Joint
/// JOINT_ROPE). Its points are world state (the og.physics state block);
/// og.physics.ropePoints reads them and the renderer draws the rope as a
/// strip of `width` and `color`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Rope")]
pub struct Rope {
    /// The number of links (points minus one).
    #[reflect(min = 1, max = 256, default = 16)]
    pub segments: u32,
    /// The rope's length at rest.
    #[reflect(min = 0, unit = "m", default = 4, tunable)]
    pub length: f32,
    /// The start's offset from the entity's position (world axes).
    #[reflect(unit = "m")]
    pub offset: Vec3,
    /// The entity the far end is tied to; 0 leaves it free.
    pub end: u64,
    /// The far end's offset from `end`'s position.
    #[reflect(name = "endOffset", unit = "m")]
    pub end_offset: Vec3,
    /// Constraint passes per step: more is stiffer, at more cost.
    #[reflect(min = 1, max = 64, default = 12, tunable)]
    pub iterations: u32,
    /// How much of its velocity each point keeps per tick (1: none lost).
    #[reflect(min = 0, max = 1, default = 0.99, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// A steady acceleration (wind).
    #[reflect(unit = "m/s2", tunable)]
    pub wind: Vec3,
    /// The thickness it collides at (0: no collision).
    #[reflect(min = 0, unit = "m", default = 0.05, tunable)]
    pub radius: f32,
    /// The drawn width.
    #[reflect(min = 0, unit = "m", default = 0.08)]
    pub width: f32,
    /// The drawn linear colour.
    #[reflect(default = [0.55, 0.42, 0.28])]
    pub color: Vec3,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Rope {
    fn default() -> Self {
        Rope {
            segments: 16,
            length: 4.0,
            offset: Vec3::ZERO,
            end: 0,
            end_offset: Vec3::ZERO,
            iterations: 12,
            damping: 0.99,
            gravity_scale: 1.0,
            wind: Vec3::ZERO,
            radius: 0.05,
            width: 0.08,
            color: Vec3::new(0.55, 0.42, 0.28),
            visible: true,
        }
    }
}

/// `Cloth.pins` bit: the whole top row is pinned.
pub const PIN_TOP: u32 = 1;
/// `Cloth.pins` bit: the top-left and top-right corners are pinned.
pub const PIN_TOP_CORNERS: u32 = 2;
/// `Cloth.pins` bit: the whole left column is pinned (a flag on a pole).
pub const PIN_LEFT: u32 = 4;
/// `Cloth.pins` bit: every fourth point of the top row (curtain rings).
pub const PIN_TOP_RINGS: u32 = 8;

/// A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart,
/// hanging from the entity's world position (its og.Transform after its
/// parents': its top-left corner, the grid laid out along the entity's
/// local x and down -y), pinned by `pins` (pinned
/// points follow the entity), with stretch and shear constraints, gravity,
/// wind, collision against every solid collider and, with `thickness`,
/// against itself. One-way like og.Rope;
/// its points are world state. Drawn as a double-sided mesh with
/// `material` (og.render.standardMaterial; 0: a plain cloth colour).
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Cloth")]
pub struct Cloth {
    /// Points across.
    #[reflect(min = 2, max = 64, default = 12)]
    pub columns: u32,
    /// Points down.
    #[reflect(min = 2, max = 64, default = 12)]
    pub rows: u32,
    /// The rest distance between neighbouring points.
    #[reflect(min = 0.001, unit = "m", default = 0.15, tunable)]
    pub spacing: f32,
    /// Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings).
    #[reflect(default = 1)]
    pub pins: u32,
    /// Constraint passes per step.
    #[reflect(min = 1, max = 64, default = 8, tunable)]
    pub iterations: u32,
    /// How much of its velocity each point keeps per tick.
    #[reflect(min = 0, max = 1, default = 0.985, tunable)]
    pub damping: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// A steady acceleration (wind); the cloth flutters with it.
    #[reflect(unit = "m/s2", tunable)]
    pub wind: Vec3,
    /// How much the wind varies over the cloth and the tick (0: steady).
    #[reflect(min = 0, max = 2, default = 0.6, tunable)]
    pub gust: f32,
    /// The thickness it collides at (0: no collision).
    #[reflect(min = 0, unit = "m", default = 0.04, tunable)]
    pub radius: f32,
    /// Self-collision: how far apart its points keep from the points of the
    /// cloth that are not their near neighbours (0: none). About `spacing`
    /// keeps a cloth that folds or piles up from passing through itself.
    #[reflect(min = 0, unit = "m", tunable)]
    pub thickness: f32,
    /// Simulation steps per tick, each integrating and then solving
    /// `iterations` passes; with self-collision enough that a point moves
    /// less than `thickness` per step (a point moves at most half the
    /// thickness a step).
    #[reflect(min = 1, max = 16, default = 1)]
    pub substeps: u32,
    /// How much of its sliding a point touching a solid loses each step
    /// (0: it slides freely; near 1 it sticks where it lands, so a cloth
    /// lowered onto the floor folds instead of spreading).
    #[reflect(min = 0, max = 1, tunable)]
    pub friction: f32,
    /// The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth.
    pub material: u64,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Cloth {
    fn default() -> Self {
        Cloth {
            columns: 12,
            rows: 12,
            spacing: 0.15,
            pins: PIN_TOP,
            iterations: 8,
            damping: 0.985,
            gravity_scale: 1.0,
            wind: Vec3::ZERO,
            gust: 0.6,
            radius: 0.04,
            thickness: 0.0,
            substeps: 1,
            friction: 0.0,
            material: 0,
            visible: true,
        }
    }
}

// ---- fluids and water -----------------------------------------------------

/// A 2D particle fluid (any profile, in the xy plane): `count` particles
/// spawned in a `size` box centred on the entity, simulated with double
/// density relaxation (Clavet et al. 2005: pressure, near pressure,
/// viscosity) on a fixed grid in a fixed order, so it is deterministic and
/// part of the og.physics state block. Particles collide with every solid
/// collider (pushed out of bodies; they push nothing back) and stay inside
/// `bounds` around the entity when it is set. Fluids of one `group`
/// interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the
/// particles blurred into a field the renderer thresholds into one surface
/// of `color`, on sprite layer `layer`.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Fluid2D")]
pub struct Fluid2D {
    /// How many particles (spawned when the component appears or count changes).
    #[reflect(min = 1, max = 8192, default = 1000)]
    pub count: u32,
    /// The spawn box, centred on the entity.
    #[reflect(unit = "m", default = [4, 3])]
    pub size: Vec2,
    /// The container around the entity (0 0: none; colliders still hold it).
    #[reflect(unit = "m")]
    pub bounds: Vec2,
    /// The interaction radius: particles closer than this push and drag each other.
    #[reflect(min = 0.01, unit = "m", default = 0.18, tunable)]
    pub radius: f32,
    /// The density it settles at (neighbours within radius, weighted).
    #[reflect(name = "restDensity", min = 0, default = 1, tunable)]
    pub rest_density: f32,
    /// The pressure's strength (above about 0.15 it gets springy, then unstable).
    #[reflect(min = 0, default = 0.1, tunable)]
    pub stiffness: f32,
    /// The near pressure's strength (keeps particles apart, gives surface tension).
    #[reflect(name = "nearStiffness", min = 0, default = 0.5, tunable)]
    pub near_stiffness: f32,
    /// Linear viscosity (honey above 1).
    #[reflect(min = 0, max = 4, default = 0.15, tunable)]
    pub viscosity: f32,
    /// Fluids sharing a group above 0 push, drag and (by `mixing`) mix with
    /// each other as one fluid, at most four to a group (in entity order;
    /// a fifth is simulated alone), and are drawn as one surface whose
    /// colour follows the mix; 0 keeps the fluid to itself.
    pub group: u32,
    /// How heavy its particles are against the group's other fluids:
    /// a heavier fluid sinks under a lighter one.
    #[reflect(min = 0.05, max = 20, default = 1, tunable)]
    pub density: f32,
    /// How readily it mixes with the group's other fluids: 0 keeps them
    /// apart in layers (the boundary held), 1 blends them where they touch
    /// (their mix, colour and weight even out).
    #[reflect(min = 0, max = 1, tunable)]
    pub mixing: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Simulation steps per tick.
    #[reflect(min = 1, max = 8, default = 2)]
    pub substeps: u32,
    /// The seed of the spawn jitter.
    pub seed: u32,
    /// The surface's linear colour.
    #[reflect(default = [0.15, 0.45, 0.95])]
    pub color: Vec3,
    /// The surface's opacity.
    #[reflect(min = 0, max = 1, default = 0.9)]
    pub opacity: f32,
    /// The drawn size of each particle's blob (times radius).
    #[reflect(name = "blobScale", min = 0.1, max = 4, default = 1.6)]
    pub blob_scale: f32,
    /// The field value where the surface starts (higher: thinner, separate drops).
    #[reflect(min = 0.01, max = 4, default = 0.45)]
    pub threshold: f32,
    /// The sprite layer it is drawn on.
    pub layer: i32,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Fluid2D {
    fn default() -> Self {
        Fluid2D {
            count: 1000,
            size: Vec2::new(4.0, 3.0),
            bounds: Vec2::ZERO,
            radius: 0.18,
            rest_density: 1.0,
            stiffness: 0.1,
            near_stiffness: 0.5,
            viscosity: 0.15,
            group: 0,
            density: 1.0,
            mixing: 0.0,
            gravity_scale: 1.0,
            substeps: 2,
            seed: 0,
            color: Vec3::new(0.15, 0.45, 0.95),
            opacity: 0.9,
            blob_scale: 1.6,
            threshold: 0.45,
            layer: 0,
            visible: true,
        }
    }
}

/// A 3D particle fluid (the 3D profile): `count` particles spawned in a
/// `size` box centred on the entity, simulated with double density
/// relaxation (Clavet et al. 2005: pressure, near pressure that gives
/// surface tension, viscosity) over a spatial hash in a fixed order, so it
/// is deterministic and part of the og.physics state block. Particles
/// collide with every solid collider, the terrain's height field included
/// (pushed out; they push nothing back), and stay inside `bounds` around
/// the entity when it is set. An emitter adds particles at `emitRate` up
/// to `maxCount`; a sink box removes those that enter it. Fluids of one
/// `group` interact: by `density` they layer, by `mixing` they blend.
/// Drawn as a screen-space surface: the particles' depth smoothed into one
/// surface, refracting what is behind by its thickness and reflecting the
/// sky, lit by the scene's lights.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Fluid3D")]
pub struct Fluid3D {
    /// How many particles to spawn in the box (when the component appears
    /// or count or seed changes; 0: none, the emitter fills it).
    #[reflect(max = 32768, default = 2000)]
    pub count: u32,
    /// The spawn box, centred on the entity.
    #[reflect(unit = "m", default = [2, 1, 2])]
    pub size: Vec3,
    /// The container around the entity (0 on an axis: open on that axis;
    /// colliders and the terrain still hold it).
    #[reflect(unit = "m")]
    pub bounds: Vec3,
    /// The interaction radius: particles closer than this push and drag
    /// each other. Cost grows with the particles, not the radius.
    #[reflect(min = 0.02, unit = "m", default = 0.2, tunable)]
    pub radius: f32,
    /// The density it settles at (neighbours within radius, weighted).
    #[reflect(name = "restDensity", min = 0, default = 10, tunable)]
    pub rest_density: f32,
    /// The pressure's strength (above about 0.05 it jitters, then springs apart).
    #[reflect(min = 0, default = 0.02, tunable)]
    pub stiffness: f32,
    /// The near pressure's strength (keeps particles apart, gives surface tension).
    #[reflect(name = "nearStiffness", min = 0, default = 0.4, tunable)]
    pub near_stiffness: f32,
    /// Linear viscosity (it also calms the surface; honey above 1).
    #[reflect(min = 0, max = 4, default = 0.4, tunable)]
    pub viscosity: f32,
    /// Fluids sharing a group above 0 push, drag and (by `mixing`) mix with
    /// each other as one fluid, at most four to a group (in entity order;
    /// a fifth is simulated alone); 0 keeps the fluid to itself.
    pub group: u32,
    /// How heavy its particles are against the group's other fluids: a
    /// heavier fluid sinks under a lighter one.
    #[reflect(min = 0.05, max = 20, default = 1, tunable)]
    pub density: f32,
    /// How readily it mixes with the group's other fluids: 0 keeps them
    /// apart in layers (the boundary held), 1 blends them where they touch
    /// (their mix, colour and weight even out).
    #[reflect(min = 0, max = 1, tunable)]
    pub mixing: f32,
    /// Multiplies og.Physics.gravity.
    #[reflect(name = "gravityScale", min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Simulation steps per tick (each costs a full pass over the particles).
    #[reflect(min = 1, max = 8, default = 2)]
    pub substeps: u32,
    /// The seed of the spawn and emitter jitter.
    pub seed: u32,
    /// The most particles alive at once (the emitter stops there).
    #[reflect(name = "maxCount", min = 1, max = 32768, default = 8192)]
    pub max_count: u32,
    /// Particles the emitter adds per second (0: no emitter).
    #[reflect(name = "emitRate", min = 0, max = 20000, tunable)]
    pub emit_rate: f32,
    /// Where the emitter is, from the entity (world axes).
    #[reflect(name = "emitOffset", unit = "m")]
    pub emit_offset: Vec3,
    /// The velocity emitted particles start with.
    #[reflect(name = "emitVelocity", unit = "m/s", tunable)]
    pub emit_velocity: Vec3,
    /// The half-size of the cube new particles appear in around the emitter.
    #[reflect(name = "emitRadius", min = 0, unit = "m", default = 0.1, tunable)]
    pub emit_radius: f32,
    /// The sink's centre, from the entity (world axes).
    #[reflect(name = "sinkOffset", unit = "m")]
    pub sink_offset: Vec3,
    /// The sink box: particles that enter it are removed (0 0 0: no sink).
    #[reflect(name = "sinkSize", unit = "m")]
    pub sink_size: Vec3,
    /// The fluid's linear colour (what light passing through it keeps).
    #[reflect(default = [0.2, 0.55, 0.9])]
    pub color: Vec3,
    /// How opaque a metre of it is (0: clear as glass, 1: opaque).
    #[reflect(min = 0, max = 1, default = 0.6)]
    pub opacity: f32,
    /// How much it reflects (scales the Fresnel term).
    #[reflect(min = 0, max = 2, default = 1)]
    pub reflectivity: f32,
    /// How far refraction bends what is seen through it.
    #[reflect(min = 0, max = 4, default = 1)]
    pub refraction: f32,
    /// The drawn size of each particle (times radius).
    #[reflect(name = "drawScale", min = 0.1, max = 2, default = 0.6)]
    pub draw_scale: f32,
    /// How much the surface is smoothed (0: beads, 1: one surface).
    #[reflect(min = 0, max = 2, default = 1)]
    pub smoothing: f32,
    /// Whether the renderer draws it.
    #[reflect(default = 1)]
    pub visible: bool,
}

impl Default for Fluid3D {
    fn default() -> Self {
        Fluid3D {
            count: 2000,
            size: Vec3::new(2.0, 1.0, 2.0),
            bounds: Vec3::ZERO,
            radius: 0.2,
            rest_density: 10.0,
            stiffness: 0.02,
            near_stiffness: 0.4,
            viscosity: 0.4,
            group: 0,
            density: 1.0,
            mixing: 0.0,
            gravity_scale: 1.0,
            substeps: 2,
            seed: 0,
            max_count: 8192,
            emit_rate: 0.0,
            emit_offset: Vec3::ZERO,
            emit_velocity: Vec3::ZERO,
            emit_radius: 0.1,
            sink_offset: Vec3::ZERO,
            sink_size: Vec3::ZERO,
            color: Vec3::new(0.2, 0.55, 0.9),
            opacity: 0.6,
            reflectivity: 1.0,
            refraction: 1.0,
            draw_scale: 0.6,
            smoothing: 1.0,
            visible: true,
        }
    }
}

/// Height-field water (3D): a `resolution` × `resolution` grid of heights
/// over a `size` square centred on the entity (its y is the rest level),
/// waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies
/// crossing it push waves (splashes; og.Splash events) and float by
/// `buoyancy`. The heights are world state; an og.Water on the same entity
/// draws them.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.WaterSurface")]
pub struct WaterSurface {
    /// Grid points along each side.
    #[reflect(min = 8, max = 256, default = 64)]
    pub resolution: u32,
    /// The side of the square of water.
    #[reflect(min = 0.1, unit = "m", default = 16, tunable)]
    pub size: f32,
    /// How fast waves travel.
    #[reflect(name = "waveSpeed", min = 0, unit = "m/s", default = 3, tunable)]
    pub wave_speed: f32,
    /// How much of a wave survives a second (1: no loss).
    #[reflect(min = 0, max = 1, default = 0.4, tunable)]
    pub damping: f32,
    /// How strongly bodies push waves.
    #[reflect(min = 0, default = 1, tunable)]
    pub splash: f32,
    /// The upward force on a submerged body per cubic metre of its box under water (0: bodies sink).
    #[reflect(min = 0, unit = "N/m3", default = 9.81, tunable)]
    pub buoyancy: f32,
    /// How much a submerged body's velocity is slowed.
    #[reflect(min = 0, unit = "1/s", default = 1.5, tunable)]
    pub drag: f32,
}

impl Default for WaterSurface {
    fn default() -> Self {
        WaterSurface {
            resolution: 64,
            size: 16.0,
            wave_speed: 3.0,
            damping: 0.4,
            splash: 1.0,
            buoyancy: 9.81,
            drag: 1.5,
        }
    }
}

/// 2D water for side-scrollers: a `width` × `depth` pool whose top edge
/// (at the entity's y, centred on its x) is `columns` springs, each pulled
/// to its rest height by `tension`, damped by `damping` and passing waves to
/// its neighbours by `spread`. Bodies (and characters) crossing the surface
/// splash it (og.Splash events) and float by `buoyancy`. The springs are
/// world state; an og.Water2D on the same entity draws it.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.WaterLine")]
pub struct WaterLine {
    /// The pool's width.
    #[reflect(min = 0.1, unit = "m", default = 12, tunable)]
    pub width: f32,
    /// The pool's depth below the surface.
    #[reflect(min = 0, unit = "m", default = 3, tunable)]
    pub depth: f32,
    /// Springs along the surface.
    #[reflect(min = 4, max = 512, default = 96)]
    pub columns: u32,
    /// How hard each spring pulls back to rest.
    #[reflect(min = 0, default = 60, tunable)]
    pub tension: f32,
    /// How fast a spring's motion dies.
    #[reflect(min = 0, default = 2.5, tunable)]
    pub damping: f32,
    /// How much of a spring's height its neighbours feel each pass (waves travel).
    #[reflect(min = 0, max = 0.5, default = 0.25, tunable)]
    pub spread: f32,
    /// How strongly crossing bodies splash.
    #[reflect(min = 0, default = 0.04, tunable)]
    pub splash: f32,
    /// The upward force on a submerged body per square metre of its box under water (0: bodies sink).
    #[reflect(min = 0, unit = "N/m2", default = 12, tunable)]
    pub buoyancy: f32,
    /// How much a submerged body's velocity is slowed.
    #[reflect(min = 0, unit = "1/s", default = 2, tunable)]
    pub drag: f32,
}

impl Default for WaterLine {
    fn default() -> Self {
        WaterLine {
            width: 12.0,
            depth: 3.0,
            columns: 96,
            tension: 60.0,
            damping: 2.5,
            spread: 0.25,
            splash: 0.04,
            buoyancy: 12.0,
            drag: 2.0,
        }
    }
}

/// Something crossed a water surface (og.WaterSurface or og.WaterLine):
/// for splash particles and sounds. Read the next tick.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Splash")]
pub struct Splash {
    /// The water's entity.
    pub water: u64,
    /// What crossed it.
    pub entity: u64,
    /// Where it crossed the surface.
    #[reflect(unit = "m")]
    pub point: Vec3,
    /// Its vertical speed (negative going in).
    #[reflect(unit = "m/s")]
    pub speed: f32,
}

// ---- ragdolls -------------------------------------------------------------

/// A ragdoll over an og.Skeleton: a capsule body per bone with a parent,
/// joints with limits between them (ball joints with cone limits in 3D; in
/// 2D hinges about z, the bones rapier2d bodies in the xy plane at their
/// animated z, for a skeleton laid out in that plane), built from the
/// skeleton's rest pose. `weight` is where the pose comes from: 0 the
/// animation (the bodies follow it kinematically), 1 the physics (the
/// bodies are dynamic, starting with the animation's velocity); the pose
/// drawn blends between them over `blendTime`, both ways. The bodies are
/// world state (the og.physics state block), not entities.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Ragdoll")]
pub struct Ragdoll {
    /// 0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover.
    #[reflect(min = 0, max = 1, tunable)]
    pub weight: f32,
    /// How long the blend between animation and physics takes, both ways.
    #[reflect(name = "blendTime", min = 0, unit = "s", default = 0.25, tunable)]
    pub blend_time: f32,
    /// The whole body's mass, shared by the bones by length.
    #[reflect(min = 0.1, unit = "kg", default = 60, tunable)]
    pub mass: f32,
    /// A bone capsule's radius as a fraction of its length.
    #[reflect(min = 0.01, max = 1, default = 0.22, tunable)]
    pub thickness: f32,
    /// How far each joint bends from the rest pose.
    #[reflect(
        name = "jointLimit",
        min = 0,
        max = 3.1,
        unit = "rad",
        default = 0.9,
        tunable
    )]
    pub joint_limit: f32,
    /// How much of their motion the bones lose per second.
    #[reflect(min = 0, max = 20, unit = "1/s", default = 0.6, tunable)]
    pub damping: f32,
    /// An impulse given to every bone when weight first goes up (a hit).
    #[reflect(unit = "N*s")]
    pub impulse: Vec3,
    /// Written: the blend in use (0 animation .. 1 physics).
    pub blend: f32,
}

impl Default for Ragdoll {
    fn default() -> Self {
        Ragdoll {
            weight: 0.0,
            blend_time: 0.25,
            mass: 60.0,
            thickness: 0.22,
            joint_limit: 0.9,
            damping: 0.6,
            impulse: Vec3::ZERO,
            blend: 0.0,
        }
    }
}
