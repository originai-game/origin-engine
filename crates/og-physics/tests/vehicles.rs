//! og.Vehicle in both profiles: the 3D raycast car and the 2D wheel-joint car.
//!
//! Ways this could fail, each caught below: the wheels are mounted on the
//! wrong axes (the car sinks, flips or drives backwards on positive
//! throttle); steering turns the wrong way or not at all; the brake does
//! not slow it; the outputs (speed, contacts) are not written; the wheel
//! poses for drawing are missing or detached from the chassis; the 2D
//! wheels are left behind when the vehicle is removed (they keep
//! colliding); a rebuild of the chassis leaves orphaned wheels; the wheel
//! rays stand a car on another whose layers it does not meet (Meridian
//! Loop's FINDINGS #3); the defaults barely move a car of a car's mass (#1);
//! a car tips over at full lock with nothing to stop it (#2); a sliding
//! tyre is not told (#16); a chassis with locked axes sizes its tyres'
//! impulses with the mass it does not have (a car on rails rolls on
//! turned wheels); the wheels push nothing back on a dynamic ground (a
//! car on a raft neither drives it nor weighs on it).

use crate::support;

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

/// A road car as the og.Vehicle doc and og-feel §12 give it: a 1,000 kg
/// box chassis on the defaults.
fn car3d(w: &mut World) -> Entity {
    car_at(
        w,
        Vec3::new(0.0, 1.2, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
        Vehicle::default(),
    )
}

fn car_at(w: &mut World, p: Vec3, body: RigidBody, collider: Collider, veh: Vehicle) -> Entity {
    let e = spawn_body(
        w,
        at(p.x, p.y, p.z),
        RigidBody {
            mass: 1000.0,
            ..body
        },
        collider,
    );
    w.insert(e, veh).unwrap();
    e
}

/// A 4 km flat road whose top is y = 0.
fn road(w: &mut World) -> Entity {
    spawn_static(
        w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(4000.0, 1.0, 4000.0)),
    )
}

fn up(w: &World, e: Entity) -> f32 {
    (transform(w, e).quaternion * Vec3::Y).y
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
            brake_force: 30.0,
            wheel_mass: 1.0,
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

#[test]
fn cars_on_layers_that_do_not_meet_pass_through_each_other() {
    // The road is on layer 1; each car meets only the road, so neither
    // chassis touches the other and neither car's wheels may stand on the
    // other. Before the rays honoured the chassis's layers and mask, the
    // upper car stood on the lower one's roof and both climbed.
    let (mut w, _) = world(Physics::default(), 1);
    road(&mut w);
    let car = |layer| Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)).on_layers(layer, 1);
    let braked = Vehicle {
        brake: 1.0,
        ..Vehicle::default()
    };
    let a = car_at(
        &mut w,
        Vec3::new(0.0, 1.0, 0.0),
        RigidBody::dynamic(),
        car(2),
        braked,
    );
    let b = car_at(
        &mut w,
        Vec3::new(0.0, 1.6, 0.0),
        RigidBody::dynamic(),
        car(4),
        braked,
    );
    run(&mut w, 120);
    let (ya, yb) = (transform(&w, a).position.y, transform(&w, b).position.y);
    assert!((ya - yb).abs() < 0.02, "both on the road: {ya} and {yb}");
    assert!(ya > 0.5 && ya < 1.0, "on its wheels on the road at {ya}");
    assert_eq!(vehicle(&w, a).contacts, 4);
    assert_eq!(vehicle(&w, b).contacts, 4);

    // On layers that meet, the upper car's rays find the lower one's roof.
    let (mut w, _) = world(Physics::default(), 1);
    road(&mut w);
    let lower = car_at(
        &mut w,
        Vec3::new(0.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)).on_layers(2, 1 | 4),
        braked,
    );
    let upper = car_at(
        &mut w,
        Vec3::new(0.0, 2.6, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)).on_layers(4, 1 | 2),
        braked,
    );
    run(&mut w, 60);
    let gap = transform(&w, upper).position.y - transform(&w, lower).position.y;
    assert!(gap > 0.9, "the upper car stands on the lower one: {gap}");
}

