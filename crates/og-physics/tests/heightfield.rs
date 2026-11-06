//! Height-field colliders (phase 2's terrain): `SHAPE_HEIGHTFIELD` over a
//! field registered with `PhysicsSystem::set_heightfield`.
//!
//! Ways this could fail, each caught below: the heights are transposed
//! (rows read as columns), mirrored or scaled wrongly; a cell is split
//! along the other diagonal than the renderer draws (a raised corner then
//! answers a different height between samples); the field is not centred
//! on its entity; an edit of the heights is ignored (the stale collider
//! stays); a character cannot walk up the field's slopes or sinks into
//! it; a dynamic body falls through; a missing field or a bad one is
//! accepted silently; the 2D profile takes it.

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::World;
use og_physics::*;
use std::sync::Arc;
use support::*;

/// A field `columns` × `rows` over `size`, heights from `f(x, z)` at the
/// samples (x and z relative to the centre).
fn field(columns: u32, rows: u32, size: Vec2, f: impl Fn(f32, f32) -> f32) -> HeightfieldDesc {
    let mut heights = Vec::new();
    for r in 0..rows {
        for c in 0..columns {
            let x = -size.x * 0.5 + c as f32 * size.x / (columns - 1) as f32;
            let z = -size.y * 0.5 + r as f32 * size.y / (rows - 1) as f32;
            heights.push(f(x, z));
        }
    }
    HeightfieldDesc {
        columns,
        rows,
        size,
        heights: Arc::from(heights),
        holes: None,
    }
}

fn terrain_collider(handle: u64) -> Collider {
    Collider {
        shape: SHAPE_HEIGHTFIELD,
        mesh: handle,
        ..Collider::default()
    }
}

fn down(p: &PhysicsSystem, w: &World, x: f32, z: f32) -> Option<RaycastHit> {
    p.query(w)
        .unwrap()
        .raycast(&RaycastDesc {
            origin: Vec3::new(x, 100.0, z),
            direction: Vec3::NEG_Y,
            ..Default::default()
        })
        .unwrap()
}

/// A character walks up a 15 degree slope of a field and stays on it (it
/// is grounded and never below the surface); a ball dropped on it comes to
/// rest on it.
#[test]
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn a_character_walks_up_a_field_and_a_ball_rests_on_it() {
    let (mut w, p) = world(Physics::default(), 1);
    let slope = 15f32.to_radians().tan();
    let surface = |x: f32| (x + 20.0).max(0.0) * slope;
    p.set_heightfield(1, field(81, 41, Vec2::new(80.0, 40.0), |x, _| surface(x)))
        .unwrap();
    spawn_static(&mut w, at(0.0, 0.0, 0.0), terrain_collider(1));
    let c = w
        .spawn((
            at(-25.0, 1.0, 0.0),
            Collider::capsule(0.4, 1.0),
            CharacterController::default(),
        ))
        .unwrap();
    run(&mut w, 20);
    let mut cc = w.get::<CharacterController>(c).unwrap();
    cc.movement = Vec3::new(3.0, 0.0, 0.0);
    w.insert(c, cc).unwrap();
    let mut lowest = f32::MAX;
    for _ in 0..240 {
        w.step().unwrap();
        let t = transform(&w, c).position;
        lowest = lowest.min(t.y - 0.9 - surface(t.x));
    }
    let t = transform(&w, c).position;
    assert!(t.x > -15.0, "it walked up the slope to {t}");
    assert!(
        t.y - 0.9 > surface(t.x) - 0.1 && t.y - 0.9 < surface(t.x) + 0.15,
        "{t}"
    );
    assert!(lowest > -0.1, "never below the surface ({lowest})");
    assert!(w.get::<CharacterController>(c).unwrap().grounded);

    let ball = spawn_body(
        &mut w,
        at(10.0, 30.0, 5.0),
        RigidBody::default(),
        Collider::sphere(0.5),
    );
    run(&mut w, 300);
    let b = transform(&w, ball).position;
    assert!(
        b.y > surface(b.x) + 0.3 && b.y < surface(b.x) + 0.7,
        "the ball on the field at {b} (surface {})",
        surface(b.x)
    );
}

