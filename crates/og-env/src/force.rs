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
