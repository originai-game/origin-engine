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
    // A river's level is its course's, a mesh's its triangles': only its
    // own point says how high.
    if w.river.is_none() && w.mesh.is_none() {
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
