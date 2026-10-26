# Debt round 3, `cull-lane-grain`: the CPU frustum cull's lanes threshold (row 57)

Branch `debt/cull-lane-grain`, from main `30376326`. Measured in an orb:
16 vCPUs (Intel Xeon @ 2.60 GHz), Linux under gVisor (the E2B sandbox),
31 GB. As briefed, this stream did not run `scripts/check.sh`, `og test`,
web-reference, a reference capture or a wasm build; the Rust checks were
`cargo check`, `cargo clippy -D warnings` and og-render's unit test, and
the bench below is a standalone release crate (not a workspace member,
built into its own target directory under /tmp).

## Commits

| Commit | What |
|---|---|
| `b6aab9aa` | og-render: the CPU cull splits only from 131,072 bounds (`Frustum::test_spheres`), its unit test; lanes.test.ts and the lane-panic fixture follow. |
| this one | The evidence, row 57 closed in docs/phase-1.md, dropped from docs/release.md's deferred list. |

## The row

`tests/evidence/P4/web-host-tools/culling-threshold.md` (M4 Pro, load 20
to 33): natively with lanes on, the CPU cull split across the pool from
2,048 bounds (`CULL_GRAIN` 1,024, og-lanes' "two grains") and the CPU
path's frame then cost two to three times its lanes-off time (13.2 ms
against 4.9 ms at 65,536 instances on the street).

## Where the cull and its split are

- `Renderer::cull` (og-render `lib.rs`) marks the included batches'
  bounds in a `Vec<bool>`, then tests every marked sphere against the
  pass's frustum, then builds the draw ranges serially from the flags.
  It runs once per CPU-culled view: the camera, each shadow cascade and
  light face, the planar reflection, every due probe face. Only the
  middle step was on the lanes:
  `og_lanes::for_each_indexed(Site::Cull, inside, 1024, ..)`.
- og-lanes splits a call from two grains (`Least::Grains(grain, 2)`) and
  runs it as `imp::ranges`: natively `pool.install` of a rayon
  `into_par_iter` over the 1,024-flag ranges; in the lanes wasm the same
  loop sent through the relay Worker, the engine thread waiting on
  og-lanes' latch. There is no `cfg` between the two: the web's lanes
  variant reaches the cull through the same call and the same threshold.
  The single-threaded wasm never splits.

## Why the split lost

A sphere test is 5 ns here (10 µs for 2,048 bounds, 513 µs for 100,000),
the same per bound on the pool as serially, so what the split adds is
fixed cost:

