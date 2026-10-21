# The dogfood games' frame time (wave 12, perf-games)

Branch `perf-games` (lane 4), from main at 7526ff3c, main merged at
c955082d. Per game: `hollowbell.md`, `kestrel-stair.md`,
`lantern-vale.md` (the before/after tables, the systems by name and a
flame-style breakdown of the engine thread).

## The load these were taken at

The machine never fell under a load average of 8 while this round ran:
the wave-end lavapipe pass held a Linux VM at 5 to 8 cores from 20:00
to past 22:40, an iOS simulator kept its Metal host busy throughout, and
the coordinator's gate ran tests beside it. Every number here was taken
at a load average of 15 to 27 (each table says its range), so the
absolute times are the loaded machine's, and the GPU-bound one (Lantern
Vale's render) is the GPU shared with the simulator's. What the tables
compare is before against after, interleaved round by round on the same
machine at the same time; the idle-machine figures are owed (below).

## Method

- **Builds.** "Before" is main at c955082d with only this branch's
  measuring commit (2b61462a) applied, "after" is the branch head; each
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
- **Rounds.** Three rounds; in each, the release before, release after,
  dev before and dev after addons in turn, each running the three games.
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
  PNGs (sha prefixes 6777e6f85d015978 e906b85052f87437 8428129f8c263066;
  4dbea6fbbeeb79e7 ec1f49dfb70e2366 2ef72fec2dbba2b7; d5f3df40426b317f
  c469db61cd326c9b 26ea1def30738eb8), and release ends on the same hashes
  as dev. `og-gfx --all` links none of the crates this branch changed
  (it builds on og-render, og-core and og-tilemap, which are main's
  here byte for byte); its run here gave RMSE 0 against the committed
  references but for terrain-near 0.0066, water-planar 0.0049,
  terrain-grass 0.0050, terrain-mid 0.0036, water-shadows 0.0017,
  water-3d 0.0006, water-underwater 0.0004, fx-2d 0.00004 and
  materials-array 0.00001, all within their tolerance and all main's own
  (no input of theirs differs from main).

## Summary

The release figures at a load average of 20 to 25 (the per-game files
have dev and the rest):

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 1.27 → 1.09 | 3.71 → 2.85 | 1.17 → 1.14 | 3.21 → 2.56 |
| The Kestrel Stair | 6.59 → 3.53 | 17.20 → 17.66 | 2.02 → 1.87 | 3.91 → 3.23 |
| The Kestrel Stair, `OG_LANES=off` | 5.39 → 2.09 | 10.48 → 4.45 | 1.66 → 1.69 | 4.45 → 4.39 |
| Lantern Vale | 1.52 → 1.28 | 3.74 → 3.08 | 3.35 → 5.53 | 13.97 → 16.41 |

and dev:

| Game | step median | step p95 | render median | render p95 |
|---|---|---|---|---|
| Hollowbell | 4.95 → 3.80 | 9.86 → 9.62 | 4.45 → 3.12 | 7.75 → 6.61 |
| The Kestrel Stair | 14.42 → 8.46 | 33.48 → 23.33 | 3.95 → 3.94 | 7.19 → 6.63 |
| The Kestrel Stair, `OG_LANES=off` | 10.29 → 6.58 | 17.64 → 12.52 | 3.31 → 3.46 | 7.12 → 8.44 |
| Lantern Vale | 5.67 → 4.12 | 14.85 → 9.33 | 4.78 → 3.88 | 9.53 → 8.43 |

Against the targets (release step p95 and render under 8 ms, dev step
p95 under 16 ms): on this loaded machine Hollowbell and Lantern Vale meet
the step targets on both addons; The Kestrel Stair meets them with the
lanes off (release 4.45, dev 12.52) and misses them with the lanes on
(17.66, 23.33), where the cloth's jobs wait for pool threads the machine
does not schedule (kestrel-stair.md); Lantern Vale's render, which is the
GPU's time (the encoder's finish waits on Metal's command buffers), was
3.3 to 7.5 ms median and 8 to 16 ms p95 on a GPU shared with a simulator,
and its look's bloom is most of it (lantern-vale.md).

## Owed

- The same tables and og polish's frame-time rows on an idle machine
  (load under 8), release and dev, before and after.
- The lanes on a loaded machine: a soft-body or physics job sent to the
  pool waits for threads the OS does not run (The Kestrel Stair's
  og.physics 0.93 ms a tick serial against 2.9 to 4.2 ms on the pool at a
  load of 20 to 25). The grains were measured idle
  (tests/evidence/P4/physics-3/); og-lanes has no notion of a pool that
  is slower than the calling thread right now.
- Release extraction of sprites reads each field by a binary search over
  the layout's names (Layout::slot, memcmp most of it): 7 % of
  Hollowbell's release frame. Resolving a record's field slots once per
  frame would take it to the reads alone.
- og.ui.end's layout measures every text again each tick (widest_line,
  break_lines: about a third of og.ui.end, 0.07-0.16 ms release, 0.35-0.8
  dev).
