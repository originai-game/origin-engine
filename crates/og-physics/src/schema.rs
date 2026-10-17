//! The reflected types physics reads and writes. Every field here is what
//! the inspector, the probe, scene text and the TypeScript IDL show, so the
//! doc comments are the user-facing documentation.
//!
//! Enumerations are `u32` codes (the committed reflection has no enum field
//! type yet); the codes are the `pub const`s below and are stable.

use og_core::glam::{Quat, Vec3};
use og_core::Reflect;
use serde::{Deserialize, Serialize};

// ---- shared with the rest of the engine -------------------------------------

/// Where an entity is, relative to the world (physics bodies are roots).
///
/// The same schema as the engine's `og.Transform` (position, quaternion,
/// scale): a world that already registered it from the IDL adopts this type.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    /// The position, as three.js Object3D.position.
    #[reflect(unit = "m")]
    pub position: Vec3,
    /// The rotation, as three.js Object3D.quaternion.
    #[reflect(default = [0, 0, 0, 1])]
    pub quaternion: Quat,
    /// The scale, as three.js Object3D.scale. Physics scales collider sizes by it.
    #[reflect(default = [1, 1, 1])]
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// An unrotated, unscaled transform at `position`.
    pub fn at(position: Vec3) -> Transform {
        Transform {
            position,
            ..Transform::default()
        }
    }
}

// ---- components ---------------------------------------------------------------

/// `RigidBody.kind`: moved by the solver; its Transform and Velocity are written back every tick.
pub const BODY_DYNAMIC: u32 = 0;
/// `RigidBody.kind`: never moves; collides with dynamic bodies.
pub const BODY_STATIC: u32 = 1;
/// `RigidBody.kind`: moved by game code through its Transform; its Velocity (if any) is how fast it pushes.
pub const BODY_KINEMATIC: u32 = 2;

/// `RigidBody.lock` bits: no translation along x, y, z; no rotation about x, y, z.
pub const LOCK_TRANSLATION_X: u32 = 1;
pub const LOCK_TRANSLATION_Y: u32 = 2;
pub const LOCK_TRANSLATION_Z: u32 = 4;
pub const LOCK_ROTATION_X: u32 = 8;
pub const LOCK_ROTATION_Y: u32 = 16;
pub const LOCK_ROTATION_Z: u32 = 32;

/// A body the physics step simulates. Needs an `og.Transform`; usually has
/// an `og.Collider`. In the 2D profile the body moves in the xy plane and
/// rotates about z.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.RigidBody")]
pub struct RigidBody {
    /// 0 dynamic (simulated, written back), 1 static (never moves), 2 kinematic (moved by game code through its Transform).
    #[reflect(min = 0, max = 2, label = "Kind")]
    pub kind: u32,
    /// Mass in kilograms; 0 derives it from the collider's density and volume.
    #[reflect(min = 0, unit = "kg", tunable)]
    pub mass: f32,
    /// How fast linear velocity decays, per second (0 = none).
    #[reflect(min = 0, max = 100, unit = "1/s", tunable)]
    pub linear_damping: f32,
    /// How fast angular velocity decays, per second (0 = none).
    #[reflect(min = 0, max = 100, unit = "1/s", tunable)]
    pub angular_damping: f32,
    /// Multiplies og.Physics.gravity for this body (0 = floats, 1 = normal).
    #[reflect(min = -10, max = 10, default = 1, tunable)]
    pub gravity_scale: f32,
    /// Locked axes, a sum of bits: 1/2/4 no translation along x/y/z, 8/16/32 no rotation about x/y/z (2D uses 1, 2 and 32).
    #[reflect(min = 0, max = 63)]
    pub lock: u32,
    /// 1 enables continuous collision detection (fast, small bodies that must not tunnel); 0 off.
    #[reflect(min = 0, max = 1)]
    pub ccd: u32,
}

impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            kind: BODY_DYNAMIC,
            mass: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            lock: 0,
            ccd: 0,
        }
    }
}

impl RigidBody {
    pub fn dynamic() -> RigidBody {
        RigidBody::default()
    }
    pub fn fixed() -> RigidBody {
        RigidBody {
            kind: BODY_STATIC,
            ..RigidBody::default()
        }
    }
    pub fn kinematic() -> RigidBody {
        RigidBody {
            kind: BODY_KINEMATIC,
            ..RigidBody::default()
        }
    }
}

