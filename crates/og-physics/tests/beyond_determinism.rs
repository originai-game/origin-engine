//! Determinism of physics beyond rigid bodies, in the shape of the pile
//! test: a 3D world (a character walking onto a moving platform, a motor
//! hinge, a chain of ball joints, a raycast car, a Verlet rope and a cloth,
//! height-field water with crates dropped in, a ragdoll that goes limp and
//! recovers) and a 2D one (a platformer running, jumping through a one-way
//! platform and off a wall, a wheel-joint car, a particle fluid, a water
//! line), each run 1200 ticks twice to the same world hash and snapshot,
//! and restored at tick 600 into a fresh world (another seed) to end in the
//! same world as the uninterrupted run.
//!
//! Ways this could fail, each caught here: state of a feature kept outside
//! the og.physics state block or the components (ropes, cloth, fluid,
//! water, joints, wheels, ragdoll bodies, controller timers): the restored
//! run diverges; an iteration order that is not fixed or a platform-
//! dependent function: the two runs diverge (and the host and wasm32 hashes
//! of tools/wasm-determinism would); a feature that does nothing: the
//! checks on what moved fail.

mod support;

use og_core::glam::{Quat, Vec2, Vec3};
use og_core::math::sin;
use og_ecs::{Entity, World, WorldDesc};
use og_physics::*;
use std::time::Instant;

const TICKS: u64 = 1200;
const HALF: u64 = 600;

fn new_world(seed: u32, profile: u32) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    let physics = install(
        &mut w,
        Physics {
            profile,
            ..Physics::default()
        },
    )
    .unwrap();
    (w, physics)
}

fn spawn(w: &mut World, b: impl og_ecs::Bundle) -> Entity {
    w.spawn(b).unwrap()
}

fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::at(Vec3::new(x, y, z))
}

fn rig() -> RagdollRig {
    let bone = |parent: Option<u32>, y: f32, length: f32| RagdollBone {
        parent,
        rest: (Vec3::new(0.0, y, 0.0), Quat::IDENTITY),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, 1.0, 0.3),
            bone(Some(0), 1.3, 0.5),
            bone(Some(1), 1.8, 0.3),
        ],
    }
}

/// The named entities a scene's inputs are driven through.
struct Cast {
    walker: Entity,
    platform: Entity,
    car: Entity,
    ragdoll: Option<Entity>,
}

fn scene3d(w: &mut World, physics: &PhysicsSystem) -> Cast {
    spawn(
        w,
        (
            at(0.0, -0.5, 0.0),
            Collider::cuboid(Vec3::new(600.0, 1.0, 600.0)),
        ),
    );
    let platform = spawn(
        w,
        (
            at(6.0, 0.6, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(3.0, 0.2, 3.0)),
        ),
    );
    let walker = spawn(
        w,
        (
            at(0.0, 0.95, 0.0),
            Collider::capsule(0.4, 1.0),
            CharacterController::default(),
        ),
    );
    // A motor hinge and a chain of ball joints hanging from the world.
    let wheel = spawn(
        w,
        (
            at(-6.0, 3.0, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::new(2.0, 0.2, 0.2)),
        ),
    );
    w.insert(
        wheel,
        Joint {
            kind: JOINT_REVOLUTE,
            other_anchor: Vec3::new(-6.0, 3.0, 0.0),
            axis: Vec3::Z,
            motor: MOTOR_VELOCITY,
            motor_target: 2.0,
            ..Joint::default()
        },
    )
    .unwrap();
    let mut previous = 0u64;
    for i in 0..6 {
        let link = spawn(
            w,
            (
                at(-10.0 + i as f32 * 0.6, 6.0, 2.0),
                RigidBody::dynamic(),
                Collider::sphere(0.2),
            ),
        );
        w.insert(
            link,
            Joint {
                kind: JOINT_BALL,
                other: previous,
                anchor: Vec3::new(-0.3, 0.0, 0.0),
                other_anchor: if previous == 0 {
                    Vec3::new(-10.3, 6.0, 2.0)
                } else {
                    Vec3::new(0.3, 0.0, 0.0)
                },
                ..Joint::default()
            },
        )
        .unwrap();
        previous = link.to_bits();
    }
    let car = spawn(
        w,
        (
            at(0.0, 1.2, -10.0),
            RigidBody {
                mass: 150.0,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::new(1.8, 0.6, 3.6)),
            Vehicle {
                engine_force: 300.0,
                ..Vehicle::default()
            },
        ),
    );
    spawn(
        w,
        (
            at(4.0, 5.0, 4.0),
            Rope {
                segments: 12,
                length: 3.0,
                wind: Vec3::new(1.0, 0.0, 0.5),
                ..Rope::default()
            },
        ),
    );
    spawn(w, (at(4.0, 2.5, 4.0), Collider::sphere(0.6)));
    spawn(
        w,
        (
            at(-4.0, 4.0, 6.0),
            Cloth {
                columns: 8,
                rows: 8,
                wind: Vec3::new(0.0, 0.0, 6.0),
                ..Cloth::default()
            },
        ),
    );
    spawn(
        w,
        (
            at(0.0, 0.0, 20.0),
            WaterSurface {
                resolution: 32,
                size: 10.0,
                ..WaterSurface::default()
            },
        ),
    );
    for i in 0..3 {
        spawn(
            w,
            (
                at(-2.0 + i as f32 * 2.0, 3.0 + i as f32, 20.0),
                RigidBody::dynamic(),
                Collider {
                    density: 0.5,
                    ..Collider::cuboid(Vec3::splat(0.8))
                },
            ),
        );
    }
    let ragdoll = spawn(w, (at(10.0, 0.0, 10.0), Ragdoll::default()));
    physics.set_ragdoll_rig(ragdoll, rig()).unwrap();
    Cast {
        walker,
        platform,
        car,
        ragdoll: Some(ragdoll),
    }
}

