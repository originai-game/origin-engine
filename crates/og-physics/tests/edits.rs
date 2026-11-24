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

#[test]
fn pixel_edits_keep_holes_and_replay_across_chunk_boundaries() {
    use og_physics::destruction::terrain2d::{BEDROCK, PixelTerrain, collider::chunk_polyline};
    let mut grid = PixelTerrain::new(13, 9, 4);
    for x in 0..13 {
        grid.set(x, 0, BEDROCK);
    }
    for y in 1..8 {
        for x in 1..12 {
            grid.set(x, y, 1);
        }
    }
    assert!(grid.detach().is_empty());
    grid.take_dirty();
    grid.circle(6.0, 4.0, 1.4, 0);
    let replay = grid.clone();
    assert!(
        grid.detach().is_empty(),
        "a hole does not sever the surrounding ground"
    );
    // The collider has the exact hole at tolerance zero, without filling
    // it through a convex hull or introducing a wall at x=4 or x=8.
    let (mut w, ph) = world(
        Physics {
            profile: PROFILE_2D,
            gravity: Vec3::ZERO,
            ..Default::default()
        },
        91,
    );
    for k in 0..12 {
        if let Some(mesh) = chunk_polyline(&grid, k, 0.0, 1.0) {
            ph.set_polyline(100 + k as u64, mesh).unwrap();
            spawn_static(
                &mut w,
                at((k % 4 * 4) as f32, (k / 4 * 4) as f32, 0.0),
                Collider {
                    shape: SHAPE_POLYLINE,
                    mesh: 100 + k as u64,
                    ..Default::default()
                },
            );
        }
    }
    run(&mut w, 1);
    let hit = ph
        .query(&w)
        .unwrap()
        .raycast(&RaycastDesc {
            origin: Vec3::new(6.0, 4.0, 0.0),
            direction: Vec3::X,
            max_distance: 10.0,
            ..Default::default()
        })
        .unwrap()
        .unwrap();
    assert!(
        (hit.point.x - 7.0).abs() < 1e-4,
        "the hole has an inner boundary: {hit:?}"
    );
    grid.circle(6.0, 1.5, 20.0, 0);
    let islands = grid.detach();
    let mut back = replay;
    back.circle(6.0, 1.5, 20.0, 0);
    assert_eq!(islands, back.detach());
    assert_eq!(grid.solid(), 13);
    assert_eq!(grid.cells(), back.cells());
}

#[test]
fn destruction_warm_starts_restore_and_foundations_cannot_pull() {
    use og_physics::destruction::{
        Chunk, FractureAsset, Instance, Material, NONE, stress::StressLimits,
    };
    let a = FractureAsset {
        support: 1,
        chunks: vec![Chunk {
            centroid: Vec3::Y,
            volume: 2.0,
            parent: NONE,
            anchor_area: 1.0,
            anchor_centroid: Vec3::ZERO,
            anchor_normal: Vec3::Y,
            ..Default::default()
        }],
        ..Default::default()
    };
    let material = Material {
        density: 700.0,
        toughness: 1000.0,
        limits: StressLimits::from_strength(1e6, 0.6, 0.0),
    };
    let mut inst = Instance::new(&a);
    inst.dirty = true;
    let mut budget = u64::MAX;
    for _ in 0..100 {
        inst.stress(
            &a,
            &inst.islands(&a),
            &[],
            -Vec3::Y * 10.0,
            1.0,
            &material,
            1,
            &mut budget,
        );
        if !inst.dirty {
            break;
        }
    }
    let reaction = inst.impulses[0].2.lin;
    assert!(
        (reaction.y + 14000.0).abs() < 100.0,
        "foundation carries actual mass: {reaction:?}"
    );
    let mut state = destruction::prop::state::State::default();
    // Instance is saved through the actual destruction state block.
    let prop = destruction::prop::state::Prop {
        asset: 1,
        model: Vec::new(),
        inst: inst.clone(),
        frame: [0.0; 10],
        entities: vec![1],
        body: vec![0],
        loose: vec![u64::MAX],
        collider: None,
        placed: true,
        anchors: true,
        fractured: false,
        island_cache: (0, Vec::new()),
    };
    state.props.insert(1, prop);
    let mut bytes = Vec::new();
    state.save(&mut bytes);
    let restored = destruction::prop::state::State::decode(&bytes).unwrap();
    assert_eq!(restored.props[&1].inst, inst);
    inst.stress(
        &a,
        &inst.islands(&a),
        &[],
        Vec3::Y * 10.0,
        1.0,
        &material,
        100,
        &mut budget,
    );
    assert_eq!(
        inst.anchors[0], 0.0,
        "a separating foundation opens instead of pulling"
    );
    assert_eq!(inst.impulses[0].2.lin, Vec3::ZERO);
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