/// `Collider.shape`: a box of `size` (both profiles).
pub const SHAPE_BOX: u32 = 0;
/// `Collider.shape`: a sphere of `radius` (a circle in 2D).
pub const SHAPE_SPHERE: u32 = 1;
/// `Collider.shape`: a capsule along y: `radius`, `height` between the cap centres (both profiles).
pub const SHAPE_CAPSULE: u32 = 2;
/// `Collider.shape`: a cylinder along y: `radius`, `height` (3D only).
pub const SHAPE_CYLINDER: u32 = 3;
/// `Collider.shape`: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D).
pub const SHAPE_CONVEX: u32 = 4;
/// `Collider.shape`: the triangles of `mesh` (3D only; for static and kinematic bodies).
pub const SHAPE_TRIMESH: u32 = 5;

/// The shape an entity collides with, attached to its RigidBody (or, with
/// no RigidBody, a static collider at its Transform). Sizes are scaled by
/// og.Transform.scale.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Collider")]
pub struct Collider {
    /// 0 box, 1 sphere (circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (polygon in 2D), 5 triangle mesh of `mesh` (3D).
    #[reflect(min = 0, max = 5, label = "Shape")]
    pub shape: u32,
    /// Full extents of a box, as three.js BoxGeometry(width, height, depth).
    #[reflect(min = 0, unit = "m", default = [1, 1, 1], tunable)]
    pub size: Vec3,
    /// Radius of a sphere, circle, capsule or cylinder.
    #[reflect(min = 0, unit = "m", default = 0.5, tunable)]
    pub radius: f32,
    /// Capsule: distance between the cap centres (three.js CapsuleGeometry length); cylinder: its height.
    #[reflect(min = 0, unit = "m", default = 1, tunable)]
    pub height: f32,
    /// The mesh handle whose vertices a convex, polygon or triangle-mesh shape uses (registered with PhysicsSystem::set_mesh).
    pub mesh: u64,
    /// Position of the shape relative to the entity.
    #[reflect(unit = "m")]
    pub offset: Vec3,
    /// Rotation of the shape relative to the entity.
    #[reflect(default = [0, 0, 0, 1])]
    pub rotation: Quat,
    /// Coulomb friction (0 = ice, 1 = rubber); a pair uses the mean of both colliders.
    #[reflect(min = 0, max = 2, default = 0.5, tunable)]
    pub friction: f32,
    /// Bounciness (0 = no bounce, 1 = elastic); a pair uses the mean of both colliders.
    #[reflect(min = 0, max = 1, tunable)]
    pub restitution: f32,
    /// Density in kg per cubic metre (per square metre in 2D); ignored when RigidBody.mass is set.
    #[reflect(min = 0, unit = "kg/m3", default = 1, tunable)]
    pub density: f32,
    /// 1 makes this a sensor: it reports SensorEntered/SensorExited and pushes nothing; 0 a solid.
    #[reflect(min = 0, max = 1)]
    pub sensor: u32,
    /// The collision layers this collider is on (bits).
    #[reflect(default = 1)]
    pub layers: u32,
    /// The layers it collides with (bits). Two colliders interact when each one's layers meet the other's mask.
    #[reflect(default = 4294967295)]
    pub mask: u32,
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            shape: SHAPE_BOX,
            size: Vec3::ONE,
            radius: 0.5,
            height: 1.0,
            mesh: 0,
            offset: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            sensor: 0,
            layers: 1,
            mask: u32::MAX,
        }
    }
}

impl Collider {
    /// A box of full extents `size`.
    pub fn cuboid(size: Vec3) -> Collider {
        Collider {
            shape: SHAPE_BOX,
            size,
            ..Collider::default()
        }
    }
    /// A sphere (a circle in 2D).
    pub fn sphere(radius: f32) -> Collider {
        Collider {
            shape: SHAPE_SPHERE,
            radius,
            ..Collider::default()
        }
    }
    /// A capsule along y with `height` between the cap centres.
    pub fn capsule(radius: f32, height: f32) -> Collider {
        Collider {
            shape: SHAPE_CAPSULE,
            radius,
            height,
            ..Collider::default()
        }
    }
    /// A cylinder along y (3D only).
    pub fn cylinder(radius: f32, height: f32) -> Collider {
        Collider {
            shape: SHAPE_CYLINDER,
            radius,
            height,
            ..Collider::default()
        }
    }
    /// The convex hull of a registered mesh (a convex polygon in 2D).
    pub fn convex(mesh: u64) -> Collider {
        Collider {
            shape: SHAPE_CONVEX,
            mesh,
            ..Collider::default()
        }
    }
    /// The triangles of a registered mesh (3D only).
    pub fn trimesh(mesh: u64) -> Collider {
        Collider {
            shape: SHAPE_TRIMESH,
            mesh,
            ..Collider::default()
        }
    }
    /// The same collider as a sensor.
    pub fn as_sensor(self) -> Collider {
        Collider { sensor: 1, ..self }
    }
    /// The same collider on `layers`, colliding with `mask`.
    pub fn on_layers(self, layers: u32, mask: u32) -> Collider {
        Collider {
            layers,
            mask,
            ..self
        }
    }
}

