//! Shared by the og-physics integration tests: a world with physics
//! installed, and spawning helpers.
#![allow(dead_code)]

pub mod pile;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World, WorldDesc};
use og_physics::{Collider, Physics, PhysicsSystem, RigidBody, Transform, Velocity, install};

/// A 60 Hz world with `og.Transform` registered as the physics crate's Rust
/// type and physics installed with `settings`.
pub fn world(settings: Physics, seed: u32) -> (World, PhysicsSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    w.register::<Transform>().unwrap();
    let physics = install(&mut w, settings).unwrap();
    (w, physics)
}

pub fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::at(Vec3::new(x, y, z))
}

pub fn rotated(t: Transform, q: Quat) -> Transform {
    Transform { quaternion: q, ..t }
}

pub fn spawn_body(w: &mut World, t: Transform, body: RigidBody, collider: Collider) -> Entity {
    w.spawn((t, body, collider)).unwrap()
}

pub fn spawn_static(w: &mut World, t: Transform, collider: Collider) -> Entity {
    w.spawn((t, collider)).unwrap()
}

/// A 40 × 1 × 40 static floor whose top is y = 0.
pub fn floor(w: &mut World) -> Entity {
    spawn_static(
        w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(40.0, 1.0, 40.0)),
    )
}

pub fn transform(w: &World, e: Entity) -> Transform {
    w.get::<Transform>(e).unwrap()
}

pub fn velocity(w: &World, e: Entity) -> Velocity {
    w.get::<Velocity>(e).unwrap()
}

pub fn run(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        w.step().unwrap();
    }
}
