# Physics on the lanes (2026-10-21)

The physics step through `og_lanes::install` (crates/og-physics README,
"The step on the lanes"): rapier's `parallel` build on the lanes pool from
`og_physics::BODY_GRAIN` (200) bodies for every lane thread, the contact
readout split by 512 pairs, and the calling thread below that. Branch
`web-physics-lanes`; the timed binaries were built at 88aadf41 (the grain
at 200), whose Rust is the branch head's.

## Commands

From the repository root, release (thin LTO, one codegen unit):

```sh
cargo build --release -p og-physics --example lanes_costs
for m in off 4 8; do OG_LANES=$m target/release/examples/lanes_costs 16 64 128 256 512 768 1000 1500 2000; done
```

three rounds, interleaved (`native-grain.txt`, with the machine load before
each run). `native-forced.txt` is the same with `BODY_GRAIN = 1` in a local
build (not committed), so every step goes to the pool: what the grain avoids
and where it starts to pay. Each run times the mean `World::step` (physics
and the ECS around it) over 600 ticks after 240: the 500-body pile of
`tests/pile.rs` (a kinematic paddle, a sensor, one body churned every 20
ticks) and pens of n of the pile's mixed shapes (`tests/support/pile.rs`
`pen`, falling into a walled pen sized for them). `OG_LANES` is read once
per process, as the runtime reads it.

## Machine and load

Apple M4 Pro, 12 cores (8 performance, 4 efficiency), macOS 27.2, shared
with other agents' work: 1-minute load 6.5 to 8.8 during these runs (earlier
attempts at load 20 to 34 were discarded).

## Results (ms per tick, the three rounds sorted)

"(serial)" and "(pool)" say where the committed grain put the step;
the physics site's parallel calls in each run are in the text files.

| scene | bodies | off | 4 lanes | 8 lanes | 4 lanes, every step on the pool | 8 lanes, every step on the pool | hash |
|---|---|---|---|---|---|---|---|
| the pile | 500 | 1.444/1.450/1.453 | 1.438/1.450/1.458 (serial) | 1.447/1.461/1.462 (serial) | 1.004/1.026/1.065 | 1.250/1.318/1.326 | 2be8de152ac7b522 |
| a pen | 16 | 0.022/0.022/0.022 | 0.022/0.022/0.023 (serial) | 0.021/0.022/0.022 (serial) | 0.044/0.045/0.046 | 0.076/0.077/0.079 | 6b174a884413198d |
| a pen | 64 | 0.092/0.092/0.093 | 0.090/0.092/0.094 (serial) | 0.092/0.093/0.093 (serial) | 0.134/0.143/0.146 | 0.200/0.205/0.208 | eab3912f89e904a0 |
| a pen | 128 | 0.199/0.200/0.201 | 0.192/0.199/0.200 (serial) | 0.198/0.199/0.200 (serial) | 0.232/0.243/0.249 | 0.329/0.340/0.350 | 480a1565699adffb |
| a pen | 256 | 0.404/0.406/0.408 | 0.387/0.400/0.405 (serial) | 0.401/0.404/0.404 (serial) | 0.418/0.435/0.444 | 0.572/0.591/0.600 | a24ec14d9364821c |
| a pen | 512 | 0.880/0.881/0.895 | 0.884/0.886/0.890 (serial) | 0.877/0.886/0.894 (serial) | 0.756/0.821/0.886 | 1.150/1.177/1.205 | 0cc7778b55d593bf |
| a pen | 768 | 1.381/1.396/1.396 | 1.347/1.387/1.396 (serial) | 1.374/1.395/1.401 (serial) | 1.120/1.181/1.227 | 1.672/1.782/1.847 | 11a894f0ef44bc2f |
| a pen | 1000 | 1.899/1.914/1.916 | 1.402/1.523/1.572 (pool) | 1.913/1.958/1.999 (serial) | 1.425/1.533/1.753 | 2.118/2.122/2.166 | d58922edef589f4e |
| a pen | 1500 | 3.159/3.225/3.232 | 2.328/2.336/2.360 (pool) | 3.197/3.201/3.218 (serial) | 2.239/2.345/2.544 | 2.752/3.196/3.332 | 258291b09de2bf0e |
| a pen | 2000 | 4.633/4.664/4.701 | 3.091/3.194/3.197 (pool) | 3.560/3.982/4.181 (pool) | 3.096/3.181/3.386 | 3.588/3.826/4.180 | d0b5c28399fbdc50 |

Every scene ends in the same world hash in every mode, forced or not (the
last column: one hash per scene).

## Reading

- **Small worlds are flat**: below the grain the step never wakes the pool
  and costs what lanes off costs (16 to 768 bodies on four lanes, to 1,500
  on eight). Every step on the pool, a 16-body pen paid 2x on four lanes and
  3.5x on eight; 64 to 256 bodies paid 55 to 7 % on four lanes and 123 to
  46 % on eight.
- **Four lanes pay from about 512 bodies** (pen 512: 0.82 against 0.88 ms;
  768: 1.18 against 1.39; 1,000: 1.52 against 1.91, -20 %; 2,000: 3.19
  against 4.66, -32 %). **Eight lanes pay only from about 1,500** (1,000:
  2.12 against 1.91, slower; 1,500: even; 2,000: 3.98 against 4.66, -15 %):
  rapier's staged solver waits for its slowest worker at every stage, and
  with eight threads on this machine some land on efficiency cores or on
  cores the shared load holds. Eight lanes are this host's default
  (`OG_LANES` unset: the cores up to 8).
- **The grain** is one number for every thread count, 200 bodies a thread
  (800 on four lanes, 1,600 on eight): eight lanes are never slower than
  serial, at the price of leaving four lanes' 7 to 15 % at 512 to 800
  bodies on the table. The pile (500 bodies, kept agitated by the paddle
  and the churn) gains more than the pens (-30 % on four lanes, -9 % on
  eight, forced) but stays below the grain.
- og-physics's own passes (a temporary timer, not committed): on the
  2,000-body pen the step was about 3.2 ms of rapier and 1.2 ms of
  og-physics around it (gathering, syncing, the readout) with lanes off;
  the contact readout (0.5 ms, two passes over the pairs) is now one pass,
  split by pairs on the pool. The rest stays serial.

## Determinism

`tools/wasm-determinism/run.sh` at dba5ceb3 (`determinism.md`; the grain was
250 then, so the same 1,000-body pen went to four lanes; not rerun after the
grain moved to 200, which changes where steps run, not what they compute):
host debug, host release with lanes off and on four threads (483 parallel calls: the
1,000-body pen's 300 steps and its pair splits), wasm32 scalar and simd128
(rapier without `parallel`) all agree, physics_hash 18126409214070009362.
og-physics's `tests/pile.rs` steps a pen of the pile's bodies (200 a lane
thread) on the pool and with lanes switched off: the same hash and snapshot.

## Not measured here

The lanes wasm (rapier's `parallel` build on wasm-bindgen-rayon's Workers,
through the relay) is not timed or hashed in this round: the browser lanes
(`bench/lanes.ts --chrome`, the web tests) are the coordinator's run.
