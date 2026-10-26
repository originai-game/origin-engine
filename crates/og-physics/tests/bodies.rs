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
use og_ecs::Entity;
use og_physics::*;
use support::*;

fn events_of<T: og_core::Reflect>(w: &og_ecs::World) -> Vec<T> {
    w.pending_events::<T>().unwrap()
}

/// Steps until `e` has been still for 30 consecutive ticks; returns the tick
/// it came to rest, or `None` after `limit` ticks.
fn settle(w: &mut og_ecs::World, e: Entity, limit: u32) -> Option<u32> {
    let mut still = 0;
    for tick in 0..limit {
        w.step().unwrap();
        let v = velocity(w, e);
        if v.linear.length() < 0.01 && v.angular.length() < 0.01 {
            still += 1;
            if still == 30 {
                return Some(tick + 1 - 30);
            }
        } else {
            still = 0;
        }
    }
    None
}

#[test]
fn a_dropped_box_comes_to_rest_on_a_static_floor_and_its_transform_is_written_back() {
    let (mut w, _p) = world(Physics::default(), 1);
    let ground = floor(&mut w);
    let crate_ = spawn_body(
        &mut w,
        at(0.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    assert!(!w.has::<Velocity>(crate_).unwrap());

    w.step().unwrap();
    // One tick in: it has started falling, and the step gave it a Velocity.
    let t = transform(&w, crate_);
    assert!(t.position.y < 5.0, "{t:?}");
    assert!(velocity(&w, crate_).linear.y < 0.0);

    // Free fall from 4.5 m above contact takes ~0.96 s (58 ticks); it must be
    // resting within two seconds.
    let rest = settle(&mut w, crate_, 300).expect("the box never came to rest");
    assert!(rest < 120, "came to rest at tick {rest}");
    let t = transform(&w, crate_);
    assert!(
        (t.position.y - 0.5).abs() < 0.02,
        "rests on the floor top: {t:?}"
    );
    // rapier's impact pushes a flat-landing box ~2 cm sideways; then it stays.
    assert!(
        t.position.x.abs() < 0.03 && t.position.z.abs() < 0.03,
        "{t:?}"
    );
    run(&mut w, 600);
    let later = transform(&w, crate_);
    assert_eq!(later, t, "a resting box does not creep");
    assert_eq!(velocity(&w, crate_), Velocity::default());
    assert!(t.quaternion.is_normalized());
    // The floor is static: still where it was.
    assert_eq!(transform(&w, ground).position, Vec3::new(0.0, -0.5, 0.0));
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
fn a_sensor_reports_entry_and_exit_and_does_not_stop_what_passes() {
    let (mut w, _p) = world(Physics::default(), 3);
    let gate = spawn_static(
        &mut w,
        at(0.0, 5.0, 0.0),
        Collider::cuboid(Vec3::new(4.0, 1.0, 4.0)).as_sensor(),
    );
    let ball = spawn_body(
        &mut w,
        at(0.0, 8.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.25),
    );

    let (mut entered, mut exited) = (Vec::new(), Vec::new());
    let (mut entered_at, mut exited_at) = (None, None);
    for tick in 0..120 {
        w.step().unwrap();
        let e = events_of::<SensorEntered>(&w);
        let x = events_of::<SensorExited>(&w);
        if !e.is_empty() {
            entered_at.get_or_insert(tick);
        }
        if !x.is_empty() {
            exited_at.get_or_insert(tick);
        }
        entered.extend(e);
        exited.extend(x);
        assert!(
            events_of::<CollisionStarted>(&w).is_empty(),
            "a sensor is not a collision"
        );
    }
    let want_in = SensorEntered {
        sensor: gate.to_bits(),
        other: ball.to_bits(),
    };
    let want_out = SensorExited {
        sensor: gate.to_bits(),
        other: ball.to_bits(),
    };
    assert_eq!(entered, vec![want_in]);
    assert_eq!(exited, vec![want_out]);
    assert!(entered_at < exited_at, "{entered_at:?} {exited_at:?}");
    // It fell straight through: 8 m less two seconds of free fall.
    assert!(transform(&w, ball).position.y < 0.0);
}

#[test]
fn layers_filter_collisions() {
    let (mut w, _p) = world(Physics::default(), 4);
    // The floor is on layer 1 and collides only with layer 1.
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(40.0, 1.0, 40.0)).on_layers(1, 1),
    );
    let kept = spawn_body(
        &mut w,
        at(-2.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    );
    let ghost = spawn_body(
        &mut w,
        at(2.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5).on_layers(2, u32::MAX),
    );
    // A third body that collides with everything except layer 1.
    let deaf = spawn_body(
        &mut w,
        at(0.0, 1.0, 3.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5).on_layers(1, !1),
    );
    run(&mut w, 120);
    assert!((transform(&w, kept).position.y - 0.5).abs() < 0.02);
    assert!(
        transform(&w, ghost).position.y < -5.0,
        "layer 2 is not in the floor's mask"
    );
    assert!(
        transform(&w, deaf).position.y < -5.0,
        "the floor is not in its mask"
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

#[test]
fn kinematic_bodies_without_a_velocity_follow_their_transform() {
    let (mut w, _p) = world(Physics::default(), 5);
    floor(&mut w);
    let pusher = w
        .spawn((
            at(-3.0, 0.5, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(1.0, 1.0, 4.0)),
        ))
        .unwrap();
    let crate_ = spawn_body(
        &mut w,
        at(0.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let dt = 1.0 / 60.0;
    for i in 1..=120 {
        // Game code moves the kinematic body; physics only reads it.
        let mut t = transform(&w, pusher);
        t.position.x = -3.0 + 2.0 * dt * i as f32;
        w.insert(pusher, t).unwrap();
        w.step().unwrap();
        assert_eq!(transform(&w, pusher).position, t.position);
    }
    // The pusher's face reached x = 0.5; the crate is ahead of it.
    assert!(
        transform(&w, crate_).position.x > 0.9,
        "{:?}",
        transform(&w, crate_)
    );
}

#[test]
fn locked_axes_and_gravity_scale_hold() {
    let (mut w, _p) = world(Physics::default(), 6);
    let floating = spawn_body(
        &mut w,
        at(0.0, 3.0, 0.0),
        RigidBody {
            gravity_scale: 0.0,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.5),
    );
    let rail = w
        .spawn((
            at(5.0, 3.0, 0.0),
            RigidBody {
                lock: LOCK_TRANSLATION_Y | LOCK_ROTATION_X | LOCK_ROTATION_Y | LOCK_ROTATION_Z,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::ONE),
            Velocity {
                linear: Vec3::new(1.0, 0.0, 0.0),
                angular: Vec3::new(0.0, 3.0, 0.0),
            },
        ))
        .unwrap();
    run(&mut w, 60);
    assert_eq!(transform(&w, floating).position, Vec3::new(0.0, 3.0, 0.0));
    let t = transform(&w, rail);
    assert_eq!(t.position.y, 3.0);
    assert!((t.position.x - 6.0).abs() < 1e-3, "{t:?}");
    assert_eq!(t.quaternion, og_core::glam::Quat::IDENTITY);
}

#[test]
fn a_broken_collider_stops_the_tick_with_the_entity_named() {
    let (mut w, _p) = world(Physics::default(), 7);
    let e = spawn_body(
        &mut w,
        at(0.0, 0.0, 0.0),
        RigidBody::dynamic(),
        Collider {
            radius: -1.0,
            ..Collider::sphere(1.0)
        },
    );
    let err = w.step().unwrap_err();
    assert!(err.reason().contains(&e.to_string()), "{}", err.reason());
    assert!(err.reason().contains("radius"), "{}", err.reason());
    let missing = spawn_body(
        &mut w,
        at(0.0, 0.0, 0.0),
        RigidBody::dynamic(),
        Collider::convex(99),
    );
    w.despawn(e).unwrap();
    let err = w.step().unwrap_err();
    assert!(err.reason().contains("mesh 99"), "{}", err.reason());
    assert!(
        err.reason().contains(&missing.to_string()),
        "{}",
        err.reason()
    );
}
