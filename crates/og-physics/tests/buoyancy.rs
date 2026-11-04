//! og.Buoyancy: rigid bodies floating on og.Water (flat or a sea) and
//! og.WaterSurface by their voxelised hull.
//!
//! Ways this could fail, each caught below: the lift is not the water the
//! hull displaces (a body floats at the wrong depth, or its volume is the
//! colliders' box rather than their volume); cells are pushed at the
//! centre of mass (a tipped hull never rights itself); the sea's waves are
//! not the ones the renderer draws or not the absolute world's (a hull
//! moved by og.WorldOrigin rides other waves); a restore or a second run
//! diverges; the surface's box buoyancy also acts on a body with its own
//! (it floats too high); a world that never names og.Buoyancy registers it.

use crate::support;

use og_core::Reflect;
use og_core::glam::{DVec3, Quat, Vec2, Vec3};
use og_core::ocean::Ocean;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn world(seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(Physics::default(), seed);
    support::enable_lazy(&mut w);
    (w, p)
}

/// A flat lake 200 m across, its surface at y 0.
fn lake() -> WaterBody {
    WaterBody {
        entity: 1 << 40,
        position: Vec3::ZERO,
        size: Vec2::splat(200.0),
        segments: 64,
        sea: None,
        zero: DVec3::ZERO,
        shore: None,
    }
}

/// A gale's sea 6 km across (the storm frame's) centred on the absolute
/// world's zero, seen with og.WorldOrigin's zero at `zero` (the water's
/// entity moved by it, as every place in the world is).
fn sea(zero: DVec3) -> WaterBody {
    WaterBody {
        position: -zero.as_vec3(),
        sea: Ocean::new(Vec2::new(7.0, -18.7), 1.0, 1.0),
        size: Vec2::splat(6000.0),
        segments: 256,
        zero,
        ..lake()
    }
}

fn floater(w: &mut World, t: Transform, size: Vec3, density: f32, b: Buoyancy) -> Entity {
    let collider = Collider {
        density,
        ..Collider::cuboid(size)
    };
    let e = spawn_body(w, t, RigidBody::dynamic(), collider);
    w.insert(e, b).unwrap();
    e
}

