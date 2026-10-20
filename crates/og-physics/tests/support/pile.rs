//! The 500-body pile: boxes, spheres, capsules, cylinders and convex hulls
//! dropped into a walled pen on a static floor, a kinematic paddle sweeping
//! through them, a sensor in one corner, and a churn system that despawns a
//! random body and drops a new one every 20 ticks (so rapier bodies appear
//! and vanish with their components). Everything random comes from the
//! world's Rng. [`pen`] is the same bodies without the paddle, the sensor
//! and the churn, as many as asked, in a pen that grows with them.

use super::{at, floor, spawn_static};
use og_core::glam::Vec3;
use og_core::math::{quat_from_axis_angle, sin};
use og_core::{Reflect, Rng};
use og_ecs::{QueryDesc, SystemDesc, World, WorldDesc};
use og_physics::*;

pub const BODIES: usize = 500;
pub const CONVEX_MESH: u64 = 7;

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct PileStats {
    pub started: u64,
    pub ended: u64,
    pub entered: u64,
    pub exited: u64,
    pub churned: u64,
}

/// Paddle marker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct Paddle {}

fn random_body(rng: &mut Rng) -> (Transform, RigidBody, Collider) {
    random_body_in(rng, 4.0)
}

/// A random body over `half` metres either side of the origin.
fn random_body_in(rng: &mut Rng, half: f32) -> (Transform, RigidBody, Collider) {
    let kind = rng.integer(0, 4) as u32;
    let s = rng.range_f32(0.25, 0.6);
    let collider = match kind {
        0 => Collider::cuboid(Vec3::new(
            rng.range_f32(0.3, 1.0),
            rng.range_f32(0.3, 1.0),
            rng.range_f32(0.3, 1.0),
        )),
        1 => Collider::sphere(s),
        2 => Collider::capsule(s * 0.6, s),
        3 => Collider::cylinder(s, s * 1.5),
        _ => Collider::convex(CONVEX_MESH),
    };
    let collider = Collider {
        friction: rng.range_f32(0.3, 0.9),
        restitution: rng.range_f32(0.0, 0.3),
        ..collider
    };
    let axis = Vec3::new(rng.range_f32(-1.0, 1.0), 1.0, rng.range_f32(-1.0, 1.0)).normalize();
    let t = Transform {
        position: Vec3::new(
            rng.range_f32(-half, half),
            rng.range_f32(1.0, 40.0),
            rng.range_f32(-half, half),
        ),
        quaternion: quat_from_axis_angle(axis, rng.range_f32(0.0, 3.0)),
        scale: Vec3::ONE,
    };
    (t, RigidBody::dynamic(), collider)
}

