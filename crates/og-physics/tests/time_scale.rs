//! Slow motion and hit stop through the world's og.TimeScale.
//!
//! Ways this could fail: physics ignores the scale (a body at scale 0.5
//! falls as fast as at 1); a hit stop still moves bodies, or makes the
//! resting pair look separated (CollisionEnded during the freeze, then a
//! second CollisionStarted after it); the hit stop never ends, or ends a
//! tick early or late; a snapshot taken during a hit stop restores a world
//! that resumes differently.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use support::*;

fn drop_box(scale: f32) -> (og_ecs::World, og_ecs::Entity) {
    let (mut w, _p) = world(Physics::default(), 1);
    floor(&mut w);
    let b = spawn_body(
        &mut w,
        at(0.0, 5.0, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    w.set_time_scale(scale).unwrap();
    (w, b)
}

#[test]
fn slow_motion_scales_the_step_and_zero_stops_it() {
    let (mut normal, a) = drop_box(1.0);
    let (mut slow, b) = drop_box(0.5);
    let (mut stopped, c) = drop_box(0.0);
    for _ in 0..20 {
        normal.step().unwrap();
        slow.step().unwrap();
        stopped.step().unwrap();
    }
    let fall = |w: &og_ecs::World, e| 5.0 - transform(w, e).position.y;
    let (fn_, fs) = (fall(&normal, a), fall(&slow, b));
    // Free fall goes with t^2: half the time, about a quarter of the drop.
    assert!(fs > 0.15 * fn_ && fs < 0.4 * fn_, "normal {fn_}, slow {fs}");
    assert_eq!(fall(&stopped, c), 0.0, "scale 0 holds the body");
    // Scaled time is counted; the tick is not.
    assert_eq!(slow.time().tick, 20);
    assert!((slow.time_scale().elapsed - 20.0 / 60.0 * 0.5).abs() < 1e-6);
    assert!(normal.set_time_scale(-1.0).is_err());
    assert!(normal.set_time_scale(f32::NAN).is_err());
}

#[test]
fn a_hit_stop_freezes_bodies_keeps_contacts_and_resumes_exactly() {
    let (mut w, b) = drop_box(1.0);
    // Let it land and settle onto the floor.
    for _ in 0..150 {
        w.step().unwrap();
    }
    assert!(w.pending_events::<CollisionStarted>().unwrap().is_empty());
    let resting = transform(&w, b);
    // Kick it upwards, then stop time for 5 ticks right away.
    w.insert(
        b,
        Velocity {
            linear: Vec3::new(1.0, 4.0, 0.0),
            angular: Vec3::ZERO,
        },
    )
    .unwrap();
    w.hit_stop(5).unwrap();
    w.hit_stop(2).unwrap(); // never shortens the one in progress
    let mut copy = None;
    for tick in 0..5 {
        w.step().unwrap();
        assert_eq!(transform(&w, b), resting, "frozen at freeze tick {tick}");
        assert!(w.pending_events::<CollisionEnded>().unwrap().is_empty());
        if tick == 2 {
            copy = Some(w.snapshot());
        }
    }
    assert_eq!(w.time_scale().hit_stop, 0);
    // After the freeze the kick plays, and the pair that touched all along
    // ends once (it never looked separated, so no second start comes first).
    let mut first = None;
    for _ in 0..31 {
        w.step().unwrap();
        if first.is_none() {
            if !w.pending_events::<CollisionEnded>().unwrap().is_empty() {
                first = Some("ended");
            } else if !w.pending_events::<CollisionStarted>().unwrap().is_empty() {
                first = Some("started");
            }
        }
    }
    assert_eq!(first, Some("ended"));

    // The snapshot taken during the freeze resumes into the same world.
    let (mut r, _) = drop_box(1.0);
    r.restore(&copy.unwrap()).unwrap();
    assert_eq!(r.time_scale().hit_stop, 2);
    for _ in 0..33 {
        r.step().unwrap();
    }
    assert_eq!(r.world_hash(), w.world_hash());
}
