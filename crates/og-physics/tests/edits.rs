//! The rapier world persists between ticks, so a restore into a world that
//! has run on must replace the newer rapier state with the snapshot's.
//!
//! Ways this could fail, caught here: the restored world keeps the newer
//! rapier state and replays differently.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use support::*;

#[test]
fn a_restore_into_a_world_that_ran_on_continues_from_the_snapshot() {
    let (mut w, _p) = world(Physics::default(), 17);
    floor(&mut w);
    for i in 0..20 {
        spawn_body(
            &mut w,
            at((i % 4) as f32 * 0.7 - 1.0, 1.0 + i as f32 * 1.1, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.6)),
        );
    }
    run(&mut w, 30);
    let snap = w.snapshot();
    run(&mut w, 90);
    let end = w.snapshot();
    w.restore(&snap).unwrap();
    assert_eq!(w.snapshot(), snap);
    run(&mut w, 90);
    assert_eq!(
        w.snapshot(),
        end,
        "the rewound world replays the same 90 ticks"
    );
}

/// One public controller path in both profiles: land, crouch under a roof,
/// restore, stand, jump at large world coordinates, then push a body with
/// finite mass. This also guards sequential controller/sensor publication.
#[test]
fn virtual_controllers_move_resize_and_restore_in_both_profiles() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        let (mut w, p) = world(
            Physics {
                profile,
                substeps: 4,
                ..Physics::default()
            },
            31,
        );
        enable_lazy(&mut w);
        spawn_static(
            &mut w,
            at(0.0, 4095.5, 0.0),
            Collider::cuboid(Vec3::new(40.0, 1.0, 10.0)),
        );
        let actor = w
            .spawn((
                at(0.0, 4098.0, 0.0),
                Collider::capsule(0.3, 0.8),
                CharacterController {
                    up: Vec3::Y,
                    step_height: 0.0,
                    ..CharacterController::default()
                },
            ))
            .unwrap();
        let mut landed = false;
        for _ in 0..60 {
            w.step().unwrap();
            for e in w.pending_events::<CharacterEvent>().unwrap() {
                if e.entity == actor.to_bits() && e.kind == CHARACTER_LANDED {
                    assert!(
                        e.speed < -1.0,
                        "landing reports impact, not the previous frame or zero"
                    );
                    landed = true;
                }
            }
        }
        assert!(landed && w.get::<CharacterController>(actor).unwrap().grounded);
        let before = transform(&w, actor);
        let roof = spawn_static(
            &mut w,
            at(0.0, 4097.8, 0.0),
            Collider::cuboid(Vec3::new(2.0, 0.2, 2.0)),
        );
        assert!(resize_character(&mut w, &p, actor, 0.2, 0.3).unwrap());
        assert!(!resize_character(&mut w, &p, actor, 1.4, 0.3).unwrap());
        assert_eq!(
            transform(&w, actor),
            before,
            "resize keeps the entity pose and adjusts its capsule offset"
        );
        w.step().unwrap();
        let checkpoint = w.snapshot();
        run(&mut w, 20);
        let end = w.snapshot();
        w.restore(&checkpoint).unwrap();
        run(&mut w, 20);
        assert_eq!(
            w.snapshot(),
            end,
            "controller support state restores bit for bit"
        );
        w.despawn(roof).unwrap();
        assert!(resize_character(&mut w, &p, actor, 0.8, 0.3).unwrap());
        let mut c = w.get::<CharacterController>(actor).unwrap();
        c.jump = true;
        w.insert(actor, c).unwrap();
        w.step().unwrap();
        assert!(
            w.get::<CharacterController>(actor).unwrap().velocity.y > 5.0,
            "a large world position must not masquerade as a ceiling hit"
        );

        // Teleport beside a floating crate, then drive into it. Zero virtual
        // mass pushes nothing despite the collider's kinematic solver body.
        let crate_body = spawn_body(
            &mut w,
            at(8.0, 4100.0, 0.0),
            RigidBody {
                gravity_scale: 0.0,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::ONE),
        );
        w.insert(actor, at(6.5, 4100.0, 0.0)).unwrap();
        c = CharacterController {
            up: Vec3::Y,
            gravity_scale: 0.0,
            movement: Vec3::X * 6.0,
            ..CharacterController::default()
        };
        w.insert(actor, c).unwrap();
        run(&mut w, 30);
        assert!((transform(&w, crate_body).position.x - 8.0).abs() < 0.001);
        c = w.get::<CharacterController>(actor).unwrap();
        c.mass = 10.0;
        c.push_strength = 100.0;
        w.insert(actor, c).unwrap();
        run(&mut w, 15);
        assert!(
            transform(&w, crate_body).position.x > 8.01,
            "finite-mass contact pushes the crate"
        );
        let pushed = w.snapshot();
        run(&mut w, 10);
        let end = w.snapshot();
        w.restore(&pushed).unwrap();
        run(&mut w, 10);
        assert_eq!(w.snapshot(), end);

        // A second controller must see the first controller's new pose,
        // while sensors see the interpolated path in this same tick.
        let sensor = spawn_static(
            &mut w,
            at(-10.0, 4120.0, 0.0),
            Collider {
                sensor: true,
                ..Collider::cuboid(Vec3::splat(0.8))
            },
        );
        let first = w
            .spawn((
                at(-12.0, 4120.0, 0.0),
                Collider::sphere(0.3),
                CharacterController {
                    gravity_scale: 0.0,
                    movement: Vec3::X * 120.0,
                    ..CharacterController::default()
                },
            ))
            .unwrap();
        let second = w
            .spawn((
                at(-8.0, 4120.0, 0.0),
                Collider::sphere(0.3),
                CharacterController {
                    gravity_scale: 0.0,
                    movement: -Vec3::X * 120.0,
                    ..CharacterController::default()
                },
            ))
            .unwrap();
        w.step().unwrap();
        assert!(
            transform(&w, second).position.x - transform(&w, first).position.x > 0.59,
            "later controllers query earlier controllers at their new poses"
        );
        assert!(
            w.pending_events::<SensorEntered>()
                .unwrap()
                .iter()
                .any(|e| e.sensor == sensor.to_bits() && e.other == first.to_bits()),
            "controller sensor entry is visible in the same tick"
        );
    }
}
