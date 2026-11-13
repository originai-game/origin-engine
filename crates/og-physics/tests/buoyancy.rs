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

use og_core::glam::{DVec3, Quat, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_env::ocean::Ocean;
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
        sea: None,
        zero: DVec3::ZERO,
        shore: None,
        river: None,
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

#[test]
fn floaters_stay_in_the_water_through_the_gales_crests() {
    // The boats sample (the former test sample three box boats and
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

/// Hips, a 0.5 m spine and a 0.3 m neck, straight up from the entity.
fn rig() -> RagdollRig {
    let bone = |parent: Option<u32>, y: f32, length: f32| RagdollBone {
        parent,
        rest: (Vec3::new(0.0, y, 0.0), Quat::IDENTITY),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, 0.0, 0.3),
            bone(Some(0), 0.3, 0.5),
            bone(Some(1), 0.8, 0.3),
        ],
    }
}

/// A limp ragdoll dropped from `y` over the lake at `x`.
fn limp(w: &mut World, p: &PhysicsSystem, x: f32, y: f32) -> Entity {
    let ragdoll = Ragdoll {
        weight: 1.0,
        ..Ragdoll::default()
    };
    let e = w.spawn((at(x, y, 0.0), ragdoll)).unwrap();
    p.set_ragdoll_rig(e, rig()).unwrap();
    e
}

fn mean_height(p: &PhysicsSystem, e: Entity) -> f32 {
    let pose = p.ragdoll_pose(e).unwrap();
    pose.iter().map(|b| b.0.y).sum::<f32>() / pose.len() as f32
}

#[test]
fn a_limp_ragdoll_with_buoyancy_floats_half_under_and_splashes_and_one_without_sinks() {
    let (mut w, p) = world(5);
    p.set_waters(vec![lake()]);
    let floats = limp(&mut w, &p, 0.0, 1.5);
    w.insert(floats, Buoyancy::default()).unwrap();
    let sinks = limp(&mut w, &p, 6.0, 1.5);
    let mut splashed = Vec::new();
    for _ in 0..900 {
        w.step().unwrap();
        splashed.extend(w.pending_events::<Splash>().unwrap());
    }
    // Water of density 2 holds half its mass under, as it does a crate's
    // volume (a bone's thin capsule displaces its mass, not its volume:
    // by volume a 60 kg ragdoll would sink like the one beside it).
    let b = w.get::<Buoyancy>(floats).unwrap();
    assert!((b.submerged - 0.5).abs() < 0.08, "{}", b.submerged);
    let y = mean_height(&p, floats);
    assert!(y.abs() < 0.3, "it lies at the surface: {y}");
    let pose = p.ragdoll_pose(floats).unwrap();
    assert!(
        (pose[0].0 - pose[1].0).length() < 0.35 && (pose[1].0 - pose[2].0).length() < 0.55,
        "its joints hold: {pose:?}"
    );
    // The water slowed it to rest.
    w.step().unwrap();
    assert!((mean_height(&p, floats) - y).abs() < 2e-3);
    assert!(
        mean_height(&p, sinks) < -20.0,
        "no og.Buoyancy, no lift: {}",
        mean_height(&p, sinks)
    );
    // Going in is an og.Splash of the ragdoll's entity; the one without
    // og.Buoyancy crosses in silence, as it did (its saves are unchanged).
    let of = |e: Entity| splashed.iter().filter(|s| s.entity == e.to_bits()).count();
    assert!(of(floats) >= 1, "{splashed:?}");
    assert_eq!(of(sinks), 0);
}

/// A capsule character (0.4 radius, 1.8 m tall) with its middle at `y`.
fn swimmer(w: &mut World, x: f32, y: f32, b: Option<Buoyancy>) -> Entity {
    let e = w
        .spawn((
            at(x, y, 0.0),
            Collider::capsule(0.4, 1.0),
            CharacterController::default(),
        ))
        .unwrap();
    if let Some(b) = b {
        w.insert(e, b).unwrap();
    }
    e
}