fn scene2d(w: &mut World) -> Cast {
    spawn(
        w,
        (
            at(0.0, -0.5, 0.0),
            Collider::cuboid(Vec3::new(600.0, 1.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(8.0, 4.0, 0.0),
            Collider::cuboid(Vec3::new(1.0, 8.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(3.0, 2.0, 0.0),
            Collider::cuboid(Vec3::new(3.0, 0.2, 1.0)).on_layers(2, u32::MAX),
        ),
    );
    let platform = spawn(
        w,
        (
            at(-6.0, 1.5, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(2.0, 0.2, 1.0)),
        ),
    );
    let walker = spawn(
        w,
        (
            at(0.0, 0.81, 0.0),
            Collider::cuboid(Vec3::new(0.8, 1.6, 1.0)),
            PlatformerController {
                one_way_layers: 2,
                air_jumps: 1,
                ..PlatformerController::default()
            },
        ),
    );
    let car = spawn(
        w,
        (
            at(100.0, 1.5, 0.0),
            RigidBody {
                mass: 4.0,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::new(3.0, 0.6, 1.0)),
            Vehicle {
                wheel_base: 2.2,
                wheel_radius: 0.45,
                engine_force: 40.0,
                max_speed: 6.0,
                ..Vehicle::default()
            },
        ),
    );
    // A tank of fluid, and a pool a ball falls into.
    spawn(
        w,
        (
            at(-40.0, -0.25, 0.0),
            Collider::cuboid(Vec3::new(5.0, 0.5, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(-42.75, 2.0, 0.0),
            Collider::cuboid(Vec3::new(0.5, 5.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(-37.25, 2.0, 0.0),
            Collider::cuboid(Vec3::new(0.5, 5.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(-41.0, 3.0, 0.0),
            Fluid2D {
                count: 300,
                size: Vec2::new(1.6, 2.0),
                seed: 5,
                ..Fluid2D::default()
            },
        ),
    );
    spawn(
        w,
        (
            at(30.0, 0.0, 0.0),
            WaterLine {
                width: 8.0,
                columns: 64,
                ..WaterLine::default()
            },
        ),
    );
    spawn(
        w,
        (
            at(30.0, -3.5, 0.0),
            Collider::cuboid(Vec3::new(10.0, 1.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(29.0, 4.0, 0.0),
            RigidBody::dynamic(),
            Collider::sphere(0.4),
        ),
    );
    Cast {
        walker,
        platform,
        car,
        ragdoll: None,
    }
}

/// The inputs at `tick`, from the tick alone (what a game's systems would do).
fn drive(w: &mut World, physics: &PhysicsSystem, cast: &Cast, tick: u64, profile: u32) {
    let s = tick as f32 / 60.0;
    if profile == PROFILE_3D {
        let mut c = w.get::<CharacterController>(cast.walker).unwrap();
        c.movement = Vec3::new(if tick < 400 { 2.0 } else { -1.0 }, 0.0, sin(s) * 0.5);
        c.jump = tick % 150 < 10;
        w.insert(cast.walker, c).unwrap();
        w.insert(cast.platform, at(6.0 + sin(s * 0.7) * 2.0, 0.6, 0.0))
            .unwrap();
        if let Some(r) = cast.ragdoll {
            let mut rd = w.get::<Ragdoll>(r).unwrap();
            rd.weight = if (300..800).contains(&tick) { 1.0 } else { 0.0 };
            rd.impulse = Vec3::new(30.0, 10.0, 0.0);
            w.insert(r, rd).unwrap();
            let targets = rig()
                .bones
                .iter()
                .map(|b| {
                    (
                        Vec3::new(10.0 + sin(s) * 0.2 * b.rest.0.y, b.rest.0.y, 10.0),
                        Quat::IDENTITY,
                    )
                })
                .collect();
            physics.set_ragdoll_targets(r, targets);
        }
    } else {
        let mut p = w.get::<PlatformerController>(cast.walker).unwrap();
        p.move_x = if (tick / 120).is_multiple_of(2) {
            1.0
        } else {
            -0.6
        };
        p.jump = tick % 70 < 14 || (tick % 70 > 30 && tick % 70 < 34);
        p.down = tick % 400 > 380;
        w.insert(cast.walker, p).unwrap();
        w.insert(
            cast.platform,
            at(-6.0 + sin(s) * 2.0, 1.5 + sin(s * 0.5), 0.0),
        )
        .unwrap();
    }
    let mut v = w.get::<Vehicle>(cast.car).unwrap();
    v.throttle = if tick < 700 { 1.0 } else { -0.5 };
    v.steer = sin(s * 0.3) * if profile == PROFILE_3D { 0.25 } else { 0.0 };
    v.brake = if tick > 1100 { 1.0 } else { 0.0 };
    w.insert(cast.car, v).unwrap();
}

fn run(w: &mut World, physics: &PhysicsSystem, cast: &Cast, from: u64, to: u64, profile: u32) {
    for tick in from..to {
        drive(w, physics, cast, tick, profile);
        w.step().unwrap();
    }
}

fn build(seed: u32, profile: u32) -> (World, PhysicsSystem, Cast) {
    let (mut w, physics) = new_world(seed, profile);
    let cast = if profile == PROFILE_3D {
        scene3d(&mut w, &physics)
    } else {
        scene2d(&mut w)
    };
    (w, physics, cast)
}

fn check(profile: u32) -> String {
    let started = Instant::now();
    let second = std::thread::spawn(move || {
        let (mut b, pb, cb) = build(7, profile);
        run(&mut b, &pb, &cb, 0, TICKS, profile);
        b
    });
    let (mut a, pa, ca) = build(7, profile);
    let start = a.world_hash();
    run(&mut a, &pa, &ca, 0, HALF, profile);
    let half = a.snapshot();
    let half_hash = a.world_hash();
    run(&mut a, &pa, &ca, HALF, TICKS, profile);
    let b = second.join().unwrap();
    assert_eq!(
        a.world_hash(),
        b.world_hash(),
        "two {profile}D runs diverged"
    );
    assert_eq!(a.snapshot(), b.snapshot());

    // A fresh world (another seed), its assets (the ragdoll rig) registered
    // again, restored at tick 600, ends where the uninterrupted run did.
    let (mut c, pc, cc) = build(99, profile);
    c.restore(&half).unwrap();
    assert_eq!(c.world_hash(), half_hash);
    run(&mut c, &pc, &cc, HALF, TICKS, profile);
    assert_eq!(
        c.world_hash(),
        a.world_hash(),
        "the restored {profile}D run diverged"
    );
    // What physics keeps beyond the components ends the same too (it is
    // in the state block, so in the hash, but these read it back).
    for (i, (ea, ec)) in a.entities().into_iter().zip(c.entities()).enumerate() {
        assert_eq!(ea, ec, "entity {i}");
        assert_eq!(pa.rope_points(ea), pc.rope_points(ec), "rope {ea}");
        assert_eq!(pa.cloth_points(ea), pc.cloth_points(ec), "cloth {ea}");
        assert_eq!(pa.fluid_particles(ea), pc.fluid_particles(ec), "fluid {ea}");
        assert_eq!(pa.water_heights(ea), pc.water_heights(ec), "water {ea}");
        assert_eq!(pa.water_line(ea), pc.water_line(ec), "water line {ea}");
        assert_eq!(pa.vehicle_wheels(ea), pc.vehicle_wheels(ec), "wheels {ea}");
        assert_eq!(pa.ragdoll_pose(ea), pc.ragdoll_pose(ec), "ragdoll {ea}");
    }
    assert_ne!(a.world_hash(), start);

    // It all moved.
    let walked = a.get::<Transform>(ca.walker).unwrap().position;
    assert!(walked.length() > 1.0, "the controller stayed at {walked}");
    let car = a.get::<Vehicle>(ca.car).unwrap();
    let ct = a.get::<Transform>(ca.car).unwrap();
    assert!(
        car.contacts > 0,
        "{car:?} {ct:?} up {}",
        ct.quaternion * Vec3::Y
    );
    if let Some(r) = ca.ragdoll {
        assert!(pa.ragdoll_pose(r).is_some());
    }
    format!(
        "{{\"profile\":{profile},\"ticks\":{TICKS},\"hash_tick_0\":\"{start:016x}\",\"hash_tick_{HALF}\":\"{half_hash:016x}\",\"hash_tick_{TICKS}\":\"{:016x}\",\"hash_tick_{TICKS}_restored\":\"{:016x}\",\"snapshot_bytes\":{},\"seconds\":{:.3}}}",
        a.world_hash(),
        c.world_hash(),
        half.len(),
        started.elapsed().as_secs_f64()
    )
}

#[test]
fn physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run() {
    let three = check(PROFILE_3D);
    let two = check(PROFILE_2D);
    let report = format!("[{three},\n{two}]\n");
    print!("{report}");
    if let Some(path) = std::env::var_os("OG_PHYSICS_BEYOND_REPORT") {
        std::fs::write(path, report).unwrap();
    }
}
