//! The components every engine module shares: where an entity is
//! (`og.Transform`), what it is relative to (`og.Parent`) and how it moves
//! (`og.Velocity`). One Rust type for each, so physics, animation, audio
//! and navigation read and write the same rows through the same type; a
//! world that registered the schema from the IDL adopts it.

use og_core::Reflect;
use og_core::glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Where an entity is, relative to its parent (or the world).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    /// The position, as three.js Object3D.position.
    #[reflect(unit = "m")]
    pub position: Vec3,
    /// The rotation, as three.js Object3D.quaternion.
    #[reflect(default = [0, 0, 0, 1])]
    pub quaternion: Quat,
    /// The scale, as three.js Object3D.scale.
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

/// The entity this one's Transform is relative to. Set by SpawnDesc.parent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.Parent")]
pub struct Parent {
    /// The parent (an entity's bits).
    pub entity: u64,
}

/// Linear and angular velocity. Physics writes it back for dynamic bodies
/// every tick (and adds it to them when missing) and moves a kinematic
/// body by it; navigation steers an agent by it. Set it to launch or stop
/// a body.
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
