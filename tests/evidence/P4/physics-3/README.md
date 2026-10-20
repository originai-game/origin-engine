# Physics on the lanes, round three (2026-10-21)

Branch `physics-3` (crates/og-physics README, "The step on the lanes";
crates/og-lanes README; docs/web-export.md §Lanes). What changed since
`tests/evidence/P4/physics-lanes-2/`:

- `og.physics.stats` (`PhysicsSystem::stats`): each phase of the last
  step, its wall-clock ms and whether it ran on the lanes pool. Every
  number below is read from it (`lanes_costs --phases`, the scene's own
  phase in `soft_lanes_costs`).
- Around rapier: the gather decodes rows by ranges (`Grains::rows`); the
  sync plans each body and the readout reads each body back by ranges of
  bodies (`Grains::bodies`), every rapier call still made in body order;
  the contact graph is read by ranges of edges and the touching pairs
  sorted with `og_lanes::sort` (`Grains::pairs`) when the step is on the
  pool; the bodies that left are found by one walk of the sorted keys and
  the tracked values kept by one walk instead of a map insert per body.
- Ragdolls are planned alone (`Grains::ragdolls`) and synced to rapier in
  ragdoll order.
- Five and six threads have soft-body grains of their own.

## Commands

From the repository root, release builds, each configuration run once per
round after the 1-minute load fell below 9 (`quiet.sh`), two rounds:

```sh
cargo build --release -p og-physics --example lanes_costs --example soft_lanes_costs
# pairs.txt: the readout's edges and sort, below and on the pool
for m in 4 8; do for p in 1000000000 512 2048; do OG_LANES=$m target/release/examples/lanes_costs --phases --pairs $p 256 512 1000 2000; done; done
# bodies.txt: the sync and readout by ranges of bodies (and, then, the gather)
for m in off 2 4 5 6 8; do for b in 1000000000 512; do OG_LANES=$m target/release/examples/lanes_costs --phases --bodies $b 1000 2000 4000; done; done
# before-after.txt: dca74cbb's lanes_costs (lanes_old) against this branch's (lanes2), and the rows grain off
for m in off 2 4 5 6 8; do OG_LANES=$m lanes_old 500 1000 2000 4000; OG_LANES=$m target/release/examples/lanes_costs --phases 500 1000 2000 4000; done
for m in 4 8; do OG_LANES=$m target/release/examples/lanes_costs --phases --rows 1000000000 500 1000 2000 4000; done
# ragdolls.txt
OG_LANES=off target/release/examples/soft_lanes_costs --only ragdolls
for m in 4 8; do for r in 1000000000 1; do OG_LANES=$m target/release/examples/soft_lanes_costs --only ragdolls --ragdolls $r; done; done
# soft-5-6.txt: five and six threads with eight's grains, four's, and every soft body split from two
for m in 5 6; do OG_LANES=$m target/release/examples/soft_lanes_costs --only ropes,cloths,fluids3,fluids2; OG_LANES=$m target/release/examples/soft_lanes_costs --only ropes,cloths,fluids3,fluids2 --rope-work 800 --cloth-work 1100; OG_LANES=$m target/release/examples/soft_lanes_costs --only ropes,cloths,fluids3,fluids2 --split; done
# fluid3d-density.txt: a 3D fluid with its density pass scattered (soft_old) and gathered on the lanes
for m in off 4 5 6 8; do OG_LANES=$m soft_old --only fluid3; OG_LANES=$m soft2 --only fluid3; done
```

`lanes_costs` times the mean `World::step` over 600 ticks after 240 (the
500-body pile of `tests/pile.rs` and pens of n of its mixed shapes);
`soft_lanes_costs` over 300 after 120. Every scene ends in one world hash
in every mode and with either binary (the last column; before-after.txt:
dca74cbb's code and this branch's give the same hashes).

## Machine and load

Apple M4 Pro, 12 cores (8 performance, 4 efficiency), macOS 27.2, shared
with other agents' builds: the runs waited for a 1-minute load under 9
and ran at 7 to 9 (each run's load is in its text file; a first attempt
at load 24 to 29 was thrown away). Tables give the fastest of the two
rounds; single runs vary by 10 to 20 % at this load.

## Where the time goes around rapier (before-after.txt)

