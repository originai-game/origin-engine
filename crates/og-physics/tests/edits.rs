//! The rapier world persists between ticks, so changes made to the
//! components outside physics have to reach it.
//!
//! Ways this could fail, each caught here: a despawned body keeps colliding
//! (a ghost holds up what rested on it); a teleport, a velocity set by game
//! code, a resized collider or a body turned static is ignored because the
//! rapier body keeps its own copy; a static collider moved by game code
//! stays where it was; a mesh re-registered with new geometry keeps the old
//! shape, or one re-registered unchanged disturbs the simulation; a restore
//! into a world that has run on keeps the newer rapier state.

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A unit box resting on the floor after a second, with its floor.
fn resting_box(seed: u32) -> (World, PhysicsSystem, Entity, Entity) {
    let (mut w, p) = world(Physics::default(), seed);
    let ground = floor(&mut w);
    let b = spawn_body(
        &mut w,
        at(0.0, 0.6, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 60);
    let y = transform(&w, b).position.y;
    assert!((y - 0.5).abs() < 0.02, "settled at {y}");
    (w, p, b, ground)
}

#[test]
fn a_despawned_body_stops_holding_up_what_rests_on_it() {
    let (mut w, _p) = world(Physics::default(), 11);
    floor(&mut w);
    let under = spawn_body(
        &mut w,
        at(0.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let top = spawn_body(
        &mut w,
        at(0.0, 1.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 60);
    assert!((transform(&w, top).position.y - 1.5).abs() < 0.03);
    w.despawn(under).unwrap();
    run(&mut w, 60);
    let y = transform(&w, top).position.y;
    assert!((y - 0.5).abs() < 0.03, "the top box fell to the floor: {y}");
}

#[test]
fn a_teleport_moves_the_body_and_it_falls_from_there() {
    let (mut w, _p, b, _) = resting_box(12);
    let mut t = transform(&w, b);
    t.position = Vec3::new(3.0, 6.0, 0.0);
    w.insert(b, t).unwrap();
    w.step().unwrap();
    let after = transform(&w, b).position;
    assert!((after.x - 3.0).abs() < 1e-3, "{after}");
    assert!(after.y < 6.0 && after.y > 5.9, "falling from 6 m: {after}");
    run(&mut w, 120);
    let landed = transform(&w, b).position;
    assert!(
        (landed.y - 0.5).abs() < 0.02 && (landed.x - 3.0).abs() < 0.05,
        "{landed}"
    );
}

#[test]
fn a_velocity_set_by_game_code_launches_the_body() {
    let (mut w, _p, b, _) = resting_box(13);
    w.insert(
        b,
        Velocity {
            linear: Vec3::new(0.0, 5.0, 0.0),
            angular: Vec3::ZERO,
        },
    )
    .unwrap();
    run(&mut w, 15);
    let y = transform(&w, b).position.y;
    // Up at 5 m/s against gravity for a quarter second: about 0.95 m higher.
    assert!(y > 1.2, "launched: {y}");
}

#[test]
fn a_resized_collider_and_a_body_made_static_take_effect() {
    let (mut w, _p, b, _) = resting_box(14);
    w.insert(b, Collider::cuboid(Vec3::splat(3.0))).unwrap();
    run(&mut w, 90);
    let y = transform(&w, b).position.y;
    assert!(
        (y - 1.5).abs() < 0.03,
        "the 3 m box rests on its new half-height: {y}"
    );

    // Static: lifted into the air, it stays there.
    let mut t = transform(&w, b);
    t.position.y = 4.0;
    w.insert(b, t).unwrap();
    w.insert(b, RigidBody::fixed()).unwrap();
    run(&mut w, 30);
    assert_eq!(transform(&w, b).position.y, 4.0);

    // A dynamic box dropped onto it lands on its top.
    let dropped = spawn_body(
        &mut w,
        at(0.0, 7.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 120);
    let y = transform(&w, dropped).position.y;
    assert!((y - 6.0).abs() < 0.03, "rests on the static box: {y}");
}

#[test]
fn a_static_collider_moved_by_game_code_moves_in_the_simulation() {
    let (mut w, _p, b, ground) = resting_box(15);
    let mut t = transform(&w, ground);
    t.position.y -= 2.0;
    w.insert(ground, t).unwrap();
    run(&mut w, 90);
    let y = transform(&w, b).position.y;
    assert!((y + 1.5).abs() < 0.03, "followed the floor down: {y}");
}

#[test]
fn a_mesh_registered_again_changes_its_colliders_only_when_its_geometry_changes() {
    fn cube(half: f32) -> MeshDesc {
        let mut positions = Vec::new();
        for i in 0..8 {
            let s = |bit: i32| if i & bit != 0 { half } else { -half };
            positions.push(Vec3::new(s(1), s(2), s(4)));
        }
        MeshDesc {
            positions,
            indices: Vec::new(),
        }
    }
    let build = || {
        let (mut w, p) = world(Physics::default(), 16);
        p.set_mesh(1, cube(0.5)).unwrap();
        floor(&mut w);
        let b = spawn_body(
            &mut w,
            at(0.0, 3.0, 0.0),
            RigidBody::dynamic(),
            Collider::convex(1),
        );
        run(&mut w, 30);
        (w, p, b)
    };
    let (mut a, _pa, _) = build();
    let (mut same, p_same, _) = build();
    p_same.set_mesh(1, cube(0.5)).unwrap();
    run(&mut a, 90);
    run(&mut same, 90);
    assert_eq!(same.snapshot(), a.snapshot(), "unchanged geometry");

    let (mut big, p_big, b) = build();
    p_big.set_mesh(1, cube(1.5)).unwrap();
    run(&mut big, 120);
    let y = transform(&big, b).position.y;
    assert!((y - 1.5).abs() < 0.03, "rests on the 3 m hull: {y}");
}

#[test]
fn a_restore_into_a_world_that_ran_on_continues_from_the_snapshot() {
    let (mut w, _p) = world(Physics::default(), 17);
    floor(&mut w);
    for i in 0..20 {
        spawn_body(
            &mut w,
            at((i % 4) as f32 * 0.7 - 1.0, 1.0 + i as f32 * 1.1, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.6)),
        );
    }
    run(&mut w, 30);
    let snap = w.snapshot();
    run(&mut w, 90);
    let end = w.snapshot();
    w.restore(&snap).unwrap();
    assert_eq!(w.snapshot(), snap);
    run(&mut w, 90);
    assert_eq!(
        w.snapshot(),
        end,
        "the rewound world replays the same 90 ticks"
    );
}
