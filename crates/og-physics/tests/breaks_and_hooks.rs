//! G2-3: joints that break (og.Joint.breakForce, og.JointBroken) and
//! contact hooks (og.ContactHook) replay from a restore.
//!
//! Ways they could fail, and the test that catches each:
//! - breaking depends on anything but the world: a restore before the
//!   break breaks it at another tick or force →
//!   `a_break_replays_from_a_restore`
//! - hooks change a world after a restore (the hook flags or table lost)
//!   → `hooks_replay_from_a_restore`

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A world with physics and G2-3's lazy types registered (a game naming them).
fn world(settings: Physics, seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(settings, seed);
    support::enable_lazy(&mut w);
    (w, p)
}

fn broken_events(w: &World) -> Vec<JointBroken> {
    w.pending_events::<JointBroken>().unwrap()
}

/// A `mass` kg ball hanging 1 m under a world point on a ball joint.
fn hang(w: &mut World, mass: f32, break_force: f32) -> Entity {
    let bob = spawn_body(
        w,
        at(0.0, 4.0, 0.0),
        RigidBody {
            mass,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.2),
    );
    w.insert(
        bob,
        Joint {
            kind: JOINT_BALL,
            anchor: Vec3::new(0.0, 1.0, 0.0),
            other_anchor: Vec3::new(0.0, 5.0, 0.0),
            break_force,
            ..Joint::default()
        },
    )
    .unwrap();
    bob
}

#[test]
fn a_break_replays_from_a_restore() {
    // 10 kg on a 200 N joint under gravity growing by 2 m/s² a tick from
    // tick 40: a load held through each step, which breaks it.
    let run = |w: &mut World, bob: Entity, from: u32| {
        let mut events = Vec::new();
        for t in from..120 {
            if t >= 40 {
                w.resource_mut::<Physics>().unwrap().gravity.y = -9.81 - 2.0 * (t - 39) as f32;
            }
            w.step().unwrap();
            for e in broken_events(w) {
                assert_eq!(e.joint, bob.to_bits());
                events.push((t, e.force.to_bits(), e.torque.to_bits()));
            }
        }
        (events, w.world_hash())
    };
    let (mut a, _pa) = world(Physics::default(), 1);
    let bob = hang(&mut a, 10.0, 200.0);
    for _ in 0..30 {
        a.step().unwrap();
    }
    let saved = a.snapshot();
    let first = run(&mut a, bob, 30);
    assert_eq!(first.0.len(), 1, "{:?}", first.0);

    let (mut b, _pb) = world(Physics::default(), 1);
    hang(&mut b, 10.0, 200.0);
    b.restore(&saved).unwrap();
    assert_eq!(run(&mut b, bob, 30), first);
}

#[test]
fn hooks_replay_from_a_restore() {
    let build = || {
        let (mut w, p) = world(Physics::default(), 1);
        let f = floor(&mut w);
        let caster = w.spawn((at(20.0, 10.0, 0.0),)).unwrap();
        w.insert(
            f,
            ContactHook {
                surface_velocity: Vec3::new(0.5, 0.0, 0.5),
                friction: 0.8,
                ignore: og_core::List::from_slice(&[caster.to_bits()]).unwrap(),
                ..ContactHook::default()
            },
        )
        .unwrap();
        for k in 0..4 {
            spawn_body(
                &mut w,
                at(k as f32 * 0.6, 0.5 + k as f32, 0.0),
                RigidBody::dynamic(),
                Collider::sphere(0.25),
            );
        }
        (w, p, f, caster)
    };
    let (mut a, pa, floor, caster) = build();
    let ignored = PointOverlapDesc {
        point: Vec3::new(0.0, -0.5, 0.0),
        filter: QueryFilter {
            exclude: Some(caster),
            ..Default::default()
        },
    };
    assert!(
        pa.query(&a)
            .unwrap()
            .overlap_point(&ignored)
            .unwrap()
            .is_empty()
    );
    run(&mut a, 30);
    let saved = a.snapshot();
    let hashes = |w: &mut World| {
        (0..90)
            .map(|_| {
                w.step().unwrap();
                w.world_hash()
            })
            .collect::<Vec<_>>()
    };
    let first = hashes(&mut a);
    let (mut b, pb, _, _) = build();
    b.restore(&saved).unwrap();
    let query = pb.query(&b).unwrap();
    assert!(
        query.overlap_point(&ignored).unwrap().is_empty(),
        "the saved hook filters queries immediately after restore"
    );
    assert!(
        query
            .overlap_point(&PointOverlapDesc {
                filter: QueryFilter::default(),
                ..ignored
            })
            .unwrap()
            .contains(&floor)
    );
    assert_eq!(
        b.snapshot(),
        saved,
        "queries leave restored hook state untouched"
    );
    assert_eq!(hashes(&mut b), first);
}
