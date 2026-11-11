//! The rapier world persists between ticks, so changes made to the
//! components outside physics have to reach it.
//!
//! Ways this could fail, each caught here: a despawned body keeps colliding
//! (a ghost holds up what rested on it); a teleport, a velocity set by game
//! code, a resized collider or a body turned static is ignored because the
//! rapier body keeps its own copy; a static collider moved by game code
//! stays where it was; a mesh re-registered with new geometry keeps the old
//! shape, or one re-registered unchanged disturbs the simulation; a restore
//! into a world that has run on keeps the newer rapier state; the step
//! reads a collider or a body as it decoded it a tick ago, after game code
//! wrote it, another took its row, or a restore put an older one there.

use crate::support;

use og_core::Reflect;
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

/// Beside some of the ledges below, so they are in a second archetype.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
struct Prop {}

/// The step keeps the colliders and bodies it decoded for as long as
/// nothing writes them. After each kind of write, the world that kept them
/// runs on as a world that never had any does: one built now and restored
/// from its snapshot. Every write here leaves each archetype's row count
/// where it was, so only the write itself can tell the step.
#[test]
fn what_the_step_kept_of_colliders_and_bodies_does_not_outlive_a_write() {
    let build = || {
        let (mut w, p) = world(Physics::default(), 18);
        w.register::<Prop>().unwrap();
        (w, p)
    };
    let (mut w, _p) = build();
    floor(&mut w);
    let ledge = Collider::cuboid(Vec3::new(2.0, 0.5, 2.0));
    let x = |i: usize| i as f32 * 3.0 - 10.5;
    let mut ledges = Vec::new();
    let mut boxes = Vec::new();
    for i in 0..8 {
        let t = at(x(i), 2.0, 0.0);
        ledges.push(if i % 2 == 0 {
            spawn_static(&mut w, t, ledge)
        } else {
            w.spawn((t, ledge, Prop {})).unwrap()
        });
        boxes.push(spawn_body(
            &mut w,
            at(x(i), 2.6, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.6)),
        ));
    }
    run(&mut w, 30);
    let settled = w.snapshot();
    let y = |w: &World, e: Entity| transform(w, e).position.y;
    assert!((y(&w, boxes[2]) - 2.55).abs() < 0.02, "on its ledge");

    let check = |w: &mut World, what: &str| {
        let (mut fresh, _p) = build();
        fresh.restore(&w.snapshot()).unwrap();
        run(w, 20);
        run(&mut fresh, 20);
        assert!(w.snapshot() == fresh.snapshot(), "after {what}");
    };
    check(&mut w, "nothing");

    // A static collider set in place: the box on it falls through it.
    w.insert(
        ledges[2],
        Collider {
            sensor: true,
            ..ledge
        },
    )
    .unwrap();
    check(&mut w, "a ledge was made a sensor");
    assert!(y(&w, boxes[2]) < 2.4, "fell through the sensor");

    // A static despawned (the last row takes its place) and another
    // spawned: the same number of rows, none where it was.
    w.despawn(ledges[0]).unwrap();
    spawn_static(&mut w, at(x(0), 4.0, 0.0), Collider::sphere(0.5));
    check(&mut w, "a ledge was replaced");

    // One static into the second archetype, one out of it.
    w.insert(ledges[4], Prop {}).unwrap();
    w.remove::<Prop>(ledges[1]).unwrap();
    check(&mut w, "two ledges changed archetype");

    // A body set in place, and its collider.
    w.insert(boxes[6], RigidBody::fixed()).unwrap();
    w.insert(boxes[5], Collider::sphere(0.3)).unwrap();
    check(&mut w, "a body and a body's collider were set");

    // A restore: every row count as now, the rows of 30 ticks in.
    w.restore(&settled).unwrap();
    check(&mut w, "a restore");
    assert!((y(&w, boxes[2]) - 2.55).abs() < 0.02, "on its ledge again");
}
