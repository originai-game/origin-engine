//! The one shape of a region of the world (og.Volume): a box along its
//! entity's axes or a sphere about it, with a fade past its surface. A
//! region component on the same entity (og.ReverbZone, og.StreamCell)
//! takes its extent from it, so every region is placed, turned, scaled
//! and faded the same way, and one entity may be several regions of one
//! shape.
//!
//! Subtractions, a square root and a quaternion product: the same on
//! every host, so the tick may branch on it.

use og_core::Reflect;
use og_core::glam::{Quat, Vec3};

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A box follows its entity: turned a quarter about y, its long side
    /// lies along the world's z, and its scale stretches it.
    #[test]
    fn a_box_follows_its_entity() {
        let v = Volume {
            size: Vec3::new(10.0, 2.0, 2.0),
            radius: 0.0,
            fade: 2.0,
        };
        let mut at = Placement::at(Vec3::new(100.0, 0.0, 0.0));
        assert_eq!(v.weight(&at, Vec3::new(104.0, 0.0, 0.0)), 1.0);
        assert_eq!(v.weight(&at, Vec3::new(100.0, 0.0, 4.0)), 0.0);
        at.quaternion = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        assert_eq!(v.weight(&at, Vec3::new(100.0, 0.0, 4.0)), 1.0);
        assert!(v.weight(&at, Vec3::new(104.0, 0.0, 0.0)) < 1e-5);
        at.scale = Vec3::new(2.0, 1.0, 1.0);
        assert_eq!(v.weight(&at, Vec3::new(100.0, 0.0, 9.0)), 1.0);
    }

    /// Past the surface the weight falls linearly over the fade; a hard
    /// edge has none; a radius makes it a sphere.
    #[test]
    fn the_fade_is_linear_from_the_surface() {
        let at = Placement::at(Vec3::ZERO);
        let ball = Volume {
            size: Vec3::splat(100.0),
            radius: 3.0,
            fade: 2.0,
        };
        assert_eq!(ball.outside(&at, Vec3::new(0.0, 4.0, 0.0)), 1.0);
        assert_eq!(ball.weight(&at, Vec3::new(0.0, 4.0, 0.0)), 0.5);
        assert_eq!(ball.weight(&at, Vec3::new(0.0, 5.0, 0.0)), 0.0);
        let hard = Volume { fade: 0.0, ..ball };
        assert_eq!(hard.weight(&at, Vec3::new(0.0, 3.0, 0.0)), 1.0);
        assert_eq!(hard.weight(&at, Vec3::new(0.0, 3.01, 0.0)), 0.0);
    }

    /// A region without a volume reaches everywhere; with one, only it.
    #[test]
    fn a_region_without_a_volume_has_no_bounds() {
        let at = Placement::at(Vec3::ZERO);
        let far = Vec3::new(1e6, 0.0, 0.0);
        assert_eq!(weight(None, &at, far), 1.0);
        assert_eq!(weight(Some(&Volume::default()), &at, far), 0.0);
        assert_eq!(weight(Some(&Volume::default()), &at, Vec3::splat(0.4)), 1.0);
    }
}