/// The pile world for `seed` at tick 0, and its physics handle.
pub fn build(seed: u32) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    w.register::<PileStats>().unwrap();
    w.register::<Paddle>().unwrap();
    // The physics types are registered before the game systems that name
    // them; physics itself is installed after those systems, so it steps
    // what they moved.
    for r in [
        w.register::<RigidBody>(),
        w.register::<Collider>(),
        w.register::<Velocity>(),
        w.register::<CollisionStarted>(),
        w.register::<CollisionEnded>(),
        w.register::<SensorEntered>(),
        w.register::<SensorExited>(),
    ] {
        r.unwrap();
    }
    w.insert_resource(PileStats::default()).unwrap();

    floor(&mut w);
    for (x, z, sx, sz) in [
        (5.5, 0.0, 1.0, 12.0),
        (-5.5, 0.0, 1.0, 12.0),
        (0.0, 5.5, 12.0, 1.0),
        (0.0, -5.5, 12.0, 1.0),
    ] {
        spawn_static(
            &mut w,
            at(x, 15.0, z),
            Collider::cuboid(Vec3::new(sx, 30.0, sz)),
        );
    }
    spawn_static(
        &mut w,
        at(-4.0, 1.0, -4.0),
        Collider::cuboid(Vec3::new(2.0, 2.0, 2.0)).as_sensor(),
    );
    w.spawn((
        at(0.0, 0.4, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::new(1.0, 0.8, 6.0)),
        Velocity::default(),
        Paddle {},
    ))
    .unwrap();
    let mut rng = *w.resource::<Rng>().unwrap();
    for _ in 0..BODIES {
        w.spawn(random_body(&mut rng)).unwrap();
    }
    *w.resource_mut::<Rng>().unwrap() = rng;

    let position = w.field::<Transform>("position").unwrap();
    let linear = w.field::<Velocity>("linear").unwrap();
    w.add_system(
        SystemDesc::new("paddle")
            .write::<Transform>()
            .write::<Velocity>()
            .read::<Paddle>()
            .query(
                QueryDesc::new()
                    .read::<Paddle>()
                    .write::<Transform>()
                    .write::<Velocity>(),
            ),
        move |ctx| {
            let t = ctx.time().elapsed as f32;
            let x = 3.5 * sin(t * 0.8);
            let next = 3.5 * sin((t + ctx.time().dt) * 0.8);
            for a in ctx.query(0)? {
                for p in a.write::<Vec3>(position)?.iter_mut() {
                    p.x = x;
                }
                for v in a.write::<Vec3>(linear)?.iter_mut() {
                    v.x = (next - x) / ctx.time().dt;
                }
            }
            Ok(())
        },
    )
    .unwrap();

    let kind = w.field::<RigidBody>("kind").unwrap();
    w.add_system(
        SystemDesc::new("churn")
            .write::<Rng>()
            .write::<PileStats>()
            .write::<Transform>()
            .write::<RigidBody>()
            .write::<Collider>()
            .query(
                QueryDesc::new()
                    .read::<RigidBody>()
                    .read::<Collider>()
                    .without::<Paddle>(),
            ),
        move |ctx| {
            if ctx.time().tick % 20 != 19 {
                return Ok(());
            }
            let mut dynamic = Vec::new();
            for a in ctx.query(0)? {
                let k = a.read::<u32>(kind)?;
                for (e, k) in a.entities().iter().zip(k.iter()) {
                    if *k == BODY_DYNAMIC {
                        dynamic.push(*e);
                    }
                }
            }
            let (victim, body) = {
                let mut rng = ctx.resource_mut::<Rng>()?;
                let i = rng.integer(0, dynamic.len() as i64 - 1) as usize;
                (dynamic[i], random_body(&mut rng))
            };
            ctx.despawn(victim)?;
            ctx.spawn(body)?;
            ctx.resource_mut::<PileStats>()?.churned += 1;
            Ok(())
        },
    )
    .unwrap();

    w.add_system(
        SystemDesc::new("count")
            .write::<PileStats>()
            .read::<CollisionStarted>()
            .read::<CollisionEnded>()
            .read::<SensorEntered>()
            .read::<SensorExited>(),
        |ctx| {
            let started = ctx.events::<CollisionStarted>()?.len() as u64;
            let ended = ctx.events::<CollisionEnded>()?.len() as u64;
            let entered = ctx.events::<SensorEntered>()?.len() as u64;
            let exited = ctx.events::<SensorExited>()?.len() as u64;
            let mut s = ctx.resource_mut::<PileStats>()?;
            s.started += started;
            s.ended += ended;
            s.entered += entered;
            s.exited += exited;
            Ok(())
        },
    )
    .unwrap();

    let physics = install(&mut w, Physics::default()).unwrap();
    physics.set_mesh(CONVEX_MESH, octahedron(0.45)).unwrap();
    (w, physics)
}

/// `n` of the pile's bodies dropped into a walled pen that grows with them
/// (the pile's 8 × 8 m for 500), and nothing else: physics alone.
pub fn pen(n: usize, seed: u32) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    let physics = install(&mut w, Physics::default()).unwrap();
    physics.set_mesh(CONVEX_MESH, octahedron(0.45)).unwrap();
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(80.0, 1.0, 80.0)),
    );
    let half = 4.0 * (n as f32 / 500.0).sqrt().max(0.5);
    let wall = half + 1.5;
    for (x, z, sx, sz) in [
        (wall, 0.0, 1.0, 2.0 * wall + 1.0),
        (-wall, 0.0, 1.0, 2.0 * wall + 1.0),
        (0.0, wall, 2.0 * wall + 1.0, 1.0),
        (0.0, -wall, 2.0 * wall + 1.0, 1.0),
    ] {
        spawn_static(
            &mut w,
            at(x, 15.0, z),
            Collider::cuboid(Vec3::new(sx, 30.0, sz)),
        );
    }
    let mut rng = *w.resource::<Rng>().unwrap();
    for _ in 0..n {
        w.spawn(random_body_in(&mut rng, half)).unwrap();
    }
    *w.resource_mut::<Rng>().unwrap() = rng;
    (w, physics)
}

/// An octahedron of radius `r`, a convex hull every run builds the same.
pub fn octahedron(r: f32) -> MeshDesc {
    MeshDesc {
        positions: vec![
            Vec3::new(r, 0.0, 0.0),
            Vec3::new(-r, 0.0, 0.0),
            Vec3::new(0.0, r, 0.0),
            Vec3::new(0.0, -r, 0.0),
            Vec3::new(0.0, 0.0, r),
            Vec3::new(0.0, 0.0, -r),
        ],
        indices: vec![
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ],
    }
}

pub fn run(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        w.step().unwrap();
    }
}