ms a tick, `World::step`, before this round (dca74cbb) and after:

| scene | off: before / after | 2: before / after | 4: before / after | 5: before / after | 6: before / after | 8: before / after |
|---|---|---|---|---|---|---|
| the pile (500) | 1.55 / 1.51 | 1.43 / 1.37 | 1.27 / 1.36 | 1.58 / 1.57 | 1.60 / 1.55 | 1.52 / 1.59 |
| a pen of 500 | 0.88 / 0.92 | 0.97 / 0.93 | 0.97 / 0.99 | 0.97 / 0.96 | 0.92 / 0.91 | 0.96 / 0.91 |
| a pen of 1000 | 2.08 / 1.95 | 1.94 / 1.80 | 1.88 / 2.09 | 2.41 / 2.15 | 2.73 / 2.20 | 2.10 / 2.15 |
| a pen of 2000 | 5.55 / 4.95 | 4.53 / 4.02 | 3.67 / 4.00 | 4.70 / 3.80 | 4.84 / 3.93 | 4.86 / 5.12 |
| a pen of 4000 | 12.62 / 12.30 | 9.50 / 9.02 | 8.41 / 8.14 | 8.98 / 8.29 | 9.38 / 7.21 | 9.89 / 8.91 |

Each phase after, fastest of two rounds (`*`: split on the pool):

| pen | lanes | gather | sync | rapier | readout | writeBack | around rapier |
|---|---|---|---|---|---|---|---|
| 1000 | off | 0.130 | 0.035 | 1.423 | 0.223 | 0.111 | 0.499 |
| 1000 | 2 | 0.136 | 0.052 | 1.215* | 0.222* | 0.131 | 0.541 |
| 1000 | 4 | 0.144 | 0.057 | 1.434* | 0.263* | 0.138 | 0.602 |
| 1000 | 5 | 0.143 | 0.057 | 1.496* | 0.260* | 0.140 | 0.600 |
| 1000 | 6 | 0.143 | 0.057 | 1.556* | 0.253* | 0.140 | 0.593 |
| 1000 | 8 | 0.137 | 0.042 | 1.561 | 0.265 | 0.117 | 0.561 |
| 2000 | off | 0.260 | 0.093 | 3.659 | 0.644 | 0.241 | 1.238 |
| 2000 | 2 | 0.271 | 0.124 | 2.740* | 0.537* | 0.274 | 1.206 |
| 2000 | 4 | 0.272 | 0.121 | 2.728* | 0.530* | 0.273 | 1.196 |
| 2000 | 5 | 0.264 | 0.122 | 2.563* | 0.507* | 0.271 | 1.164 |
| 2000 | 6 | 0.266 | 0.122 | 2.667* | 0.514* | 0.271 | 1.173 |
| 2000 | 8 | 0.275 | 0.132 | 3.717* | 0.633* | 0.276 | 1.316 |
| 4000 | off | 0.518 | 0.241 | 9.453 | 1.517 | 0.479 | 2.755 |
| 4000 | 2 | 0.411* | 0.271 | 6.482* | 1.220* | 0.515 | 2.417 |
| 4000 | 4 | 0.378* | 0.278 | 5.831* | 0.989* | 0.520 | 2.165 |
| 4000 | 5 | 0.415* | 0.284 | 5.938* | 0.991* | 0.532 | 2.222 |
| 4000 | 6 | 0.378* | 0.276 | 5.005* | 0.889* | 0.520 | 2.063 |
| 4000 | 8 | 0.469* | 0.287 | 6.488* | 1.027* | 0.518 | 2.301 |

Before this round (the stats' first run on dca74cbb's step, one round,
load 8): a pen of 2,000 spent gather 0.272, sync 0.227, readout 0.758 and
write-back 0.253 ms lanes off (1.51 ms around rapier), and 0.291, 0.264,
0.804 and 0.285 on four lanes (1.64). Now 1.24 lanes off and 1.20 on four
lanes: the sync from one walk of the sorted keys (0.23 to 0.09 lanes off),
the readout from the sort of keys and indices instead of whole pairs and
from the edges read by ranges on the pool (0.80 to 0.53 on four lanes).
At 4,000 bodies, 2.76 lanes off, 2.07 to 2.30 on the pool. What stays
serial: every rapier call (touching a body pushes it on rapier's modified
list, in the snapshot and the hash, so the calls keep body order), the
deepest-contact fold, the tracked map, and the write-back (component
columns and one encoded event a touching pair, 0.27 ms at 2,000).

