//! The force fields of a world (og.ForceField): an acceleration on what is
//! inside a region, as one function of a point. Everything the tick moves
//! asks [`Forces::at`], so bodies, ropes, cloths, fluids and characters
//! cannot disagree about where a field pulls.
//!
//! Additions, a square root and a division: the same on every host.

use crate::volume::{Placement, Volume};
use og_core::glam::Vec3;

/// How a field accelerates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// Along its direction.
    #[default]
    Directional,
    /// Away from its centre (towards it with a negative strength).
    Radial,
    /// Around its direction through its centre, counter-clockwise seen
    /// against the direction.
    Vortex,
}

/// One force field (og.ForceField), in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    /// Where its entity puts it.
    pub at: Placement,
    /// Its region (og.Volume); `None`: everywhere.
    pub volume: Option<Volume>,
    pub kind: FieldKind,
    /// A directional field's direction and a vortex's axis (unit, world).
    pub direction: Vec3,
    /// Its acceleration inside the region (m/s²).
    pub strength: f32,
}

impl Field {
    /// Its acceleration at `p` (m/s²): its strength its kind's way, times
    /// the region's weight there.
    pub fn at(&self, p: Vec3) -> Vec3 {
        if !self.strength.is_finite() {
            return Vec3::ZERO;
        }
        let weight = crate::volume::weight(self.volume.as_ref(), &self.at, p);
        if weight <= 0.0 {
            return Vec3::ZERO;
        }
        let from = p - self.at.position;
        let dir = match self.kind {
            FieldKind::Directional => self.direction,
            FieldKind::Radial => from.normalize_or_zero(),
            FieldKind::Vortex => {
                let out = from - self.direction * self.direction.dot(from);
                self.direction.cross(out.normalize_or_zero())
            }
        };
        dir * (self.strength * weight)
    }
}

/// The force fields of a world this tick, in entity order. None by default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Forces {
    pub fields: Vec<Field>,
}

impl Forces {
    /// Whether nothing is accelerated anywhere.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// The acceleration at `p` (m/s²): every field's there, in their order.
    pub fn at(&self, p: Vec3) -> Vec3 {
        self.fields.iter().fold(Vec3::ZERO, |a, f| a + f.at(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(kind: FieldKind) -> Field {
        Field {
            at: Placement::at(Vec3::new(10.0, 0.0, 0.0)),
            volume: Some(Volume {
                size: Vec3::ONE,
                radius: 4.0,
                fade: 2.0,
            }),
            kind,
            direction: Vec3::Y,
            strength: 6.0,
        }
    }

    /// Each kind accelerates its way inside the region: along the
    /// direction, away from the centre, around the axis.
    #[test]
    fn a_field_accelerates_its_way_inside_its_volume() {
        let p = Vec3::new(12.0, 5.0, 0.0);
        // Outside the sphere (5.4 m from the centre, 1.4 m past it, of a
        // 2 m fade): 0.3 of the strength is left.
        let w = 1.0 - ((29.0f32).sqrt() - 4.0) / 2.0;
        let up = field(FieldKind::Directional).at(p);
        assert!((up - Vec3::Y * 6.0 * w).length() < 1e-5, "{up}");
        let inside = Vec3::new(12.0, 1.0, 0.0);
        assert_eq!(field(FieldKind::Directional).at(inside), Vec3::Y * 6.0);
        let out = field(FieldKind::Radial).at(Vec3::new(13.0, 0.0, 0.0));
        assert_eq!(out, Vec3::X * 6.0);
        // Counter-clockwise seen from above (against +y): at +x it goes
        // to -z, whatever the height.
        let round = field(FieldKind::Vortex).at(inside);
        assert!((round - Vec3::NEG_Z * 6.0).length() < 1e-5, "{round}");
        assert_eq!(
            field(FieldKind::Radial).at(Vec3::new(20.0, 0.0, 0.0)),
            Vec3::ZERO
        );
    }

    /// A negative strength pulls in; without a volume a field is
    /// everywhere; fields add.
    #[test]
    fn fields_pull_reach_everywhere_without_a_volume_and_add() {
        let well = Field {
            strength: -9.0,
            volume: None,
            ..field(FieldKind::Radial)
        };
        assert_eq!(well.at(Vec3::new(1010.0, 0.0, 0.0)), Vec3::NEG_X * 9.0);
        let forces = Forces {
            fields: vec![well, field(FieldKind::Directional)],
        };
        assert_eq!(
            forces.at(Vec3::new(11.0, 0.0, 0.0)),
            Vec3::new(-9.0, 6.0, 0.0)
        );
        assert!(Forces::default().is_empty());
        assert_eq!(Forces::default().at(Vec3::ONE), Vec3::ZERO);
    }
}
