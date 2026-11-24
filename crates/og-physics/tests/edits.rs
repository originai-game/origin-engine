//! The rapier world persists between ticks, so a restore into a world that
//! has run on must replace the newer rapier state with the snapshot's.
//!
//! Ways this could fail, caught here: the restored world keeps the newer
//! rapier state and replays differently.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use support::*;

/// One vehicle path per profile: explicit multi-axle mounts, live visual
/// updates, torque/braking, teleport and bit-exact save/resume telemetry.
#[test]
fn vehicles_drive_edit_teleport_and_restore_in_both_profiles() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        let (mut w, p) = world(
            Physics {
                profile,
                substeps: 4,
                ..Physics::default()
            },
            29,
        );
        enable_lazy(&mut w);
        let ground = spawn_static(
            &mut w,
            at(0.0, -0.5, 0.0),
            Collider {
                friction: 1.0,
                ..Collider::cuboid(Vec3::new(500.0, 1.0, 500.0))
            },
        );
        let flat = profile == PROFILE_2D;
        let car = w
            .spawn((
                at(0.0, 1.1, 7.0),
                RigidBody {
                    mass: 1000.0,
                    ..RigidBody::dynamic()
                },
                Collider::cuboid(if flat {
                    Vec3::new(3.0, 0.3, 1.0)
                } else {
                    Vec3::new(1.5, 0.3, 3.0)
                }),
                Vehicle {
                    model: VEHICLE_DRIVETRAIN,
                    engine_inertia: 0.3,
                    clutch_strength: 450.0,
                    compression_damping: 4.0,
                    rebound_damping: 5.0,
                    ..Vehicle::default()
                },
            ))
            .unwrap();
        let mut children = Vec::new();
        for i in 0..if flat { 3 } else { 6 } {
            let along = (if flat { i } else { i / 2 }) as f32 - 1.0;
            let mount = if flat {
                Vec3::new(along * 1.2, -0.2, 0.0)
            } else {
                Vec3::new(if i % 2 == 0 { 0.8 } else { -0.8 }, -0.2, along * 1.2)
            };
            children.push(
                w.spawn((
                    Transform::at(mount),
                    Parent {
                        entity: car.to_bits(),
                    },
                    VehicleWheel {
                        mount: if i == 0 { Vec3::ZERO } else { mount },
                        drive: true,
                        compression_damping: 4.0,
                        rebound_damping: 5.0,
                        ..VehicleWheel::default()
                    },
                ))
                .unwrap(),
            );
        }
        if !flat {
            spawn_static(
                &mut w,
                at(0.0, 0.3, 7.0),
                Collider {
                    sensor: true,
                    ..Collider::cuboid(Vec3::new(20.0, 0.05, 20.0))
                },
            );
            let ignored = spawn_static(
                &mut w,
                at(0.0, 0.15, 7.0),
                Collider::cuboid(Vec3::new(20.0, 0.05, 20.0)),
            );
            let mut ignore = og_core::List::new();
            ignore.push(car.to_bits()).unwrap();
            w.insert(
                ignored,
                ContactHook {
                    ignore,
                    ..ContactHook::default()
                },
            )
            .unwrap();
            for cast in 0..3 {
                let mut v = w.get::<Vehicle>(car).unwrap();
                v.wheel_cast = cast;
                w.insert(car, v).unwrap();
                w.step().unwrap();
                let poses = p.vehicle_wheels(car).unwrap();
                assert!(
                    poses
                        .iter()
                        .all(|p| p.contact && p.ground_entity == ground.to_bits()),
                    "full droop cast {cast}: {poses:?}"
                );
                assert!(poses.iter().any(|p| p.suspension_compression < 0.0));
            }
        }
        run(&mut w, 90);
        let poses = p.vehicle_wheels(car).unwrap();
        assert_eq!(poses.len(), if flat { 3 } else { 6 });
        assert!(
            poses
                .iter()
                .any(|p| p.contact && p.load > 0.0 && p.ground_entity == ground.to_bits()),
            "{profile}D: {poses:?}"
        );
        let start = transform(&w, car).position;
        let mut v = w.get::<Vehicle>(car).unwrap();
        v.throttle = 0.8;
        w.insert(car, v).unwrap();
        for _ in 0..120 {
            // The same entities may be animated without becoming hard points.
            for pose in p.vehicle_wheels(car).unwrap() {
                let e = children
                    .iter()
                    .find(|e| e.to_bits() == pose.entity)
                    .unwrap();
                w.insert(*e, Transform::at(Vec3::splat(-99.0))).unwrap();
            }
            w.step().unwrap();
        }
        let driven = transform(&w, car).position;
        let distance = if flat {
            driven.x - start.x
        } else {
            driven.z - start.z
        };
        assert!(
            distance > 1.0,
            "{profile}D did not drive: {distance}, {:?}",
            w.get::<Vehicle>(car)
        );
        assert!(
            p.vehicle_wheels(car)
                .unwrap()
                .iter()
                .any(|w| w.angular_velocity > 1.0)
        );
        let mut wheel = w.get::<VehicleWheel>(children[0]).unwrap();
        wheel.radius += 0.02;
        wheel.suspension_rest += 0.01;
        wheel.grip = 0.8;
        w.insert(children[0], wheel).unwrap();
        w.step().unwrap();
        assert!(
            p.vehicle_wheels(car).unwrap()[0].angular_velocity > 1.0,
            "geometry/grip edits must preserve wheel speed"
        );
        let snap = w.snapshot();
        run(&mut w, 30);
        let end = w.snapshot();
        let poses = p.vehicle_wheels(car).unwrap();
        w.restore(&snap).unwrap();
        run(&mut w, 30);
        assert_eq!(w.snapshot(), end, "{profile}D vehicle restore");
        assert_eq!(p.vehicle_wheels(car).unwrap(), poses);
        let mut t = transform(&w, car);
        t.position += Vec3::new(50.0, 4.0, 0.0);
        w.insert(car, t).unwrap();
        w.step().unwrap();
        for wheel in p.vehicle_wheels(car).unwrap() {
            assert!(
                (wheel.position - transform(&w, car).position).length() < 4.0,
                "teleport left wheel behind: {wheel:?}"
            );
            assert!(wheel.position.is_finite() && wheel.angular_velocity.is_finite());
            if flat {
                assert_eq!(wheel.position.z, 7.0);
            }
        }
        let mut v = w.get::<Vehicle>(car).unwrap();
        v.brake = 1.0;
        w.insert(car, v).unwrap();
        // Include the fall from the teleported height before expecting a stop.
        run(&mut w, 300);
        assert!(
            w.get::<Vehicle>(car).unwrap().speed.abs() < 0.01,
            "drive and brake together must stop: {:?}",
            w.get::<Vehicle>(car)
        );
    }
}

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

