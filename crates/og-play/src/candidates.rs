//! What is around a point: every given entity within a range and a cone,
//! with its distance, its angle off a forward direction and whether it is
//! in sight, nearest first. Facts only: which one a game picks is its own
//! rule.

use og_core::glam::Vec3;
use og_core::math;

/// One entity in range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub entity: u64,
    pub distance: f32,
    /// Radians between the forward direction and the direction to it (0
    /// to π); 0 for one at the point itself.
    pub angle: f32,
    /// Whether nothing stands between (true when sight was not asked).
    pub visible: bool,
}

/// Where to look from and how far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Around {
    pub from: Vec3,
    /// A unit direction.
    pub forward: Vec3,
    /// How far; 0 finds nothing.
    pub range: f32,
    /// The cone's half angle in radians; 0 (or π and more) takes every
    /// direction.
    pub cone: f32,
}

/// The entities of `points` within `around`, nearest first (equal
/// distances in entity order). `sight` answers whether one at a point is
/// in sight; `None` takes every one as visible.
pub fn candidates(
    around: &Around,
    points: &[(u64, Vec3)],
    sight: Option<&dyn Fn(u64, Vec3) -> bool>,
) -> Vec<Candidate> {
    let cone = around.cone > 0.0 && around.cone < std::f32::consts::PI;
    let mut out: Vec<Candidate> = points
        .iter()
        .filter_map(|&(entity, at)| {
            let to = at - around.from;
            let distance = to.length();
            // A NaN distance is out of range too.
            if distance.is_nan() || distance > around.range || around.range <= 0.0 {
                return None;
            }
            let angle = if distance > 0.0 {
                math::acos((to.dot(around.forward) / distance).clamp(-1.0, 1.0))
            } else {
                0.0
            };
            if cone && angle > around.cone {
                return None;
            }
            Some(Candidate {
                entity,
                distance,
                angle,
                visible: true,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.distance
            .total_cmp(&b.distance)
            .then(a.entity.cmp(&b.entity))
    });
    if let Some(sight) = sight {
        for c in &mut out {
            let at = points
                .iter()
                .find(|p| p.0 == c.entity)
                .map_or(around.from, |p| p.1);
            c.visible = sight(c.entity, at);
        }
    }
    out
}