#[test]
fn a_box_floats_at_the_depth_archimedes_gives() {
    // A 2 × 0.5 × 1 box of density 0.6 in water of density 2 is 0.3 under:
    // 0.15 m of its 0.5, its centre 0.1 m above the surface. Its volume
    // and its box are the same here; a sphere (below) tells them apart.
    let (mut w, p) = world(3);
    p.set_waters(vec![lake()]);
    let b = floater(
        &mut w,
        at(0.0, 2.0, 0.0),
        Vec3::new(2.0, 0.5, 1.0),
        0.6,
        Buoyancy::default(),
    );
    // A ball of radius 0.5 and density 1 displaces half its volume:
    // its centre sits at the surface (a box's volume would hold it 0.12 m
    // higher, as its 4/3 π r³ is 0.52 of its box).
    let ball = spawn_body(
        &mut w,
        at(5.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    w.insert(
        ball,
        Buoyancy {
            samples: 12,
            ..Buoyancy::default()
        },
    )
    .unwrap();
    run(&mut w, 840);
    // Still by then: it holds its height through a second. (Its written
    // velocity is not 0 at rest: the lift is an impulse at the tick's start,
    // as og.physics.applyForce, and gravity acts over the substeps.)
    let mut heights = Vec::new();
    for _ in 0..60 {
        w.step().unwrap();
        heights.push(transform(&w, b).position.y);
    }
    let (lo, hi) = heights
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi - lo < 0.002, "it still bobs {} m", hi - lo);
    let y = transform(&w, b).position.y;
    assert!((y - 0.1).abs() < 0.005, "the box's centre at {y}, not 0.1");
    let under = w.get::<Buoyancy>(b).unwrap().submerged;
    assert!(
        (under - 0.3).abs() < 0.01,
        "written submerged {under}, not 0.3"
    );
    let t = transform(&w, b);
    assert!(
        t.quaternion.angle_between(Quat::IDENTITY) < 0.01,
        "it floats level: {:?}",
        t.quaternion
    );
    let y = transform(&w, ball).position.y;
    assert!(
        y.abs() < 0.03,
        "the ball's centre at {y}, not at the surface"
    );
}

#[test]
fn a_tipped_hull_rights_itself_and_a_heavy_one_sinks() {
    let (mut w, p) = world(4);
    p.set_waters(vec![lake()]);
    // A wide, flat hull on its side: the lift at its cells under water
    // turns it back flat (a lift at the centre of mass never would).
    let hull = floater(
        &mut w,
        rotated(at(0.0, 0.2, 0.0), Quat::from_rotation_z(1.2)),
        Vec3::new(3.0, 0.6, 1.5),
        0.5,
        Buoyancy::default(),
    );
    let stone = floater(
        &mut w,
        at(10.0, 1.0, 0.0),
        Vec3::ONE,
        5.0,
        Buoyancy::default(),
    );
    run(&mut w, 900);
    let up = transform(&w, hull).quaternion * Vec3::Y;
    assert!(up.y > 0.99, "the hull is flat again: its up is {up}");
    let s = transform(&w, stone).position.y;
    assert!(s < -5.0, "a stone of density 5 sinks: at {s}");
    assert_eq!(w.get::<Buoyancy>(stone).unwrap().submerged, 1.0);
}

/// The hull's pose after `ticks`, bit for bit.
fn bits(w: &World, e: Entity) -> [u32; 7] {
    let t = transform(w, e);
    [
        t.position.x.to_bits(),
        t.position.y.to_bits(),
        t.position.z.to_bits(),
        t.quaternion.x.to_bits(),
        t.quaternion.y.to_bits(),
        t.quaternion.z.to_bits(),
        t.quaternion.w.to_bits(),
    ]
}

fn boat(w: &mut World, x: f32) -> Entity {
    floater(
        w,
        at(x, 1.0, -30.0),
        Vec3::new(2.0, 0.8, 5.0),
        0.5,
        Buoyancy {
            samples: 6,
            ..Buoyancy::default()
        },
    )
}

#[test]
fn a_hull_rides_the_waves_the_same_through_a_restore_and_a_second_run() {
    let ride = |zero: DVec3, x: f32| {
        let (mut w, p) = world(5);
        p.set_waters(vec![sea(zero)]);
        let e = boat(&mut w, x);
        let mut heights = Vec::new();
        for _ in 0..600 {
            w.step().unwrap();
            heights.push(transform(&w, e).position.y);
        }
        (w, p, e, heights)
    };
    let (w, _, e, heights) = ride(DVec3::ZERO, 0.0);
    // It rides the swell: metres up and down, not floating still.
    let tail = &heights[240..];
    let (lo, hi) = tail
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi - lo > 1.0, "the gale lifts and drops it {} m", hi - lo);
    let tilt = (transform(&w, e).quaternion * Vec3::Y).y;
    assert!(tilt < 0.9999, "and pitches it: its up's y is {tilt}");

    // A second run, bit for bit.
    let (again, _, e2, _) = ride(DVec3::ZERO, 0.0);
    assert_eq!(bits(&again, e2), bits(&w, e), "two runs differ");

    // The same sea seen with og.WorldOrigin's zero 2 km east: the hull
    // 2 km west of it in the shifted frame stands on the same water and
    // rides the same waves (up to the float precision of its position).
    let (shifted, _, e3, moved) = ride(DVec3::new(2048.0, 0.0, 0.0), -2048.0);
    let d = moved
        .iter()
        .zip(&heights)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(d < 0.05, "the shifted frame rides other waves: {d} m apart");
    let _ = (shifted, e3);

    // A restore at tick 300 into a fresh engine, the host setting its
    // waters as it would, runs on to the same pose.
    let (mut w, p) = world(5);
    p.set_waters(vec![sea(DVec3::ZERO)]);
    let e = boat(&mut w, 0.0);
    run(&mut w, 300);
    let saved = w.snapshot();
    run(&mut w, 300);
    let (mut r, rp) = world(5);
    rp.set_waters(vec![sea(DVec3::ZERO)]);
    r.restore(&saved).unwrap();
    run(&mut r, 300);
    assert_eq!(bits(&r, e), bits(&w, e), "a restore diverges");
}

