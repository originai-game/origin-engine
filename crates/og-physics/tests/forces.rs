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
//! moved below a box does not right it; og.RigidBody.airDrag does nothing,
//! overshoots the air's speed, moves a body that asked for none, takes the
//! world's wind where a motor blows, or blows a 2D body out of its plane.
//!
//! og.ForceField (`set_forces`). Ways this could fail: a field accelerates
//! by mass (a heavy body lags), reaches a body outside its volume, moves a
//! static body, leaves a body that fell asleep in it asleep when it is
//! switched on, pushes a 2D body out of its plane, or never reaches a
//! character (an updraft that lifts crates and not the hero).

use crate::support;

use og_core::glam::Vec3;
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

fn wind(x: f32, y: f32, z: f32) -> og_env::air::Air {
    og_env::air::Air {
        wind: Vec3::new(x, y, z),
        ..Default::default()
    }
}

/// A body with airDrag closes on the wind's velocity by the implicit rule,
/// tick by tick, and never passes it; a body without one is not moved.
#[test]
fn air_drag_carries_a_body_towards_the_wind_and_only_one_that_asks() {
    let (mut w, p, still) = floating(PROFILE_3D);
    let leaf = spawn_body(
        &mut w,
        at(10.0, 0.0, 0.0),
        RigidBody {
            mass: 2.0,
            air_drag: 2.0,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.5),
    );
    p.set_air(wind(6.0, 0.0, -3.0));
    run(&mut w, 1);
    // One tick: (wind - 0) × k / (1 + k), k = 2 / 60.
    let v = velocity(&w, leaf).linear;
    assert!(close(v, Vec3::new(6.0, 0.0, -3.0) / 31.0), "{v}");
    run(&mut w, 59);
    let v = velocity(&w, leaf).linear;
    let closed = 1.0 - og_core::math::pow(30.0 / 31.0, 60.0);
    assert!(close(v, Vec3::new(6.0, 0.0, -3.0) * closed), "{v}");
    run(&mut w, 600);
    let v = velocity(&w, leaf).linear;
    assert!(
        v.x <= 6.0 && v.x > 5.99 && v.z >= -3.0,
        "past the wind: {v}"
    );
    assert_eq!(velocity(&w, still).linear, Vec3::ZERO, "no airDrag");
}

/// The air is asked where the body is: a fan lifts the body over it and
/// not the one out of its reach; in still air the drag slows a moving body.
#[test]
fn air_drag_takes_the_air_where_the_body_is() {
    let (mut w, p) = world(
        Physics {
            gravity: Vec3::ZERO,
            ..Physics::default()
        },
        1,
    );
    let light = RigidBody {
        mass: 1.0,
        air_drag: 3.0,
        ..RigidBody::dynamic()
    };
    let [over, beyond] =
        [0.0, 30.0].map(|x| spawn_body(&mut w, at(x, 1.0, 0.0), light, Collider::sphere(0.25)));
    p.set_air(og_env::air::Air {
        motors: vec![og_env::air::Motor {
            position: Vec3::ZERO,
            radius: 6.0,
            strength: 10.0,
            ..Default::default()
        }],
        ..Default::default()
    });
    run(&mut w, 30);
    let up = velocity(&w, over).linear;
    assert!(up.y > 1.0 && up.x.abs() < 1e-4, "over the fan: {up}");
    assert_eq!(velocity(&w, beyond).linear, Vec3::ZERO, "out of reach");
    // Still air: a thrown body is slowed, as by linear damping.
    p.set_air(Default::default());
    p.apply_impulse(beyond, Vec3::new(4.0, 0.0, 0.0), None)
        .unwrap();
    run(&mut w, 1);
    let thrown = velocity(&w, beyond).linear.x;
    run(&mut w, 60);
    let slowed = velocity(&w, beyond).linear.x;
    assert!(
        (slowed - thrown * og_core::math::pow(20.0 / 21.0, 60.0)).abs() < 1e-3,
        "{thrown} to {slowed}"
    );
}

/// In the 2D profile the wind's z does not push a body out of its plane.
#[test]
fn air_drag_stays_in_the_plane_in_2d() {
    let (mut w, p, _) = floating(PROFILE_2D);
    let leaf = spawn_body(
        &mut w,
        at(5.0, 0.0, 0.0),
        RigidBody {
            air_drag: 5.0,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.5),
    );
    p.set_air(wind(2.0, 0.0, 9.0));
    run(&mut w, 120);
    let v = velocity(&w, leaf).linear;
    assert!(v.x > 1.9 && v.z == 0.0, "{v}");
    assert_eq!(transform(&w, leaf).position.z, 0.0);
}

/// A field over `volume` at the origin.
fn field(
    kind: og_env::force::FieldKind,
    direction: Vec3,
    strength: f32,
    volume: Option<og_env::volume::Volume>,
) -> og_env::force::Forces {
    og_env::force::Forces {
        fields: vec![og_env::force::Field {
            at: og_env::volume::Placement::at(Vec3::ZERO),
            volume,
            kind,
            direction,
            strength,
        }],
    }
}

