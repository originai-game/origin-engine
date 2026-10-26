# Physics on the lanes, round two (2026-10-21)

Branch `physics-lanes-2` (crates/og-physics README, "The step on the
lanes"; crates/og-lanes README). What changed since
`tests/evidence/P4/physics-lanes/`: the step goes to the pool from a number
of bodies per thread count (`og_physics::pool_bodies`, through
`og_lanes::install_from`) instead of 200 a thread; ropes, cloths and fluid
sets step side by side on the lanes and a fluid's neighbour search and
collisions split by ranges of particles (`og_physics::Grains`); queries run
through `og_lanes::serial`.

## Commands

From the repository root, release (thin LTO, one codegen unit):

```sh
cargo build --release -p og-physics --example lanes_costs --example soft_lanes_costs
# where the pool pays (native-forced.txt, three rounds interleaved)
OG_LANES=off target/release/examples/lanes_costs 256 384 512 640 768 1000 1250 1500 2000
for m in 2 3 4 6 8; do OG_LANES=$m target/release/examples/lanes_costs --forced 256 384 512 640 768 1000 1250 1500 2000; done
# the soft bodies' grains (soft-grains.txt, two rounds)
OG_LANES=off target/release/examples/soft_lanes_costs
for m in 4 8; do for a in "" --split "--split --particles 256" "--split --particles 512" "--split --particles 2048"; do OG_LANES=$m target/release/examples/soft_lanes_costs $a; done; done
# the committed grains (native-grain.txt, soft-measured.txt, two rounds)
for m in off 2 4 6 8; do OG_LANES=$m target/release/examples/lanes_costs 256 384 512 640 768 1000 1250 1500 2000; done
for m in off 4 8; do OG_LANES=$m target/release/examples/soft_lanes_costs; done
```

`lanes_costs` times the mean `World::step` over 600 ticks after 240 (the
500-body pile of `tests/pile.rs` and pens of n of its mixed shapes);
`soft_lanes_costs` over 300 after 120 (each scene on a floor with a bar to
drape over; ropes of 16 segments and 12 iterations, cloths of 12 × 12 and
8 iterations, fluids at two substeps, ragdolls of three bones, half limp).
`OG_LANES` is read once per process. Every scene ends in one world hash in
every mode (the last column of each table: the number of distinct hashes).

## Machine and load

Apple M4 Pro, 12 cores (8 performance, 4 efficiency), macOS 27.2, shared
with other agents' builds and gates: 1-minute load 7.8 to 28.8 over these
runs (each run's load is in the text files). The step tables give every
round's ms per tick, sorted (the first is the least disturbed; single runs
went far over it: 8 lanes at 2,000 bodies, 4.3 to 17 ms); the soft-body
tables the fastest round. The readings use the fastest.

## Where the step pays on the pool, by thread count (native-forced.txt)

Every step on the pool (`--forced`) against lanes off.

| scene | bodies | off | 2 | 3 | 4 | 6 | 8 | hashes |
|---|---|---|---|---|---|---|---|---|
| the pile | 500 | 1.463/1.762/2.262 | 1.154/1.195/1.305 | 1.159/1.239/1.547 | 1.063/1.098/1.152 | 1.073/1.148/1.426 | 1.406/1.418/2.210 | 1 |
| a pen | 256 | 0.405/0.427/0.880 | 0.407/0.419/0.456 | 0.435/0.441/0.474 | 0.452/0.499/0.503 | 0.481/0.538/0.580 | 0.664/0.715/0.996 | 1 |
| a pen | 384 | 0.638/0.659/1.139 | 0.594/0.685/0.703 | 0.638/0.646/0.657 | 0.651/0.673/0.681 | 0.719/0.807/0.820 | 1.025/1.187/1.229 | 1 |
| a pen | 512 | 0.883/0.924/1.240 | 0.813/0.919/1.240 | 0.819/0.864/2.004 | 0.830/0.888/0.914 | 0.938/1.000/1.058 | 1.318/1.582/1.670 | 1 |
| a pen | 640 | 1.169/1.172/1.519 | 1.092/1.175/1.198 | 1.021/1.060/2.140 | 0.962/1.156/1.578 | 1.069/1.356/1.571 | 1.682/2.148/5.084 | 1 |
| a pen | 768 | 1.424/1.719/1.810 | 1.385/1.396/1.489 | 1.241/1.245/1.311 | 1.144/1.291/1.302 | 1.363/1.427/2.013 | 1.999/2.009/3.110 | 1 |
| a pen | 1000 | 1.952/2.100/3.350 | 1.946/1.979/5.332 | 1.655/2.244/6.010 | 1.497/1.658/1.930 | 1.764/1.934/2.518 | 2.400/2.419/3.370 | 1 |
| a pen | 1250 | 2.583/3.475/3.685 | 2.398/2.780/3.945 | 2.170/2.198/3.320 | 2.100/2.143/2.231 | 2.236/2.294/2.377 | 2.900/2.956/3.770 | 1 |
| a pen | 1500 | 3.634/3.741/4.215 | 2.911/2.959/3.411 | 2.723/2.804/4.007 | 2.560/2.584/2.791 | 2.730/2.868/4.877 | 3.482/3.894/4.762 | 1 |
| a pen | 2000 | 4.991/5.211/5.530 | 4.087/4.919/6.086 | 3.879/3.996/4.822 | 3.301/3.402/3.813 | 3.684/4.275/5.363 | 4.335/4.941/17.072 | 1 |

The minima: two lanes pay little anywhere (-8 % at 512, -20 % at 1,500);
three and four from about 500 bodies (four: -6 % at 512, -18 % at 640,
-23 % at 1,000, -34 % at 2,000); six from about 700 to 1,000 (-9 % at
1,000, -25 % at 1,500); eight from about 1,500 (-4 % at 1,500, -13 % at
2,000, slower than serial below). Hence `pool_bodies`: 500 on two to four
threads, 1,000 on five or six, 1,500 from seven.

## The committed thresholds (native-grain.txt)

| scene | bodies | off | 2 | 4 | 6 | 8 | hashes |
|---|---|---|---|---|---|---|---|
| the pile | 500 | 1.445/1.446 | 1.252/1.431 | 1.069/1.224 | 1.474/1.850 | 1.595/4.523 | 1 |
| a pen | 256 | 0.404/0.408 | 0.409/0.421 | 0.406/0.407 | 0.402/0.415 | 0.407/1.137 | 1 |
| a pen | 384 | 0.644/0.650 | 0.645/0.652 | 0.646/0.648 | 0.648/0.673 | 0.712/2.844 | 1 |
| a pen | 512 | 0.893/0.898 | 0.852/1.045 | 0.900/1.032 | 0.922/1.869 | 0.919/4.196 | 1 |
| a pen | 640 | 1.118/1.130 | 1.131/1.217 | 1.092/1.525 | 1.183/1.193 | 1.173/5.882 | 1 |
| a pen | 768 | 1.400/1.578 | 1.306/1.436 | 1.231/1.504 | 1.478/2.442 | 1.488/9.344 | 1 |
| a pen | 1000 | 2.009/2.260 | 1.756/1.850 | 1.701/1.741 | 2.395/3.019 | 2.097/10.925 | 1 |
| a pen | 1250 | 2.677/2.703 | 2.289/2.367 | 2.168/2.332 | 2.857/15.443 | 3.940/11.187 | 1 |
| a pen | 1500 | 3.294/3.356 | 2.840/3.294 | 2.656/3.851 | 3.050/5.341 | 4.015/18.366 | 1 |
| a pen | 2000 | 5.145/5.212 | 4.071/4.227 | 3.706/7.021 | 4.479/21.160 | 5.595/14.381 | 1 |

Four lanes take the pile (500) and every pen from 512 to the pool: the
pile 1.45 to 1.07 ms, 1,000 bodies 2.01 to 1.70, 2,000 5.15 to 3.71. Eight
lanes stay serial below 1,500, so their columns there are lanes off's work
under another run's load; at 1,500 and 2,000 they were within the noise of
serial on this load (the previous round measured -15 % at 2,000 at load 7).

## The soft bodies' grains (soft-grains.txt)

Columns: `off`; `4`, `8`: lanes with the grains then in the code (ropes
4,096 point passes a task, cloths 8,192, fluids 2,048, particles 1,024);
`S`: ropes, cloths and fluid sets split from two of them; `S pN`: also
the per-particle passes by N particles a task. Fastest of two rounds.

| scene | off | 4 | 4 S | 4 S p256 | 4 S p512 | 4 S p2048 | 8 | 8 S | 8 S p256 | 8 S p512 | 8 S p2048 | hashes |
| 8 ropes (17 points, 12 iterations) | 0.051 | 0.052 | 0.038 | 0.050 | 0.040 | 0.043 | 0.051 | 0.083 | 0.062 | 0.052 | 0.062 | 1 |
| 16 ropes (17 points, 12 iterations) | 0.102 | 0.100 | 0.055 | 0.082 | 0.058 | 0.058 | 0.103 | 0.098 | 0.084 | 0.068 | 0.079 | 1 |
| 32 ropes (17 points, 12 iterations) | 0.199 | 0.204 | 0.086 | 0.101 | 0.087 | 0.089 | 0.204 | 0.107 | 0.110 | 0.102 | 0.103 | 1 |
| 64 ropes (17 points, 12 iterations) | 0.405 | 0.149 | 0.150 | 0.174 | 0.149 | 0.150 | 0.147 | 0.151 | 0.158 | 0.127 | 0.159 | 1 |
| 128 ropes (17 points, 12 iterations) | 0.899 | 0.290 | 0.281 | 0.299 | 0.286 | 0.289 | 0.251 | 0.258 | 0.268 | 0.234 | 0.269 | 1 |
| 2 cloths (12 × 12, 8 iterations) | 0.060 | 0.060 | 0.049 | 0.049 | 0.053 | 0.050 | 0.059 | 0.070 | 0.067 | 0.068 | 0.063 | 1 |
| 4 cloths (12 × 12, 8 iterations) | 0.118 | 0.121 | 0.061 | 0.068 | 0.063 | 0.059 | 0.126 | 0.079 | 0.082 | 0.087 | 0.083 | 1 |
| 8 cloths (12 × 12, 8 iterations) | 0.235 | 0.235 | 0.097 | 0.105 | 0.107 | 0.103 | 0.242 | 0.128 | 0.117 | 0.115 | 0.127 | 1 |
| 16 cloths (12 × 12, 8 iterations) | 0.489 | 0.190 | 0.184 | 0.188 | 0.192 | 0.192 | 0.161 | 0.172 | 0.182 | 0.169 | 0.186 | 1 |
| 32 cloths (12 × 12, 8 iterations) | 1.220 | 0.387 | 0.379 | 0.380 | 0.381 | 0.383 | 0.291 | 0.309 | 0.320 | 0.309 | 0.330 | 1 |
| 2 3D fluids of 250 (2 substeps) | 0.265 | 0.266 | 0.189 | 0.194 | 0.184 | 0.191 | 0.258 | 0.198 | 0.236 | 0.193 | 0.198 | 1 |
| 2 3D fluids of 500 (2 substeps) | 0.603 | 0.598 | 0.379 | 0.386 | 0.381 | 0.391 | 0.605 | 0.386 | 0.453 | 0.398 | 0.395 | 1 |
| 4 3D fluids of 500 (2 substeps) | 1.246 | 1.244 | 0.415 | 0.429 | 0.422 | 0.425 | 1.264 | 0.430 | 0.511 | 0.455 | 0.439 | 1 |
| 2 3D fluids of 1000 (2 substeps) | 1.488 | 1.487 | 0.862 | 0.733 | 0.859 | 0.867 | 1.480 | 0.866 | 0.908 | 0.839 | 0.893 | 1 |
| 4 3D fluids of 1000 (2 substeps) | 3.035 | 0.910 | 0.884 | 0.915 | 0.900 | 0.899 | 0.919 | 0.936 | 1.199 | 0.919 | 0.916 | 1 |
| 8 3D fluids of 1000 (2 substeps) | 6.462 | 1.755 | 1.760 | 1.966 | 1.772 | 1.826 | 1.403 | 1.485 | 2.044 | 1.474 | 1.365 | 1 |
| a 3D fluid of 1000 (2 substeps) | 0.700 | 0.690 | 0.690 | 0.685 | 0.700 | 0.700 | 0.699 | 0.694 | 0.951 | 0.702 | 0.698 | 1 |
| a 3D fluid of 2000 (2 substeps) | 1.812 | 1.804 | 1.820 | 1.475 | 1.467 | 1.825 | 1.795 | 1.800 | 1.641 | 1.557 | 1.845 | 1 |
| a 3D fluid of 4000 (2 substeps) | 5.174 | 3.542 | 3.512 | 3.511 | 3.527 | 5.124 | 3.633 | 3.577 | 3.809 | 3.499 | 5.242 | 1 |
| a 3D fluid of 8000 (2 substeps) | 11.581 | 7.709 | 7.715 | 7.640 | 8.028 | 7.718 | 7.490 | 7.549 | 7.478 | 7.370 | 8.063 | 1 |
| 2 2D fluids of 250 (2 substeps) | 0.459 | 0.469 | 0.301 | 0.338 | 0.361 | 0.318 | 0.463 | 0.309 | 0.312 | 0.315 | 0.313 | 1 |
| 2 2D fluids of 500 (2 substeps) | 1.192 | 1.180 | 0.705 | 0.705 | 0.690 | 0.704 | 1.221 | 0.693 | 0.716 | 0.697 | 0.701 | 1 |
| 4 2D fluids of 500 (2 substeps) | 2.467 | 2.456 | 0.752 | 0.784 | 0.753 | 0.794 | 2.480 | 0.782 | 0.756 | 0.750 | 0.777 | 1 |
| 4 2D fluids of 1000 (2 substeps) | 5.731 | 1.627 | 1.673 | 1.649 | 1.633 | 1.603 | 1.674 | 1.624 | 1.327 | 1.598 | 1.641 | 1 |
| a 2D fluid of 1000 (2 substeps) | 1.393 | 1.396 | 1.397 | 0.768 | 1.388 | 1.378 | 1.396 | 1.398 | 0.906 | 1.389 | 1.405 | 1 |
| a 2D fluid of 2000 (2 substeps) | 3.144 | 3.116 | 3.137 | 1.492 | 1.563 | 3.113 | 3.122 | 3.129 | 1.589 | 1.618 | 3.205 | 1 |
| a 2D fluid of 4000 (2 substeps) | 6.820 | 3.219 | 3.092 | 3.169 | 3.038 | 6.815 | 3.185 | 3.191 | 2.915 | 2.751 | 6.833 | 1 |
| a 2D fluid of 8000 (2 substeps) | 15.148 | 6.796 | 7.085 | 6.613 | 6.590 | 6.623 | 5.948 | 6.177 | 5.601 | 5.725 | 8.073 | 1 |
| 16 ragdolls (3 bones, half limp) | 0.031 | 0.030 | 0.031 | 0.031 | 0.031 | 0.031 | 0.029 | 0.030 | 0.031 | 0.031 | 0.030 | 1 |
| 64 ragdolls (3 bones, half limp) | 0.114 | 0.111 | 0.113 | 0.113 | 0.113 | 0.111 | 0.112 | 0.113 | 0.113 | 0.113 | 0.112 | 1 |
| 256 ragdolls (3 bones, half limp) | 0.336 | 0.327 | 0.337 | 0.321 | 0.319 | 0.349 | 0.339 | 0.334 | 0.331 | 0.331 | 0.336 | 1 |

- Ropes split pay from 8 on four lanes, from about 16 to 24 on eight
  (8 ropes: 0.083 against 0.051 ms): 800 point passes a task up to four
  threads, 2,400 above.
- Cloths: from two on four lanes (0.049 against 0.060), from four on eight
  (two: 0.070 against 0.059): 1,100 and 2,000.
- Fluid sets pay from two fluids of 250 particles (0.19 against 0.27 ms in
  3D, 0.30 against 0.46 in 2D) on both: 500 particle substeps a task.
- One fluid's passes: a 2D fluid pays from 1,000 particles at 256 a task
  (1.39 to 0.77 ms), its neighbour search (binary searches in a sorted
  grid) being most of its step; a 3D one from 2,000 at 512 (1.81 to 1.47;
  at 1,000 even), its serial relaxation weighing more: 256 and 512.
- Ragdolls are flat in every mode (256 of them: 0.33 ms, rapier's step of
  their 768 bones included); their sync writes rapier's bodies and stays
  serial.

## The committed grains (soft-measured.txt)

| scene | off | 4 | 8 | hashes |
| 8 ropes (17 points, 12 iterations) | 0.050 | 0.043 | 0.049 | 1 |
| 16 ropes (17 points, 12 iterations) | 0.099 | 0.055 | 0.099 | 1 |
| 32 ropes (17 points, 12 iterations) | 0.195 | 0.086 | 0.080 | 1 |
| 64 ropes (17 points, 12 iterations) | 0.400 | 0.147 | 0.130 | 1 |
| 128 ropes (17 points, 12 iterations) | 0.933 | 0.284 | 0.243 | 1 |
| 2 cloths (12 × 12, 8 iterations) | 0.060 | 0.050 | 0.060 | 1 |
| 4 cloths (12 × 12, 8 iterations) | 0.122 | 0.060 | 0.089 | 1 |
| 8 cloths (12 × 12, 8 iterations) | 0.249 | 0.097 | 0.116 | 1 |
| 16 cloths (12 × 12, 8 iterations) | 0.486 | 0.190 | 0.170 | 1 |
| 32 cloths (12 × 12, 8 iterations) | 1.256 | 0.387 | 0.317 | 1 |
| 2 3D fluids of 250 (2 substeps) | 0.265 | 0.194 | 0.191 | 1 |
| 2 3D fluids of 500 (2 substeps) | 0.610 | 0.394 | 0.377 | 1 |
| 4 3D fluids of 500 (2 substeps) | 1.282 | 0.465 | 0.432 | 1 |
| 2 3D fluids of 1000 (2 substeps) | 1.464 | 0.865 | 0.831 | 1 |
| 4 3D fluids of 1000 (2 substeps) | 3.038 | 0.970 | 0.916 | 1 |
| 8 3D fluids of 1000 (2 substeps) | 6.380 | 1.863 | 1.262 | 1 |
| a 3D fluid of 1000 (2 substeps) | 0.686 | 0.701 | 0.693 | 1 |
| a 3D fluid of 2000 (2 substeps) | 1.813 | 1.624 | 1.509 | 1 |
| a 3D fluid of 4000 (2 substeps) | 5.233 | 3.754 | 3.591 | 1 |
| a 3D fluid of 8000 (2 substeps) | 11.962 | 7.668 | 7.347 | 1 |
| 2 2D fluids of 250 (2 substeps) | 0.483 | 0.302 | 0.339 | 1 |
| 2 2D fluids of 500 (2 substeps) | 1.220 | 0.683 | 0.700 | 1 |
| 4 2D fluids of 500 (2 substeps) | 2.510 | 0.734 | 0.778 | 1 |
| 4 2D fluids of 1000 (2 substeps) | 5.793 | 1.599 | 1.372 | 1 |
| a 2D fluid of 1000 (2 substeps) | 1.400 | 0.750 | 0.860 | 1 |
| a 2D fluid of 2000 (2 substeps) | 3.131 | 1.508 | 1.482 | 1 |
| a 2D fluid of 4000 (2 substeps) | 6.982 | 3.013 | 2.895 | 1 |
| a 2D fluid of 8000 (2 substeps) | 15.190 | 6.703 | 5.781 | 1 |
| 16 ragdolls (3 bones, half limp) | 0.030 | 0.031 | 0.030 | 1 |
| 64 ragdolls (3 bones, half limp) | 0.112 | 0.112 | 0.112 | 1 |
| 256 ragdolls (3 bones, half limp) | 0.331 | 0.339 | 0.338 | 1 |

## Determinism

The determinism tool's physics scenarios (`tools/wasm-determinism`
`physics_hash`: the beyond scenario 3D and 2D for 600 ticks, a 1,000-body
pen for 300), run natively from a temporary og-physics example at
b6ff2494: 18126409214070009362 with lanes off, on four threads (483
parallel calls) and with every pass split (`Grains::always`) on four and on
eight (11,327 parallel calls each): the hash of
`tests/evidence/P4/physics-lanes/determinism.md`, unchanged.
og-physics's `tests/lanes.rs` checks the same per test run.