#[test]
fn a_character_with_buoyancy_floats_sinks_by_its_density_and_drifts_with_a_river() {
    let (mut w, p) = world(7);
    // A river 12 m wide flowing 2 m/s along +x at y 0, 40 m off the lake's
    // swimmers in z.
    let course = [Vec3::new(-300.0, 0.0, 40.0), Vec3::new(300.0, 0.0, 40.0)];
    let river = WaterBody {
        entity: 2 << 40,
        position: Vec3::new(0.0, 0.0, 40.0),
        size: Vec2::new(600.0, 12.0),
        river: og_env::river::River::new(&course, 12.0, 2.0).map(std::sync::Arc::new),
        ..lake()
    };
    assert!(river.river.is_some());
    let lake = WaterBody {
        size: Vec2::new(200.0, 40.0),
        ..lake()
    };
    p.set_waters(vec![lake, river]);
    let floats = swimmer(&mut w, 0.0, 3.0, Some(Buoyancy::default()));
    let heavy = swimmer(
        &mut w,
        4.0,
        3.0,
        Some(Buoyancy {
            water_density: 0.5,
            ..Buoyancy::default()
        }),
    );
    let plain = swimmer(&mut w, 8.0, 3.0, None);
    let drifts = w
        .spawn((
            at(0.0, 0.0, 40.0),
            Collider::capsule(0.4, 1.0),
            CharacterController::default(),
            Buoyancy::default(),
        ))
        .unwrap();
    run(&mut w, 600);
    let y = |w: &World, e: Entity| transform(w, e).position.y;
    // Density 2 against its 1: half of its 1.8 m under, its middle at the
    // surface, at rest, and not on any ground.
    let b = w.get::<Buoyancy>(floats).unwrap();
    assert!((b.submerged - 0.5).abs() < 0.02, "{}", b.submerged);
    assert!(y(&w, floats).abs() < 0.04, "{}", y(&w, floats));
    let c = w.get::<CharacterController>(floats).unwrap();
    assert!(
        !c.grounded && c.velocity.length() < 0.02,
        "{:?}",
        c.velocity
    );
    // Water lighter than it lifts half its weight: it sinks, slower than
    // the one with no og.Buoyancy falls, which the water never touches.
    let (h, f) = (y(&w, heavy), y(&w, plain));
    assert!(h < -5.0 && f < h - 20.0, "heavy {h}, plain {f}");
    assert_eq!(w.get::<Buoyancy>(heavy).unwrap().submerged, 1.0);
    // The river carries the one afloat in it downstream at its current,
    // as it does a floating crate: 2 m/s for 10 s.
    let t = transform(&w, drifts).position;
    assert!((t.x - 20.0).abs() < 0.5 && (t.z - 40.0).abs() < 0.1, "{t}");
    assert!(y(&w, drifts).abs() < 0.04);
}

#[test]
fn a_swimmer_and_a_floating_ragdoll_run_the_same_twice_and_through_a_restore() {
    let build = || {
        let (mut w, p) = world(11);
        p.set_waters(vec![sea(DVec3::ZERO)]);
        let c = swimmer(&mut w, 2.0, 2.0, Some(Buoyancy::default()));
        let r = limp(&mut w, &p, -3.0, 2.0);
        w.insert(r, Buoyancy::default()).unwrap();
        (w, p, c, r)
    };
    let state = |w: &World, p: &PhysicsSystem, c: Entity, r: Entity| {
        let t = transform(w, c).position;
        let mut bits = vec![t.x.to_bits(), t.y.to_bits(), t.z.to_bits()];
        for (at, _) in p.ragdoll_pose(r).unwrap() {
            bits.extend([at.x.to_bits(), at.y.to_bits(), at.z.to_bits()]);
        }
        bits.push(w.get::<Buoyancy>(c).unwrap().submerged.to_bits());
        bits.push(w.get::<Buoyancy>(r).unwrap().submerged.to_bits());
        bits
    };
    let (mut a, pa, c, r) = build();
    run(&mut a, 120);
    let saved = a.snapshot();
    run(&mut a, 240);
    let first = state(&a, &pa, c, r);
    // In a gale's sea they ride the waves: in the water, not flung or sunk.
    let t = transform(&a, c).position;
    assert!(t.y.abs() < 8.0, "{t}");
    assert!(mean_height(&pa, r).abs() < 8.0);
    let (mut b, pb, ..) = build();
    run(&mut b, 360);
    assert_eq!(state(&b, &pb, c, r), first);
    let (mut d, pd, ..) = build();
    d.restore(&saved).unwrap();
    run(&mut d, 240);
    assert_eq!(state(&d, &pd, c, r), first);
}
