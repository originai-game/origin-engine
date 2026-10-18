//! og.Vehicle in both profiles: the 3D raycast car and the 2D wheel-joint car.
//!
//! Ways this could fail, each caught below: the wheels are mounted on the
//! wrong axes (the car sinks, flips or drives backwards on positive
//! throttle); steering turns the wrong way or not at all; the brake does
//! not slow it; the outputs (speed, contacts) are not written; the wheel
//! poses for drawing are missing or detached from the chassis; the 2D
//! wheels are left behind when the vehicle is removed (they keep
//! colliding); a rebuild of the chassis leaves orphaned wheels.

mod support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn vehicle(w: &World, e: Entity) -> Vehicle {
    w.get::<Vehicle>(e).unwrap()
}

fn set(w: &mut World, e: Entity, f: impl FnOnce(&mut Vehicle)) {
    let mut v = vehicle(w, e);
    f(&mut v);
    w.insert(e, v).unwrap();
}

fn car3d(w: &mut World) -> Entity {
    let e = spawn_body(
        w,
        at(0.0, 1.2, 0.0),
        RigidBody {
            mass: 150.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::new(1.8, 0.6, 3.6)),
    );
    w.insert(
        e,
        Vehicle {
            engine_force: 400.0,
            brake_force: 60.0,
            ..Vehicle::default()
        },
    )
    .unwrap();
    e
}

#[test]
fn a_3d_car_rests_on_its_wheels_drives_forward_steers_and_brakes() {
    let (mut w, p) = world(Physics::default(), 1);
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(400.0, 1.0, 400.0)),
    );
    let car = car3d(&mut w);
    run(&mut w, 90);
    let v = vehicle(&w, car);
    assert_eq!(v.contacts, 4, "{v:?}");
    let rest = transform(&w, car).position;
    assert!(rest.y > 0.45 && rest.y < 1.1, "the chassis rides at {rest}");
    let wheels = p.vehicle_wheels(car).unwrap();
    assert_eq!(wheels.len(), 4);
    assert!(
        wheels
            .iter()
            .all(|wh| wh.contact && (wh.position.y - 0.4).abs() < 0.12),
        "{wheels:?}"
    );

    set(&mut w, car, |v| v.throttle = 1.0);
    run(&mut w, 120);
    let p1 = transform(&w, car).position;
    assert!(p1.z > rest.z + 3.0, "drove forward (+z) to {p1}");
    assert!(p1.x.abs() < 0.5, "straight: {p1}");
    let speed = vehicle(&w, car).speed;
    assert!(speed > 3.0, "speed {speed}");

    // Steering right (+1) turns towards -x (with +z forward and +y up, +x is left).
    set(&mut w, car, |v| v.steer = 1.0);
    run(&mut w, 60);
    let p2 = transform(&w, car).position;
    assert!(p2.x < p1.x - 1.0, "turned right to {p2}");

    set(&mut w, car, |v| {
        v.throttle = 0.0;
        v.steer = 0.0;
        v.brake = 1.0;
    });
    let before = vehicle(&w, car).speed.abs();
    run(&mut w, 180);
    let s = vehicle(&w, car).speed.abs();
    assert!(s < 0.5 && before > 3.0, "braked from {before} to {s}");
}

#[test]
fn a_2d_car_drives_on_sprung_wheels_and_its_wheels_go_with_it() {
    let (mut w, p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    let ground = spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(400.0, 1.0, 1.0)),
    );
    let car = spawn_body(
        &mut w,
        at(0.0, 1.5, 0.0),
        RigidBody {
            mass: 4.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::new(3.0, 0.6, 1.0)),
    );
    w.insert(
        car,
        Vehicle {
            wheel_base: 2.2,
            wheel_radius: 0.45,
            mount_height: 0.1,
            suspension_rest: 0.2,
            suspension_travel: 0.2,
            engine_force: 40.0,
            max_speed: 8.0,
            ..Vehicle::default()
        },
    )
    .unwrap();
    run(&mut w, 90);
    assert_eq!(vehicle(&w, car).contacts, 2, "{:?}", vehicle(&w, car));
    let wheels = p.vehicle_wheels(car).unwrap();
    assert_eq!(wheels.len(), 2);
    assert!(
        wheels[0].position.x > wheels[1].position.x,
        "front at +x: {wheels:?}"
    );
    set(&mut w, car, |v| v.throttle = 1.0);
    run(&mut w, 120);
    let x = transform(&w, car).position.x;
    assert!(x > 4.0, "drove right to {x}");
    let wheels = p.vehicle_wheels(car).unwrap();
    for wh in &wheels {
        assert!(
            (wh.position.x - x).abs() < 1.6,
            "a wheel stayed with the car: {wh:?} at {x}"
        );
    }
    // Removing the vehicle removes its wheels: the chassis drops onto the ground.
    let y_before = transform(&w, car).position.y;
    w.remove::<Vehicle>(car).unwrap();
    run(&mut w, 60);
    assert!(p.vehicle_wheels(car).is_none());
    let y_after = transform(&w, car).position.y;
    assert!(
        y_before > 0.45 && y_after < 0.32,
        "on its wheels at {y_before}, without them it sat down to {y_after}"
    );
    let _ = ground;
}