## Chrome acceptance, 2026-10-24

The coordinator ran the measure checkout at `27649c3a`, using its full
gate's publish wasm, Chrome for Testing 150, WebGPU, and a 1280×720 canvas
on the 12-core M4 Pro. No wasm rebuild or threshold change was needed.
From `packages/engine`, five rounds of:

```sh
bun bench/lanes.ts --chrome --pile --bodies 1000,2000 --frames 600 --out <round-directory>
```

Each round runs non-isolated, isolated with lanes off, then isolated with
lanes on for each pile. Each process warms for 60 frames and measures
600. `chrome-2026-10-24.json` retains all 30 runs and their frame samples.
The table shows the median and range of the five per-run p50 values,
in milliseconds. Frame time is the Worker call's round trip, not a GPU
completion fence. Step is the engine's whole simulation step.

| Bodies | Mode | Frame p50 median | Frame p50 range | Step p50 median |
|---|---|---:|---|---:|
| 1,000 | Not isolated | 5.400 | 5.300 to 5.500 | 4.900 |
| 1,000 | Isolated, lanes off | 5.420 | 5.350 to 5.465 | 4.985 |
| 1,000 | Isolated, eight lanes | 5.565 | 5.540 to 5.650 | 5.130 |
| 2,000 | Not isolated | 11.400 | 11.400 to 11.500 | 10.800 |
| 2,000 | Isolated, lanes off | 11.430 | 11.340 to 11.435 | 10.835 |
| 2,000 | Isolated, eight lanes | 6.400 | 6.345 to 6.630 | 5.845 |

