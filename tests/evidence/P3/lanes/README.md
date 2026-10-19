# Lanes: frame times with lanes off and on (2026-10-19)

The design's §7 reference scene (`crates/og-tests/reference-scene`:
500 instanced objects, 2 shadowed lights, 20 skinned characters, PBR, TAA
and post) measured with lanes off and on, natively on Metal and in Chrome
over WebGPU with and without cross-origin isolation. A heavier crowd
(`--crowd`: the reference scene plus 2000 boxes, 160 walkers and 48 falling
crates, `packages/engine/tests/support/lanes-crowd.ts`) shows what lanes do
once work crosses the grains; it runs natively only.

## Commands

From `packages/engine`, after a release build of the addon and both wasm
modules (the bench refuses a non-release native build):

```sh
OG_RELEASE=1 sh scripts/build.sh
bun bench/lanes.ts --native --frames 600           # native.json
bun bench/lanes.ts --native --crowd --frames 600   # native-crowd.json
bun bench/lanes.ts --chrome --frames 600           # chrome.json, chrome-*.png
```

Each run warms 60 frames, then times 600 at a fixed 1/60 s step and
1280x720. Natively a frame is `engine.frame` plus the GPU wait
(`og_node_wait_gpu`) and `cpu` is `engine.frame` alone; each mode is its own
process (`OG_LANES=off`, `OG_LANES` unset = cores up to 8). In Chrome a frame
is the round trip of the shell's `frame` call to the engine Worker; `step` and
`render` are the engine's own timings from the probe's state. Chrome modes:
a page without COOP/COEP; an isolated page started with `?lanes=0` (the page
offers no lanes variant, so the shell reports "the build has no lanes
variant"); an isolated page that offers it.

## Machine and load

Apple M4 Pro (12 cores), macOS 27.2, Chrome stable, release build. The host
was shared with other agents' builds; `load.txt` has `uptime` before and
after each command (1-minute load 27 to 35 for the native runs, rising to
110 during the Chrome run).

## Results (ms)

Native, Metal, reference scene:

| lanes | threads | parallel calls | p50 | p95 | p99 | max | cpu p50 | world hash |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| off | 1 | 0 | 6.968 | 14.668 | 17.598 | 19.665 | 1.417 | bda28bdc44a50796 |
| on | 8 | 660 | 6.475 | 9.031 | 10.329 | 20.637 | 1.276 | bda28bdc44a50796 |

Native, Metal, crowd:

| lanes | threads | parallel calls | p50 | p95 | p99 | max | cpu p50 | world hash |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| off | 1 | 0 | 6.063 | 9.528 | 10.413 | 14.547 | 2.848 | a5735ce013e90817 |
| on | 8 | 5940 | 5.808 | 6.717 | 7.831 | 12.819 | 2.755 | a5735ce013e90817 |

Chrome, WebGPU, reference scene (publish build):

| page | lanes | pool | parallel calls | p50 | p95 | p99 | max | mean | step p50 | render p50 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| not isolated | off | none | 0 | 1.2 | 2.3 | 3.1 | 33.1 | 1.358 | 0.2 | 0.8 |
| isolated, `?lanes=0` | off | none | 0 | 1.19 | 2.235 | 3.225 | 5.005 | 1.339 | 0.2 | 0.87 |
| isolated | on, 8 | web-workers | 0 | 1.265 | 2.305 | 3.16 | 4.555 | 1.413 | 0.225 | 0.88 |

Identity: both native pairs give the same world hash with lanes off and on
(the bench fails otherwise). Publish builds report no world hash, so the
three Chrome runs compare the RGBA of the frame captured at tick 660: sha256
`29ec2115e792…` in all three (`chrome-*.png`). `packages/engine/tests/lanes.test.ts`
covers per-tick hashes, pixels and replays.

## Verdict on §7

The 16.7 ms budget held at p99 in every mode in these runs. The worst p99
is 17.598 ms natively with lanes off, above the budget by 0.9 ms; with lanes
on the same scene's p99 is 10.3 ms. The native tail is GPU-wait under host
load, not CPU (cpu p99 3.8 ms off, 2.3 ms on). Earlier runs of the same build
at 1-minute load 75 to 220 had native p95/p99 of 23 to 59 ms in both modes,
so the result depends on the host being otherwise quiet; a dedicated machine
should repeat it before the budget is called held.

In Chrome lanes change nothing measurable for the reference scene: no work
crosses the grains, and rapier's step is serial on the web (its parallel
feature is native only), so the isolated lanes page runs the same serial
code on the lanes module, about 0.07 ms slower a frame (the atomics build).
The Chrome crowd has not been measured.

The Three.js comparison of §7 is an owner measurement and was not done.

## Why the grains are where they are

A per-system trace of the reference scene (serial, release) measured:
animation evaluation of 20 characters 21 µs, their palettes 12 µs, extraction
about 0.6 µs a row, culling of 501 items 4 µs. Waking the rayon pool costs
tens of µs on an idle machine but 0.5 to 2.6 ms a call under the load above
(a hop microbench: an 8-thread `install` p50 0.5 to 0.9 ms). So og-anim splits
from 128 skeletons (grain 64), extraction from 1024 rows (grain 512) and
culling from 2048 bounds (grain 1024): the reference scene stays serial in
all three, and only the physics step (rapier's own rayon, native) runs
parallel there. The off path's `install` runs inline on a one-thread pool
built with `use_current_thread` (hop p50 1 µs, against 110 µs and a 3.5 ms
p95 for the earlier separate one-thread pool).
