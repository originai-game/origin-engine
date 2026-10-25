# Idle GPU measurements, 2026-10-25

Source remains `1bb341e51b22bdcdaf69a3eb60aac5265366196f` (0.4.0),
not the later candidate or atlas branches. No engine source or budget was
changed. Mac M4 Pro, 12 cores, macOS 27.2; Chrome for Testing 150.
The native capture and web packages were rebuilt in release at this source.

After explicit owner authorization, confirmed EchoPlay PID 81366 and
SimMetalHost PID 95801 with `ps -o pid,args -p`, then stopped exactly those
two. SimMetalHost respawned; the authorized `xcrun simctl shutdown all`
removed it. No other owner process was stopped.

`idle-check.sh` holds both gate locks. Before each accepted batch it waits
for the **one-minute load average to stay below 8 for a full minute**, sampled
every five seconds. During each batch it samples every second and rejects
the entire batch if any sample reaches 8. Both pre-run and in-run samples,
exact command, source revision, exit code, and stdout/stderr are retained.
This is a load-qualified run, not proof that all background CPU/GPU work
has stopped. The five- and fifteen-minute averages can still reflect earlier
work; they are not the acceptance metric.

## Capture budgets are not met across these scenes

All images are 1280×720, decoded and dimension-checked outside the timed
region. Final native and web images were visually inspected: the game
template's Start scene and Meridian Loop's menu/track/city are rendered,
not empty frames. Thirty captures per scene/host follow 60 warm frames and
three discarded captures, with one new frame between captures. p50 and p95
are sorted samples at indexes 15 and 28. Load throughout the accepted
capture batch: **5.04–5.54**.

| Scene | Host | p50 ms | p95 ms | max ms | Target ms | p95 result | Hard-cap result |
|---|---|---:|---:|---:|---:|---|---|
| game template | native Metal | 25.861 | 26.369 | 31.535 | ≤30 | Pass | Fail |
| game template | Chrome WebGPU | 49.090 | 65.080 | 65.330 | ≤60 | Fail | Fail |
| Meridian Loop | native Metal | 97.934 | 102.639 | 104.435 | ≤30 | Fail | Fail |
| Meridian Loop | Chrome WebGPU | 148.335 | 150.620 | 165.060 | ≤60 | Fail | Fail |

Design §7 does not specify a percentile. Both the requested percentiles and
the maximum are reported, rather than treating a passing p95 as proof of a
hard cap. The capture row is **Fail**, even when judged by p95.

Native measures the synchronous public `engine.probe.capture(1280,720)`
call, including extraction, render/readback, PNG, base64 and ABI parsing.
Web measures the awaited `shell.probe("capture", {width:1280,height:720})`
through the normal Worker host, including its asynchronous readback and
transport. Every web sample returned on the first outer call. Web uses
the non-publish module, as the diagnostic probe path does; this is not a
publish-only download-size or first-frame test.

The real `og run --commands` screenshot path was also exercised: 33
screenshots at simulation seconds 2–34, discarding the first three. A Bun
preload wraps the unchanged native frame ABI and records wall time. The
command pumps at the pre-step tick; its frame answer is tick + 1. Every
PNG is validated; only the last is retained. Startup/cook and file writes
are outside this timed span.

| Scene | Screenshot-bearing frame wall p50/p95/max ms | Capture residual p50/p95/max ms |
|---|---|---|
| game template | 24.644 / 25.053 / 25.054 | 23.816 / 24.255 / 24.280 |
| Meridian Loop | 138.501 / 145.618 / 146.241 | 136.905 / 143.399 / 144.338 |

Frame wall includes one ordinary tick and normal render in addition to the
screenshot. The residual subtracts reported step/render time; it estimates
the internal capture/PNG span plus ABI overhead, not a separately timed
function. These later simulation times differ from the direct-capture
sample window, so differences between the two methods are not an ABI-cost
comparison. Meridian's capture path is expensive relative to its ordinary
step/render work; no profile was taken to separate GPU readback from PNG
compression, so neither is assigned the blame here.

