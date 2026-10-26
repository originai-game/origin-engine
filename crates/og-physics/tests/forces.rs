//! og.physics.applyForce, applyImpulse, applyTorque and applyTorqueImpulse
//! (Meridian Loop's FINDINGS #2: no force, torque or impulse call, so an
//! anti-roll assist or downforce had none), and og.RigidBody.centerOfMass.
//!
//! Ways this could fail, each caught below: a force is applied as an
//! impulse (60 times too strong) or lingers after its step; an impulse is
//! scaled by the tick; a push at a point off the centre of mass does not
//! spin the body; a torque turns about the wrong axis; a static or
//! kinematic body is moved; a hit stop lets a force through or drops an
//! impulse; pushes not yet applied are missing from a snapshot (the
//! restored world diverges); the 2D profile ignores them; a centre of mass
//! moved below a box does not right it.

use crate::support;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A 2 kg sphere floating (no gravity) at the origin.
fn floating(profile: u32) -> (World, PhysicsSystem, Entity) {
    let (mut w, p) = world(
        Physics {
            profile,
            gravity: Vec3::ZERO,
            ..Physics::default()
        },
        1,
    );
    let e = spawn_body(
        &mut w,
        at(0.0, 0.0, 0.0),
        RigidBody {
            mass: 2.0,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.5),
    );
    (w, p, e)
}

fn close(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-3
}

#[test]
fn a_force_acts_over_one_step_and_an_impulse_at_once() {
    let (mut w, p, e) = floating(PROFILE_3D);
    run(&mut w, 1);
    // 10 N on 2 kg for one tick: 5 m/s² × 1/60 s.
    p.apply_force(e, Vec3::new(10.0, 0.0, 0.0), None).unwrap();
    run(&mut w, 1);
    let v = velocity(&w, e).linear;
    assert!(close(v, Vec3::new(5.0 / 60.0, 0.0, 0.0)), "{v}");
    // Gone after its step.
    run(&mut w, 3);
    assert!(close(velocity(&w, e).linear, v), "the force lingered");
    // 10 N every tick for a second: 5 m/s more.
    for _ in 0..60 {
        p.apply_force(e, Vec3::new(10.0, 0.0, 0.0), None).unwrap();
        run(&mut w, 1);
    }
    let v2 = velocity(&w, e).linear;
    assert!(close(v2, v + Vec3::new(5.0, 0.0, 0.0)), "{v2}");
    // 4 N·s on 2 kg: 2 m/s at once, whatever the tick.
    p.apply_impulse(e, Vec3::new(0.0, 4.0, 0.0), None).unwrap();
    run(&mut w, 1);
    let v3 = velocity(&w, e).linear;
    assert!(close(v3, v2 + Vec3::new(0.0, 2.0, 0.0)), "{v3}");
    assert!(close(velocity(&w, e).angular, Vec3::ZERO));

    // Refused when not finite.
    assert!(
        p.apply_force(e, Vec3::new(f32::NAN, 0.0, 0.0), None)
            .is_err()
    );
    assert!(
        p.apply_impulse(e, Vec3::X, Some(Vec3::splat(f32::INFINITY)))
            .is_err()
    );
}

#[test]
fn a_push_off_the_centre_spins_and_a_torque_turns_about_its_axis() {
    let (mut w, p, e) = floating(PROFILE_3D);
    run(&mut w, 1);
    // An impulse along +x at a point above the centre: it moves and spins
    // about -z (r × J = (0,1,0) × (1,0,0) = (0,0,-1)).
    p.apply_impulse(e, Vec3::X, Some(Vec3::new(0.0, 1.0, 0.0)))
        .unwrap();
    run(&mut w, 1);
    let vel = velocity(&w, e);
    assert!(close(vel.linear, Vec3::new(0.5, 0.0, 0.0)), "{vel:?}");
    assert!(
        vel.angular.z < -0.1 && vel.angular.x.abs() < 1e-4,
        "{vel:?}"
    );

    let (mut w, p, e) = floating(PROFILE_3D);
    run(&mut w, 1);
    p.apply_torque_impulse(e, Vec3::new(0.0, 0.2, 0.0)).unwrap();
    run(&mut w, 1);
    let spin = velocity(&w, e).angular;
    // A 2 kg sphere of 0.5 m: I = 2/5 m r² = 0.2 kg·m², so 1 rad/s.
    assert!(close(spin, Vec3::new(0.0, 1.0, 0.0)), "{spin}");
    p.apply_torque(e, Vec3::new(0.0, 12.0, 0.0)).unwrap();
    run(&mut w, 1);
    let spin = velocity(&w, e).angular;
    assert!(close(spin, Vec3::new(0.0, 2.0, 0.0)), "{spin}");
    assert!(close(velocity(&w, e).linear, Vec3::ZERO));
}