#[test]
fn the_defaults_drive_a_car_of_a_thousand_kilograms() {
    // og-feel §12's numbers: rear drive at 2.5 kN a wheel (5 m/s² off the
    // line), 2 kN of brake a wheel (8 m/s²), a top speed of 40 m/s. The
    // old defaults (80 N, 30 N) moved a 1,000 kg car at 0.16 m/s².
    let (mut w, _) = world(Physics::default(), 1);
    road(&mut w);
    let car = car3d(&mut w);
    run(&mut w, 60);
    set(&mut w, car, |v| v.throttle = 1.0);
    run(&mut w, 60);
    let one = vehicle(&w, car).speed;
    assert!(one > 4.0 && one < 6.0, "{one} m/s after 1 s");
    run(&mut w, 540);
    let ten = vehicle(&w, car).speed;
    assert!(ten > 25.0 && ten <= 40.0, "{ten} m/s after 10 s");
    assert!(up(&w, car) > 0.99);
    set(&mut w, car, |v| {
        v.throttle = 0.0;
        v.brake = 1.0;
    });
    let mut ticks = 0;
    while vehicle(&w, car).speed > 0.5 && ticks < 600 {
        run(&mut w, 1);
        ticks += 1;
    }
    let decel = ten / (ticks as f32 / 60.0);
    assert!(
        decel > 6.5 && decel < 10.0,
        "braked at {decel} m/s² ({ticks} ticks)"
    );
}