## The readout's pairs (pairs.txt)

Below the pool (a pen of 512 on eight lanes, its step serial) splitting
the edges and the sort cost more than it saved: readout 0.099 ms not
split, 0.267 by 512 edges, 0.097 by 2,048 (fewer than two grains). On the
pool, 2,048 edges a task beat 512: a pen of 2,000 on four lanes 0.480
(2,048) against 0.485 (512) and 0.648 not split; on eight lanes 0.558,
0.592 and 0.702. Hence `Grains::pairs` 2,048, used only when the step is
on the pool.

## The body grain (bodies.txt)

Planning the sync and reading the bodies back by 512 bodies a task never
paid, on any thread count to 4,000 bodies: sync 0.352 lanes off against
0.386 split off-pool and 0.338 to 0.371 on the pools at 4,000; readout
0.937 (four lanes, not split) against 1.053 (split). What splits (the
comparisons, a pose per body, the rest rule's reads) is a third of those
phases; the rest is the rapier calls in body order and the walks, and
waking the pool costs what the third saves. `Grains::bodies` stays
`usize::MAX` (the split is there, tested with `Grains::always`). The
gather's row decode pays from about 4,000 rows: 0.518 lanes off against
0.378 to 0.469 by 1,024 rows on two to eight lanes (before-after.txt,
and 0.514 with the rows grain off); even at 2,000 (two tasks). Hence
`Grains::rows` 1,024.

## Ragdolls (ragdolls.txt)

ms a tick (the ragdolls phase in brackets): planning each ragdoll on the
lanes does not pay at any count measured. 1,024 ragdolls (3,072 bones,
half limp) spend 0.20 ms in their sync lanes off and 0.21 to 0.27 split;
at 16 to 256, splitting costs 0.04 to 0.13 ms more. Most of the phase is
the rapier calls (a kinematic target or two dampings a bone, the limp
switch), made in ragdoll order. `Grains::ragdolls` stays `usize::MAX`.

| scene | off | 4, not split | 4, split from two | 8, not split | 8, split from two | hashes |
|---|---|---|---|---|---|---|
| 16 ragdolls (3 bones, half limp) | 0.034 (0.003) | 0.032 (0.003) | 0.085 (0.044) | 0.034 (0.003) | 0.121 (0.074) | 1 |
| 64 ragdolls (3 bones, half limp) | 0.116 (0.010) | 0.116 (0.010) | 0.184 (0.062) | 0.125 (0.010) | 0.243 (0.116) | 1 |
| 256 ragdolls (3 bones, half limp) | 0.358 (0.042) | 0.460 (0.056) | 0.536 (0.088) | 0.344 (0.040) | 0.512 (0.165) | 1 |
| 1024 ragdolls (3 bones, half limp) | 1.942 (0.203) | 1.574 (0.225) | 1.530 (0.205) | 1.843 (0.228) | 1.847 (0.269) | 1 |

## Five and six threads (soft-5-6.txt)

Five and six threads used eight's soft grains; here with eight's (ropes
2,400 point passes a task, cloths 2,000), four's (800 and 1,100) and every
soft body split from two (ms a tick; the soft or fluid phase in brackets):

| scene | off | 5, eight's | 5, four's | 5, from two | 6, eight's | 6, four's | 6, from two | hashes |
|---|---|---|---|---|---|---|---|---|
| 8 ropes (17 points, 12 iterations) | 0.058 (0.053) | 0.054 (0.050) | 0.044 (0.037) | 0.060 (0.051) | 0.053 (0.050) | 0.063 (0.053) | 0.065 (0.054) | 1 |
| 16 ropes (17 points, 12 iterations) | 0.110 (0.105) | 0.104 (0.100) | 0.075 (0.065) | 0.088 (0.075) | 0.105 (0.100) | 0.079 (0.068) | 0.079 (0.068) | 1 |
| 32 ropes (17 points, 12 iterations) | 0.218 (0.211) | 0.106 (0.094) | 0.093 (0.082) | 0.108 (0.096) | 0.109 (0.096) | 0.104 (0.091) | 0.113 (0.100) | 1 |
| 64 ropes (17 points, 12 iterations) | 0.442 (0.430) | 0.166 (0.151) | 0.161 (0.146) | 0.175 (0.159) | 0.187 (0.169) | 0.165 (0.149) | 0.164 (0.147) | 1 |
| 128 ropes (17 points, 12 iterations) | 0.959 (0.940) | 0.285 (0.263) | 0.290 (0.267) | 0.306 (0.283) | 0.302 (0.279) | 0.301 (0.277) | 0.269 (0.247) | 1 |
| 2 cloths (12 × 12, 8 iterations) | 0.065 (0.062) | 0.064 (0.060) | 0.069 (0.062) | 0.068 (0.061) | 0.062 (0.059) | 0.078 (0.068) | 0.074 (0.066) | 1 |
| 4 cloths (12 × 12, 8 iterations) | 0.125 (0.121) | 0.100 (0.090) | 0.086 (0.079) | 0.087 (0.079) | 0.092 (0.083) | 0.085 (0.077) | 0.087 (0.079) | 1 |
| 8 cloths (12 × 12, 8 iterations) | 0.245 (0.239) | 0.113 (0.106) | 0.111 (0.104) | 0.127 (0.118) | 0.127 (0.119) | 0.123 (0.115) | 0.120 (0.112) | 1 |
| 16 cloths (12 × 12, 8 iterations) | 0.505 (0.498) | 0.190 (0.182) | 0.192 (0.183) | 0.203 (0.194) | 0.188 (0.180) | 0.195 (0.185) | 0.187 (0.178) | 1 |
| 32 cloths (12 × 12, 8 iterations) | 1.284 (1.274) | 0.375 (0.364) | 0.376 (0.365) | 0.377 (0.365) | 0.342 (0.330) | 0.352 (0.341) | 0.389 (0.374) | 1 |
| 2 3D fluids of 250 (2 substeps) | 0.268 (0.264) | 0.221 (0.215) | 0.213 (0.206) | 0.225 (0.217) | 0.232 (0.224) | 0.222 (0.215) | 0.228 (0.220) | 1 |
| 2 3D fluids of 500 (2 substeps) | 0.623 (0.618) | 0.430 (0.422) | 0.421 (0.413) | 0.429 (0.421) | 0.430 (0.423) | 0.415 (0.407) | 0.457 (0.448) | 1 |
| 4 3D fluids of 500 (2 substeps) | 1.270 (1.262) | 0.551 (0.541) | 0.500 (0.491) | 0.496 (0.488) | 0.505 (0.496) | 0.476 (0.468) | 0.497 (0.489) | 1 |
| 2 3D fluids of 1000 (2 substeps) | 1.576 (1.567) | 0.906 (0.896) | 0.897 (0.887) | 0.951 (0.939) | 0.920 (0.910) | 0.907 (0.898) | 0.946 (0.937) | 1 |
| 4 3D fluids of 1000 (2 substeps) | 3.207 (3.195) | 1.058 (1.047) | 1.022 (1.010) | 1.053 (1.041) | 1.042 (1.030) | 1.024 (1.011) | 0.994 (0.983) | 1 |
| 8 3D fluids of 1000 (2 substeps) | 6.828 (6.809) | 1.846 (1.828) | 1.900 (1.881) | 1.906 (1.888) | 1.902 (1.883) | 1.880 (1.862) | 1.802 (1.785) | 1 |
| 2 2D fluids of 250 (2 substeps) | 0.476 (0.472) | 0.348 (0.341) | 0.363 (0.355) | 0.351 (0.343) | 0.358 (0.350) | 0.367 (0.359) | 0.357 (0.349) | 1 |
| 2 2D fluids of 500 (2 substeps) | 1.234 (1.227) | 0.754 (0.744) | 0.754 (0.743) | 0.748 (0.738) | 0.784 (0.774) | 0.797 (0.788) | 0.774 (0.763) | 1 |
| 4 2D fluids of 500 (2 substeps) | 2.580 (2.571) | 0.869 (0.858) | 0.870 (0.859) | 0.879 (0.866) | 0.870 (0.859) | 0.860 (0.849) | 0.907 (0.895) | 1 |
| 4 2D fluids of 1000 (2 substeps) | 6.230 (6.201) | 1.693 (1.678) | 1.731 (1.716) | 1.653 (1.637) | 1.516 (1.502) | 1.592 (1.576) | 1.701 (1.684) | 1 |

Five threads split ropes from eight as four do (8 ropes 0.044 against
0.054 ms not split), six from 16 (8 ropes: 0.063 split against 0.053;
16: 0.079 against 0.105): 800 and 1,600 point passes. Cloths split from
four on both (two: 0.069 and 0.078 split against 0.064 and 0.062): 2,000,
eight's. Fluid sets pay from two of 250 particles on both (500, as on
every count). One fluid's particle passes: 256 particles a task in 2D
(1,000: 0.96 against 1.47 by 512 on five lanes), 512 in 3D (below).