- **The wake and the join.** The engine thread hands the job to a
  sleeping pool, rayon wakes workers one by one as they steal, and the
  engine thread waits for the last range. On this orb that costs about
  50 µs a call on two threads and 300 to 600 µs on eight to sixteen (each
  woken thread is another futex wake, and gVisor makes those dear; the
  M4's idle hop was tens of µs, its loaded one 0.5 to 2.6 ms). That is 5
  to 60 times the work of 2,048 bounds.
- **Load.** With other processes on the cores, a woken lane waits for a
  scheduler quantum. The serial loop is untouched (its p50 and mean do
  not move with load), the split's p50 sometimes still wins, but its p90
  reaches 1 to 13 ms and its mean is 2 to 3 times serial at 100,000
  bounds, and more below.
- **Not false sharing.** Each range owns 1,024 one-byte flags (16 cache
  lines) and shares at most one line with its neighbour; at 524,288
  bounds eight threads reach 0.25 of serial, near the ideal less the hop.
- **Not the chunk size.** Coarser tasks (4,096, 16,384, 65,536 bounds,
  one task a thread) are slower than 1,024 at every count where a split
  wins: a thread that wakes late holding a big range delays the join,
  while with small ranges the early threads steal its work.
- **Per view.** The loss is paid once per culled view (camera and three
  cascades in the M4's city), which is how 1 to 2 ms a hop became 8 ms a
  frame there.

## How it was measured

`bench/` is a standalone crate (its own `[workspace]`, og-lanes by path,
glam with the engine's `scalar-math`, the release profile of the
engine's `Cargo.toml`). Its loop is a copy of the engine's: mark every
bound, `for_each_indexed` over the flags with `intersects_sphere`
(`frustum_planes` copied from `frame.rs`), a grid of spheres under a
perspective camera so about half are kept. Each round runs serial and
every grain in turn, 300 µs of spinning between calls (the pool sleeps
again, as between a frame's culls; with no gap the figures are of the
same order), and asserts every variant's flags equal the serial
ones. Cells are p50 / mean in µs per call over 150 to 300 rounds after
20 warm-up rounds; the ratio is the mean against serial's. "1,024
(serial)" is the old code's own result there: one grain does not split.

    cd tests/evidence/debt3/cull-lane-grain/bench
    cargo build --release --target-dir /tmp/cullbench-target   # outside the checkout's target/
    ln -s /tmp/cullbench-target target; mkdir -p out
    for t in 2 4 8 16; do OG_LANES=$t ./target/release/cullbench 300 300 > out/idle-t$t.txt; done
    ./load.sh 16; ./load.sh 32                                    # 16 or 32 busy loops beside it
    python3 summarize.py idle 2,4,8,16 [g1024|g4096|g16384|g65536|perthread]

`raw/` has every run's output. Idle: load average under 1. Loaded: 16
busy loops on the 16 vCPUs (`load16`), and 32 (`load32`, twice
oversubscribed; the one-minute load average climbed from 7 to 13 and 15
during the 20-second runs, the run queue itself was above 32 throughout).

## The table: serial against the old split (1,024 a task from 2,048)

Idle:

| bounds | serial | lanes, 2 threads | lanes, 4 threads | lanes, 8 threads | lanes, 16 threads |
|---:|---:|---:|---:|---:|---:|
| 1,024 | 5 / 6 | (serial) | (serial) | (serial) | (serial) |
| 2,048 | 11 / 12 | 97 / 102 (8.98×) | 112 / 118 (10.63×) | 120 / 126 (10.88×) | 121 / 127 (6.81×) |
| 8,192 | 42 / 46 | 135 / 143 (3.14×) | 144 / 149 (3.47×) | 392 / 377 (8.64×) | 487 / 489 (7.76×) |
| 32,768 | 164 / 181 | 231 / 243 (1.29×) | 200 / 210 (1.21×) | 259 / 317 (1.82×) | 782 / 762 (4.20×) |
| 100,000 | 513 / 526 | 409 / 423 (0.83×) | 316 / 323 (0.63×) | 310 / 355 (0.66×) | 529 / 655 (1.25×) |

16 busy loops:

| bounds | serial | lanes, 2 threads | lanes, 4 threads | lanes, 8 threads | lanes, 16 threads |
|---:|---:|---:|---:|---:|---:|
| 1,024 | 5 / 6 | (serial) | (serial) | (serial) | (serial) |
| 2,048 | 11 / 11 | 53 / 127 (10.93×) | 78 / 538 (48.04×) | 152 / 982 (88.43×) | 160 / 868 (78.91×) |
| 8,192 | 42 / 44 | 89 / 107 (2.15×) | 110 / 534 (12.44×) | 200 / 692 (15.72×) | 335 / 711 (16.05×) |
| 32,768 | 165 / 170 | 222 / 243 (1.25×) | 162 / 1233 (7.34×) | 266 / 956 (5.61×) | 437 / 672 (3.98×) |
| 100,000 | 512 / 521 | 372 / 1155 (2.26×) | 296 / 1538 (3.02×) | 320 / 1260 (2.40×) | 663 / 1443 (2.77×) |

32 busy loops:

| bounds | serial | lanes, 2 threads | lanes, 4 threads | lanes, 8 threads | lanes, 16 threads |
|---:|---:|---:|---:|---:|---:|
| 1,024 | 5 / 5 | (serial) | (serial) | (serial) | (serial) |
| 2,048 | 10 / 11 | 54 / 966 (80.48×) | 73 / 662 (65.59×) | 66 / 683 (63.82×) | 69 / 774 (68.46×) |
| 8,192 | 41 / 43 | 89 / 685 (15.82×) | 98 / 590 (13.66×) | 117 / 740 (17.04×) | 282 / 890 (20.41×) |
| 32,768 | 162 / 171 | 222 / 1018 (6.12×) | 167 / 1075 (5.75×) | 310 / 865 (5.05×) | 405 / 1060 (6.30×) |
| 100,000 | 507 / 512 | 607 / 1003 (1.92×) | 616 / 1034 (2.03×) | 497 / 1591 (3.14×) | 622 / 1662 (3.25×) |

The split loses at 2,048 to 32,768 bounds everywhere, by 1.2 to 11 times
idle and up to 90 times loaded. At 100,000 it wins idle on two to eight
threads (0.63 to 0.83) and loses loaded (2 to 3 times, mean).

## Where it starts to win

Idle, the old split from 65,536 bounds up:

| bounds | serial | lanes, 2 threads | lanes, 4 threads | lanes, 8 threads | lanes, 16 threads |
|---:|---:|---:|---:|---:|---:|
| 65,536 | 312 / 328 | 299 / 310 (0.93×) | 254 / 274 (0.84×) | 284 / 317 (1.00×) | 693 / 773 (2.39×) |
| 131,072 | 627 / 649 | 477 / 490 (0.74×) | 337 / 362 (0.56×) | 351 / 369 (0.57×) | 530 / 674 (1.04×) |
| 262,144 | 1248 / 1314 | 881 / 889 (0.71×) | 518 / 534 (0.41×) | 483 / 494 (0.39×) | 538 / 651 (0.48×) |

32 busy loops:

| bounds | serial | lanes, 2 threads | lanes, 4 threads | lanes, 8 threads | lanes, 16 threads |
|---:|---:|---:|---:|---:|---:|
| 65,536 | 313 / 325 | 392 / 763 (2.45×) | 392 / 1331 (4.10×) | 412 / 1351 (3.28×) | 566 / 1152 (3.56×) |
| 131,072 | 623 / 653 | 740 / 1054 (1.65×) | 491 / 1060 (1.65×) | 677 / 762 (0.81×) | 660 / 1143 (1.75×) |
| 262,144 | 1274 / 1491 | 1409 / 1411 (1.11×) | 1209 / 1544 (1.04×) | 1073 / 1723 (1.04×) | 1009 / 2137 (1.70×) |

Grains, idle, from 131,072 bounds (`raw/grains-t*.txt`), mean against
serial on four / eight threads:

| bounds | 1,024 | 4,096 | 16,384 | 65,536 | one a thread |
|---:|---:|---:|---:|---:|---:|
| 131,072 | 0.56 / 0.58 | 0.72 / 0.80 | 0.76 / 0.90 | 1.11 / 1.12 | 0.82 / 0.90 |
| 262,144 | 0.43 / 0.35 | 0.52 / 0.52 | 0.54 / 0.51 | 0.61 / 0.67 | 0.61 / 0.58 |
| 524,288 | 0.37 / 0.25 | 0.42 / 0.33 | 0.43 / 0.35 | 0.46 / 0.38 | 0.51 / 0.38 |

## The change

`Frustum::test_spheres` (og-render `frame.rs`) now holds the cull's
tests, and `Renderer::cull` calls it. It splits only from
`CULL_LANES_FROM` = 131,072 bounds, still 1,024 a task (`CULL_GRAIN`);
below, it passes og-lanes one grain of everything, which runs on the
calling thread (the idiom og-physics' soft bodies use). 131,072 is the
first count where the idle split wins on every thread count to eight by
a quarter or more (0.56 to 0.74); at 65,536 it wins by at most 16 % idle
on four threads, nothing on eight, and loses 2.5 to 4 times loaded.
Every count the row named (2,048 to 100,000) now runs serially, which is
the faster or equal choice in every condition measured but idle
65,536 to 100,000 bounds on two to eight threads (up to 0.2 ms a view
given up there, against 0.5 to 1 ms a view saved at 100,000 loaded).

What a fixed threshold cannot do: above it, a loaded machine still pays
(1.0 to 1.7 times serial at 131,072 and 262,144, mean). A gauge like
og-physics' soft-body `Gauge` (time a pool call, back off to serial
after a loss) would cover that. It was not added: natively a scene that
large takes GPU culling (`GPU_CULLING_MIN_INSTANCES` 16,384) and reaches
the CPU cull only under `Culling::Cpu`, object motion blur, past the
storage-binding limit, or for probe faces; and the gauge is og-physics'
private type, which would first have to move into og-lanes.

## Determinism

The culled set does not depend on where the tests run: each flag is
written once, by its own sphere's test, and the draw ranges are built
from the flags serially afterwards, so the instance records a draw reads
are the same bytes. og-render's new unit test
`frame::tests::the_cull_splits_only_from_its_threshold_and_keeps_the_same_bounds`
checks, at 0, 1, 2,048, 8,192, 100,000, 131,071, 131,072, 132,113 and
262,151 bounds (every seventh one unmarked though in view; a partial view
asserted): the flags equal each marked sphere's own test computed apart,
with lanes on and off, and the call goes to the pool (`calls_here`)
exactly when lanes have threads and the count is at least 131,072. Run at
every thread count:

    cargo test -p og-render --lib frame::tests        # the default pool, 8 threads
    OG_LANES=off|2|3|4|8|16 target/debug/deps/og_render-<hash> frame::tests

all six pass. On the old code (the split from two grains, forced back
for the check) it fails: `2048 bounds, lanes true: parallel calls`,
left 1, right 0.

No reference frame can move: below 131,072 bounds the flags are the
same and so is everything drawn from them, and no scene in og-tests,
the templates or the browser tiers comes near 131,072 CPU-culled bounds.
Nothing was re-recorded.

## What else moved with it

The two tests that relied on the cull splitting in small scenes:

- `packages/engine/tests/lanes.test.ts`: the enlarged reference scene
  (about 2,500 bounds) asserted the `cull` site split. It now asserts
  `anim` split and `cull` did not; the pixels and hashes with lanes on
  and off are compared as before.
- `packages/engine/tests/web/lane-panic.ts` injected its fault at the
  `cull` site of a 2,304-box fixture, which no longer splits. The fixture
  is now 160 skinned walkers (the reference scene's `walker.glb`, copied
  into the fixture's assets), whose animation evaluation splits (more
  than two of og-anim's 64-skeleton grains, in the lanes wasm as
  natively), and the fault is injected at `anim`: `lane: "anim"`, "a
  fault injected at the anim site", the card's "in a lane (anim)".

Neither was run here (bun and browser tests are the coordinator's and the
Mac lane's).

## Not measured here

- The lanes wasm in Chrome: the same threshold applies there (one code
  path). The Worker hop is dearer than rayon's natively and a wasm test
  costs more than 5 ns, so the crossover may sit elsewhere; but the web
  CPU-culls large scenes only on WebGL2, and nothing there approaches
  131,072 bounds.
- The M4 Pro: its idle hop is cheaper (tens of µs), so its idle crossover
  is likely lower than 131,072; its loaded figures (the row's) lose at
  every count, as here.