The 2,000-body lanes runs report `pool: web-workers`, eight threads and
1,921 parallel calls each. The 1,000-body runs report zero parallel calls:
the eight-thread threshold is 1,500 bodies in `og-physics/src/lanes.rs`.
They are a serial control, not evidence of parallel physics at 1,000.
Simulation accounts for most measured frame time; no CPU profile was
taken to separate Rapier, synchronization and component work. Load samples
during rounds 2 through 5 ranged from 6.63 to 11.70, above row 89's idle
limit. These timings describe this run and do not justify retuning grains
or claiming an idle-machine performance budget passed.

All modes and rounds captured tick 660 with the same RGBA SHA-256 per pile:

- 1,000: `200390c7afbf30c1028a27038d855ab5eda150ed647b1d36813fa87f457ffa22`
- 2,000: `5a65e93417f80efc45344a8a82463f0aebf481b06455c912243af8a5c1b58784`

The coordinator inspected the eight-lane captures of both piles: orange
boxes render on the ground, with no blank or error frame. Every frame
call succeeded. These are pixel hashes, not whole-world hashes; publish
strips the world dump. The direct web `og.physics.stats` acceptance and
whole-world Chrome parity remain unverified, so owed row 52 stays partly
open.

## Chrome whole-world parity and direct physics stats (2026-10-26)

