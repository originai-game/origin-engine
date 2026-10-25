# Round 4 measurements on the owner's Mac

Source: `1bb341e51b22bdcdaf69a3eb60aac5265366196f`, version 0.4.0.
Apple M4 Pro, 12 cores, macOS 27.2 arm64, Chrome for Testing 150.
The persistent `origin-engine-measure` worktree holds the measurements.
Local main's five later commits were preserved on `r4/mac-pending`, not
included here. No budget, baseline, or engine source changed.

## Acceptance recorded so far

The coordinator allowed rows 52, 88, and Android measurements under recorded
load on 2026-10-25. EchoPlay and SimMetalHost were still running for that
first batch. They were subsequently stopped with owner authorization;
`GPU.md` records the later load-qualified capture, comparison and perf-games
runs, with a full minute below load 8 before each accepted batch.

| Requested row | Result | Measurement and limit |
|---|---|---|
| Probe capture, native and web | Fail | Meridian p95 102.639 ms native / 150.620 ms web versus 30/60 ms; template p95 26.369/65.080 ms. Native template also has a 31.535 ms maximum. See `GPU.md` for full distributions |
| 51, three.js comparison | Pass WebGPU; fail WebGL2 reference target | 1.937× / 1.151× against the existing three.js WebGL2 instanced reference comparator, target ≥1.5×. Sphere-field same-tier instanced ratios 0.365× / 0.694×, no separate numeric sphere budget |
| 89, perf-games and polish | Fail one release target; pass remaining play targets | Lantern unpaced release render p95 8.030 ms versus <8 ms. Every dev step p95 <16 ms. Polish frame scores release 100/100/100, dev 93/100/82; all worst frame p95 <16.7 ms |
| 88, terrain first-frame settle | Pass, measurement and simulation isolation | Five fresh runs per tier. Five tiles settle at tick 0, no simulation tick or world-hash change. No separate numeric settle budget is specified |
| 52, Chrome whole-world hashes and physics stats | Pass | 30 runs, 1,000 and 2,000 boxes, three modes, five rounds. Whole-world hashes agree at ticks 60, 360, and 660 |
| Android first frame | Pass on this device; specified hardware/network not-measurable | Pixel 10, USB localhost: median 1,677 ms, range 1,559–1,836 ms, below 2,000 ms. This is not a 2022 mid-range Android on 4G |
| Android 10,000-entity TS system | Pass on this device; specified hardware not-measurable | Move system median 0.111 ms per tick, range 0.110–0.117, below 1 ms. Whole engine tick 0.523 ms, range 0.518–0.552 |
| iPhone first frame and frame times | Not-measurable yet | Last preflight: physical iPhone Air offline in Xcode, Mac Safari Remote Automation disabled. Owner action remains pending; no iPhone 13 result |

`summary.json` contains the calculated tables. `bun
tests/evidence/r4/measure/summarize.mjs` regenerates them and checks the
world hashes, physics pool usage, terrain residency, tick counts, and
Android entity movement. `summarize-gpu.mjs` generates the later
`gpu-summary.json`, including load qualification. Unsuccessful and
load-disqualified attempts are retained and labeled, not counted as passes.

## Terrain cost is outside the tick

`web.mjs terrain` exports fx-showcase with the non-publish wasm, starts
its normal Worker, selects `terrain_stream:stream:0.25` and the terrain
station through commands, then calls `frame(0)` at tick 0. The first
terrain render decodes and installs five tiles. Its before/after whole-world
hash is identical, and the answer reports `ticks: 0`, `stepMs: 0`.
The following tick's `dump_stream` runs before its render and reads that
first render's tile statistics. A capture comes after the dump, so it
cannot hide missing tiles by settling them itself.

| Chrome tier | First frame wall ms, median (range) | Render ms, median (range) | Tile settle ms, median (range) | 1-minute load range |
|---|---:|---:|---:|---:|
| WebGPU | 8.320 (7.945–16.820) | 7.850 (7.370–16.205) | 4.220 (3.925–8.150) | 8.137–8.767 |
| WebGL2 | 107.850 (102.810–119.930) | 104.610 (100.355–116.440) | 4.105 (3.875–4.630) | 7.852–8.264 |

Tile settle is `TerrainStreamer::update`'s `worstFrameMs`, measured by the
engine around decoding, cache work, and installation into the render scene.
It excludes subsequent GPU rendering. The first render also includes terrain
extraction and renderer setup; the large WebGL2 first-frame remainder is
not simulation time. No profile was taken to divide that remainder further.
These five-run ranges are startup measurements, not steady-state p95s.

Both retained images were inspected: a lake, sandy shore, green valley,
mountain, and foreground path are visible without missing terrain tiles.
Only one representative capture per tier is kept; every run's JSON remains.

```sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" bun tests/evidence/r4/measure/web.mjs terrain templates/fx-showcase tests/evidence/r4/measure/terrain-tick0-webgpu-1
# Repeat with fresh processes for runs 2–5, then with --webgl for WebGL2.
```

