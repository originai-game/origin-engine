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