## A 3D fluid's densities (fluid3d-density.txt)

The density pass scatters each pair's q² and q³ to both particles. Each
particle's sums can be gathered instead in the scatter's order (its pairs
with the particles before it, then its own), bit for bit, which splits by
particles; it needs the pairs' transpose each substep. Measured first by
512 particles a task with the elementwise passes split too (the file's first
half: 8,000 particles on four lanes 10.29 ms a tick gathered against
8.45 scattered), then one range a pool thread (below): it does not pay
once the transpose is counted, so the scatter stays.

| scene | off | 4, scatter | 4, gathered | 5, scatter | 5, gathered | 6, scatter | 6, gathered | 8, scatter | 8, gathered | hashes |
|---|---|---|---|---|---|---|---|---|---|---|
| a 3D fluid of 1000 (2 substeps) | 0.720 (0.714) | 0.747 (0.740) | 0.758 (0.750) | 0.777 (0.770) | 0.743 (0.736) | 0.762 (0.754) | 0.733 (0.727) | 0.742 (0.736) | 0.765 (0.759) | 1 |
| a 3D fluid of 2000 (2 substeps) | 1.908 (1.896) | 1.558 (1.545) | 1.835 (1.819) | 1.741 (1.720) | 1.957 (1.936) | 1.719 (1.703) | 1.854 (1.840) | 1.679 (1.662) | 2.031 (2.011) | 1 |
| a 3D fluid of 4000 (2 substeps) | 5.426 (5.400) | 3.978 (3.945) | 4.078 (4.050) | 3.772 (3.745) | 4.264 (4.228) | 3.798 (3.772) | 4.168 (4.142) | 3.714 (3.688) | 4.157 (4.129) | 1 |
| a 3D fluid of 8000 (2 substeps) | 12.288 (12.251) | 8.372 (8.327) | 8.453 (8.414) | 8.270 (8.227) | 8.509 (8.463) | 7.831 (7.798) | 8.283 (8.248) | 7.894 (7.857) | 8.236 (8.198) | 1 |

