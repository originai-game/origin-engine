//! Bodies in an og.Water (a lake, a sea, a river): what crosses its surface and what cuts it.

use super::{Crossing, Floater, WaterBody, WaterCut};
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use std::collections::BTreeMap;

/// How near a water's surface a body's box is followed across it (m),
/// beside twice what it falls in a tick: the sea's level moves under a
/// body by less in a tick.
const NEAR: f32 = 0.25;

/// By og.Water, then by body: whether each body near its surface had its
/// centre under it after the last step. World state (a crossing is the
/// change from it), saved in the block's extension; a water no body is
/// near has no entry, so a world of dry bodies saves as it did.
pub(crate) type Under = IdMap<u64, IdMap<u64, bool>>;

/// What one tick's bodies do at the og.Water surfaces: the crossings (each
/// with its water) and, per water, the bodies cutting its surface.
#[derive(Default)]
pub(crate) struct Wet {
    pub crossings: Vec<(u64, Crossing)>,
    pub cuts: BTreeMap<u64, Vec<WaterCut>>,
}

/// The surface of `w` under the floater `f` (world y) when its box is near
/// it; `None` away from the water, over it or under it. The sea is asked
/// only for a body its waves can reach.
fn level_near(w: &WaterBody, f: &Floater, seconds: f64, dt: f32) -> Option<f32> {
    let near = NEAR + 2.0 * f.velocity.y.abs() * dt;
    // A river's level is its course's: only its own point says how high.
    if w.river.is_none() {
        let reach = w.reach() + near;
        if f.min.y > w.position.y + reach || f.max.y < w.position.y - reach {
            return None;
        }
    }
    let at = w.surface(f64::from(f.centre.x), f64::from(f.centre.z), seconds)?;
    let level = w.position.y + at.height;
    (f.min.y <= level + near && f.max.y >= level - near).then_some(level)
}

