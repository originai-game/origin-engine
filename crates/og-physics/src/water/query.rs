//! The water at a point: one answer for every kind (a lake, a sea, a river, a height field, a line).

use super::{LineState, SurfaceState, WaterBody};
use crate::schema_ext::WaterSurface;
use og_core::IdMap;
use og_core::glam::{Vec2, Vec3};
use og_ecs::Entity;
use og_env::ocean::SurfacePoint;

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

/// The water at a point ([`at`]): whose it is, where its surface stands
/// over the point and how it moves there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterAt {
    /// The water's entity.
    pub water: u64,
    /// The surface over the point (world y), as it is drawn.
    pub height: f32,
    /// How far the point is under the surface (negative over it).
    pub depth: f32,
    /// The water's velocity and acceleration at the surface there.
    pub velocity: Vec3,
    pub acceleration: Vec3,
    /// The surface's unit normal there (up on still water).
    pub normal: Vec3,
}

/// How far apart the surface is read for its slope (m).
const SLOPE: f32 = 0.05;

/// The surface of one water (its entity and world position) over world
/// (`x`, `z`), with the waves bodies raised on its own height field or
/// springs: its world height and its motion; `None` outside it.
fn drawn(
    waters: &[WaterBody],
    surfaces: &IdMap<u64, SurfaceState>,
    lines: &IdMap<u64, LineState>,
    (entity, centre): (u64, Vec3),
    (x, z): (f32, f32),
    seconds: f64,
) -> Option<SurfacePoint> {
    let body = waters.iter().find(|w| w.entity == entity);
    let sea = body.map(|w| w.surface(f64::from(x), f64::from(z), seconds));
    if let Some(s) = surfaces.get(&entity) {
        let half = s.size * 0.5;
        if (x - centre.x).abs() > half || (z - centre.z).abs() > half {
            return None;
        }
        let sea = sea.flatten().unwrap_or_default();
        return Some(SurfacePoint {
            height: centre.y + sea.height + s.height_at(centre, x, z),
            velocity: sea.velocity + Vec3::Y * s.velocity_at(centre, x, z),
            ..sea
        });
    }
    if let Some(l) = lines.get(&entity) {
        if (x - centre.x).abs() > l.width * 0.5 {
            return None;
        }
        let flat = Vec2::new(centre.x, centre.y);
        return Some(SurfacePoint {
            height: centre.y + l.height_at(flat, x),
            velocity: Vec3::Y * l.velocity_at(flat, x),
            acceleration: Vec3::ZERO,
        });
    }
    let p = sea??;
    Some(SurfacePoint {
        height: body?.position.y + p.height,
        ..p
    })
}