The first `capture/` batch is retained as **provisional**, not used for the
verdict above: it lacked the later full-minute idle precondition. Its
Meridian native/web p95 were 107.831/167.350 ms, at recorded one-minute loads
6.24–6.60 and 6.06–6.42 respectively. Those recorded values were below 8,
although the machine's subsequent Storage/Chrome activity invalidated the
first two full comparison batches. The accepted repeat resolves that
uncertainty without selecting the fastest samples.

## Three.js: WebGPU meets the reference target, WebGL2 does not

The unchanged `tools/bench/threejs-compare.ts` and pinned three.js 0.180.0
ran 600 measured frames after 60 warm frames, two rounds per mode. The
first two all-mode batches completed but exceeded load 8; their JSON and
logs are retained in `threejs-loaded*` and `threejs-idle*` and excluded.

For the accepted run, `threejs-idle.sh` invokes the same runner separately
for each scene/mode, with a full idle minute before each. This avoids
accumulating process/shader-startup load across all nine modes. The two
rounds of each mode are adjacent, rather than interleaved with other modes;
that method change and every round's number are recorded. All nine modes
passed the load check: **2.97–6.38** across the accepted batches.

| Scene | Mode | p50 ms | p95 ms | Sustained ms | FPS |
|---|---|---:|---:|---:|---:|
| reference | engine WebGPU | 1.078 | 2.360 | 1.310 | 763 |
| reference | engine WebGL2 | 2.195 | 3.263 | 2.205 | 454 |
| reference | three WebGL2 instanced | 3.008 | 3.683 | 2.538 | 394 |
| reference | three WebGL2 one Mesh each | 2.353 | 4.735 | 3.051 | 328 |
| 20,000 spheres | engine WebGPU | 3.670 | 4.158 | 3.608 | 277 |
| 20,000 spheres | engine WebGL2 | 3.400 | 4.928 | 3.725 | 268 |
| 20,000 spheres | three WebGL2 instanced | 2.620 | 2.920 | 2.586 | 387 |
| 20,000 spheres | three WebGL2 one Mesh each | 10.933 | 11.518 | 10.879 | 92 |
| 20,000 spheres | three WebGPU instanced | 1.288 | 1.590 | 1.316 | 760 |

Against the instanced reference comparator, the frame-rate ratios are
**1.937× WebGPU (Pass)** and **1.151× WebGL2 (Fail)** against ≥1.5×.
The existing reference harness has a three.js WebGL2 comparator, not a
three.js WebGPU reference renderer. Against its one-Mesh-per-object form,
the ratios are 2.329× and 1.384×. The engine and three.js scene rebuilds
retain the harness's different AA/shadow implementations; screenshots show
the objects and skinned figures, not identical pixel output.

The sphere field has no separate numeric §7 budget. Its same-tier instanced
ratios are **0.365× WebGPU** and **0.694× WebGL2**: the engine remains slower
than hand-instanced three.js, but beats the one-Mesh-per-object WebGL2 form
by 3.015×/2.921×. This is a completed comparison, not a new pass threshold.
Both engine sphere images were inspected and show the dense sphere field.

Times retain the existing harness semantics: engine Worker frame round
trips versus three.js requestAnimationFrame intervals with the frame-rate
limit disabled; every frame advances one simulation step. Sustained time
is total wall time per frame, not GPU timestamp time. Back-to-back submission
can be throttled by GPU queues. No phase profile or causal speedup over the
old, differently loaded pre-lean run is claimed.

## Perf-games: one release render target misses, all dev step targets pass

`perf-games.mjs` repeats the three W12 scripts from
`tests/evidence/P4/perf-games/README.md`, five rounds per profile, interleaving
games and unpaced/paced runs. `--headless` is unpaced; omitting it is paced.
Each invocation requests `--ticks 600 --size 1280x720 --json`. The existing
CLI excludes the first five frames from its timing distributions. Table
cells are medians across five runs of each run's reported percentile, not
a pooled percentile over all frames. Full per-system means and raw runs
are retained. No historical W12 before-build was rebuilt: all source is
the agreed current revision, so these tables do not claim a before/after
speedup for that old optimization stream.

Release load was **4.46–6.45** throughout all play runs and polish;
dev load was **3.71–5.60**. Profiles were built separately before their idle
waits, not while timing. The W12 targets are release step/render p95 under
8 ms and dev step p95 under 16 ms.

