//! G2-3's drivetrain car and its wheels: og.Vehicle with `model`
//! VEHICLE_DRIVETRAIN (gears, rpm, a torque curve), the handbrake and drift
//! grip, and og.VehicleWheel children in place of the four implicit wheels.
//!
//! Ways this could fail, each caught below: the gearbox never leaves first
//! or shifts past its top gear; the rev limiter does not cap the speed a
//! gear reaches; reverse is never selected; the gearbox's state is lost by
//! a restore, so a restored drive shifts at other ticks
//! (`a_drivetrain_car_shifts_up_through_its_gears_and_restores_mid_drive`);
//! the handbrake leaves the rear tyres' grip alone, so a car turning at
//! speed does not swing its tail out; wheel children are ignored, counted
//! in the wrong order, or steer the wrong way; a 2D car ignores the model.

use crate::support;

use og_core::List;
use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A world with physics and G2-3's lazy types registered (a game naming them).
fn world(settings: Physics, seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(settings, seed);
    support::enable_lazy(&mut w);
    (w, p)
}

fn vehicle(w: &World, e: Entity) -> Vehicle {
    w.get::<Vehicle>(e).unwrap()
}

fn set(w: &mut World, e: Entity, f: impl FnOnce(&mut Vehicle)) {
    let mut v = vehicle(w, e);
    f(&mut v);
    w.insert(e, v).unwrap();
}

fn road(w: &mut World) {
    spawn_static(
        w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(4000.0, 1.0, 4000.0)),
    );
}

