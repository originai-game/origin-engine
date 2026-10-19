//! The beyond-rigid-bodies scenario: a 3D world (a character walking onto
//! a moving platform, a motor hinge, a chain of ball joints, a raycast car,
//! a Verlet rope and two cloths, one piling up on itself with
//! self-collision, height-field water with crates dropped in, a ragdoll that
//! goes limp and recovers) and a 2D one (a platformer, a wheel-joint car, a
//! particle fluid, two fluids of one group layering and mixing, a water
//! line, a 2D ragdoll knocked over and recovering), every input derived from
//! the tick. `tests/beyond_determinism.rs` runs it twice and across a
//! restore; `tools/wasm-determinism` hashes it on the host and on wasm32.
#![allow(dead_code)]

use og_core::glam::{Quat, Vec2, Vec3};
use og_core::math::sin;
use og_ecs::{Entity, World, WorldDesc};
use og_physics::*;

pub const TICKS: u64 = 1200;
pub const HALF: u64 = 600;

pub fn new_world(seed: u32, profile: u32) -> (World, PhysicsSystem) {
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

pub fn spawn(w: &mut World, b: impl og_ecs::Bundle) -> Entity {
    w.spawn(b).unwrap()
}

pub fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::at(Vec3::new(x, y, z))
}

pub fn rig() -> RagdollRig {
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

/// A side-on figure in the plane z = 0 for the 2D ragdoll: pelvis, torso
/// and head up, a thigh and shin down (turned half a turn about z).
pub fn rig_2d() -> RagdollRig {
    let down = Quat::from_xyzw(0.0, 0.0, 1.0, 0.0);
    let bone = |parent: Option<u32>, x: f32, y: f32, rotation: Quat, length: f32| RagdollBone {
        parent,
        rest: (Vec3::new(x, y, 0.0), rotation),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, 0.0, 1.0, Quat::IDENTITY, 0.3),
            bone(Some(0), 0.0, 1.3, Quat::IDENTITY, 0.5),
            bone(Some(1), 0.0, 1.8, Quat::IDENTITY, 0.3),
            bone(Some(0), 0.1, 1.0, down, 0.45),
            bone(Some(3), 0.1, 0.55, down, 0.45),
        ],
    }
}

/// The named entities a scene's inputs are driven through.
pub struct Cast {
    pub walker: Entity,
    pub platform: Entity,
    pub car: Entity,
    pub ragdoll: Option<Entity>,
    /// The self-colliding cloth (3D), its top row lowered onto the floor.
    pub folding: Option<Entity>,
}

/// Where the 3D folding cloth's top row is at `tick`: lowered from 1.4 m
/// to 0.5 m over two seconds, swaying.
pub fn folding_at(tick: u64) -> Transform {
    let s = (tick as f32 / 120.0).min(1.0);
    at(-12.0, 1.4 - 0.9 * s, -8.0 + 0.1 * sin(tick as f32 * 0.08))
}

pub fn scene3d(w: &mut World, physics: &PhysicsSystem) -> Cast {
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
    // A cloth lowered onto the floor, folding on itself.
    let folding = spawn(
        w,
        (
            folding_at(0),
            Cloth {
                columns: 12,
                rows: 16,
                spacing: 0.06,
                gust: 0.0,
                radius: 0.025,
                thickness: 0.06,
                substeps: 2,
                iterations: 6,
                friction: 0.9,
                ..Cloth::default()
            },
        ),
    );
    Cast {
        walker,
        platform,
        car,
        ragdoll: Some(ragdoll),
        folding: Some(folding),
    }
}

pub fn scene2d(w: &mut World, physics: &PhysicsSystem) -> Cast {
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
    // A tank of two fluids of one group: the heavy one poured on the light.
    spawn(
        w,
        (
            at(-60.0, -0.25, 0.0),
            Collider::cuboid(Vec3::new(5.0, 0.5, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(-62.25, 3.0, 0.0),
            Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
        ),
    );
    spawn(
        w,
        (
            at(-57.75, 3.0, 0.0),
            Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
        ),
    );
    for (y, density, seed) in [(2.2, 3.0, 11), (0.6, 1.0, 12)] {
        spawn(
            w,
            (
                at(-60.0, y, 0.0),
                Fluid2D {
                    count: 150,
                    size: Vec2::new(3.6, 0.8),
                    group: 1,
                    density,
                    mixing: 0.3,
                    seed,
                    ..Fluid2D::default()
                },
            ),
        );
    }
    // A 2D ragdoll on a ledge, knocked over and recovering.
    spawn(
        w,
        (
            at(50.0, 0.75, 0.0),
            Collider::cuboid(Vec3::new(2.0, 1.5, 1.0)),
        ),
    );
    let ragdoll = spawn(
        w,
        (
            at(50.0, 1.5, 0.0),
            Ragdoll {
                impulse: Vec3::new(8.0, 3.0, 0.0),
                ..Ragdoll::default()
            },
        ),
    );
    physics.set_ragdoll_rig(ragdoll, rig_2d()).unwrap();
    Cast {
        walker,
        platform,
        car,
        ragdoll: Some(ragdoll),
        folding: None,
    }
}

/// The inputs at `tick`, from the tick alone (what a game's systems would do).
pub fn drive(w: &mut World, physics: &PhysicsSystem, cast: &Cast, tick: u64, profile: u32) {
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
        if let Some(r) = cast.ragdoll {
            let mut rd = w.get::<Ragdoll>(r).unwrap();
            rd.weight = if (200..700).contains(&tick) { 1.0 } else { 0.0 };
            w.insert(r, rd).unwrap();
            let targets = rig_2d()
                .bones
                .iter()
                .map(|b| {
                    (
                        Vec3::new(
                            50.0 + b.rest.0.x + sin(s) * 0.1 * b.rest.0.y,
                            1.5 + b.rest.0.y,
                            0.0,
                        ),
                        b.rest.1,
                    )
                })
                .collect();
            physics.set_ragdoll_targets(r, targets);
        }
    }
    if let Some(f) = cast.folding {
        w.insert(f, folding_at(tick)).unwrap();
    }
    let mut v = w.get::<Vehicle>(cast.car).unwrap();
    v.throttle = if tick < 700 { 1.0 } else { -0.5 };
    v.steer = sin(s * 0.3) * if profile == PROFILE_3D { 0.25 } else { 0.0 };
    v.brake = if tick > 1100 { 1.0 } else { 0.0 };
    w.insert(cast.car, v).unwrap();
}

pub fn run(w: &mut World, physics: &PhysicsSystem, cast: &Cast, from: u64, to: u64, profile: u32) {
    for tick in from..to {
        drive(w, physics, cast, tick, profile);
        w.step().unwrap();
    }
}

pub fn build(seed: u32, profile: u32) -> (World, PhysicsSystem, Cast) {
    let (mut w, physics) = new_world(seed, profile);
    let cast = if profile == PROFILE_3D {
        scene3d(&mut w, &physics)
    } else {
        scene2d(&mut w, &physics)
    };
    (w, physics, cast)
}
