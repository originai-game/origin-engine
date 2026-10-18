//! Joints between bodies (og.Joint), in both profiles: what each kind keeps
//! together, limits, motors, springs, ropes, world anchors, and that the
//! joint follows its bodies through rebuilds and removal.
//!
//! Ways this could fail, each caught below: the second frame ignores the
//! bodies' relative pose, so a joint snaps its bodies together on the
//! first tick (the hinge pendulum and the fixed pair would jump); limits
//! or motors never reach rapier (the pendulum swings past its limit, the
//! wheel never spins); the written `value` stays 0; a body rebuilt for a
//! collider change loses its joint (the pendulum falls); a removed joint
//! keeps holding; a world anchor (`other` 0) is not a fixed point; the 2D
//! profile builds 3D frames (its hinge drifts out of the plane or fails).

mod support;

use og_core::glam::{Quat, Vec3};
use og_ecs::Entity;
use og_physics::*;
use support::*;

fn joint(w: &og_ecs::World, e: Entity) -> Joint {
    w.get::<Joint>(e).unwrap()
}

fn pendulum(w: &mut og_ecs::World, limits: Option<(f32, f32)>) -> Entity {
    // A 1 m arm hanging from a world point at (0, 5, 0), started level so it swings.
    let bob = spawn_body(
        w,
        at(1.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.2),
    );
    let (limited, min, max) = match limits {
        Some((a, b)) => (true, a, b),
        None => (false, 0.0, 0.0),
    };
    w.insert(
        bob,
        Joint {
            kind: JOINT_REVOLUTE,
            anchor: Vec3::new(-1.0, 0.0, 0.0),
            other_anchor: Vec3::new(0.0, 5.0, 0.0),
            axis: Vec3::Z,
            limits: limited,
            min,
            max,
            ..Joint::default()
        },
    )
    .unwrap();
    bob
}

#[test]
fn a_hinge_swings_about_its_world_anchor_and_reports_its_angle() {
    let (mut w, _p) = world(Physics::default(), 1);
    let bob = pendulum(&mut w, None);
    w.step().unwrap();
    // No snap on the first tick: the arm starts where it was built.
    let p = transform(&w, bob).position;
    assert!((p - Vec3::new(1.0, 5.0, 0.0)).length() < 0.05, "{p}");
    let mut lowest = f32::MAX;
    for _ in 0..90 {
        w.step().unwrap();
        let p = transform(&w, bob).position;
        // Always 1 m from the pivot, in the z = 0 plane.
        let r = (p - Vec3::new(0.0, 5.0, 0.0)).length();
        assert!((r - 1.0).abs() < 0.03, "arm length {r}");
        assert!(p.z.abs() < 0.01, "{p}");
        lowest = lowest.min(p.y);
    }
    assert!(lowest < 4.1, "the pendulum swung down to {lowest}");
    // The angle written back moved from 0 (where it was built).
    assert!(joint(&w, bob).value.abs() > 0.3, "{:?}", joint(&w, bob));
}

#[test]
fn a_hinge_with_limits_stops_at_them() {
    let (mut w, _p) = world(Physics::default(), 1);
    let bob = pendulum(&mut w, Some((-0.4, 0.4)));
    for _ in 0..120 {
        w.step().unwrap();
        let a = joint(&w, bob).value;
        assert!(a.abs() < 0.48, "angle {a} past the 0.4 rad limit");
    }
    let p = transform(&w, bob).position;
    assert!(p.y > 4.5, "a 0.4 rad limit keeps the bob high, at {p}");
}

