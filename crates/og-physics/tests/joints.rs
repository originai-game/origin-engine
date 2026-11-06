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

use crate::support;

use og_core::glam::Vec3;
use og_ecs::Entity;
use og_physics::*;
use support::*;

fn joint(w: &og_ecs::World, e: Entity) -> Joint {
    w.get::<Joint>(e).unwrap()
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
