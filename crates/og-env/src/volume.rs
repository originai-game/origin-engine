//! The one shape of a region of the world: a box along its entity's axes
//! or a sphere about it, with a fade past its surface. A zone that changes
//! what is inside it (og.ReverbZone, og.FogVolume) reads its weight here,
//! so every region is placed, turned, scaled and faded the same way.
//!
//! Subtractions, a square root and a quaternion product: the same on
//! every host, so the tick may branch on it.

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

/// A region's shape. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    /// The box's full size along the entity's axes, centred on it (times
    /// its scale).
    pub size: Vec3,
    /// A sphere of this radius about the entity instead of the box (times
    /// the largest of its scale); 0 uses the box.
    pub radius: f32,
    /// How far past the surface the region fades out, linearly (m); 0 is a
    /// hard edge.
    pub fade: f32,
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
}