#[test]
fn a_velocity_motor_spins_a_wheel_and_a_position_motor_turns_to_its_target() {
    let (mut w, _p) = world(
        Physics {
            gravity: Vec3::ZERO,
            ..Physics::default()
        },
        1,
    );
    let wheel = spawn_body(
        &mut w,
        at(0.0, 2.0, 0.0),
        RigidBody::dynamic(),
        Collider::cylinder(0.5, 0.2),
    );
    w.insert(
        wheel,
        Joint {
            kind: JOINT_REVOLUTE,
            other_anchor: Vec3::new(0.0, 2.0, 0.0),
            axis: Vec3::Y,
            motor: MOTOR_VELOCITY,
            motor_target: 3.0,
            motor_stiffness: 20.0,
            ..Joint::default()
        },
    )
    .unwrap();
    run(&mut w, 60);
    let spin = velocity(&w, wheel).angular;
    assert!((spin.y - 3.0).abs() < 0.3, "spinning at {spin}");
    assert!(spin.x.abs() < 0.05 && spin.z.abs() < 0.05, "{spin}");

    let servo = spawn_body(
        &mut w,
        at(4.0, 2.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::new(1.0, 0.2, 0.2)),
    );
    w.insert(
        servo,
        Joint {
            kind: JOINT_REVOLUTE,
            other_anchor: Vec3::new(4.0, 2.0, 0.0),
            axis: Vec3::Z,
            motor: MOTOR_POSITION,
            motor_target: 0.8,
            motor_stiffness: 40.0,
            motor_damping: 8.0,
            ..Joint::default()
        },
    )
    .unwrap();
    run(&mut w, 180);
    let a = joint(&w, servo).value;
    assert!((a - 0.8).abs() < 0.05, "the servo stopped at {a}");
}

