//! A Rust-only "bouncing boxes" world: boxes fall, bounce off the walls of
//! an arena with random jitter, emit events, retire after enough bounces and
//! are replaced by newly spawned ones. Advanced 3600 ticks (one minute at
//! 60 Hz) twice from the same seed, it ends with the same world hash; and a
//! run snapshotted halfway and restored into a fresh world ends there too.

#[path = "support/bouncing_boxes.rs"]
mod world;

use world::{build, run, Stats};

#[test]
fn the_same_seed_gives_the_same_world_after_3600_ticks() {
    let mut a = build(42);
    let mut b = build(42);
    assert_eq!(a.world_hash(), b.world_hash());
    run(&mut a, 3600);
    run(&mut b, 3600);
    assert_eq!(a.time().tick, 3600);
    assert_eq!(a.world_hash(), b.world_hash());
    assert_eq!(a.snapshot(), b.snapshot());

    // The minute did something: boxes bounced, retired and were replaced.
    let stats = *a.resource::<Stats>().unwrap();
    assert!(stats.bounces > 1_000, "{stats:?}");
    assert!(stats.spawned > 100 && stats.retired > 100, "{stats:?}");
    assert_eq!(a.entity_count() as u64, 200 + stats.spawned - stats.retired);
    assert!(stats.energy > 0.0);

    // A different seed is a different world.
    let mut c = build(43);
    run(&mut c, 3600);
    assert_ne!(c.world_hash(), a.world_hash());
}

#[test]
fn a_run_snapshotted_halfway_and_restored_ends_in_the_same_world() {
    let mut whole = build(7);
    run(&mut whole, 3600);

    let mut first = build(7);
    run(&mut first, 1800);
    let half = first.snapshot();
    let mut second = build(99); // a different seed: the snapshot carries the rng
    second.restore(&half).unwrap();
    assert_eq!(second.world_hash(), first.world_hash());
    run(&mut second, 1800);
    run(&mut first, 1800);
    assert_eq!(second.time().tick, 3600);
    assert_eq!(second.world_hash(), whole.world_hash());
    assert_eq!(first.world_hash(), whole.world_hash());
}