/// The public ragdoll path: COM activation, owner filtering, dynamic
/// stacking, point impulses, in-place tuning, a partial PD hinge, saves and despawn.
#[test]
fn ragdolls_activate_drive_tune_and_restore_in_both_profiles() {
    use og_core::List;
    use og_core::glam::Quat;
    for profile in [PROFILE_3D, PROFILE_2D] {
        let (mut w, p) = world(
            Physics {
                profile,
                gravity: Vec3::ZERO,
                ..Default::default()
            },
            73,
        );
        enable_lazy(&mut w);
        let bone = RagdollBone {
            rest: (Vec3::ZERO, Quat::IDENTITY),
            length: 2.0,
            axis: Vec3::Y,
            mass: 2.0,
            shape: 2,
            ..Default::default()
        };
        let r = Ragdoll {
            bone_mask: List::from_slice(&[0]).unwrap(),
            layers: 1,
            mask: 1,
            blend_time: 0.0,
            damping: 0.0,
            thickness: 0.1,
            drive_transform: true,
            ..Default::default()
        };
        let e = w
            .spawn((
                at(0.0, 0.0, 0.0),
                RigidBody::kinematic(),
                Collider::capsule(0.6, 2.0),
                r,
            ))
            .unwrap();
        p.set_ragdoll_rig(
            e,
            RagdollRig {
                bones: vec![bone.clone()],
            },
        )
        .unwrap();
        p.query(&w).unwrap();
        w.step().unwrap();
        let (position, rotation) = (
            Vec3::new(0.1, 0.2, 0.0),
            og_core::math::quat_from_axis_angle(Vec3::Z, 0.1),
        );
        let com = position + rotation * Vec3::Y;
        p.set_ragdoll_targets(e, vec![(position, rotation)]);
        w.insert(
            e,
            Ragdoll {
                weight: 1.0,
                impulse: Vec3::X * 6.0,
                ..r
            },
        )
        .unwrap();
        w.step().unwrap();
        let (at, q) = p.ragdoll_pose(e).unwrap()[0];
        // dt = 1/60, mass = 2; the TOTAL impulse contributes 0.05 m.
        let expected = com + (com - Vec3::Y) + Vec3::X * 0.05;
        assert!(
            (at + q * Vec3::Y - expected).length() < 1e-4,
            "profile {profile}: activation COM {:?}, expected {expected:?}",
            at + q * Vec3::Y
        );
        assert!((transform(&w, e).position - at).length() < 1e-5);
        w.despawn(e).unwrap();
        w.step().unwrap();
        assert!(p.ragdoll_pose(e).is_none() && !p.has_ragdoll_rig(e));

        // Two unrelated ragdolls must collide, unlike the removed global bit.
        let mut pair = Vec::new();
        for x in [10.0, 10.1] {
            let e = w
                .spawn((
                    Transform::at(Vec3::X * x),
                    Ragdoll {
                        weight: 1.0,
                        drive_transform: false,
                        ..r
                    },
                ))
                .unwrap();
            p.set_ragdoll_rig(
                e,
                RagdollRig {
                    bones: vec![bone.clone()],
                },
            )
            .unwrap();
            pair.push(e);
        }
        run(&mut w, 30);
        let a = p.ragdoll_pose(pair[0]).unwrap()[0].0;
        let b = p.ragdoll_pose(pair[1]).unwrap()[0].0;
        assert!(
            a.distance(b) > 0.38,
            "different ragdolls separate their 0.2 m spheres"
        );
        p.apply_ragdoll_impulse(pair[1], 0, Vec3::X * 12.0, b + Vec3::Y)
            .unwrap();
        run(&mut w, 10);
        let before = p.ragdoll_pose(pair[1]).unwrap()[0].0;
        let mut tuned = w.get::<Ragdoll>(pair[1]).unwrap();
        tuned.mass *= 2.0;
        tuned.thickness *= 0.8;
        tuned.joint_limit = 0.5;
        w.insert(pair[1], tuned).unwrap();
        w.step().unwrap();
        assert!(
            p.ragdoll_pose(pair[1]).unwrap()[0].0.x > before.x,
            "tuning must not rebuild from stale animation targets"
        );

        // A kinematic root, a hinged dynamic child, and a PD drive. No
        // collision layers here: this isolates drive from contact impulses.
        let h = w
            .spawn((
                Transform::at(Vec3::X * 20.0),
                Ragdoll {
                    bone_mask: List::from_slice(&[1]).unwrap(),
                    weight: 1.0,
                    motor_stiffness: 50.0,
                    motor_damping: 10.0,
                    layers: 0,
                    mask: 0,
                    drive_transform: false,
                    ..r
                },
            ))
            .unwrap();
        let mut child = bone.clone();
        child.parent = Some(0);
        child.rest.0 = Vec3::Y * 2.0;
        child.joint_kind = 1;
        child.hinge_axis = Vec3::Z;
        child.twist_min = -1.0;
        child.twist_max = 1.0;
        p.set_ragdoll_rig(
            h,
            RagdollRig {
                bones: vec![bone, child],
            },
        )
        .unwrap();
        w.step().unwrap();
        let anchor = p.ragdoll_pose(h).unwrap()[0];
        let checkpoint = w.snapshot();
        let kick = |w: &mut og_ecs::World| {
            p.apply_ragdoll_impulse(h, 1, Vec3::X * 4.0, Vec3::new(20.0, 4.0, 0.0))
                .unwrap();
            run(w, 120);
        };
        kick(&mut w);
        let driven = p.ragdoll_pose(h).unwrap()[1].1;
        assert!(
            driven.dot(Quat::IDENTITY).abs() > 0.999,
            "profile {profile}: the PD motor restores the target after a point hit: {driven:?}"
        );
        assert_eq!(
            p.ragdoll_pose(h).unwrap()[0],
            anchor,
            "boneMask leaves the root animated"
        );
        let end = w.snapshot();
        w.restore(&checkpoint).unwrap();
        kick(&mut w);
        assert_eq!(
            w.snapshot(),
            end,
            "ragdoll motors and filtering restore bit for bit"
        );
        let mut off = w.get::<Ragdoll>(h).unwrap();
        off.motor_stiffness = 0.0;
        off.motor_damping = 0.0;
        w.insert(h, off).unwrap();
        w.step().unwrap();
        p.apply_ragdoll_impulse(h, 1, Vec3::X * 4.0, Vec3::new(20.0, 4.0, 0.0))
            .unwrap();
        run(&mut w, 30);
        assert!(
            p.ragdoll_pose(h).unwrap()[1].1.dot(Quat::IDENTITY).abs() < 0.995,
            "zero PD coefficients must disable an existing motor"
        );
    }
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
        let roof = spawn_static(
            &mut w,
            at(0.0, 4097.8, 0.0),
            Collider::cuboid(Vec3::new(2.0, 0.2, 2.0)),
        );
        assert!(resize_character(&mut w, &p, actor, 0.2, 0.3).unwrap());
        // Queries see the stepped world, so the roof blocks once a step has taken it in.
        w.step().unwrap();
        let before = transform(&w, actor);
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

        // The default axis follows world gravity, not a hard-coded Y axis.
        // An explicit axis overrides it; zero gravity supplies no axis.
        for (gravity, up, movement, expected) in [
            (
                Vec3::new(6.0, -8.0, 0.0),
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::new(0.1, -8.0 / 60.0, 0.0),
            ),
            (
                Vec3::new(6.0, -8.0, 0.0),
                Vec3::Y,
                Vec3::ZERO,
                Vec3::new(0.0, -8.0 / 60.0, 0.0),
            ),
            (
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::new(1.0, 2.0, 0.0),
                Vec3::new(1.0, 2.0, 0.0),
            ),
        ] {
            let (mut w, _) = world(
                Physics {
                    profile,
                    gravity,
                    ..Physics::default()
                },
                31,
            );
            enable_lazy(&mut w);
            let actor = w
                .spawn((
                    at(0.0, 0.0, 0.0),
                    Collider::sphere(0.3),
                    CharacterController {
                        up,
                        movement,
                        ..CharacterController::default()
                    },
                ))
                .unwrap();
            w.step().unwrap();
            let c = w.get::<CharacterController>(actor).unwrap();
            assert!((c.velocity - expected).abs().max_element() < 1e-5);
            assert_eq!(c.up, up, "deriving an axis does not overwrite the input");
        }
    }
}