/// A field of more than one chunk (256 cells a side) is built as one
/// collider per chunk: rays answer the same surface on both sides of a
/// chunk's edge, and a region edit rebuilds only the chunks under it. A
/// crate asleep on a chunk the edit does not reach stays asleep (its
/// ground was not rebuilt, so rapier kept its contact); the crate over the
/// edit falls onto the lowered ground.
#[test]
fn an_edit_rebuilds_only_the_chunks_of_a_large_field_under_it() {
    let settings = Physics {
        sleep_ticks: 20,
        ..Physics::default()
    };
    let (mut w, p) = world(settings, 1);
    // 601 × 601 samples, 0.5 m apart: 3 × 3 chunks, their edges at
    // samples 256 and 512 (x and z of -22 m and 106 m).
    let size = Vec2::splat(300.0);
    let slope = |x: f32, z: f32| 0.05 * x + 0.03 * z;
    let f = field(601, 601, size, slope);
    p.set_heightfield(9, f.clone()).unwrap();
    spawn_static(&mut w, at(0.0, 0.0, 0.0), terrain_collider(9));
    run(&mut w, 1);
    for (x, z) in [
        (-22.0f32, 3.3f32),
        (-22.25, 3.3),
        (-21.75, 3.3),
        (106.0, 106.0),
        (-140.2, 140.7),
        (5.0, -22.0),
    ] {
        let y = down(&p, &w, x, z).expect("the field").point.y;
        assert!((y - slope(x, z)).abs() < 1e-3, "{x},{z}: {y}");
    }
    // A crate on the far chunk, one over the middle chunk's ground.
    let crate_at = |w: &mut World, x: f32, z: f32| {
        spawn_body(
            w,
            at(x, slope(x, z) + 0.6, z),
            RigidBody::default(),
            Collider::cuboid(Vec3::splat(1.0)),
        )
    };
    let far = crate_at(&mut w, -120.0, -120.0);
    let near = crate_at(&mut w, 40.0, 40.0);
    run(&mut w, 120);
    let sleeping = |w: &World, e| w.get::<RigidBody>(e).unwrap().sleeping;
    assert!(
        sleeping(&w, far) && sleeping(&w, near),
        "both crates came to rest asleep"
    );
    let rested = transform(&w, far).position;

    // Lower a 4 m square under the near crate by 3 m.
    let mut heights = f.heights.to_vec();
    for r in 0..601usize {
        for c in 0..601usize {
            let (x, z) = (c as f32 * 0.5 - 150.0, r as f32 * 0.5 - 150.0);
            if (x - 40.0).abs() <= 2.0 && (z - 40.0).abs() <= 2.0 {
                heights[r * 601 + c] -= 3.0;
            }
        }
    }
    let edited = HeightfieldDesc {
        heights: Arc::from(heights),
        ..f
    };
    p.edit_heightfield(9, edited, Vec2::new(38.0, 38.0), Vec2::new(42.0, 42.0))
        .unwrap();
    run(&mut w, 1);
    assert!(
        sleeping(&w, far),
        "the far crate's ground was rebuilt: it woke"
    );
    assert!(
        !sleeping(&w, near),
        "the edited ground woke the crate on it"
    );
    // Beside the crate, inside the lowered square.
    let y = down(&p, &w, 41.5, 41.5).unwrap().point.y;
    assert!(
        (y - (slope(41.5, 41.5) - 3.0)).abs() < 1e-3,
        "the edit: {y}"
    );
    run(&mut w, 120);
    let fell = transform(&w, near).position.y;
    assert!(
        (fell - (slope(40.0, 40.0) - 3.0 + 0.5)).abs() < 0.2,
        "the near crate rests on the lowered ground at {fell}"
    );
    assert_eq!(transform(&w, far).position, rested);
}

/// A snapshot of a field in chunks carries each chunk by its key, not its
/// heights, and restores to the same world.
#[test]
fn a_field_in_chunks_saves_by_its_asset_and_restores() {
    let (mut w, p) = world(Physics::default(), 1);
    let f = field(301, 513, Vec2::new(150.0, 256.0), |x, z| {
        0.02 * x - 0.04 * z
    });
    p.set_heightfield(5, f.clone()).unwrap();
    spawn_static(&mut w, at(0.0, 1.0, 0.0), terrain_collider(5));
    let ball = spawn_body(
        &mut w,
        at(0.0, 4.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    run(&mut w, 30);
    let saved = w.snapshot();
    let hash = w.world_hash();
    // 301 × 513 heights are 600 KiB: the save holds none of them.
    assert!(
        saved.len() < 64 * 1024,
        "the snapshot is {} bytes",
        saved.len()
    );
    run(&mut w, 60);

    let (mut r, rp) = world(Physics::default(), 2);
    rp.set_heightfield(5, f).unwrap();
    r.restore(&saved).unwrap();
    assert_eq!(r.world_hash(), hash);
    assert_eq!(r.snapshot(), saved, "a restored world saves the same bytes");
    run(&mut r, 60);
    assert_eq!(r.world_hash(), w.world_hash());
    let y = transform(&r, ball).position.y;
    assert!(y > -10.0 && y < 4.0, "the ball is on the field at {y}");
}
