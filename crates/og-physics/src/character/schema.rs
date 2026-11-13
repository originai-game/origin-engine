//! The reflected types of the controllers (og.CharacterController,
//! og.PlatformerController, their events) and of ragdolls.

use og_core::Reflect;
use og_core::glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

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
    #[reflect(name = "snapDistance", min = 0, unit = "m", default = 0.5, tunable)]
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
    /// Push dynamic bodies it walks into (with at most pushStrength).
    #[reflect(name = "pushBodies", default = 1)]
    pub push_bodies: bool,
    /// The most force it pushes with: each tick the bodies it walks into
    /// share an impulse of at most this times the tick's length, so a body
    /// whose friction holds more (a heavy crate) does not move.
    #[reflect(name = "pushStrength", min = 0, unit = "N", default = 1000, tunable)]
    pub push_strength: f32,
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
            push_strength: DEFAULT_PUSH_STRENGTH,
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

/// The controllers' `pushStrength` default (N): a body of a few kilograms
/// is shoved at the controller's speed, one of a tonne on the ground holds.
pub const DEFAULT_PUSH_STRENGTH: f32 = 1000.0;

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
/// or capsule); it becomes a kinematic body and pushes the dynamic bodies
/// it runs into (`pushBodies`) with at most `pushStrength`, so a heavy one
/// holds where a light one is shoved.
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
    /// Push dynamic bodies it runs into (with at most pushStrength).
    #[reflect(name = "pushBodies", default = 1)]
    pub push_bodies: bool,
    /// The most force it pushes with: each tick the bodies it runs into
    /// share an impulse of at most this times the tick's length, so a body
    /// whose friction holds more (a heavy crate) does not move.
    #[reflect(name = "pushStrength", min = 0, unit = "N", default = 1000, tunable)]
    pub push_strength: f32,
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
            push_bodies: true,
            push_strength: DEFAULT_PUSH_STRENGTH,
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