#[test]
fn full_lock_at_speed_tips_a_tall_car_only_with_roll_influence_and_a_high_centre_of_mass() {
    // The default car at 36 m/s, full lock for three seconds: with the
    // defaults (rollInfluence 0) it slides and stays on its wheels; with
    // rapier's own 0.1 (what og.Vehicle had, unsettable) it ends on its
    // roof; with 0.1 and the centre of mass lowered below the wheel
    // mounts it stays up again.
    let trial = |roll: f32, centre: f32| {
        let (mut w, p) = world(Physics::default(), 1);
        road(&mut w);
        let car = car_at(
            &mut w,
            Vec3::new(0.0, 1.0, 0.0),
            RigidBody {
                center_of_mass: Vec3::new(0.0, centre, 0.0),
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
            Vehicle {
                engine_force: 5000.0,
                roll_influence: roll,
                ..Vehicle::default()
            },
        );
        set(&mut w, car, |v| v.throttle = 1.0);
        run(&mut w, 600);
        assert_eq!(vehicle(&w, car).slip, 0.0, "a straight line slides no tyre");
        set(&mut w, car, |v| v.steer = 1.0);
        let (mut lowest, mut slip, mut side) = (1.0f32, 0.0f32, 0.0f32);
        for _ in 0..180 {
            run(&mut w, 1);
            lowest = lowest.min(up(&w, car));
            slip = slip.max(vehicle(&w, car).slip);
            for wh in p.vehicle_wheels(car).unwrap() {
                side = side.max(wh.side_speed);
                assert!(wh.slip <= vehicle(&w, car).slip + 1e-6, "{wh:?}");
            }
        }
        (lowest, slip, side)
    };
    let (lowest, slip, side) = trial(0.0, 0.0);
    assert!(lowest > 0.95, "the default car leaned to {lowest}");
    assert!(
        slip > 0.5 && side > 2.0,
        "full lock at 36 m/s slides: slip {slip}, {side} m/s"
    );
    let (lowest, ..) = trial(0.1, 0.0);
    assert!(lowest < 0.0, "rollInfluence 0.1 left it upright ({lowest})");
    let (lowest, ..) = trial(0.1, -0.7);
    assert!(lowest > 0.95, "the low centre of mass leaned to {lowest}");
}

#[test]
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn a_car_on_rails_with_its_front_wheels_turned_scrubs_them() {
    // The chassis can move only along z (x and every rotation locked) and
    // its front wheels are steered a little, s = 0.05 × maxSteer: they
    // roll at an angle to the only way the car can go, so they slide
    // sideways at v sin s and their side impulse is all spent along z. A
    // side impulse sized with the chassis's free inverse mass (1/m, where
    // only sin²s of it moves the chassis) removes 0.2 sin²s of that slide
    // a tick, a drag of 2 × 0.2 m sin²s v / dt = 18.2 v N against the
    // rear wheels' 2 × 2,500 (1 − v/40): 34.9 m/s, no tyre sliding. Sized
    // with its effective inverse mass (sin²s/m) the impulse wants
    // 0.2 m v / sin s, past the friction circle's cap of
    // (m g / 4) dt × 5 grip: both front tyres slide, each dragging
    // (m g / 4) × 5 grip × sin s, and 5,000 (1 − v/40) = 1,349 N at
    // 29.2 m/s.
    let (mut w, _) = world(Physics::default(), 1);
    road(&mut w);
    let rails = LOCK_TRANSLATION_X | LOCK_ROTATION_X | LOCK_ROTATION_Y | LOCK_ROTATION_Z;
    let car = car_at(
        &mut w,
        Vec3::new(0.0, 1.2, 0.0),
        RigidBody {
            lock: rails,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
        Vehicle::default(),
    );
    run(&mut w, 60);
    set(&mut w, car, |v| {
        v.throttle = 1.0;
        v.steer = 0.05;
    });
    // Eight seconds is the speed's time constant (1,000 kg / 125 N s/m).
    run(&mut w, 3000);
    let d = Vehicle::default();
    let (m, g, sin_s) = (1000.0f32, 9.81f32, (0.05 * d.max_steer).sin());
    let drag = 2.0 * (m * g / 4.0) * 5.0 * d.grip * sin_s;
    let engine = 2.0 * d.engine_force;
    let expected = (engine - drag) / (engine / d.max_speed);
    let v = vehicle(&w, car);
    assert!(
        (v.speed - expected).abs() < 0.3,
        "{} m/s, the scrub predicts {expected}",
        v.speed
    );
    assert!(v.slip > 0.5, "the front tyres slide: {v:?}");
}

#[test]
fn a_car_on_a_raft_drives_it_backwards_and_weighs_on_it() {
    // A 4,000 kg raft on a frictionless road and a 1,000 kg car on it:
    // nothing outside the pair pushes along z, so their momentum there
    // stays 0 while the car drives (the raft goes back at a quarter of
    // the car's speed). Before the wheels pushed back on their ground the
    // raft never moved and the car drove off with all the momentum.
    let (mut w, _) = world(Physics::default(), 1);
    let ice = |c: Collider| Collider { friction: 0.0, ..c };
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        ice(Collider::cuboid(Vec3::new(400.0, 1.0, 400.0))),
    );
    let raft_body = RigidBody {
        mass: 4000.0,
        ..RigidBody::dynamic()
    };
    let raft = spawn_body(
        &mut w,
        at(0.0, 0.2, 0.0),
        raft_body,
        ice(Collider::cuboid(Vec3::new(8.0, 0.4, 40.0))),
    );
    let car = car_at(
        &mut w,
        Vec3::new(0.0, 1.6, -10.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
        Vehicle::default(),
    );
    let momentum = |w: &World| 1000.0 * velocity(w, car).linear + 4000.0 * velocity(w, raft).linear;
    run(&mut w, 60);
    set(&mut w, car, |v| v.throttle = 1.0);
    run(&mut w, 120);
    assert_eq!(vehicle(&w, car).contacts, 4);
    let (vc, vr) = (velocity(&w, car).linear, velocity(&w, raft).linear);
    assert!(vc.z > 8.0, "the car drives forward: {vc}");
    assert!(
        (vr.z + vc.z / 4.0).abs() < 0.01 * vc.z,
        "the raft goes back at a quarter of {vc}: {vr}"
    );
    assert!(
        momentum(&w).z.abs() < 1.0,
        "momentum along z: {}",
        momentum(&w)
    );

    // The same raft afloat (gravityScale 0, nothing under it, held
    // level): the car's weight is the only outside force on the pair, so
    // their momentum along y falls at m g however the car bounces on it.
    // Before the springs pushed back on their ground the raft did not
    // move and they held the car up from nowhere.
    let (mut w, _) = world(Physics::default(), 1);
    let raft = spawn_body(
        &mut w,
        at(0.0, 0.2, 0.0),
        RigidBody {
            gravity_scale: 0.0,
            lock: LOCK_ROTATION_X | LOCK_ROTATION_Y | LOCK_ROTATION_Z,
            ..raft_body
        },
        Collider::cuboid(Vec3::new(8.0, 0.4, 40.0)),
    );
    let car = car_at(
        &mut w,
        Vec3::new(0.0, 1.6, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.8, 0.6, 4.0)),
        Vehicle::default(),
    );
    let momentum = |w: &World| 1000.0 * velocity(w, car).linear + 4000.0 * velocity(w, raft).linear;
    run(&mut w, 30);
    let before = momentum(&w).y;
    run(&mut w, 60);
    let fell = before - momentum(&w).y;
    let weight = 1000.0 * 9.81;
    assert!(
        (fell - weight).abs() < 0.02 * weight,
        "a second of the car's weight ({weight}) took {fell} from p_y"
    );
}
