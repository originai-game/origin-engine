# Remaining candidate timings

Measured `7ca4e25178cde62abcb797a8c726dfda8434bffa`, the same engine source
as `70c120c0`. The coordinator authorised continuing after the WebGL2
comparison missed its target. No engine source, reference frame, or baseline
changed. Mac main remained at `308f7ff2`.

All runs completed. Each batch used `../idle-run.sh`, with the machine and
target locks, a full minute of one-minute load below 8 before starting, and
one-second load samples throughout. No other build or gate ran alongside.
`command.txt`, `head.txt`, `status.txt`, and load TSVs retain the commands and
qualification. `../summarize-outstanding.mjs` checks receipt counts, errors,
loads, and game hashes, and regenerates `summary.json`.

## Lantern passes the release render target

Five rounds per game, interleaved in Hollowbell, Kestrel, Lantern order.
`../perf-unpaced.mjs` uses the earlier perf-games input scripts unchanged,
600 ticks, `--headless`, and 1280×720. The packaged native addon is release.
Each run reports 595 measured renders after warmup, 600 final ticks, Metal,
no warnings, and no unchecked commands. Each game's final hash agrees across
all five runs. Load was 3.15–3.62.

Each table entry is the median of five per-run percentiles, not a percentile
of pooled frames. All times are milliseconds.

| Game | Step p50 | Step p95 | Render p50 | Render p95 | Render p95 range |
|---|---:|---:|---:|---:|---:|
| Hollowbell | 0.451 | 0.827 | 0.482 | 0.625 | 0.611–0.666 |
| Kestrel Stair | 0.784 | 1.135 | 1.190 | 1.679 | 1.665–1.805 |
| Lantern Vale | 0.538 | 0.839 | 1.613 | 7.333 | 6.995–7.759 |

Lantern's render p95 is below 8 ms in every run: PASS. The median is 0.697 ms
below the earlier 8.03 ms receipt. These are sequential historical runs,
not an interleaved before-and-after isolation of perf-ui. The unpaced render
includes submission and GPU waiting; this run does not profile that split.

## Both Chrome tiers pass the device timing checks

Five fresh browser sessions per game and tier, interleaved by round. Each
uses `og run PROJECT --device chrome --seconds 10 --size 1280x720 --json`,
with `--webgl` for WebGL2. The command builds the publish page using the
shipping wasm profile, serves it locally, passes the play gate, discards
one second, and measures ten seconds of played frames in the Worker.
First-frame time starts at navigation, not at the play-gate tap.

Chrome for Testing 150 runs headless on the M4 Pro. Although the command
requests a 1280×720 window, the recorded canvas and viewport are
**1280×577**, DPR 1, in all 20 receipts. The historical P3 device receipts
have this same canvas size. These are localhost timings without network
throttling, not phone or 4G acceptance results. Each new browser session has
a fresh browser profile, but OS caches are not flushed between sessions.
Load was 3.43–5.69. All sessions report the requested tier, isolation, live
frames and ticks, zero console errors, and zero device findings.

| Game | Tier | First frame median (range), ms | Work p50, ms | Work p95 median (range), ms | Display p95 median, ms |
|---|---|---:|---:|---:|---:|
| Template | WebGPU | 310 (300–368) | 0.87 | 1.78 (1.76–2.05) | 8.34 |
| Template | WebGL2 | 505 (499–1107) | 1.14 | 2.35 (1.86–2.63) | 8.34 |
| Meridian | WebGPU | 332 (327–338) | 1.15 | 2.20 (2.11–2.98) | 8.34 |
| Meridian | WebGL2 | 695 (603–1482) | 1.44 | 2.38 (2.31–2.48) | 8.34 |

All 20 runs pass device.ts's existing 2000 ms first-frame, 16.7 ms work p95,
and 20 ms display p95 checks. Display cadence is capped near 120 Hz, so its
p95 is not a measure of available rendering throughput.

**Retained outlier:** template WebGPU round 1 has a 1899.98 ms maximum
display gap and only 515 ticks in ten seconds, versus 599–601 ticks in the
other runs. Its maximum engine work is 2.58 ms. Its display p95 is 9.06 ms,
so the existing p95 checks do not flag the gap. The cause is unproven; this
is not evidence of a 1.9-second engine frame. No run was discarded.

### Historical deltas do not isolate size-2

No matched template-and-Meridian timing set immediately before size-2 was
found. E7's size experiment measured fx-showcase, ui-kit, and runner through
reference.ts, not these published pages. Its first frame times exclude
navigation and cannot be subtracted from the device first-frame numbers.

The older `tests/evidence/P3/device/{chrome,chrome-webgl2}/device.json`
receipts do measure the template on Chrome 150 at the same canvas size,
but only once, for five seconds, six days before these five-run medians.

| Template metric | P3 WebGPU | Current WebGPU | Delta | P3 WebGL2 | Current WebGL2 | Delta |
|---|---:|---:|---:|---:|---:|---:|
| First frame, ms | 270 | 310 | +40 | 1125 | 505 | -620 |
| Work p50, ms | 0.54 | 0.87 | +0.33 | 1.13 | 1.14 | +0.01 |
| Work p95, ms | 0.73 | 1.78 | +1.05 | 1.41 | 2.35 | +0.94 |
| Display p95, ms | 9.82 | 8.34 | -1.48 | 37.27 | 8.34 | -28.93 |

Engine-work p95 is higher than the historical receipt. This comparison
cannot attribute the difference to opt-level s: engine and game changes
also separate the revisions. There is no comparable Meridian receipt.
An isolated s-versus-3 regression verdict is NOT-MEASURABLE from these
receipts. No profile setting was changed. The earlier candidate's complete
browser-reference comparison passed on both tiers; these timing runs do
not add a new image comparison.

## The sphere baseline failure reproduces when idle

Three serial invocations, each after its own idle minute:

```sh
RUST_TEST_THREADS=1 cargo test -p og-runtime --release --test extract_cost -- --ignored perf_extract_spheres_20k --nocapture
```

| Run | Best frame, ms | Load during measurement | Result |
|---|---:|---:|---|
| 1 | 0.799 | 4.32 | FAIL baseline limit |
| 2 | 0.785 | 3.28 | FAIL baseline limit |
| 3 | 0.783 | 3.73 | FAIL baseline limit |

Median 0.785 ms, range 0.783–0.799 ms. This is 47.8% above the unchanged
0.531 ms baseline and above its 15% tolerance limit, 0.61065 ms, in every
run. The spread is 0.016 ms, far smaller than the 0.254 ms median excess.
The earlier 0.798 ms result reproduces; this is a persistent baseline
regression, not the earlier concurrent-test noise.

Each measurement completed and asserted that the renderer received all
20,000 instances, then failed only the budget assertion, with exit 101.
No build failure, timeout, or incomplete measurement occurred. The test
times the best of seven batches of 50 frames after three warm frames. Its
counting renderer does no GPU work. It includes the tick, extraction, and
frame-result JSON handling, so the name does not mean pure extraction.
The baseline dates to `ef3ee893`, 2026-10-21. These repeats establish the
failure, not its responsible commit or CPU hot path. `OG_PERF_RECORD` was
not set, and the baseline file is unchanged.