#[test]
fn a_fixed_joint_keeps_two_bodies_as_one() {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let a = spawn_body(
        &mut w,
        rotated(at(0.0, 3.0, 0.0), Quat::from_rotation_y(0.5)),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let b = spawn_body(
        &mut w,
        at(1.2, 3.6, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.3),
    );
    let start = transform(&w, b).position - transform(&w, a).position;
    let local = transform(&w, a).quaternion.inverse() * start;
    w.insert(a, Joint::new(JOINT_FIXED, b.to_bits())).unwrap();
    run(&mut w, 120);
    let (ta, tb) = (transform(&w, a), transform(&w, b));
    assert!(ta.position.y < 1.0, "the pair fell: {ta:?}");
    let now = ta.quaternion.inverse() * (tb.position - ta.position);
    assert!((now - local).length() < 0.05, "{local} became {now}");
}

#[test]
fn a_slider_moves_along_its_axis_within_its_limits() {
    let (mut w, _p) = world(Physics::default(), 1);
    let car = spawn_body(
        &mut w,
        at(0.0, 3.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::splat(0.4)),
    );
    // A rail tilted 45 degrees down towards +x, 1.5 m of travel.
    let axis = Vec3::new(1.0, -1.0, 0.0).normalize();
    w.insert(
        car,
        Joint {
            kind: JOINT_PRISMATIC,
            other_anchor: Vec3::new(0.0, 3.0, 0.0),
            axis,
            limits: true,
            min: -0.1,
            max: 1.5,
            ..Joint::default()
        },
    )
    .unwrap();
    run(&mut w, 120);
    let p = transform(&w, car).position - Vec3::new(0.0, 3.0, 0.0);
    let along = p.dot(axis);
    let off = (p - axis * along).length();
    assert!((along - 1.5).abs() < 0.05, "slid {along}");
    assert!(off < 0.02, "left the rail by {off}");
    assert!((joint(&w, car).value - 1.5).abs() < 0.05);
}

#[test]
fn a_spring_settles_below_its_rest_length_and_a_rope_holds_at_its_length() {
    let (mut w, _p) = world(Physics::default(), 1);
    let weight = spawn_body(
        &mut w,
        at(0.0, 4.0, 0.0),
        RigidBody {
            mass: 1.0,
            linear_damping: 1.0,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.2),
    );
    w.insert(
        weight,
        Joint {
            kind: JOINT_SPRING,
            other_anchor: Vec3::new(0.0, 5.0, 0.0),
            rest_length: 1.0,
            stiffness: 50.0,
            damping: 4.0,
            ..Joint::default()
        },
    )
    .unwrap();
    let hanging = spawn_body(
        &mut w,
        at(3.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.2),
    );
    w.insert(
        hanging,
        Joint {
            kind: JOINT_ROPE,
            other_anchor: Vec3::new(3.0, 5.0, 0.0),
            length: 2.0,
            ..Joint::default()
        },
    )
    .unwrap();
    run(&mut w, 360);
    // Hooke: 1 kg at 50 N/m stretches 9.81 / 50 = 0.196 m past the rest length.
    let d = 5.0 - transform(&w, weight).position.y;
    assert!((d - 1.196).abs() < 0.05, "the spring hangs {d}");
    let r = 5.0 - transform(&w, hanging).position.y;
    assert!((r - 2.0).abs() < 0.05, "the rope hangs {r}");
}

#[test]
fn a_joint_survives_its_body_being_rebuilt_and_stops_when_removed() {
    let (mut w, _p) = world(Physics::default(), 1);
    let bob = pendulum(&mut w, None);
    run(&mut w, 30);
    // A new collider rebuilds the rapier body; the joint must be rebuilt with it.
    w.insert(bob, Collider::sphere(0.25)).unwrap();
    run(&mut w, 30);
    let r = (transform(&w, bob).position - Vec3::new(0.0, 5.0, 0.0)).length();
    assert!((r - 1.0).abs() < 0.05, "after the rebuild the arm is {r}");
    w.remove::<Joint>(bob).unwrap();
    run(&mut w, 60);
    assert!(
        transform(&w, bob).position.y < 3.0,
        "without its joint the bob falls"
    );
}

#[test]
fn a_joint_to_an_entity_that_is_not_a_body_does_nothing_until_it_is() {
    let (mut w, _p) = world(Physics::default(), 1);
    let bob = spawn_body(
        &mut w,
        at(0.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.2),
    );
    let post = w.spawn((at(0.0, 6.0, 0.0),)).unwrap();
    w.insert(
        bob,
        Joint {
            kind: JOINT_ROPE,
            other: post.to_bits(),
            length: 1.0,
            ..Joint::default()
        },
    )
    .unwrap();
    run(&mut w, 90);
    // Free fall (no floor): 0.5 g t² = 11 m in 1.5 s; any rope would have held it.
    let y = transform(&w, bob).position.y;
    assert!(y < -5.0, "no body, no joint: the bob fell to {y}");
    // Now the post becomes a static body: the rope pulls the bob back up to it.
    w.insert(post, RigidBody::fixed()).unwrap();
    w.insert(post, Collider::sphere(0.1)).unwrap();
    w.insert(bob, Velocity::default()).unwrap();
    w.insert(bob, at(0.0, 5.5, 0.0)).unwrap();
    run(&mut w, 120);
    let d = (transform(&w, bob).position - Vec3::new(0.0, 6.0, 0.0)).length();
    assert!(d <= 1.05, "the rope holds the bob {d} from the post");
}

#[test]
fn the_2d_profile_hinges_in_the_plane() {
    let (mut w, _p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    let bob = spawn_body(
        &mut w,
        at(1.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.2),
    );
    w.insert(
        bob,
        Joint {
            kind: JOINT_REVOLUTE,
            anchor: Vec3::new(-1.0, 0.0, 0.0),
            other_anchor: Vec3::new(0.0, 5.0, 0.0),
            limits: true,
            min: -1.0,
            max: 1.0,
            ..Joint::default()
        },
    )
    .unwrap();
    let mut lowest = f32::MAX;
    for _ in 0..120 {
        w.step().unwrap();
        let p = transform(&w, bob).position;
        let r = (p - Vec3::new(0.0, 5.0, 0.0)).length();
        assert!((r - 1.0).abs() < 0.03, "arm length {r}");
        lowest = lowest.min(p.y);
        assert!(joint(&w, bob).value.abs() < 1.08);
    }
    assert!(lowest < 4.5, "it swung to {lowest}");
}