fn cube(size: f32) -> Option<og_env::volume::Volume> {
    Some(og_env::volume::Volume {
        size: Vec3::splat(size),
        ..Default::default()
    })
}

/// A field accelerates what is inside its volume, a heavy body as a light
/// one, and nothing outside it; a field that pulls draws bodies in.
#[test]
fn a_force_field_accelerates_bodies_inside_its_volume_whatever_their_mass() {
    use og_env::force::FieldKind;
    let (mut w, p, light) = floating(PROFILE_3D);
    let body = |w: &mut World, x: f32, mass: f32| {
        spawn_body(
            w,
            at(x, 0.0, 0.0),
            RigidBody {
                mass,
                ..RigidBody::dynamic()
            },
            Collider::sphere(0.5),
        )
    };
    let heavy = body(&mut w, 3.0, 200.0);
    let outside = body(&mut w, 30.0, 2.0);
    let wall = spawn_static(&mut w, at(-3.0, 0.0, 0.0), Collider::sphere(0.5));
    p.set_forces(field(FieldKind::Directional, Vec3::Y, 6.0, cube(12.0)));
    run(&mut w, 30);
    // Half a second at 6 m/s²: 3 m/s, both still inside the 12 m cube.
    for e in [light, heavy] {
        let v = velocity(&w, e).linear;
        assert!(close(v, Vec3::new(0.0, 3.0, 0.0)), "{v}");
    }
    assert_eq!(velocity(&w, outside).linear, Vec3::ZERO);
    assert_eq!(transform(&w, wall).position, Vec3::new(-3.0, 0.0, 0.0));
    assert!(close(p.force_at(Vec3::new(1.0, 2.0, 0.0)), Vec3::Y * 6.0));
    assert_eq!(p.force_at(Vec3::new(30.0, 0.0, 0.0)), Vec3::ZERO);

    // A well with no volume pulls everything towards its centre.
    let (mut w, p, _) = floating(PROFILE_3D);
    let far = body(&mut w, 40.0, 2.0);
    p.set_forces(field(FieldKind::Radial, Vec3::Y, -5.0, None));
    run(&mut w, 60);
    let v = velocity(&w, far).linear;
    assert!(close(v, Vec3::new(-5.0, 0.0, 0.0)), "{v}");
}

/// A crate asleep on the floor rises when an updraft stronger than
/// gravity is switched on under it.
#[test]
fn a_force_field_switched_on_moves_a_body_that_slept_in_it() {
    use og_env::force::FieldKind;
    let settings = Physics {
        sleep_ticks: 30,
        sleep_speed: 0.1,
        ..Physics::default()
    };
    let (mut w, p) = world(settings, 1);
    enable_lazy(&mut w);
    floor(&mut w);
    let crate_ = spawn_body(
        &mut w,
        at(0.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    run(&mut w, 240);
    assert!(
        w.get::<RigidBody>(crate_).unwrap().sleeping,
        "the crate came to rest"
    );
    p.set_forces(field(FieldKind::Directional, Vec3::Y, 20.0, cube(8.0)));
    run(&mut w, 30);
    let y = transform(&w, crate_).position.y;
    assert!(y > 1.0, "lifted to {y}");
}

/// In the 2D profile a field's z does not push a body out of its plane.
#[test]
fn a_force_field_stays_in_the_plane_in_2d() {
    use og_env::force::FieldKind;
    let (mut w, p, e) = floating(PROFILE_2D);
    let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
    p.set_forces(field(FieldKind::Directional, direction, 4.0, None));
    run(&mut w, 60);
    let v = velocity(&w, e).linear;
    assert!(v.x > 2.8 && v.z == 0.0, "{v}");
    assert_eq!(transform(&w, e).position.z, 0.0);
}

/// An updraft lifts a character standing in it, and gravity has it back
/// when the field is gone.
#[test]
fn a_force_field_lifts_a_character_by_its_vertical_part() {
    use og_env::force::FieldKind;
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    let hero = w
        .spawn((
            at(0.0, 0.82, 0.0),
            Collider::capsule(0.3, 1.0),
            CharacterController::default(),
        ))
        .unwrap();
    run(&mut w, 30);
    let stood = transform(&w, hero).position.y;
    assert!(w.get::<CharacterController>(hero).unwrap().grounded);
    // Sideways and up: only the lift reaches it.
    let direction = Vec3::new(1.0, 1.0, 0.0).normalize();
    p.set_forces(field(FieldKind::Directional, direction, 40.0, cube(20.0)));
    run(&mut w, 30);
    let lifted = transform(&w, hero).position;
    assert!(lifted.y > stood + 1.0, "lifted to {}", lifted.y);
    assert_eq!(lifted.x, 0.0, "a character takes the vertical part");
    p.set_forces(Default::default());
    run(&mut w, 240);
    assert!((transform(&w, hero).position.y - stood).abs() < 0.05);
}