| Profile | Game | Pacing | Step median / p95 ms | Render median / p95 ms | Result |
|---|---|---|---|---|---|
| release | Hollowbell | unpaced | 0.463 / 0.869 | 0.537 / 0.723 | Pass |
| release | Hollowbell | paced | 1.270 / 2.476 | 1.230 / 2.092 | Pass |
| release | Kestrel Stair | unpaced | 0.809 / 1.167 | 1.218 / 1.816 | Pass |
| release | Kestrel Stair | paced | 1.662 / 2.806 | 1.461 / 2.113 | Pass |
| release | Lantern Vale | unpaced | 0.538 / 0.976 | 1.571 / 8.030 | **Fail** |
| release | Lantern Vale | paced | 1.199 / 2.169 | 1.846 / 2.810 | Pass |
| dev | Hollowbell | unpaced | 2.022 / 3.144 | 1.889 / 2.238 | Pass |
| dev | Hollowbell | paced | 2.906 / 3.776 | 2.635 / 2.930 | Pass |
| dev | Kestrel Stair | unpaced | 3.084 / 3.896 | 2.075 / 2.469 | Pass |
| dev | Kestrel Stair | paced | 3.657 / 4.600 | 2.485 / 2.881 | Pass |
| dev | Lantern Vale | unpaced | 2.317 / 3.352 | 2.300 / 2.691 | Pass |
| dev | Lantern Vale | paced | 2.795 / 3.714 | 2.752 / 3.113 | Pass |

Lantern's unpaced release render p95s were 7.520, 8.030, 7.894, 8.358,
and 8.143 ms: three of five exceed 8, and the median misses by 0.030 ms.
It is not rounded down into a pass. Unpaced rendering includes waiting on
Metal command buffers when the engine submits faster than the GPU drains;
its paced p95 is 2.810 ms. These runs do not isolate which GPU passes or
driver work account for the wait.

The script reached tick 600 in 59 of 60 runs. One paced dev Kestrel run
(round index 2) reached 601: the CLI stops after a frame whose tick is
`>= --ticks`, and a paced frame can take multiple simulation ticks. It is
retained, not silently discarded or called exactly 600 ticks. Every tick-600
run has the same whole-world hash for its game across profiles, rounds,
and pacing: Hollowbell `5c33b0733b9a10a3`, Kestrel `13596d3b7a682ef9`,
Lantern `780bf264aa0a3f1f`. The tick-601 Kestrel hash is
`aa88916a5f3c58ce`.

The per-system clocks identify the largest measured tick work, not render
or sampling-stack costs. On unpaced release, the largest mean is Ui for
Hollowbell (0.106 ms) and Lantern (0.128 ms), and og.physics for Kestrel
(0.238 ms). On dev, Ui/og.ui.end and per-entity game systems dominate the
listed means. No new stack profile or pool-on/off experiment was run.

### Polish retains its existing scoring thresholds

Every playtest ran on Metal; there are no missing-run findings. These are
the tool's unmodified scores. All worst-playtest frame p95s are below its
16.7 ms finding threshold (**Pass**); the 100-point score is reached at
8.3 ms, and the score falls to zero at 33.3 ms.

| Profile | Game | Frame score | Feel | Worst frame p95 ms | Step p95 median (worst) ms | Render p95 worst ms |
|---|---|---:|---:|---:|---:|---:|
| release | Hollowbell | 100 | 97 | 2.24 | 1.23 (1.47) | 0.83 |
| release | Kestrel Stair | 100 | 99 | 3.01 | 1.17 (1.32) | 2.15 |
| release | Lantern Vale | 100 | 99 | 2.88 | 1.47 (1.67) | 1.68 |
| dev | Hollowbell | 93 | 96 | 9.95 | 6.16 (7.52) | 2.67 |
| dev | Kestrel Stair | 100 | 99 | 7.68 | 4.42 (5.01) | 2.87 |
| dev | Lantern Vale | 82 | 96 | 12.91 | 7.77 (9.86) | 3.04 |

`bun tests/evidence/r4/measure/summarize-gpu.mjs` validates accepted load
windows, sample counts, actual native/web tiers, completed play workloads,
and rendered polish runs, then regenerates `gpu-summary.json`. Each run
directory also retains its command and original tool report. Neither budgets
nor existing performance baselines were changed.
