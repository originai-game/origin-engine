//! What is inside a sensor (`sensor_contents` over a step's og.Contact).
//!
//! Ways this could fail: only the side where the sensor has the lower id is
//! read (a body spawned before the sensor is never inside it); a solid the
//! sensor's entity touches counts as inside; two overlapping sensors miss
//! each other; an entity is listed twice; the order follows spawn or
//! contact order rather than the entity; the contents drift from what
//! SensorEntered and SensorExited add up to; a restore or a hit stop
//! loses them.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use std::collections::BTreeSet;
use support::*;

/// A 2 m sensor cube at the origin; boxes of 0.5 m overlap it while
/// |x| < 1.25 (on the y = 0 line, well inside in y and z).
const REACH: f32 = 1.25;

fn inside(w: &og_ecs::World, sensor: og_ecs::Entity) -> Vec<u64> {
    sensor_contents(&w.pending_events::<Contact>().unwrap(), sensor.to_bits())
}

fn kinematic_box(w: &mut og_ecs::World, x: f32) -> og_ecs::Entity {
    spawn_body(
        w,
        at(x, 0.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::splat(0.5)),
    )
}

/// Each box's x on tick `t`: three walks across the sensor, started at
/// different ticks and from different sides.
fn path(i: usize, t: u32) -> f32 {
    let t = t as f32;
    match i {
        0 => -3.0 + 0.3 * t,
        1 => 3.0 - 0.2 * (t - 5.0).max(0.0),
        _ => -3.0 + 0.4 * (t - 12.0).max(0.0),
    }
}

#[test]
fn contents_follow_the_boxes_in_entity_order_and_match_the_events() {
    let (mut w, _p) = world(Physics::default(), 1);
    // One box spawned before the sensor (a lower id: the pair's `b` is the
    // sensor), two after (the sensor is `a`).
    let first = kinematic_box(&mut w, path(0, 0));
    let sensor = spawn_static(
        &mut w,
        at(0.0, 0.0, 0.0),
        Collider::cuboid(Vec3::splat(2.0)).as_sensor(),
    );
    let boxes = [
        first,
        kinematic_box(&mut w, path(1, 0)),
        kinematic_box(&mut w, path(2, 0)),
    ];
    // A solid ledge over the sensor with a box resting on it: touching, never inside.
    let wall = spawn_static(
        &mut w,
        at(0.0, 3.0, 0.0),
        Collider::cuboid(Vec3::splat(1.0)),
    );
    let rider = spawn_body(
        &mut w,
        at(0.0, 3.76, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::splat(0.5)),
    );
    let mut from_events: BTreeSet<u64> = BTreeSet::new();
    let mut ever = BTreeSet::new();
    for t in 0..30u32 {
        for (i, b) in boxes.iter().enumerate() {
            w.insert(*b, at(path(i, t), 0.0, 0.0)).unwrap();
        }
        w.step().unwrap();
        let mut expected: Vec<u64> = boxes
            .iter()
            .enumerate()
            .filter(|(i, _)| path(*i, t).abs() < REACH - 0.01)
            .map(|(_, b)| b.to_bits())
            .collect();
        expected.sort();
        let now = inside(&w, sensor);
        // Away from the edges the expectation is exact.
        let near_edge = (0..3).any(|i| (path(i, t).abs() - REACH).abs() < 0.01);
        if !near_edge {
            assert_eq!(now, expected, "tick {t}");
        }
        for e in w.pending_events::<SensorEntered>().unwrap() {
            assert_eq!(e.sensor, sensor.to_bits());
            assert!(from_events.insert(e.other), "entered twice, tick {t}");
        }
        for e in w.pending_events::<SensorExited>().unwrap() {
            assert!(
                from_events.remove(&e.other),
                "left without entering, tick {t}"
            );
        }
        assert_eq!(
            now,
            from_events.iter().copied().collect::<Vec<_>>(),
            "tick {t}"
        );
        assert!(!now.contains(&wall.to_bits()) && !now.contains(&rider.to_bits()));
        ever.extend(now);
    }
    assert_eq!(ever.len(), 3, "every box was inside at some point");
    // The wall and the rider do touch: contents read only the sensor's side.
    let solid: Vec<Contact> = w.pending_events::<Contact>().unwrap();
    assert!(
        solid.iter().any(|c| c.sensor == 0),
        "the rider rests against the wall"
    );
    assert!(inside(&w, wall).is_empty(), "a solid holds nothing");
}

#[test]
fn two_overlapping_sensors_hold_each_other_once() {
    let (mut w, _p) = world(Physics::default(), 1);
    let a = spawn_static(
        &mut w,
        at(0.0, 0.0, 0.0),
        Collider::cuboid(Vec3::splat(2.0)).as_sensor(),
    );
    let b = spawn_body(
        &mut w,
        at(0.5, 0.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::splat(2.0)).as_sensor(),
    );
    let c = kinematic_box(&mut w, 0.4);
    w.step().unwrap();
    assert_eq!(inside(&w, a), vec![b.to_bits(), c.to_bits()]);
    assert_eq!(inside(&w, b), vec![a.to_bits(), c.to_bits()]);
    // The pairs as delivered twice (a hit stop re-sends them) list each once.
    let mut twice = w.pending_events::<Contact>().unwrap();
    twice.extend(twice.clone());
    assert_eq!(
        sensor_contents(&twice, a.to_bits()),
        vec![b.to_bits(), c.to_bits()]
    );
}

#[test]
fn contents_survive_a_restore_a_hit_stop_and_a_despawn() {
    let (mut w, _p) = world(Physics::default(), 1);
    let sensor = spawn_static(
        &mut w,
        at(0.0, 0.0, 0.0),
        Collider::cuboid(Vec3::splat(2.0)).as_sensor(),
    );
    let a = kinematic_box(&mut w, 0.0);
    let b = kinematic_box(&mut w, 0.5);
    w.step().unwrap();
    let both = vec![a.to_bits(), b.to_bits()];
    assert_eq!(inside(&w, sensor), both);
    let saved = w.snapshot();

    // A hit stop holds the step: moving a box out changes nothing until it ends.
    w.hit_stop(3).unwrap();
    w.insert(b, at(5.0, 0.0, 0.0)).unwrap();
    for _ in 0..3 {
        w.step().unwrap();
        assert_eq!(inside(&w, sensor), both, "held by the hit stop");
    }
    w.step().unwrap();
    assert_eq!(inside(&w, sensor), vec![a.to_bits()]);
    w.despawn(a).unwrap();
    w.step().unwrap();
    assert!(inside(&w, sensor).is_empty(), "a despawned body leaves");

    w.restore(&saved).unwrap();
    assert_eq!(inside(&w, sensor), both, "restored with the world");
    w.step().unwrap();
    assert_eq!(inside(&w, sensor), both, "and stepped on from it");
}
