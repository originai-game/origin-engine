//! G2-3's PBD soft volumes (og.SoftBody), through the world and
//! og.physics.softBodyPoints.
//!
//! Ways this could fail, each caught below: the volume is not held, so a
//! ball dropped on the floor flattens into a sheet; it is held so rigidly
//! that it never squashes on landing; points sink through the floor; the
//! written `volume` and `center` disagree with the points; the points are
//! not world state, so a restore mid-fall lands differently; two runs
//! differ; the 2D ring leaves its plane or loses its area; `pinTop` does
//! not hold the top to the entity; a world that never names og.SoftBody
//! registers it.

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

/// The lowest point's height, the written volume and the height from the
/// lowest point to the highest, each tick.
fn fall(w: &mut World, p: &PhysicsSystem, e: Entity, ticks: u32) -> Vec<(f32, f32, f32)> {
    (0..ticks)
        .map(|_| {
            w.step().unwrap();
            let pts = p.soft_body_points(e).unwrap().points;
            let low = pts.iter().fold(f32::MAX, |m, q| m.min(q.y));
            let high = pts.iter().fold(f32::MIN, |m, q| m.max(q.y));
            (low, w.get::<SoftBody>(e).unwrap().volume, high - low)
        })
        .collect()
}

#[test]
fn a_soft_ball_squashes_on_landing_and_recovers_its_volume_on_the_floor() {
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    let soft = SoftBody {
        radius: 0.5,
        detail: 2,
        stiffness: 0.3,
        ..SoftBody::default()
    };
    let e = ball(&mut w, 3.0, soft);
    let trace = fall(&mut w, &p, e, 240);
    let lowest = trace.iter().fold(f32::MAX, |m, t| m.min(t.0));
    assert!(lowest > -0.02, "a point sank to {lowest} through the floor");
    let squashed = trace.iter().fold(f32::MAX, |m, t| m.min(t.2));
    assert!(
        squashed < 0.85,
        "landing at 7 m/s squashes it: it was {squashed} m tall at least"
    );
    let least = trace.iter().fold(f32::MAX, |m, t| m.min(t.1));
    assert!(least > 0.9, "but keeps most of its volume: {least}");
    let (low, volume, _) = *trace.last().unwrap();
    assert!(
        low < 0.05,
        "it rests on the floor, its lowest point at {low}"
    );
    assert!(
        (volume - 1.0).abs() < 0.05,
        "and holds its volume there: {volume}"
    );
    let s = w.get::<SoftBody>(e).unwrap();
    let pts = p.soft_body_points(e).unwrap();
    assert_eq!(pts.points.len(), 162);
    assert_eq!(pts.indices.len(), 320 * 3);
    let mean = pts.points.iter().fold(Vec3::ZERO, |a, q| a + *q) / 162.0;
    assert!((s.center - mean).length() < 1e-4, "{} {mean}", s.center);
    assert!(
        s.center.y > 0.3 && s.center.y < 0.6,
        "it stands as a ball, its centre at {}",
        s.center.y
    );

    // Without its volume held it collapses like cloth.
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    let e = ball(
        &mut w,
        3.0,
        SoftBody {
            pressure: 0.0,
            ..soft
        },
    );
    let (_, volume, _) = *fall(&mut w, &p, e, 240).last().unwrap();
    assert!(volume < 0.6, "unpressurised it slumps: {volume}");
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

#[test]
fn a_2d_soft_ring_lands_in_its_plane_and_keeps_its_area() {
    let (mut w, p) = world(
        Physics {
            profile: PROFILE_2D,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ..Physics::default()
        },
        1,
    );
    spawn_static(
        &mut w,
        at(0.0, -0.5, 0.0),
        Collider::cuboid(Vec3::new(40.0, 1.0, 1.0)),
    );
    let e = w
        .spawn((
            at(0.0, 2.0, 0.25),
            SoftBody {
                radius: 0.6,
                detail: 1,
                ..SoftBody::default()
            },
        ))
        .unwrap();
    let trace = fall(&mut w, &p, e, 200);
    let pts = p.soft_body_points(e).unwrap();
    assert_eq!(pts.points.len(), 24);
    assert_eq!(pts.indices.len(), 22 * 3, "a fan over the ring");
    assert!(
        pts.points.iter().all(|q| q.z == 0.25),
        "it leaves its plane"
    );
    let lowest = trace.iter().fold(f32::MAX, |m, t| m.min(t.0));
    assert!(lowest > -0.03, "it sank to {lowest}");
    let (low, area, _) = *trace.last().unwrap();
    assert!(low < 0.05, "it rests on the ground: {low}");
    assert!((area - 1.0).abs() < 0.05, "and keeps its area: {area}");
}

#[test]
fn pin_top_hangs_a_soft_volume_from_its_entity() {
    let (mut w, p) = world(Physics::default(), 1);
    let e = w
        .spawn((
            at(0.0, 3.0, 0.0),
            SoftBody {
                radius: 0.4,
                pin_top: 0.05,
                stiffness: 0.2,
                ..SoftBody::default()
            },
        ))
        .unwrap();
    run(&mut w, 120);
    let pts = p.soft_body_points(e).unwrap().points;
    let top = pts.iter().fold(f32::MIN, |m, q| m.max(q.y));
    assert!((top - 3.4).abs() < 1e-4, "the pinned top stays at {top}");
    let s = w.get::<SoftBody>(e).unwrap();
    assert!(
        s.center.y < 3.0 && s.center.y > 2.5,
        "the rest hangs below it: {}",
        s.center.y
    );
    // The entity moves: the pins follow.
    w.insert(e, at(1.0, 3.0, 0.0)).unwrap();
    w.step().unwrap();
    let pts = p.soft_body_points(e).unwrap().points;
    let top = pts
        .iter()
        .copied()
        .fold(Vec3::splat(f32::MIN), |m, q| if q.y > m.y { q } else { m });
    assert!((top.x - 1.0).abs() < 1e-4, "the top moved to {top}");
}

#[test]
fn a_world_that_never_names_og_soft_body_does_not_register_it() {
    let (w, _p) = support::world(Physics::default(), 1);
    assert!(w.id_by_name("og.SoftBody").is_err());
}
