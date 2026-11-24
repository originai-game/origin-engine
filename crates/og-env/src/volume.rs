//! The one shape of a region of the world (og.Volume): a box along its
//! entity's axes or a sphere about it, with a fade past its surface. A
//! region component on the same entity (og.ReverbZone, og.StreamCell,
//! og.ForceField, og.FogVolume, og.ReflectionProbe, og.LightVolume)
//! takes its extent from it, so every region is placed, turned, scaled
//! and faded the same way, and one entity may be several regions of one
//! shape.
//!
//! Subtractions, a square root and a quaternion product: the same on
//! every host, so the tick may branch on it.

use og_core::Reflect;
use og_core::glam::{Mat3, Quat, Vec3};

/// Where a region's entity puts it in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
}

impl Placement {
    /// At `position`, unturned and unscaled.
    pub fn at(position: Vec3) -> Placement {
        Placement {
            position,
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// A region's shape (og.Volume). See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Volume")]
pub struct Volume {
    /// The box's full size along the entity's axes, centred on it (times
    /// og.Transform.scale).
    #[reflect(default = [1, 1, 1], unit = "m")]
    pub size: Vec3,
    /// A sphere of this radius about the entity instead of the box (times
    /// the largest of og.Transform.scale); 0 uses the box.
    #[reflect(min = 0, unit = "m", tunable)]
    pub radius: f32,
    /// How far past the surface the region fades out, linearly; 0 is a
    /// hard edge.
    #[reflect(min = 0, unit = "m", tunable)]
    pub fade: f32,
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            size: Vec3::ONE,
            radius: 0.0,
            fade: 0.0,
        }
    }
}

/// How much of a region reaches `p`, its entity at `at`: a region without
/// an og.Volume has no bounds, so it is whole everywhere.
pub fn weight(volume: Option<&Volume>, at: &Placement, p: Vec3) -> f32 {
    volume.map_or(1.0, |v| v.weight(at, p))
}

impl Volume {
    /// How far `p` lies outside the region placed at `at` (m): 0 inside
    /// and on its surface.
    pub fn outside(&self, at: &Placement, p: Vec3) -> f32 {
        let scale = at.scale.abs();
        if self.radius > 0.0 {
            ((p - at.position).length() - self.radius * scale.max_element()).max(0.0)
        } else {
            let local = at.quaternion.normalize().inverse() * (p - at.position);
            let half = self.size.abs() * scale * 0.5;
            (local.abs() - half).max(Vec3::ZERO).length()
        }
    }

    /// The box along the world's axes that holds the region placed at `at`
    /// (its fade not counted): what a region read along the world's axes
    /// covers (a reflection probe's box projection, a light volume's grid).
    pub fn bounds(&self, at: &Placement) -> (Vec3, Vec3) {
        let half = self.half_extent(at);
        (at.position - half, at.position + half)
    }

    /// Half the size of [`Volume::bounds`]' box along each world axis: the
    /// same wherever the region stands (its position is not read).
    pub fn half_extent(&self, at: &Placement) -> Vec3 {
        let scale = at.scale.abs();
        if self.radius > 0.0 {
            Vec3::splat(self.radius * scale.max_element())
        } else {
            let h = self.size.abs() * scale * 0.5;
            let m = Mat3::from_quat(at.quaternion.normalize());
            m.x_axis.abs() * h.x + m.y_axis.abs() * h.y + m.z_axis.abs() * h.z
        }
    }

    /// How much of the region reaches `p`: 1 inside, falling linearly to 0
    /// at `fade` metres from its surface (0 outside a hard edge).
    pub fn weight(&self, at: &Placement, p: Vec3) -> f32 {
        let outside = self.outside(at, p);
        if outside <= 0.0 {
            1.0
        } else if self.fade > 0.0 {
            (1.0 - outside / self.fade).max(0.0)
        } else {
            0.0
        }
    }
}
