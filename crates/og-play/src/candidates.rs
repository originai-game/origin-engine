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

#[cfg(test)]
mod tests {
    use super::*;

    fn around(cone: f32) -> Around {
        Around {
            from: Vec3::ZERO,
            forward: Vec3::Z,
            range: 5.0,
            cone,
        }
    }

    #[test]
    fn nearest_first_then_by_entity_within_the_range() {
        let points = [
            (4, Vec3::new(0.0, 0.0, 3.0)),
            (2, Vec3::new(3.0, 0.0, 0.0)),
            (9, Vec3::new(0.0, 0.0, 1.0)),
            (1, Vec3::new(0.0, 0.0, 6.0)),
        ];
        let found = candidates(&around(0.0), &points, None);
        let ids: Vec<u64> = found.iter().map(|c| c.entity).collect();
        // 2 and 4 are both 3 away: entity order.
        assert_eq!(ids, [9, 2, 4]);
        assert!((found[1].angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(found[2].angle, 0.0);
        assert!(
            candidates(
                &Around {
                    range: 0.0,
                    ..around(0.0)
                },
                &points,
                None
            )
            .is_empty()
        );
    }

    #[test]
    fn a_cone_keeps_what_is_ahead_and_sight_is_reported_not_filtered() {
        let points = [
            (1, Vec3::new(0.0, 0.0, 2.0)),
            (2, Vec3::new(2.0, 0.0, 2.0)),
            (3, Vec3::new(0.0, 0.0, -2.0)),
        ];
        // 50°: the one straight ahead and the one 45° off.
        let found = candidates(&around(0.87), &points, None);
        assert_eq!(found.iter().map(|c| c.entity).collect::<Vec<_>>(), [1, 2]);
        let walled = |e: u64, _: Vec3| e != 1;
        let seen = candidates(&around(0.0), &points, Some(&walled));
        assert_eq!(seen.len(), 3);
        assert_eq!(
            seen.iter().find(|c| c.entity == 1).map(|c| c.visible),
            Some(false)
        );
        assert!(seen.iter().filter(|c| c.entity != 1).all(|c| c.visible));
    }
}