**PASS:** on `db10aaf9`, all three Chrome modes produced the same complete
world snapshot and pixels per pile. This closes the two acceptance checks
left open above. `chrome-2026-10-26.json` contains the six runs, all frame
samples, direct physics stats (including every phase), wasm SHA-256s, load
samples and the temporary preload's source and command. No source, grain,
baseline or reference changed.

The machine was macrunner (12-core M4 Pro, macOS 27.2), Chrome for Testing
150.0.7871.115, WebGPU, 1280×720. The gate's publish wasm from `3e9d7f21`
was reused: the intervening changes through `db10aaf9` affect tests and
documentation, not runtime code. Each mode had a 60-second pause before
starting its page, 60 warm frames and 600 measured frames. One-minute load
samples immediately before/after each run ranged from 4.30 to 6.12; these
are endpoint samples, not continuous proof of idle load. This is one run
per mode for parity and stats acceptance, not a new performance baseline.

After the timed frames and capture, CDP read `og_physics_stats(0)` and
`og_world_snapshot(0)` directly from the engine Worker's cached wasm
module. The snapshot hash is SHA-256 of the **entire inflated OGWS world
snapshot**, excluding only the 24-byte OGSV save wrapper. It covers local
as well as shared state. It is not the canonical state-stream FNV returned
by `Engine.hash()` (publish strips that probe), the shared-only
`og_net_hash`, or a pixel hash. Identical complete snapshot bytes establish
whole-world parity without enabling a non-publish runtime. The receipt
also records the save's FNV checksum and snapshot byte count.