/// A breezy sea over a 1:20 beach (W1): a buoy in half a metre of water
/// rides the shoaled, broken sea there, far less than its twin out in deep
/// water, the same bit for bit on a second run.
#[test]
fn a_buoy_in_the_shallows_rides_less_than_in_deep_water() {
    use og_core::ocean::{Shore, ShoreTrains};
    // The waterline at x = 210, 1 m cells from x = -100.
    let (c, r) = (401u32, 81u32);
    let depth: Vec<f32> = (0..c * r)
        .map(|i| (210.0 - ((i % c) as f32 - 100.0)) / 20.0)
        .collect();
    let shore = Shore::new(Vec2::new(-100.0, -40.0), 1.0, c, r, &depth).expect("a shore");
    let ocean = Ocean::new(Vec2::new(12.0, 0.0), 1.0, 0.8);
    let trains = ShoreTrains::new(ocean.unwrap().significant_height(), 1.0, 30.0);
    let water = WaterBody {
        sea: ocean,
        size: Vec2::splat(6000.0),
        segments: 256,
        shore: Some((std::sync::Arc::new(shore), trains)),
        ..lake()
    };
    let ride = || {
        let (mut w, p) = world(9);
        p.set_waters(vec![water.clone()]);
        let buoy = |w: &mut World, x: f32| {
            floater(
                w,
                at(x, 0.5, 0.0),
                Vec3::splat(0.4),
                0.5,
                Buoyancy::default(),
            )
        };
        // 0.5 m of water at x = 200; the deep one off the seabed's grid.
        let (shallow, deep) = (buoy(&mut w, 200.0), buoy(&mut w, -400.0));
        let mut ys = (Vec::new(), Vec::new());
        for _ in 0..900 {
            w.step().unwrap();
            ys.0.push(transform(&w, shallow).position.y);
            ys.1.push(transform(&w, deep).position.y);
        }
        (bits(&w, shallow), bits(&w, deep), ys)
    };
    let (a, b, (shallow, deep)) = ride();
    let span = |y: &[f32]| {
        let (lo, hi) = y[300..]
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        hi - lo
    };
    let (s, d) = (span(&shallow), span(&deep));
    assert!(d > 1.0, "the deep buoy rides {d} m");
    assert!(
        s < 0.5 && s < 0.25 * d,
        "the shallow buoy rides {s} m against {d} m"
    );
    let (a2, b2, _) = ride();
    assert_eq!((a, b), (a2, b2), "two runs differ");
}

#[test]
fn a_body_with_its_own_hull_takes_no_box_buoyancy_from_a_surface() {
    // og.WaterSurface lifts a body by its box (buoyancy 9.81 per m³ under);
    // with og.Buoyancy that would add to the hull's lift. Water of density
    // 2 floats a density-1 box half under, its centre at the rest level.
    let (mut w, _p) = world(6);
    w.spawn((at(0.0, 0.0, 0.0), WaterSurface::default()))
        .unwrap();
    let b = floater(
        &mut w,
        at(0.0, 1.0, 0.0),
        Vec3::ONE,
        1.0,
        Buoyancy::default(),
    );
    run(&mut w, 900);
    let y = transform(&w, b).position.y;
    assert!(y.abs() < 0.02, "its centre at {y}, not at the surface");
}