/// The water at world `point`: of every og.Water (`waters`) and every
/// height field and line of springs (`places`: each one's entity and world
/// position, in entity order), the one whose surface stands highest over
/// the point, as it is drawn (a sea's waves, a river's course, the waves
/// bodies raised), with its motion and its slope there. `None` over no
/// water. A line of springs is asked at `x` alone and its normal lies in
/// xy.
pub(crate) fn at(
    waters: &[WaterBody],
    surfaces: &IdMap<u64, SurfaceState>,
    lines: &IdMap<u64, LineState>,
    places: &[(u64, Vec3)],
    point: Vec3,
    seconds: f64,
) -> Option<WaterAt> {
    let read = |entity: u64, centre: Vec3, x: f32, z: f32| {
        drawn(waters, surfaces, lines, (entity, centre), (x, z), seconds)
    };
    let fields = places
        .iter()
        .filter(|p| surfaces.contains_key(&p.0) || lines.contains_key(&p.0))
        .copied();
    let alone = waters
        .iter()
        .filter(|w| !surfaces.contains_key(&w.entity) && !lines.contains_key(&w.entity))
        .map(|w| (w.entity, w.position));
    let mut best: Option<(u64, Vec3, SurfacePoint)> = None;
    for (entity, centre) in fields.chain(alone) {
        let Some(p) = read(entity, centre, point.x, point.z) else {
            continue;
        };
        if best.is_none_or(|b| p.height > b.2.height) {
            best = Some((entity, centre, p));
        }
    }
    let (entity, centre, p) = best?;
    // The slope by central differences; a side off the water reads the middle.
    let rise = |dx: f32, dz: f32| {
        let high = read(entity, centre, point.x + dx, point.z + dz).map_or(p.height, |s| s.height);
        let low = read(entity, centre, point.x - dx, point.z - dz).map_or(p.height, |s| s.height);
        (high - low) / (2.0 * SLOPE)
    };
    let along = rise(SLOPE, 0.0);
    let across = if lines.contains_key(&entity) {
        0.0
    } else {
        rise(0.0, SLOPE)
    };
    Some(WaterAt {
        water: entity,
        height: p.height,
        depth: p.height - point.y,
        velocity: p.velocity,
        acceleration: p.acceleration,
        // Level water's normal is (0, 1, 0), not (-0, 1, -0).
        normal: Vec3::new(0.0 - along, 1.0, 0.0 - across).normalize(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_ext::WaterLine;

    fn still(entity: u64, position: Vec3, size: f32) -> WaterBody {
        WaterBody {
            entity,
            position,
            size: Vec2::splat(size),
            sea: None,
            zero: og_core::glam::DVec3::ZERO,
            shore: None,
            river: None,
        }
    }

    #[test]
    fn the_highest_water_over_a_point_answers_with_the_points_depth() {
        // A lake at y 2 and a pond at y 5 standing over one corner of it.
        let waters = [
            still(3, Vec3::new(10.0, 2.0, 0.0), 40.0),
            still(7, Vec3::new(12.0, 5.0, 1.0), 4.0),
        ];
        let none = IdMap::new();
        let ask = |p: Vec3| at(&waters, &none, &IdMap::new(), &[], p, 0.0);
        let pond = ask(Vec3::new(12.5, 3.0, 1.5)).unwrap();
        assert_eq!((pond.water, pond.height, pond.depth), (7, 5.0, 2.0));
        let lake = ask(Vec3::new(20.0, 3.0, 0.0)).unwrap();
        assert_eq!((lake.water, lake.height, lake.depth), (3, 2.0, -1.0));
        assert_eq!(lake.normal, Vec3::Y);
        assert_eq!((lake.velocity, lake.acceleration), (Vec3::ZERO, Vec3::ZERO));
        assert_eq!(ask(Vec3::new(31.0, 0.0, 0.0)), None);
    }

    #[test]
    fn a_seas_point_is_its_own_surface_and_leans_with_the_wave() {
        let sea = WaterBody {
            sea: og_core::ocean::Ocean::new(Vec2::new(0.0, -12.0), 1.0, 0.8),
            ..still(4, Vec3::new(0.0, 1.0, 0.0), 2000.0)
        };
        let waters = [sea.clone()];
        let p = Vec3::new(31.0, -3.0, -57.0);
        let w = at(&waters, &IdMap::new(), &IdMap::new(), &[], p, 12.5).unwrap();
        let own = sea.surface(31.0, -57.0, 12.5).unwrap();
        assert_eq!(w.height, 1.0 + own.height);
        assert_eq!(w.depth, w.height + 3.0);
        assert_eq!(
            (w.velocity, w.acceleration),
            (own.velocity, own.acceleration)
        );
        assert!(w.velocity.length() > 0.05, "{:?}", w.velocity);
        // The normal is square to the surface a step away along each axis.
        let h = |x: f64, z: f64| 1.0 + sea.surface(x, z, 12.5).unwrap().height;
        for (dx, dz) in [(0.01, 0.0), (0.0, 0.01)] {
            let along = Vec3::new(
                2.0 * dx as f32,
                h(31.0 + dx, -57.0 + dz) - h(31.0 - dx, -57.0 - dz),
                2.0 * dz as f32,
            );
            let off = w.normal.dot(along.normalize());
            assert!(off.abs() < 2e-2, "{off} along {along:?}");
        }
        assert!((w.normal.length() - 1.0).abs() < 1e-5 && w.normal.y > 0.5);
        assert!(w.normal.x != 0.0 && w.normal.z != 0.0);
    }

    #[test]
    fn the_waves_bodies_raised_lift_the_point_and_lean_its_normal() {
        // A 16 m height field at y 1 whose waves stand as a ramp rising
        // 0.1 m a metre along +x, rising at 0.3 m/s everywhere.
        let surface = WaterSurface {
            resolution: 65,
            size: 16.0,
            ..WaterSurface::default()
        };
        let mut state = SurfaceState::new(&surface);
        let n = 65;
        for j in 0..n {
            for i in 0..n {
                state.heights[j * n + i] = 0.1 * (i as f32 * 0.25 - 8.0);
                state.velocities[j * n + i] = 0.3;
            }
        }
        let mut surfaces = IdMap::new();
        surfaces.insert(5, state);
        let centre = Vec3::new(100.0, 1.0, -20.0);
        let ask =
            |places: &[(u64, Vec3)], p: Vec3| at(&[], &surfaces, &IdMap::new(), places, p, 0.0);
        let w = ask(&[(5, centre)], Vec3::new(102.0, 0.0, -21.0)).unwrap();
        assert_eq!(w.water, 5);
        assert!((w.height - 1.2).abs() < 1e-5, "{}", w.height);
        assert!((w.depth - 1.2).abs() < 1e-5);
        assert!((w.velocity - Vec3::Y * 0.3).length() < 1e-6);
        let lean = Vec3::new(-0.1, 1.0, 0.0).normalize();
        assert!((w.normal - lean).length() < 1e-3, "{:?}", w.normal);
        // Off its square, and without its place, it is not asked.
        assert_eq!(ask(&[(5, centre)], Vec3::new(109.0, 0.0, -21.0)), None);
        assert_eq!(ask(&[], Vec3::new(102.0, 0.0, -21.0)), None);
    }

    #[test]
    fn a_line_of_springs_answers_at_x_with_a_normal_in_xy() {
        let line = WaterLine {
            width: 12.0,
            columns: 25,
            ..WaterLine::default()
        };
        let mut state = LineState::new(&line);
        for (i, h) in state.heights.iter_mut().enumerate() {
            *h = -0.2 * (i as f32 * 0.5 - 6.0);
        }
        let mut lines = IdMap::new();
        lines.insert(8, state);
        let centre = Vec3::new(4.0, 3.0, 0.0);
        let w = at(
            &[],
            &IdMap::new(),
            &lines,
            &[(8, centre)],
            Vec3::new(5.0, 2.0, 9.0),
            0.0,
        )
        .unwrap();
        assert_eq!(w.water, 8);
        assert!((w.height - 2.8).abs() < 1e-5, "{}", w.height);
        assert!((w.depth - 0.8).abs() < 1e-5);
        let lean = Vec3::new(0.2, 1.0, 0.0).normalize();
        assert!((w.normal - lean).length() < 1e-3 && w.normal.z == 0.0);
    }
}