| Bodies | Mode | Frame p50 ms | Step p50 ms | Direct physics ms | Threads | Physics pool | Split phases |
|---|---|---:|---:|---:|---:|---|---|
| 1,000 | Not isolated | 5.100 | 4.800 | 4.600 | 1 | false | none |
| 1,000 | Isolated, lanes off | 5.135 | 4.790 | 4.455 | 1 | false | none |
| 1,000 | Isolated, eight lanes | 5.185 | 4.830 | 4.285 | 8 | false | none |
| 2,000 | Not isolated | 10.900 | 10.400 | 9.300 | 1 | false | none |
| 2,000 | Isolated, lanes off | 10.870 | 10.330 | 10.165 | 1 | false | none |
| 2,000 | Isolated, eight lanes | 6.340 | 5.830 | 5.000 | 8 | true | rapier, readout |

Direct physics times are single observations of the last completed step
(tick 659), not medians. Capture and snapshot are at the resulting boundary,
tick 660. The eight-lane 2,000-box run reports 1,921 parallel calls; every
other run reports zero. At 1,000 bodies, eight threads are available but
physics remains serial, as the 1,500-body threshold requires.

| Bodies | Whole-world snapshot SHA-256 (all three modes) | Bytes |
|---|---|---:|
| 1,000 | `3aa525ccb227f0d73899aa3da9ec343a767625e691626445d1d978723c36bb24` | 3,683,034 |
| 2,000 | `f40f709bfac749903d850d16f072362298130e0aa6469605b38659f8fc690505` | 7,695,342 |

RGBA SHA-256s also match all three modes and the October 24 captures:
`200390c7afbf30c1028a27038d855ab5eda150ed647b1d36813fa87f457ffa22`
(1,000) and
`5a65e93417f80efc45344a8a82463f0aebf481b06455c912243af8a5c1b58784`
(2,000). The 2,000-box eight-lane PNG was inspected: a pile of orange boxes
on the gray floor, not a blank/error frame. Local PNGs and raw intermediate
receipts remain in `.amp/in/lanes-52/`; the committed JSON retains their
digests and the complete measurement data.