#[test]
fn a_world_that_never_names_it_does_not_register_it() {
    let (mut w, p) = support::world(Physics::default(), 7);
    p.set_waters(vec![lake()]);
    floor(&mut w);
    spawn_body(
        &mut w,
        at(0.0, 2.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 30);
    assert!(w.id_by_name(Buoyancy::NAME).is_err());
}

#[test]
fn floaters_stay_in_the_water_through_the_gales_crests() {
    // The boats sample (tests/evidence/r5-water/boats): three box boats and
    // four buoys set down on the storm frame's gale. Through six seconds of
    // its swell each stays wet, near its resting depth, and is never flung
    // clear of a crest.
    let (mut w, p) = world(8);
    let water = sea(DVec3::ZERO);
    p.set_waters(vec![water.clone()]);
    let level = |x: f32, z: f32| water.height(f64::from(x), f64::from(z), 0.0).unwrap();
    let mut floats = Vec::new();
    for (x, z, heading) in [
        (-12.0, -30.0, 0.4f32),
        (6.0, -45.0, -0.9),
        (-2.0, -70.0, 1.6),
    ] {
        let t = rotated(at(x, level(x, z) + 0.2, z), Quat::from_rotation_y(heading));
        let b = Buoyancy {
            samples: 6,
            ..Buoyancy::default()
        };
        floats.push(floater(&mut w, t, Vec3::new(2.4, 1.0, 6.0), 0.6, b));
    }
    for (x, z) in [
        (10.0, -20.0),
        (-6.0, -38.0),
        (18.0, -55.0),
        (-20.0, -52.0f32),
    ] {
        let collider = Collider {
            density: 0.8,
            ..Collider::sphere(0.6)
        };
        let e = spawn_body(
            &mut w,
            at(x, level(x, z) + 0.1, z),
            RigidBody::dynamic(),
            collider,
        );
        let b = Buoyancy {
            samples: 5,
            ..Buoyancy::default()
        };
        w.insert(e, b).unwrap();
        floats.push(e);
    }
    // Each floater's least and mean submerged fraction after a second.
    let mut each = vec![(1.0f32, 0.0f32); floats.len()];
    for tick in 0..600 {
        w.step().unwrap();
        if tick < 60 {
            continue;
        }
        for (i, &e) in floats.iter().enumerate() {
            let sub = w.get::<Buoyancy>(e).unwrap().submerged;
            each[i] = (each[i].0.min(sub), each[i].1 + sub / 540.0);
        }
    }
    // Afloat at their drafts on average (0.3 for the boats, 0.4 for the
    // buoys), and wet through every crest: a long hull over a short crest
    // lifts its ends out, a third of its draft at the least, never clear.
    // (Lifted by gravity alone, not the water's own acceleration, and
    // dragged towards the level's rise at a fixed point, every one was
    // flung to 0.01 or less.)
    for (i, &(least, mean)) in each.iter().enumerate() {
        let rest = if i < 3 { 0.3 } else { 0.4 };
        assert!(
            (mean - rest).abs() < 0.03,
            "floater {i} rides at {mean}, not {rest}"
        );
        assert!(
            least > 0.3 * rest,
            "floater {i} all but left the water: {least}"
        );
    }
}

/// Each sea records the og.Buoyancy hulls in it for the renderer, which
/// calms its render-only chop under them (not world state): a boat turned
/// about y is a rectangle of its own length and beam along its own axes,
/// at its place; a flat lake records none.
#[test]
fn a_sea_records_the_hulls_in_it_for_the_renderer() {
    let (mut w, p) = world(6);
    p.set_waters(vec![sea(DVec3::ZERO)]);
    let turn = 0.6f32;
    let e = floater(
        &mut w,
        Transform {
            quaternion: Quat::from_rotation_y(turn),
            ..at(4.0, 1.0, -30.0)
        },
        Vec3::new(2.0, 0.8, 5.0),
        0.5,
        Buoyancy {
            samples: 6,
            ..Buoyancy::default()
        },
    );
    run(&mut w, 3);
    let wakes = p
        .water_wakes(Entity::from_bits(1 << 40))
        .expect("the sea's hulls");
    let (_, cuts) = wakes.ticks.back().expect("a tick");
    assert_eq!(cuts.len(), 1);
    let c = cuts[0];
    assert_eq!(c.entity, e.to_bits());
    let t = transform(&w, e);
    assert!(
        c.centre.distance(Vec2::new(t.position.x, t.position.z)) < 0.2,
        "{:?} at {:?}",
        c.centre,
        t.position
    );
    let x = Quat::from_rotation_y(turn) * Vec3::X;
    assert!(c.axis.dot(Vec2::new(x.x, x.z)) > 0.99, "{:?}", c.axis);
    assert!(
        (c.half - Vec2::new(1.0, 2.5)).abs().max_element() < 0.15,
        "{:?}",
        c.half
    );
    assert!(c.min.x <= c.centre.x - c.half.x && c.max.y >= c.centre.y + c.half.x);

    let (mut w, p) = world(6);
    p.set_waters(vec![lake()]);
    boat(&mut w, 0.0);
    run(&mut w, 3);
    assert!(p.water_wakes(Entity::from_bits(1 << 40)).is_none());
}