/// One tick of the bodies at the og.Water surfaces (`waters`: those
/// without an og.WaterSurface, which splashes for itself): a body whose
/// centre changed sides of a surface crosses it (og.Splash), and a body
/// whose box straddles one cuts it, for the renderer (a hull's cut is
/// og.Buoyancy's, its own footprint). Waters and bodies in entity order.
pub(crate) fn step<'a>(
    waters: impl Iterator<Item = &'a WaterBody>,
    floaters: &[Floater],
    seconds: f64,
    dt: f32,
    under: &mut Under,
) -> Wet {
    let mut wet = Wet::default();
    let mut next = Under::new();
    for w in waters {
        let was = under.get(&w.entity);
        let mut seen = IdMap::new();
        let cuts = wet.cuts.entry(w.entity).or_default();
        for f in floaters {
            let Some(level) = level_near(w, f, seconds, dt) else {
                continue;
            };
            let below = f.centre.y < level;
            seen.insert(f.entity, below);
            if was
                .and_then(|m| m.get(&f.entity))
                .is_some_and(|b| *b != below)
            {
                wet.crossings.push((
                    w.entity,
                    Crossing {
                        entity: f.entity,
                        point: Vec3::new(f.centre.x, level, f.centre.z),
                        speed: f.velocity.y,
                    },
                ));
            }
            if !f.hull && f.min.y < level && f.max.y > level {
                let centre = Vec2::new(w.position.x, w.position.z);
                cuts.push(WaterCut::boxed(
                    f.entity,
                    Vec2::new(f.min.x, f.min.z) - centre,
                    Vec2::new(f.max.x, f.max.z) - centre,
                    level - f.min.y,
                    f.velocity.y,
                ));
            }
        }
        if !seen.is_empty() {
            next.insert(w.entity, seen);
        }
    }
    *under = next;
    wet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lake() -> WaterBody {
        WaterBody {
            entity: 3,
            position: Vec3::new(10.0, 2.0, 0.0),
            size: Vec2::splat(40.0),
            sea: None,
            zero: og_core::glam::DVec3::ZERO,
            shore: None,
            river: None,
        }
    }

    /// A half-metre box centred at `y` over the lake's middle, falling at `speed`.
    fn crate_at(y: f32, speed: f32) -> Floater {
        let centre = Vec3::new(12.0, y, 1.0);
        Floater {
            entity: 9,
            min: centre - Vec3::splat(0.25),
            max: centre + Vec3::splat(0.25),
            centre,
            velocity: Vec3::new(0.0, speed, 0.0),
            dynamic: true,
            hull: false,
        }
    }

    #[test]
    fn a_body_falling_into_a_lake_crosses_once_and_cuts_while_it_straddles() {
        let lake = lake();
        let mut under = Under::new();
        let dt = 1.0 / 60.0;
        let mut crossings = Vec::new();
        let mut cut = Vec::new();
        // From 3 m over the surface to 3 m under it, 5 cm a tick.
        for tick in 0..120 {
            let y = 5.0 - 0.05 * tick as f32;
            let wet = step(
                [&lake].into_iter(),
                &[crate_at(y, -3.0)],
                0.0,
                dt,
                &mut under,
            );
            crossings.extend(
                wet.crossings
                    .iter()
                    .map(|(w, c)| (*w, c.entity, c.point, y)),
            );
            cut.extend(wet.cuts[&3].iter().map(|c| (y, *c)));
        }
        assert_eq!(crossings.len(), 1, "{crossings:?}");
        let (water, body, point, y) = crossings[0];
        assert_eq!((water, body), (3, 9));
        assert_eq!(point, Vec3::new(12.0, 2.0, 1.0));
        assert!(
            y < 2.0 && y > 1.94,
            "the first tick its centre is under: {y}"
        );
        // It cuts only while its box straddles the level, in the lake's frame.
        assert!(cut.iter().all(|(y, _)| (y - 2.0).abs() < 0.25));
        assert!(cut.len() >= 9, "{}", cut.len());
        let (y, c) = cut[0];
        assert_eq!(c.min, Vec2::new(1.75, 0.75));
        assert_eq!(c.max, Vec2::new(2.25, 1.25));
        assert!((c.depth - (2.0 - (y - 0.25))).abs() < 1e-5);
        // Deep under, it is followed no more: nothing is kept for it.
        assert!(under.is_empty());
    }

    #[test]
    fn a_body_that_starts_under_or_beside_the_water_does_not_splash() {
        let lake = lake();
        let dt = 1.0 / 60.0;
        let mut under = Under::new();
        for _ in 0..3 {
            let wet = step(
                [&lake].into_iter(),
                &[crate_at(1.9, 0.0)],
                0.0,
                dt,
                &mut under,
            );
            assert!(wet.crossings.is_empty());
        }
        assert_eq!(under.get(&3).and_then(|m| m.get(&9)), Some(&true));
        // Off the lake's square: not its body.
        let mut far = crate_at(1.9, 0.0);
        far.centre.x += 40.0;
        far.min.x += 40.0;
        far.max.x += 40.0;
        let mut under = Under::new();
        let wet = step([&lake].into_iter(), &[far], 0.0, dt, &mut under);
        assert!(wet.crossings.is_empty() && wet.cuts[&3].is_empty() && under.is_empty());
    }

    #[test]
    fn a_hull_crosses_but_leaves_its_cut_to_its_own_footprint() {
        let lake = lake();
        let mut under = Under::new();
        let dt = 1.0 / 60.0;
        let hull = |y: f32| Floater {
            hull: true,
            dynamic: false,
            ..crate_at(y, -1.0)
        };
        let a = step([&lake].into_iter(), &[hull(2.1)], 0.0, dt, &mut under);
        let b = step([&lake].into_iter(), &[hull(1.9)], 0.0, dt, &mut under);
        assert!(a.crossings.is_empty() && a.cuts[&3].is_empty());
        assert_eq!(b.crossings.len(), 1);
        assert!(b.cuts[&3].is_empty());
    }
}