## Chrome hashes include the whole world

The shipping publish module strips `probe.dump`, which `Engine.hash()`
uses. Rather than substitute pixel hashes or the network-only shared hash,
`lanes-build.sh` builds a non-publish diagnostic lanes module from the same
source, with both physics profiles like the normal non-publish web module.
It uses the existing wasm profile and persistent `target/wasm-mt`, retains
the probe, and does not replace the packaged publish variants. These are
diagnostic-module stats, not a new shipped-build throughput comparison.

`lanes.mjs` uses the existing `lanes-pile-scene.ts` and `lanes.page.ts`.
For each size it runs non-isolated, isolated with lanes off, and isolated
with lanes on, interleaved for five rounds. Each fresh world renders 660
frames. At ticks 60, 360, and 660 it reads `Engine.hash()` and a game facts
callback that calls `og.physics.stats(world)` directly. Stats describe the
just-completed step, numbered 59, 359, and 659.

| Boxes | Mode | Final physics step ms, median (range) | Threads | Parallel calls | Final whole-world hash |
|---|---|---:|---:|---:|---|
| 1,000 | Non-isolated | 4.800 (4.200–6.900) | 1 | 0 | `045cf4ed2a89cc1b` |
| 1,000 | Isolated, lanes off | 4.415 (4.200–4.735) | 1 | 0 | `045cf4ed2a89cc1b` |
| 1,000 | Isolated, lanes on | 4.705 (4.055–5.010) | 8 | 0 | `045cf4ed2a89cc1b` |
| 2,000 | Non-isolated | 10.600 (10.400–11.900) | 1 | 0 | `c3220b16926c6ace` |
| 2,000 | Isolated, lanes off | 10.520 (8.815–10.595) | 1 | 0 | `c3220b16926c6ace` |
| 2,000 | Isolated, lanes on | 6.440 (6.190–6.630) | 8 | 1,921 | `c3220b16926c6ace` |

One-minute load ranged from 6.672 to 9.210. At 1,000 bodies the eight-thread
configuration stays below the 1,500-body parallel threshold; `pool` is false.
At 2,000 bodies direct stats report `pool: true`, with `rapier` and `readout`
split, in every lanes-on sample. Rapier is the largest reported phase.
The single-step table is not a claim about sustained speedup or p95 latency.

```sh
OG_RELEASE=1 sh packages/engine/scripts/build.sh
bash tests/evidence/r4/measure/lanes-build.sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" bun tests/evidence/r4/measure/lanes.mjs
```

## Android measurements use the Pixel 10, not the budget's reference phone

Android 17, Chrome 155.0.8059.87, WebGPU on the Imagination PowerVR adapter.
The template's five device runs use the existing CLI, which measures
navigation to first frame, not the later play-gate tap. USB reverse serves
localhost without a 4G network profile. The largest observed first-frame
time is 1,836 ms, but the hardware, network, and timing origin differ from
the exact design acceptance conditions.

```sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" packages/engine/native/og.darwin-arm64 run templates/game --device android:58051FDCR007QL --seconds 5 --out tests/evidence/r4/measure/android/first-frame-1 --json
# Repeat in fresh device runs for rounds 2–5.
bun tests/evidence/r4/measure/android-tick.mjs
```

First-frame times were 1,559, 1,836, 1,677, 1,610, and 1,691 ms. The
Mac's before/after load samples range from 5.34 to 8.04. Every device run
reports WebGPU and zero findings. Its JSON retains display intervals,
engine work, adapter, viewport, isolation, and response headers.

The tick runner uses the existing hello-world Move system and Worker,
10,000 entities in one archetype, reading Velocity and writing Transform.
Health is also attached but not queried by Move. Each of five fresh pages
warms 600 ticks and then times 2,000 ticks in one awaited `advance` call.
Rendering is outside the timed region. The whole engine tick includes
all built-in systems and the Worker round trip amortized over 2,000 ticks.
Move's own time comes from differences in its cumulative system clock.

Every run advances from tick 600 to 2,600, has 10,000 entities, and ends at
whole-world hash `47bec675f12794a5`. The inspected entity's z moves from
−10.000153 to 23.333017, within 0.001 of the independent expectation
2,000 / 60 = 33.333333 units. Mac load was 7.827–8.078; phone CPU load
was not measured. No speedup over another device or build is claimed.

An initial attempt used the full existing `tests/web/page.ts` browser check.
Its CPU work completed, but the later unrelated `renderFrame` check returned
zero bytes rather than 921,600. The failed receipt is retained in
`android/tick/original-browser-check-failure.json` and `android/tick.log`.
The dedicated tick runner does not perform or claim that readback check.

Phone setup and remaining owner actions are in `phone.md`. The CLI's
`--preflight` exit 2 is recorded as a defect, not fixed in this branch.
