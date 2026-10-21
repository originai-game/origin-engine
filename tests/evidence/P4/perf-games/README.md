# The dogfood games' frame time (wave 12, perf-games)

Branch `perf-games` (lane 4), from main at 7526ff3c, main merged at
c955082d and again at 48133d07. Per game: `hollowbell.md`, `kestrel-stair.md`,
`lantern-vale.md` (the before/after tables, the systems by name and a
flame-style breakdown of the engine thread).

## The load these were taken at

The machine was never idle this round. Until 23:10 the wave-end
lavapipe pass held a Linux VM at 5 to 8 cores and the load average was
15 to 27; the first measurement (each per-game file's "at a load average
of 20 to 25" section) was taken then. After the VM stopped the load fell
to 7 to 16 with the coordinator's gate and an iOS simulator (whose Metal
host shares the GPU) still running: the second measurement, the og
polish runs and the profiles were taken then, each table naming its
load range (a few runs caught the gate's peaks, to 43). Every table
compares before against after interleaved round by round, so both sides
saw the same machine; an idle machine's figures are owed (below).

## Method

- **Builds.** "Before" is main (c955082d for the first measurement,
  48133d07 for the second) with only this branch's measuring commit
  (2b61462a) applied, "after" is the branch head merged with the same
  main; each
  in the release profile (`cargo build --release -p og-host-node -p
  og-cli`) and the dev one (`cargo build -p og-host-node -p og-cli`),
  installed into `packages/engine/native/` as
  `packages/engine/scripts/build.sh` does (copy, re-sign, align). `og` is
  the dev one throughout (it only cooks and spawns the run).
- **The play run.** `og run examples/<game> --headless --json --ticks 600
  --size 1280x720 --commands "<script>"` on metal, 600 ticks (10 s) of
  play from a fresh start:
  - Hollowbell: `0.1:play;0.5:key_down:KeyD;1.5:key_down:Space;1.8:key_up:Space;2:key_down:KeyX;2.05:key_up:KeyX;3:key_down:KeyX;3.05:key_up:KeyX;4:key_down:Space;4.4:key_up:Space;6:key_up:KeyD;6:key_down:KeyA;8:key_down:KeyX;8.05:key_up:KeyX;9.5:key_up:KeyA`
  - The Kestrel Stair: `0.1:play:0:quick;0.5:key_down:KeyW;2:key_down:KeyJ;2.05:key_up:KeyJ;3:key_down:KeyD;5:key_up:KeyD;5:key_down:Space;5.2:key_up:Space;7:key_down:KeyJ;7.05:key_up:KeyJ;9.5:key_up:KeyW`
  - Lantern Vale: `0.1:run;0.5:key_down:KeyW;3:key_down:KeyD;5:key_up:KeyD;6:key_up:KeyW;6:warp:0;6.2:key_down:KeyE;8:key_up:KeyE;8.2:key_down:KeyS;9.9:key_up:KeyS`
- **What is read.** `og run --json` now reports `frames` (each frame's
  tick and render milliseconds from the frame answer's `stepMs` and
  `renderMs`, the first five frames left out, ordered as og polish orders
  them: median `s[n/2]`, p95 `s[floor(0.95 n)]`) and `systems` (each
  system's mean milliseconds a run over the run, from the world's clock,
  `World::set_clock`). A table's cell is the median over the rounds of
  each run's figure.
- **Rounds.** Three rounds (six for the second, unpaced); in each, the
  release before, release after, dev before and dev after addons in turn,
  each running the three games. Paced runs: the same with `og run`
  without `--headless` (a frame each 16.7 ms of wall clock).
- **The breakdown.** macOS `sample` of the engine process for 8 s of a
  longer play run (1,800 ticks; Hollowbell 7,200 and Lantern Vale 3,600 on
  release, which run through 1,800 ticks too fast to sample), folded per
  stack, each row the share of `og_engine_frame`'s samples whose stack
  holds that function (nested rows are within their parent). On release,
  inlining hides `run_system`; "game systems" is then the dispatcher into
  TypeScript. The lanes' threads were sampled too (kestrel-stair.md).
- **Frames and hashes.** Each game's play run with three screenshots
  (2.1, 5.5 and 9.6 s) on before and after: the same world hash
  (Hollowbell e24062103e2b2391, The Kestrel Stair ec228f8a48073301,
  Lantern Vale 0fb19ee7ce2ba3c7, as at 7526ff3c) and byte-identical
  PNGs, at both merges (sha prefixes, Hollowbell 6777e6f85d015978
  e906b85052f87437 8428129f8c263066 at c955082d and 90a49fe2d3650526
  52bf61a3ad09a1a3 2a16c0ae7313708e at 48133d07, whose palettes moved its
  frames on main; The Kestrel Stair 4dbea6fbbeeb79e7 ec1f49dfb70e2366
  2ef72fec2dbba2b7; Lantern Vale d5f3df40426b317f c469db61cd326c9b
  26ea1def30738eb8), and release ends on the same hashes as dev. The
  og-gfx run was at the first merge (c955082d). `og-gfx --all` links none of the crates this branch changed
  (it builds on og-render, og-core and og-tilemap, which are main's
  here byte for byte); its run here gave RMSE 0 against the committed
  references but for terrain-near 0.0066, water-planar 0.0049,
  terrain-grass 0.0050, terrain-mid 0.0036, water-shadows 0.0017,
  water-3d 0.0006, water-underwater 0.0004, fx-2d 0.00004 and
  materials-array 0.00001, all within their tolerance and all main's own
  (no input of theirs differs from main).

## Summary

At the lightest load (7 to 16, a few runs to 43), six interleaved
rounds of the unpaced play run (`--headless`, as fast as it goes):

Release addon:

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 0.66 → 0.53 | 1.36 → 1.09 | 0.70 → 0.60 | 1.15 → 1.42 |
| The Kestrel Stair | 1.98 → 1.28 | 2.90 → 2.31 | 1.21 → 1.50 | 2.63 → 4.40 |
| Lantern Vale | 0.90 → 0.71 | 1.88 → 1.78 | 2.14 → 3.71 | 7.17 → 7.79 |

Dev addon:

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 3.06 → 2.06 | 4.58 → 3.70 | 2.92 → 1.90 | 3.61 → 2.60 |
| The Kestrel Stair | 5.29 → 2.98 | 8.85 → 4.00 | 2.24 → 1.98 | 3.69 → 2.50 |
| Lantern Vale | 2.77 → 2.13 | 4.55 → 4.98 | 2.49 → 2.27 | 3.29 → 3.71 |

Unpaced, a frame's render milliseconds include the wait for the GPU
whenever the engine thread outruns it (the encoder's finish blocks on
Metal's command buffers): The Kestrel Stair's render rose with its
faster step, and Lantern Vale's is its GPU's time on a GPU the gate and a
simulator were using. Paced at 60 Hz (`og run` without `--headless`),
three interleaved rounds, the render is the engine thread's own:

Release addon:

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 1.00 → 0.79 | 1.59 → 1.21 | 0.90 → 0.74 | 1.36 → 0.89 |
| The Kestrel Stair | 2.00 → 1.14 | 4.02 → 1.68 | 1.04 → 0.97 | 1.71 → 1.26 |
| Lantern Vale | 0.86 → 0.74 | 1.42 → 1.29 | 1.22 → 1.18 | 1.48 → 1.60 |

Dev addon:

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 3.19 → 2.21 | 4.08 → 3.04 | 2.95 → 1.94 | 3.34 → 2.19 |
| The Kestrel Stair | 4.96 → 2.98 | 6.89 → 3.79 | 2.14 → 1.95 | 2.62 → 2.34 |
| Lantern Vale | 2.66 → 1.93 | 3.90 → 2.78 | 2.39 → 2.07 | 2.97 → 2.40 |

og polish (every playtest of each game, frame time scored on the worst
playtest's p95 step plus render: 8.3 ms 100, 33 ms 0), before and after
on each addon, at the load named in the last column (before's, then
after's, at the start and end of each run):

| Game | Addon | frame time score | feel | worst frame p95 | step p95, median over playtests (worst) | render p95 (worst) | load |
|---|---|---|---|---|---|---|---|
| Hollowbell | release | 100 → 100 | 97 → 97 | 3.37 → 2.75 | 1.53 (2.12) → 1.24 (1.56) | 0.93 (1.37) → 0.73 (0.79) | 14.58-13.77, 13.77-12.87 |
| The Kestrel Stair | release | 100 → 100 | 99 → 99 | 4.75 → 4.91 | 2.12 (2.96) → 1.31 (2.49) | 1.73 (2.07) → 1.73 (2.16) | 12.87-11.77, 11.77-10.88 |
| Lantern Vale | release | 100 → 100 | 99 → 99 | 3.05 → 2.69 | 1.52 (1.76) → 1.39 (1.54) | 1.48 (1.72) → 1.51 (1.77) | 10.88-10.17, 10.17-10.00 |
| Hollowbell | dev | 0 → 91 | 77 → 95 | 90.07 → 10.60 | 8.81 (61.39) → 6.03 (8.26) | 4.11 (48.39) → 2.29 (2.76) | 10.00-23.69, 23.69-14.85 |
| The Kestrel Stair | dev | 81 → 100 | 95 → 99 | 13.10 → 7.19 | 6.27 (9.25) → 4.01 (4.47) | 2.44 (3.92) → 2.37 (2.67) | 14.85-9.76, 9.76-7.64 |
| Lantern Vale | dev | 82 → 85 | 96 → 96 | 12.74 → 12.00 | 8.21 (9.98) → 7.56 (9.48) | 2.81 (3.49) → 2.61 (4.06) | 7.64-7.33, 7.33-7.84 |

Hollowbell's dev "before" run caught the gate's peak (load 24): its 90 ms
frame is that, not the game. Against the targets: release step p95 and
render p95 are under 8 ms for all three games on every measurement at
this load (the worst, Lantern Vale's unpaced render p95 7.8 ms, is the
shared GPU's); the dev step p95 is under 16 ms (3.7 to 5.0 unpaced, 9.5
the worst og polish playtest). og polish scores frame time 100 on the
release addon for all three, and on dev 91 (Hollowbell), 100 (The
Kestrel Stair) and 85 (Lantern Vale, whose dev playtests' step p95 is 7
to 9.5 ms against a 2.2 ms median: owed, below).

At a load average of 20 to 25 (the wave-end VM running), three rounds:
The Kestrel Stair's release step p95 was 17.2 → 17.7 ms with the lanes on
and 10.5 → 4.5 with them off (kestrel-stair.md): on a loaded machine the
cloth's jobs wait for pool threads the OS does not run.

## Owed

- The tables and og polish's rows on an idle machine (load under 8 for
  the whole run), release and dev, before and after.
- The lanes on a loaded machine: a soft-body or physics job sent to the
  pool waits for threads the OS does not run (The Kestrel Stair's
  og.physics 0.93 ms a tick serial against 2.9 to 4.2 ms on the pool at a
  load of 20 to 25; 0.49 ms on the pool at 7 to 16). The grains were
  measured idle (tests/evidence/P4/physics-3/); og-lanes has no notion of
  a pool slower than the calling thread right now.
- Release extraction of sprites reads each field by a binary search over
  the layout's names (`Layout::slot`, memcmp most of it): 5 % of
  Hollowbell's release frame. Resolving a record's field slots once per
  frame would leave the reads alone.
- og.ui.end's layout measures every text again each tick (widest_line,
  break_lines: about a third of og.ui.end, 0.05-0.09 ms release, 0.2-0.5
  dev).
- Lantern Vale's dev playtests show step spikes (p95 7 to 9.5 ms against
  a 2.2 ms median) that the play run does not; not profiled.
