//! One path for the bodies in every og.Water (a lake, a sea, a river): what
//! crosses its surface is an og.Splash, what stands in it cuts it for the
//! renderer, and which side each body is on is world state.
//!
//! Ways this could fail, each caught below: only an og.WaterSurface
//! splashes (a lake or a sea sends nothing); only og.Buoyancy hulls are seen
//! (a plain body or a character crosses in silence); the crossing is at the
//! rest level, not the sea drawn there; a body bobbing at the surface or
//! resting beside it splashes every tick; the side a body was on is not
//! saved (a restore between two ticks loses the splash); bodies clear of the
//! water are kept (a dry world saves other bytes); a lake records no cut.

use crate::support;

use og_core::glam::{DVec3, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_env::ocean::Ocean;
use og_physics::*;
use support::*;

const LAKE: u64 = 1 << 40;

/// A flat lake 200 m across, its surface at y 0.
fn lake() -> WaterBody {
    WaterBody {
        entity: LAKE,
        position: Vec3::ZERO,
        size: Vec2::splat(200.0),
        sea: None,
        zero: DVec3::ZERO,
        shore: None,
        river: None,
    }
}

/// A gale's sea 6 km across.
fn sea() -> WaterBody {
    WaterBody {
        sea: Ocean::new(Vec2::new(7.0, -18.7), 1.0, 1.0),
        size: Vec2::splat(6000.0),
        ..lake()
    }
}

fn world(waters: Vec<WaterBody>) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(Physics::default(), 4);
    support::enable_lazy(&mut w);
    p.set_waters(waters);
    (w, p)
}

/// A half-metre crate denser than water, without og.Buoyancy.
fn crate_at(w: &mut World, x: f32, y: f32, z: f32) -> Entity {
    spawn_body(
        w,
        at(x, y, z),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::splat(0.5)),
    )
}

/// Steps `ticks`, returning every og.Splash with the tick it was sent in.
fn splashes(w: &mut World, ticks: u32) -> Vec<(u32, Splash)> {
    let mut all = Vec::new();
    for tick in 0..ticks {
        w.step().unwrap();
        for s in w.pending_events::<Splash>().unwrap() {
            all.push((tick, s));
        }
    }
    all
}

#[test]
fn a_crate_a_hull_and_a_character_each_splash_into_a_lake() {
    let (mut w, _p) = world(vec![lake()]);
    let plain = crate_at(&mut w, 0.0, 2.0, 0.0);
    let hull = crate_at(&mut w, 10.0, 2.0, 0.0);
    w.insert(hull, Buoyancy::default()).unwrap();
    let walker = w
        .spawn((
            at(20.0, 2.0, 0.0),
            Collider::capsule(0.4, 1.0),
            CharacterController::default(),
        ))
        .unwrap();
    // A crate resting on a pier beside the water's level, and one far over it.
    spawn_static(
        &mut w,
        at(40.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(4.0, 1.0, 4.0)),
    );
    let beside = crate_at(&mut w, 40.0, 0.25, 0.0);
    let all = splashes(&mut w, 90);
    let of = |e: Entity| -> Vec<Splash> {
        all.iter()
            .filter(|(_, s)| s.entity == e.to_bits())
            .map(|(_, s)| *s)
            .collect()
    };
    // The plain crate has no lift from an og.Water: it goes in once and sinks.
    let plain = of(plain);
    assert_eq!(plain.len(), 1, "the sinking crate splashed {plain:?}");
    let s = plain[0];
    assert_eq!(s.water, LAKE);
    assert!(s.point.y.abs() < 1e-6 && s.point.x.abs() < 0.01, "{s:?}");
    // 2 m of free fall: 6.3 m/s down.
    assert!((s.speed + 6.26).abs() < 0.3, "it went in at {}", s.speed);
    // The hull goes in (and may come back up through the surface as it bobs).
    let hull = of(hull);
    assert!(
        hull.first().is_some_and(|s| s.speed < -5.0),
        "the hull's splashes: {hull:?}"
    );
    let walker = of(walker);
    assert!(
        walker
            .first()
            .is_some_and(|s| s.speed < 0.0 && s.water == LAKE),
        "the character's splashes: {walker:?}"
    );
    assert!(of(beside).is_empty(), "a crate at rest splashed");
}