/// Linear and angular velocity. Written back for dynamic bodies every tick
/// (added to them by the step when missing); read for kinematic ones. Set
/// it to launch or stop a body.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(name = "og.Velocity")]
pub struct Velocity {
    /// Metres per second (2D uses x and y).
    #[reflect(unit = "m/s", tunable)]
    pub linear: Vec3,
    /// Radians per second about each axis (2D uses z).
    #[reflect(unit = "rad/s", tunable)]
    pub angular: Vec3,
}

// ---- resource -------------------------------------------------------------

/// `Physics.profile`: rapier2d, bodies in the xy plane rotating about z.
pub const PROFILE_2D: u32 = 2;
/// `Physics.profile`: rapier3d.
pub const PROFILE_3D: u32 = 3;

/// The physics settings of a world. The profile is the project's
/// (`physics2d` or `physics3d`); the same components serve both.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Physics")]
pub struct Physics {
    /// 2 for the 2D profile (rapier2d: xy plane, rotation about z), 3 for 3D (rapier3d).
    #[reflect(min = 2, max = 3, default = 3, label = "Profile")]
    pub profile: u32,
    /// Acceleration of every dynamic body (times its gravity_scale); 2D uses x and y.
    #[reflect(unit = "m/s2", default = [0, -9.81, 0], tunable)]
    pub gravity: Vec3,
    /// Physics steps per tick, each of dt / substeps (more = stiffer stacks, less tunnelling, more cost).
    #[reflect(min = 1, max = 16, default = 1, tunable)]
    pub substeps: u32,
    /// Solver iterations per step (rapier's internal substeps).
    #[reflect(min = 1, max = 32, default = 4, tunable)]
    pub solver_iterations: u32,
    /// A dynamic body touching something whose speed stays below this before and after a step holds still (velocity zeroed, pose kept); 0 disables. Same number in m/s and rad/s.
    #[reflect(min = 0, max = 1, unit = "m/s", default = 0.02, tunable)]
    pub rest_speed: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Physics {
            profile: PROFILE_3D,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            substeps: 1,
            solver_iterations: 4,
            rest_speed: 0.02,
        }
    }
}

impl Physics {
    /// Default settings for the 2D profile.
    pub fn profile_2d() -> Physics {
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        }
    }
    /// Default settings for the 3D profile.
    pub fn profile_3d() -> Physics {
        Physics::default()
    }
}

// ---- events -----------------------------------------------------------------

/// Two solid colliders started touching during the tick (`a` < `b` by id).
/// Read during the next tick, like every event.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.CollisionStarted")]
pub struct CollisionStarted {
    /// The entity with the lower id.
    pub a: u64,
    /// The entity with the higher id.
    pub b: u64,
    /// The deepest contact point, in world space (z = 0 in 2D).
    #[reflect(unit = "m")]
    pub point: Vec3,
    /// The contact normal, pointing from `a` towards `b`.
    pub normal: Vec3,
}

/// Two solid colliders stopped touching (or one of them vanished).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.CollisionEnded")]
pub struct CollisionEnded {
    /// The entity with the lower id.
    pub a: u64,
    /// The entity with the higher id.
    pub b: u64,
}

/// A collider started overlapping a sensor.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SensorEntered")]
pub struct SensorEntered {
    /// The entity whose collider is the sensor.
    pub sensor: u64,
    /// The entity that entered it.
    pub other: u64,
}

/// A collider stopped overlapping a sensor (or one of them vanished).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.SensorExited")]
pub struct SensorExited {
    /// The entity whose collider is the sensor.
    pub sensor: u64,
    /// The entity that left it.
    pub other: u64,
}

/// One pair in contact at the end of a step, sent every tick for every
/// touching pair (`a` < `b`). The physics step reads the previous tick's
/// pairs back to derive the Started/Ended/Entered/Exited events, so this is
/// also how a game asks "what is touching what right now".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(name = "og.Contact")]
pub struct Contact {
    /// The entity with the lower id.
    pub a: u64,
    /// The entity with the higher id.
    pub b: u64,
    /// 0 for two solids touching; otherwise an overlap with a sensor: 1 `a` is the sensor, 2 `b` is, 3 both are.
    #[reflect(min = 0, max = 3)]
    pub sensor: u32,
}
