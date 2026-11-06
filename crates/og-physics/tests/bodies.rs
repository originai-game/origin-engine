//! 3D rigid bodies through the world: what the physics step writes back,
//! the events it sends and how layers filter.
//!
//! Ways this could fail: the transform is never written back (the box stays
//! at its spawn height); it is written back wrongly (the box sinks through
//! the floor or hovers); velocity is not carried between ticks (the box
//! falls at constant speed or never settles); static or kinematic bodies are
//! moved by the solver; a despawned collider keeps colliding; Started or
//! Ended is sent every tick, or never; a sensor blocks what enters it, or
//! reports the wrong entity; layers are ignored.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use support::*;

fn events_of<T: og_core::Reflect>(w: &og_ecs::World) -> Vec<T> {
    w.pending_events::<T>().unwrap()
}

#[test]
fn touching_starts_once_ends_once_and_a_removed_floor_stops_holding_the_box() {
    let (mut w, _p) = world(Physics::default(), 2);
    let ground = floor(&mut w);
    let crate_ = spawn_body(
        &mut w,
        at(0.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let (a, b) = if ground.to_bits() < crate_.to_bits() {
        (ground, crate_)
    } else {
        (crate_, ground)
    };

    let mut started = Vec::new();
    for _ in 0..60 {
        w.step().unwrap();
        started.extend(events_of::<CollisionStarted>(&w));
        assert!(events_of::<CollisionEnded>(&w).is_empty());
    }
    assert_eq!(started.len(), 1, "{started:?}");
    let s = started[0];
    assert_eq!((s.a, s.b), (a.to_bits(), b.to_bits()));
    // The normal points from a to b: up if the floor is a.
    let up = if a == ground { 1.0 } else { -1.0 };
    assert!((s.normal.y - up).abs() < 1e-3, "{s:?}");
    assert!(
        s.point.y.abs() < 0.05,
        "the contact is at the floor top: {s:?}"
    );
    // Every touching pair is also reported as an og.Contact each tick.
    assert_eq!(
        events_of::<Contact>(&w),
        vec![Contact {
            a: a.to_bits(),
            b: b.to_bits(),
            sensor: 0
        }]
    );

    // The floor vanishes: the pair ends once and the box falls again.
    w.despawn(ground).unwrap();
    w.step().unwrap();
    assert_eq!(
        events_of::<CollisionEnded>(&w),
        vec![CollisionEnded {
            a: a.to_bits(),
            b: b.to_bits()
        }]
    );
    let y = transform(&w, crate_).position.y;
    run(&mut w, 30);
    assert!(events_of::<CollisionEnded>(&w).is_empty());
    assert!(
        transform(&w, crate_).position.y < y - 1.0,
        "the box fell through where the floor was"
    );
}

#[test]
fn kinematic_bodies_move_by_their_velocity_and_push_dynamic_ones() {
    let (mut w, _p) = world(Physics::default(), 5);
    floor(&mut w);
    let pusher = w
        .spawn((
            at(-3.0, 0.5, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(1.0, 1.0, 4.0)),
            Velocity {
                linear: Vec3::new(2.0, 0.0, 0.0),
                angular: Vec3::ZERO,
            },
        ))
        .unwrap();
    let crate_ = spawn_body(
        &mut w,
        at(0.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    // Nothing moves it but its velocity: 2 m/s, written back each tick.
    for i in 1..=120 {
        w.step().unwrap();
        let x = transform(&w, pusher).position.x;
        assert!(
            (x - (-3.0 + 2.0 * i as f32 / 60.0)).abs() < 1e-4,
            "tick {i}: x {x}"
        );
        assert_eq!(velocity(&w, pusher).linear, Vec3::new(2.0, 0.0, 0.0));
    }
    // The pusher's face reached x = 1.5; the crate is ahead of it.
    assert!(
        transform(&w, crate_).position.x > 1.9,
        "{:?}",
        transform(&w, crate_)
    );
    // A Transform game code writes moves it there (a teleport), and it goes
    // on from there; with its velocity zero it stays.
    w.insert(pusher, at(-6.0, 0.5, 0.0)).unwrap();
    w.step().unwrap();
    let x = transform(&w, pusher).position.x;
    assert!(
        (x - (-6.0 + 2.0 / 60.0)).abs() < 1e-4,
        "after the teleport: {x}"
    );
    w.insert(pusher, Velocity::default()).unwrap();
    run(&mut w, 10);
    assert_eq!(transform(&w, pusher).position.x, x);
}
