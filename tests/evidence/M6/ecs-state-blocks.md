# og-ecs state blocks: saved, hashed and restored with the world (d214078)

Three runs, Apple M4 Pro, 2026-10-18:

1. `CARGO_TARGET_DIR=target/agent-qa cargo test -p og-ecs` (og-ecs unmodified in the tree): every og-ecs test passes, including `tests/state_blocks.rs`:

```
test block_names_are_unique ... ok
test a_snapshot_without_a_block_resets_it ... ok
test a_refused_restore_changes_neither_the_world_nor_any_block ... ok
test the_hash_covers_every_block ... ok
test a_world_without_blocks_snapshots_and_hashes_as_before ... ok
test blocks_travel_with_the_snapshot_into_the_same_and_a_fresh_world ... ok
test a_block_this_world_lacks_is_refused_and_dropped_on_migration ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

   They assert: blocks travel in the snapshot into the same and a fresh world with an equal hash; the hash changes when only a block's state changes; a world without blocks snapshots (format 2) and hashes exactly as before; a snapshot lacking a block resets it; a refused restore changes neither the world nor any block; a block this world lacks is refused, and dropped with a warning by a migrating restore; block names are unique.

2. **A world without blocks hashes as in M1.** `scripts/check.sh` step `determinism` (`tools/wasm-determinism/run.sh`, on a `git archive 49ed935` export): the bouncing-boxes world hash is `9205731540494195112` and the math digest `6117185439544271997` on aarch64-apple-darwin debug and release and on wasm32 scalar and simd128 under node v26.3.0, identical to tests/evidence/M1/cross-target-determinism.md and tests/evidence/M6/core-cross-target-determinism.md.

3. **A world with a block covers it in the hash and the restore.** og-physics keeps the persistent rapier world (contacts, warm starts) in the `og.physics` state block: the 500-body pile restored at tick 1800 into a fresh world ends at the uninterrupted run's tick-3600 hash `1cc878a44e5a47f3` in debug and release (physics-pile-determinism.json), which would diverge if the block were not saved and applied (README of og-physics, §Determinism).
