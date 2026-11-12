//! The water at a point: one answer for every kind (a lake, a sea, a river, a height field, a line).

use super::{LineState, SurfaceState, WaterBody};
use crate::schema_ext::WaterSurface;
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use og_core::ocean::SurfacePoint;
use og_ecs::Entity;

/// The water surface over world point (`x`, `z`): its height (world y)
/// and the water's velocity and acceleration there; the highest of the og.WaterSurface squares
/// and og.Water bodies over it (one entity with both: its rest level and
/// its sea together); `None` over no water. An og.WaterSurface lifts at its
/// rest level, as its own buoyancy does: its waves are the bodies'
/// splashes, and a body riding its own wake would pump them.
pub(crate) fn level(
    surfaces: &[(Entity, Vec3, WaterSurface)],
    waters: &[WaterBody],
    x: f32,
    z: f32,
    seconds: f64,
) -> Option<SurfacePoint> {
    let mut level: Option<SurfacePoint> = None;
    let mut take = |y: f32, p: SurfacePoint| {
        if level.is_none_or(|l| y > l.height) {
            level = Some(SurfacePoint { height: y, ..p });
        }
    };
    let (x64, z64) = (f64::from(x), f64::from(z));
    for (e, position, surface) in surfaces {
        let half = surface.size * 0.5;
        if (x - position.x).abs() > half || (z - position.z).abs() > half {
            continue;
        }
        let key = e.to_bits();
        let sea = waters
            .iter()
            .find(|w| w.entity == key)
            .and_then(|w| w.surface(x64, z64, seconds))
            .unwrap_or_default();
        take(position.y + sea.height, sea);
    }
    for w in waters {
        if surfaces.iter().any(|s| s.0.to_bits() == w.entity) {
            continue;
        }
        if let Some(p) = w.surface(x64, z64, seconds) {
            take(w.position.y + p.height, p);
        }
    }
    level
}

/// How far the waves bodies raised on `entity`'s water stand over its rest
/// level at world point (`x`, `z`): its og.WaterSurface's height field or
/// its og.WaterLine's springs (`x` alone), the entity at `centre`; 0
/// outside them. `None` for an entity with neither (or not stepped yet).
pub(crate) fn ripple(
    surfaces: &IdMap<u64, SurfaceState>,
    lines: &IdMap<u64, LineState>,
    entity: u64,
    centre: Vec3,
    x: f32,
    z: f32,
) -> Option<f32> {
    if let Some(s) = surfaces.get(&entity) {
        return Some(s.height_at(centre, x, z));
    }
    lines
        .get(&entity)
        .map(|l| l.height_at(Vec2::new(centre.x, centre.y), x))
}