fn car(w: &mut World, veh: Vehicle) -> Entity {
    let e = spawn_body(
        w,
        at(0.0, 1.2, 0.0),
        RigidBody {
            mass: 1000.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
    );
    w.insert(e, veh).unwrap();
    e
}

fn drivetrain() -> Vehicle {
    Vehicle {
        model: VEHICLE_DRIVETRAIN,
        ..Vehicle::default()
    }
}

/// (gear, rpm bits, z bits) each tick for `ticks` ticks.
fn trace(w: &mut World, e: Entity, ticks: u32) -> Vec<(i32, u32, u32)> {
    (0..ticks)
        .map(|_| {
            run(w, 1);
            let v = vehicle(w, e);
            (
                v.gear,
                v.rpm.to_bits(),
                transform(w, e).position.z.to_bits(),
            )
        })
        .collect()
}

#[test]
fn a_drivetrain_car_shifts_up_through_its_gears_and_restores_mid_drive() {
    let start = || {
        let (mut w, p) = world(Physics::default(), 1);
        road(&mut w);
        let e = car(&mut w, drivetrain());
        run(&mut w, 60);
        set(&mut w, e, |v| v.throttle = 1.0);
        (w, p, e)
    };
    let (mut a, _pa, e) = start();
    let first = trace(&mut a, e, 240);
    let mid = a.snapshot();
    let rest = trace(&mut a, e, 360);

    // Off the line in first, up through at least third, never past sixth,
    // and the engine within its idle and its limit while a gear is in.
    let gears: Vec<i32> = first.iter().chain(&rest).map(|t| t.0).collect();
    assert_eq!(gears[0], 1);
    let top = *gears.iter().max().unwrap();
    assert!((3..=6).contains(&top), "top gear {top}");
    assert!(gears.windows(2).all(|g| g[1] >= g[0]), "only up: {gears:?}");
    let rpms: Vec<f32> = rest.iter().map(|t| f32::from_bits(t.1)).collect();
    assert!(
        rpms.iter().all(|&r| (900.0..=6900.0).contains(&r)),
        "{rpms:?}"
    );
    let v = vehicle(&a, e);
    assert!(v.speed > 15.0, "10 s at full throttle: {} m/s", v.speed);

    // A second run is the same tick for tick, and so is one restored
    // mid-drive (the gearbox's gear, rpm and clutch are in the block).
    let (mut b, _pb, e2) = start();
    assert_eq!(trace(&mut b, e2, 240), first);
    let (mut c, _pc, _) = start();
    c.restore(&mid).unwrap();
    assert_eq!(trace(&mut c, e, 360), rest);
}

#[test]
fn the_rev_limit_caps_a_gear_and_a_manual_box_holds_it() {
    let (mut w, _p) = world(Physics::default(), 1);
    road(&mut w);
    // First only: 6,800 rpm through 3.4 × 3.6 on 0.4 m wheels is 23.3 m/s
    // at the wheel... of which first (3.4) reaches 6800 / 60 × 2π / 12.24 × 0.4.
    let e = car(
        &mut w,
        Vehicle {
            automatic: false,
            gear: 1,
            ..drivetrain()
        },
    );
    run(&mut w, 60);
    set(&mut w, e, |v| v.throttle = 1.0);
    run(&mut w, 600);
    let v = vehicle(&w, e);
    let limit = 6800.0 / 60.0 * std::f32::consts::TAU / (3.4 * 3.6) * 0.4;
    assert_eq!(v.gear, 1);
    assert!(
        v.speed > limit * 0.85 && v.speed < limit * 1.05,
        "{} m/s against a limit of {limit}",
        v.speed
    );
    // Neutral: no drive (it coasts; the car has no rolling resistance).
    set(&mut w, e, |v| v.gear = 0);
    run(&mut w, 120);
    let n = vehicle(&w, e);
    assert!(n.speed <= v.speed + 1e-3, "{} then {}", v.speed, n.speed);
    assert!(n.rpm > 6000.0, "free-revs at full throttle: {}", n.rpm);
}

#[test]
fn an_automatic_box_backs_up_in_reverse() {
    let (mut w, _p) = world(Physics::default(), 1);
    road(&mut w);
    let e = car(&mut w, drivetrain());
    run(&mut w, 60);
    set(&mut w, e, |v| v.throttle = -1.0);
    run(&mut w, 180);
    let v = vehicle(&w, e);
    assert_eq!(v.gear, -1);
    assert!(v.speed < -2.0, "{}", v.speed);
    assert!(transform(&w, e).position.z < -2.0);
}

/// How fast the rear tyres slide sideways (their mean `sideSpeed`), 15
/// ticks into a full-lock turn at about 10 m/s with the handbrake at
/// `handbrake` (the arcade model, the defaults).
fn tail_slide(handbrake: f32) -> f32 {
    let (mut w, p) = world(Physics::default(), 1);
    road(&mut w);
    let e = car(&mut w, Vehicle::default());
    run(&mut w, 60);
    set(&mut w, e, |v| v.throttle = 1.0);
    run(&mut w, 150);
    set(&mut w, e, |v| {
        v.steer = 1.0;
        v.handbrake = handbrake;
    });
    run(&mut w, 15);
    let wheels = p.vehicle_wheels(e).unwrap();
    (wheels[2].side_speed + wheels[3].side_speed) * 0.5
}

#[test]
fn the_handbrake_swings_the_tail_out() {
    let grip = tail_slide(0.0);
    let drift = tail_slide(1.0);
    assert!(grip > 0.1, "a turn at speed slides a little: {grip}");
    assert!(
        drift > grip * 1.5,
        "rear tyres slide at {drift} m/s with the handbrake, {grip} without"
    );
}

fn wheel(w: &mut World, car: Entity, x: f32, z: f32, wheel: VehicleWheel) -> Entity {
    w.spawn((
        at(x, -0.2, z),
        wheel,
        Parent {
            entity: car.to_bits(),
        },
    ))
    .unwrap()
}

#[test]
fn six_wheel_children_carry_the_car_and_the_rear_ones_steer_against_the_front() {
    let (mut w, p) = world(Physics::default(), 1);
    road(&mut w);
    let truck = car(&mut w, Vehicle::default());
    let front = VehicleWheel {
        steer: 1.0,
        ..VehicleWheel::default()
    };
    let rear = VehicleWheel {
        steer: -0.5,
        drive: true,
        handbrake: true,
        ..VehicleWheel::default()
    };
    let mut ids = Vec::new();
    for (z, kind) in [(1.6, front), (0.0, rear), (-1.6, rear)] {
        for x in [0.8, -0.8] {
            ids.push(wheel(&mut w, truck, x, z, kind));
        }
    }
    run(&mut w, 90);
    assert_eq!(vehicle(&w, truck).contacts, 6);
    let wheels = p.vehicle_wheels(truck).unwrap();
    let mut sorted = ids.clone();
    sorted.sort_by_key(|e| e.to_bits());
    assert_eq!(
        wheels.iter().map(|wh| wh.entity).collect::<Vec<_>>(),
        sorted.iter().map(|e| e.to_bits()).collect::<Vec<_>>()
    );
    // Steering right: front wheels turn one way, rear ones half as far the other.
    set(&mut w, truck, |v| {
        v.steer = 1.0;
        v.throttle = 1.0;
    });
    run(&mut w, 30);
    let wheels = p.vehicle_wheels(truck).unwrap();
    let chassis = transform(&w, truck).quaternion;
    let angle = |e: Entity| {
        let wh = wheels.iter().find(|x| x.entity == e.to_bits()).unwrap();
        let axle = chassis.inverse() * (wh.quaternion * Vec3::X);
        og_core::math::atan2(axle.z, axle.x)
    };
    let (f, r) = (angle(ids[0]), angle(ids[2]));
    assert!(f.abs() > 0.3, "front {f}");
    assert!((r + f * 0.5).abs() < 0.05, "rear {r} against front {f}");
    // The rear four drive: it moves forward.
    assert!(transform(&w, truck).position.z > 0.5);
}

#[test]
fn a_2d_drivetrain_car_drives_in_gear() {
    let (mut w, _p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(4000.0, 1.0, 1.0)),
    );
    let e = spawn_body(
        &mut w,
        at(0.0, 1.5, 0.0),
        RigidBody {
            mass: 400.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::new(2.0, 0.5, 1.0)),
    );
    // A light car: a small engine, so it drives rather than flips.
    w.insert(
        e,
        Vehicle {
            model: VEHICLE_DRIVETRAIN,
            gears: List::from_slice(&[3.0, 1.5]).unwrap(),
            torque_curve: List::from_slice(&[30.0, 40.0, 40.0, 30.0]).unwrap(),
            ..Vehicle::default()
        },
    )
    .unwrap();
    run(&mut w, 60);
    set(&mut w, e, |v| v.throttle = 1.0);
    run(&mut w, 300);
    let v = vehicle(&w, e);
    assert!(v.gear >= 1, "{v:?}");
    assert!(v.rpm > 900.0, "{}", v.rpm);
    assert!(
        transform(&w, e).position.x > 5.0,
        "{}",
        transform(&w, e).position
    );
}
