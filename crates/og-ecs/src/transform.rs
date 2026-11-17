//! The components every engine module shares: where an entity is
//! (`og.Transform`), what it is relative to (`og.Parent`) and how it moves
//! (`og.Velocity`). One Rust type for each, so physics, animation, audio
//! and navigation read and write the same rows through the same type; a
//! world that registered the schema from the IDL adopts it.

use og_core::Reflect;
use og_core::glam::{Mat4, Quat, Vec3};
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
    /// The authored local matrix, before any parent frame.
    pub fn matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.quaternion, self.position)
    }

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

/// The simulation world matrix written after posing an attachment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.MatrixWorld")]
pub struct MatrixWorld {
    /// The composed world matrix.
    pub matrix: Mat4,
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

impl crate::World {
    /// Installs the optional module that supplies posed parent frames.
    /// Frames are derived from the module's snapshot state, never saved twice.
    pub fn set_parent_frames(
        &mut self,
        source: impl Fn(&Self) -> og_core::Result<og_core::IdMap<u64, Mat4>> + Send + Sync + 'static,
    ) {
        self.parent_frames = Some(Box::new(source));
    }

    /// Posed model-space frames between a parent and its child's local
    /// transform. Empty when no module supplies them.
    pub fn parent_frames(&self) -> og_core::Result<og_core::IdMap<u64, Mat4>> {
        match &self.parent_frames {
            Some(source) => source(self),
            None => Ok(og_core::IdMap::new()),
        }
    }

    /// The simulation matrix, including optional posed parent frames.
    pub fn matrix_world(&self, entity: crate::Entity) -> og_core::Result<Mat4> {
        let frames = self.parent_frames()?;
        Ok(world_matrix(
            entity,
            &frames,
            |e| {
                self.get::<Parent>(e)
                    .ok()
                    .map(|p| crate::Entity::from_bits(p.entity))
            },
            |e| self.get::<Transform>(e).ok(),
        ))
    }
}

/// Whether this entity's chain passes through a posed frame.
pub fn under_parent_frame(
    entity: crate::Entity,
    frames: &og_core::IdMap<u64, Mat4>,
    parent_of: impl Fn(crate::Entity) -> Option<crate::Entity>,
) -> bool {
    if frames.is_empty() {
        return false;
    }
    let mut cur = entity;
    for _ in 0..=256 {
        if frames.contains_key(&cur.to_bits()) {
            return true;
        }
        match parent_of(cur) {
            Some(parent) => cur = parent,
            None => break,
        }
    }
    false
}

/// Compose a chain in root-to-leaf order, inserting a posed frame before
/// each authored local transform. Shared by simulation consumers.
pub fn world_matrix(
    entity: crate::Entity,
    frames: &og_core::IdMap<u64, Mat4>,
    parent_of: impl Fn(crate::Entity) -> Option<crate::Entity>,
    transform_of: impl Fn(crate::Entity) -> Option<Transform>,
) -> Mat4 {
    let mut chain = Vec::new();
    let mut cur = entity;
    for _ in 0..=256 {
        if chain.contains(&cur) {
            break;
        }
        chain.push(cur);
        match parent_of(cur) {
            Some(p) => cur = p,
            None => break,
        }
    }
    let mut matrix = Mat4::IDENTITY;
    for e in chain.into_iter().rev() {
        if let Some(frame) = frames.get(&e.to_bits()) {
            matrix *= *frame;
        }
        if let Some(t) = transform_of(e) {
            matrix *= t.matrix();
        }
    }
    matrix
}
