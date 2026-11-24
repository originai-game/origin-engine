//! G2-3's PBD soft volumes (og.SoftBody) are world state.
//!
//! Ways this could fail, each caught below: the points are not world state,
//! so a restore mid-fall lands differently; two runs differ.

use crate::support;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A world with physics and G2-3's lazy types registered (a game naming them).
fn world(settings: Physics, seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(settings, seed);
    support::enable_lazy(&mut w);
    (w, p)
}

fn ball(w: &mut World, y: f32, soft: SoftBody) -> Entity {
    w.spawn((at(0.0, y, 0.0), soft)).unwrap()
}

#[test]
fn soft_volumes_are_world_state_identical_across_runs_and_after_a_restore() {
    let start = || {
        let (mut w, p) = world(Physics::default(), 3);
        floor(&mut w);
        spawn_static(
            &mut w,
            rotated(at(0.0, 1.0, 0.0), Quat::from_rotation_z(0.4)),
            Collider::cuboid(Vec3::new(4.0, 0.2, 4.0)),
        );
        let a = ball(&mut w, 3.0, SoftBody::default());
        let b = w
            .spawn((
                rotated(at(0.4, 4.5, 0.3), Quat::from_rotation_y(0.7)),
                SoftBody {
                    shape: SOFT_BOX,
                    size: Vec3::new(0.8, 0.5, 0.6),
                    detail: 3,
                    ..SoftBody::default()
                },
            ))
            .unwrap();
        (w, p, a, b)
    };
    let points = |p: &PhysicsSystem, a: Entity, b: Entity| {
        (
            p.soft_body_points(a).unwrap(),
            p.soft_body_points(b).unwrap(),
        )
    };
    let (mut w1, p1, a, b) = start();
    run(&mut w1, 50);
    let mid = w1.snapshot();
    run(&mut w1, 100);
    let end = points(&p1, a, b);
    let (mut w2, p2, a2, b2) = start();
    run(&mut w2, 150);
    assert_eq!(points(&p2, a2, b2), end, "two runs differ");
    assert_eq!(w1.world_hash(), w2.world_hash());
    let (mut w3, p3, ..) = start();
    w3.restore(&mid).unwrap();
    run(&mut w3, 100);
    assert_eq!(points(&p3, a, b), end, "a restored run lands differently");
    assert_eq!(w3.world_hash(), w1.world_hash());
    assert_eq!(w3.snapshot(), w1.snapshot());
}