Relaxation, viscosity and mixing stay serial: Gauss-Seidel over the pairs
in index order, each pair moving (or pushing, or mixing) both particles
at once and the next reading the result. Particle i's pairs read what
particle i - 1's wrote whenever the two are neighbours, which lattice,
emission and pouring order make nearly always, so the chain that must
keep this order is as long as the fluid. A two-colour or Jacobi order
splits it but moves every particle's bits and the physics hash (the
beyond scenario pours two 3D fluids); this round keeps the hash.

## Determinism

The determinism tool's physics scenarios (`tools/wasm-determinism`
`physics_hash`: the beyond scenario 3D and 2D for 600 ticks, a 1,000-body
pen for 300), run natively from a temporary og-physics example at
b748cd40: 18126409214070009362 with lanes off, on four and on eight
threads (1,660 and 600 parallel calls), and with every pass split
(`Grains::always`) on four and on eight (23,066 parallel calls each):
the hash of `tests/evidence/P4/physics-lanes/determinism.md`, unchanged.
og-physics's `tests/lanes.rs` checks the same per test run, now with 24
ragdolls a profile and the stats.

## Not measured here

The lanes wasm (`bun bench/lanes.ts --chrome --pile`) and
`og.physics.stats` through the hosts (`packages/engine/tests/e5.test.ts`
reads it natively) are the coordinator's runs.