#[test]
fn a_sea_is_crossed_where_its_waves_stand_and_a_restore_keeps_the_splash() {
    let water = sea();
    let build = || {
        let (mut w, p) = world(vec![water.clone()]);
        let crates: Vec<Entity> = (0..6)
            .map(|i| crate_at(&mut w, i as f32 * 37.0, 9.0, -20.0))
            .collect();
        (w, p, crates)
    };
    let (mut w, _p, crates) = build();
    let all = splashes(&mut w, 150);
    assert_eq!(all.len(), crates.len(), "one splash a crate: {all:?}");
    let mut furthest = 0.0f32;
    for (tick, s) in &all {
        let level = water
            .height(
                f64::from(s.point.x),
                f64::from(s.point.z),
                f64::from(*tick) / 60.0,
            )
            .unwrap();
        // The sea moves under a point by centimetres in a tick.
        assert!(
            (s.point.y - level).abs() < 0.15,
            "crossed at {} where the sea stands at {level}",
            s.point.y
        );
        furthest = furthest.max(level.abs());
    }
    assert!(furthest > 0.5, "the gale stood within {furthest} m of rest");

    // Saved one tick before the first crossing and restored into a fresh
    // engine: the same splashes, in the same ticks.
    let first = all.iter().map(|(t, _)| *t).min().unwrap();
    let (mut a, _pa, _) = build();
    run(&mut a, first);
    let saved = a.snapshot();
    let (mut r, _pr) = world(vec![water.clone()]);
    r.restore(&saved).unwrap();
    let after: Vec<(u32, Splash)> = splashes(&mut r, 150 - first)
        .into_iter()
        .map(|(t, s)| (t + first, s))
        .collect();
    assert_eq!(after, all, "a restore loses or moves a splash");
}

#[test]
fn bodies_clear_of_the_water_are_not_world_state() {
    // The same crates on the same floor 5 m over a lake and over none: the
    // water is the host's to set, and no body is near it.
    let run_with = |waters: Vec<WaterBody>| {
        let (mut w, _p) = world(waters);
        spawn_static(
            &mut w,
            at(0.0, 5.0, 0.0),
            Collider::cuboid(Vec3::new(20.0, 1.0, 20.0)),
        );
        for i in 0..4 {
            crate_at(&mut w, i as f32 * 2.0, 7.0, 0.0);
        }
        run(&mut w, 60);
        w.snapshot()
    };
    assert!(
        run_with(vec![lake()]) == run_with(Vec::new()),
        "a dry world saves other bytes over a lake"
    );
}

#[test]
fn what_stands_in_a_lake_cuts_it_for_the_renderer() {
    let (mut w, p) = world(vec![lake()]);
    // A post standing through the surface, and a hull afloat on it.
    spawn_static(
        &mut w,
        at(0.0, -3.0, 0.0),
        Collider::cuboid(Vec3::new(4.0, 1.0, 4.0)),
    );
    let post = spawn_body(
        &mut w,
        at(0.0, -1.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(0.4, 3.0, 0.6)),
    );
    let hull = spawn_body(
        &mut w,
        at(30.0, 0.2, 0.0),
        RigidBody::dynamic(),
        Collider {
            density: 0.6,
            ..Collider::cuboid(Vec3::new(2.0, 0.5, 1.0))
        },
    );
    w.insert(hull, Buoyancy::default()).unwrap();
    run(&mut w, 240);
    let wakes = p
        .water_wakes(Entity::from_bits(LAKE))
        .expect("a lake keeps its cuts");
    let (_, cuts) = wakes.ticks.back().unwrap();
    let cut = |e: Entity| cuts.iter().find(|c| c.entity == e.to_bits()).copied();
    let post = cut(post).expect("the post cuts the lake");
    assert!(
        (post.half - Vec2::new(0.2, 0.3)).abs().max_element() < 0.02 && post.centre.length() < 0.05,
        "{post:?}"
    );
    // The post stands on the bed at -2.5: 2.5 m of it under.
    assert!((post.depth - 2.5).abs() < 0.05, "{post:?}");
    let hull = cut(hull).expect("the hull cuts the lake");
    assert!(
        (hull.centre - Vec2::new(30.0, 0.0)).length() < 0.2,
        "{hull:?}"
    );
    assert_eq!(cuts.len(), 2, "each body cuts once: {cuts:?}");
}
