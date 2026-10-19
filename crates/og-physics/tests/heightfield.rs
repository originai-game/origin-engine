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

mod support;

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

/// One raised corner: the cell (0, 0) to (1, 1) has heights 0 except 4 at
/// its +x, +z corner. Split along that diagonal, the point a quarter
/// along z and three quarters along x stands 1 m high (the triangle
/// (0,0), (1,0), (1,1)); the other diagonal would give 0.
#[test]
fn heights_are_rows_along_z_split_along_the_renderers_diagonal() {
    let (mut w, p) = world(Physics::default(), 1);
    // 3 × 3 samples over 2 × 2 m centred at (10, 0, -5): cell (0,0) spans
    // x 9..10, z -6..-5; its +x, +z corner is the field's centre sample.
    let f = field(3, 3, Vec2::splat(2.0), |x, z| {
        if x.abs() < 1e-4 && z.abs() < 1e-4 {
            4.0
        } else {
            0.0
        }
    });
    p.set_heightfield(7, f).unwrap();
    spawn_static(&mut w, at(10.0, 0.0, -5.0), terrain_collider(7));
    run(&mut w, 1);
    let hit = down(&p, &w, 9.75, -5.75).expect("the field");
    assert!((hit.point.y - 1.0).abs() < 1e-3, "{hit:?}");
    // Mirrored across the diagonal: (x 1/4, z 3/4) of the cell also 1 m.
    let hit = down(&p, &w, 9.25, -5.25).expect("the field");
    assert!((hit.point.y - 1.0).abs() < 1e-3, "{hit:?}");
    // The raised corner itself, and a sample of 0 three cells away.
    assert!((down(&p, &w, 10.0, -5.0).unwrap().point.y - 4.0).abs() < 1e-3);
    assert!(down(&p, &w, 9.0, -6.0).unwrap().point.y.abs() < 1e-3);
    // Outside the field: nothing.
    assert!(down(&p, &w, 12.5, -5.0).is_none());
}

/// A sloped field rising along +x (0.25 m per m) and falling along +z:
/// rays hit the plane everywhere; an edit to flat heights moves the hit.
#[test]
fn the_surface_follows_the_field_and_its_edits() {
    let (mut w, p) = world(Physics::default(), 1);
    let size = Vec2::new(40.0, 20.0);
    p.set_heightfield(3, field(41, 21, size, |x, z| 0.25 * x - 0.1 * z))
        .unwrap();
    spawn_static(&mut w, at(0.0, 2.0, 0.0), terrain_collider(3));
    run(&mut w, 1);
    for (x, z) in [(0.0f32, 0.0f32), (7.3, -2.2), (-15.5, 9.1)] {
        let y = down(&p, &w, x, z).expect("hit").point.y;
        assert!(
            (y - (2.0 + 0.25 * x - 0.1 * z)).abs() < 1e-3,
            "{x},{z}: {y}"
        );
    }
    // A crate resting on the slope (friction holds it).
    let crate_ = spawn_body(
        &mut w,
        at(-10.0, 2.0 - 2.5 + 0.2 + 0.6, 0.0),
        RigidBody::default(),
        Collider::cuboid(Vec3::splat(1.0)),
    );
    run(&mut w, 120);
    let before = transform(&w, crate_).position.y;
    assert!(
        (before - (2.0 - 2.5 + 0.5)).abs() < 0.15,
        "resting at {before}"
    );
    // The edit: the field flat at -3 m (below the crate). The query sees
    // it at once, and the simulation's collider is rebuilt: the crate
    // drops onto it.
    p.set_heightfield(3, field(41, 21, size, |_, _| -3.0))
        .unwrap();
    run(&mut w, 1);
    let y = down(&p, &w, 7.3, -2.2).unwrap().point.y;
    assert!((y + 1.0).abs() < 1e-3, "after the edit: {y}");
    run(&mut w, 120);
    let after = transform(&w, crate_).position.y;
    assert!(
        (after + 0.5).abs() < 0.1,
        "the crate on the edited field at {after}"
    );
}

/// A character walks up a 15 degree slope of a field and stays on it (it
/// is grounded and never below the surface); a ball dropped on it comes to
/// rest on it.
#[test]
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

#[test]
fn bad_or_missing_fields_are_refused() {
    let (mut w, p) = world(Physics::default(), 1);
    assert!(
        p.set_heightfield(
            1,
            HeightfieldDesc {
                columns: 3,
                rows: 3,
                size: Vec2::ONE,
                heights: Arc::from(vec![0.0; 8]),
            }
        )
        .is_err()
    );
    assert!(
        p.set_heightfield(1, field(1, 5, Vec2::ONE, |_, _| 0.0))
            .is_err()
    );
    assert!(
        p.set_heightfield(1, field(2, 2, Vec2::ONE, |_, _| f32::NAN))
            .is_err()
    );
    // A collider naming nothing registered fails the step, naming the fix.
    spawn_static(&mut w, at(0.0, 0.0, 0.0), terrain_collider(99));
    let e = w.step().unwrap_err();
    assert!(e.to_string().contains("set_heightfield"), "{e}");

    let (mut w2, p2) = world(
        Physics {
            profile: 2,
            ..Physics::default()
        },
        1,
    );
    p2.set_heightfield(1, field(2, 2, Vec2::ONE, |_, _| 0.0))
        .unwrap();
    spawn_static(&mut w2, at(0.0, 0.0, 0.0), terrain_collider(1));
    let e = w2.step().unwrap_err();
    assert!(e.to_string().contains("3D only"), "{e}");
}
