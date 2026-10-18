//! Determinism of the 500-body pile (`support/pile.rs`), in the shape of
//! og-ecs's bouncing boxes: two runs from one seed end in the same world
//! after 3600 ticks, and a run snapshotted at tick 1800 and restored into a
//! fresh world ends there too.
//!
//! Ways this could fail, each of which this test would catch: rapier state
//! surviving the tick outside the world (restore diverges); a hash-map or
//! pointer order leaking into body insertion (runs diverge); contacts carried
//! outside the snapshot (Started/Ended differ after restore); the step doing
//! nothing (the pile does not move, events are not counted).

mod support;

use og_physics::*;
use std::time::Instant;
use support::pile::{self, BODIES, PileStats};

#[test]
fn the_pile_is_deterministic_and_a_restore_at_tick_1800_ends_in_the_same_world() {
    let started = Instant::now();
    // The second run goes on its own thread (a world is Send); it is the
    // same simulation either way.
    let second = std::thread::spawn(|| {
        let (mut b, _pb) = pile::build(42);
        let start = b.world_hash();
        pile::run(&mut b, 3600);
        (start, b)
    });
    let (mut a, _pa) = pile::build(42);
    let start_hash = a.world_hash();

    pile::run(&mut a, 1800);
    let half = a.snapshot();
    let half_hash = a.world_hash();
    pile::run(&mut a, 1800);
    let a_secs = started.elapsed().as_secs_f64();

    let (b_start, b) = second.join().unwrap();
    assert_eq!(b_start, start_hash);
    assert_eq!(a.time().tick, 3600);
    assert_eq!(
        a.world_hash(),
        b.world_hash(),
        "two runs from seed 42 diverged"
    );
    assert_eq!(a.snapshot(), b.snapshot());

    // A fresh world (another seed: the snapshot carries the rng) restored at
    // tick 1800 ends in the same world as the uninterrupted run.
    let (mut c, _pc) = pile::build(99);
    c.restore(&half).unwrap();
    assert_eq!(c.world_hash(), half_hash);
    pile::run(&mut c, 1800);
    assert_eq!(c.time().tick, 3600);
    assert_eq!(c.world_hash(), a.world_hash(), "the restored run diverged");

    // The minute did something: bodies collided, crossed the sensor and churned.
    let stats = *a.resource::<PileStats>().unwrap();
    assert!(stats.started > 1_000 && stats.ended > 500, "{stats:?}");
    assert!(stats.entered > 0 && stats.exited > 0, "{stats:?}");
    assert_eq!(stats.churned, 180, "{stats:?}");
    assert_ne!(a.world_hash(), start_hash);
    let bodies = a
        .query(&og_ecs::QueryDesc::new().read::<RigidBody>())
        .unwrap()
        .entity_count();
    assert_eq!(bodies, BODIES + 1, "500 dynamic bodies and the paddle");

    // A different seed is a different world.
    let (mut d, _pd) = pile::build(43);
    pile::run(&mut d, 1);
    let (mut e, _pe) = pile::build(42);
    pile::run(&mut e, 1);
    assert_ne!(d.world_hash(), e.world_hash());

    let total = started.elapsed().as_secs_f64();
    let report = format!(
        "{{\"bodies\":{BODIES},\"ticks\":3600,\"seed\":42,\"hash_tick_0\":\"{start_hash:016x}\",\"hash_tick_1800\":\"{half_hash:016x}\",\"hash_tick_3600\":\"{:016x}\",\"hash_tick_3600_restored\":\"{:016x}\",\"stats\":{{\"collision_started\":{},\"collision_ended\":{},\"sensor_entered\":{},\"sensor_exited\":{},\"churned\":{}}},\"first_run_seconds\":{a_secs:.3},\"ms_per_tick\":{:.3},\"test_seconds\":{total:.3},\"profile\":\"{}\"}}\n",
        a.world_hash(),
        c.world_hash(),
        stats.started,
        stats.ended,
        stats.entered,
        stats.exited,
        stats.churned,
        a_secs * 1000.0 / 3600.0,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    );
    print!("{report}");
    if let Some(path) = std::env::var_os("OG_PHYSICS_PILE_REPORT") {
        std::fs::write(path, report).unwrap();
    }
}