#[test]
fn only_dynamic_bodies_move_and_the_2d_profile_takes_pushes_too() {
    let (mut w, p, e) = floating(PROFILE_3D);
    let wall = spawn_static(&mut w, at(5.0, 0.0, 0.0), Collider::cuboid(Vec3::ONE));
    let lift = spawn_body(
        &mut w,
        at(-5.0, 0.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 1);
    for x in [wall, lift] {
        p.apply_impulse(x, Vec3::new(0.0, 100.0, 0.0), None)
            .unwrap();
    }
    // A push for an entity with no body at all is harmless.
    p.apply_impulse(Entity::from_bits(9_999), Vec3::X, None)
        .unwrap();
    run(&mut w, 2);
    assert_eq!(transform(&w, wall).position, Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(transform(&w, lift).position, Vec3::new(-5.0, 0.0, 0.0));
    assert!(close(velocity(&w, e).linear, Vec3::ZERO));

    let (mut w, p, e) = floating(PROFILE_2D);
    run(&mut w, 1);
    p.apply_impulse(e, Vec3::new(2.0, 4.0, 7.0), None).unwrap();
    p.apply_torque_impulse(e, Vec3::new(3.0, 3.0, 0.25))
        .unwrap();
    run(&mut w, 1);
    let vel = velocity(&w, e);
    assert!(close(vel.linear, Vec3::new(1.0, 2.0, 0.0)), "{vel:?}");
    assert!(vel.angular.z > 0.1 && vel.angular.x == 0.0, "{vel:?}");
}

#[test]
fn a_hit_stop_drops_forces_and_keeps_impulses_for_after_it() {
    let (mut w, p, e) = floating(PROFILE_3D);
    run(&mut w, 1);
    w.hit_stop(3).unwrap();
    p.apply_force(e, Vec3::new(600.0, 0.0, 0.0), None).unwrap();
    p.apply_impulse(e, Vec3::new(0.0, 0.0, 2.0), None).unwrap();
    run(&mut w, 3);
    assert!(
        close(velocity(&w, e).linear, Vec3::ZERO),
        "moved in the stop"
    );
    run(&mut w, 1);
    let v = velocity(&w, e).linear;
    assert!(close(v, Vec3::new(0.0, 0.0, 1.0)), "{v}");
}

#[test]
fn pushes_not_yet_applied_are_in_the_snapshot() {
    let (mut w, p, e) = floating(PROFILE_3D);
    run(&mut w, 5);
    p.apply_impulse(e, Vec3::new(1.0, 2.0, 3.0), Some(Vec3::new(0.0, 0.3, 0.0)))
        .unwrap();
    p.apply_force(e, Vec3::new(-30.0, 0.0, 0.0), None).unwrap();
    let saved = w.snapshot();
    let hash = w.world_hash();
    run(&mut w, 30);

    let (mut r, _, _) = floating(PROFILE_3D);
    r.restore(&saved).unwrap();
    assert_eq!(r.world_hash(), hash);
    run(&mut r, 30);
    assert_eq!(r.world_hash(), w.world_hash());
    assert!(velocity(&r, e).linear.length() > 1.0);
}

#[test]
fn a_centre_of_mass_below_a_box_rights_it() {
    // A 1 × 1 × 1 box set down tilted 60° on its edge: it falls over; with
    // its centre of mass moved 0.45 m down (inside, near its bottom face)
    // it rocks back onto that face, a roly-poly.
    let trial = |centre: f32| {
        let (mut w, _) = world(Physics::default(), 1);
        floor(&mut w);
        let tilt = Quat::from_rotation_z(1.05);
        let b = spawn_body(
            &mut w,
            rotated(at(0.0, 0.75, 0.0), tilt),
            RigidBody {
                mass: 5.0,
                center_of_mass: Vec3::new(0.0, centre, 0.0),
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::ONE),
        );
        run(&mut w, 240);
        (transform(&w, b).quaternion * Vec3::Y).y
    };
    let fallen = trial(0.0);
    assert!(fallen < 0.1, "the plain box stayed up ({fallen})");
    let righted = trial(-0.45);
    assert!(righted > 0.99, "the weighted box lies at {righted}");
}
