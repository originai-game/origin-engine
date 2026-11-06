# Dev-profile opt-levels for the crates the tests spend their time in (2026-11-06)

macrunner (M-series, Metal), lane-22, base 513fb9bfb. The machine was shared,
so every before/after pair below ran back to back on the same load and is
quoted in CPU user time where the wall clock was noisy. Load averages are
given where they were read.

## What changed

- `[profile.dev.package.og-ui] opt-level = 1`, and the same for `og-runtime`
  and `og-cook` (the og-nav pattern; their `codegen-units = 16` stays).
- `[profile.dev.package.{serde_json,itoa,zmij,memchr}] opt-level = 3`: the
  engine API's arguments and answers are JSON text, and at -O0 serde_json
  was 3964 of 6911 samples of a UI kit frame (`sample` of `og test
  templates/ui-kit --filter tests/patterns.og.ts` with `OG_RENDERER=none`).
- `og_ui_frame_viewport` keeps its last answer with the viewport and HUD
  aspect it was made from (as `og_ui_frame_theme` does): the kit asks for it
  many times a tick, and it was 583 samples of a frame; 87 after.

## The UI kit's components playtest, `OG_RENDERER=none` (`og test templates/ui-kit --filter tests/components.og.ts`)

| Step | user CPU | wall |
|---|---|---|
| A: main (everything -O0 but the existing per-package lines) | 77.6 s | 81.3 s |
| B: + serde_json and its formatters at O3 | 61.4 s | 65.5 s |
| C: + viewport memo + og-cook O1 (og-ui still O0) | 59.9 s | 115.6 s (load spike) |
| D: + og-ui O1 | 45.7 s | 51.8 s |
| E: + og-runtime O1 (the change) | 39.4 s | 42.1 s |

The patterns playtest, the kit's slowest: 446 s wall (419 s user) at A,
155–170 s at D.

## og-cook

`cargo nextest run -p og-cook -E 'not test(/render|draw/)'`: summed test
time 394 s before, 337 s after. `template::every_thread_count_cooks_the_same_bytes`
93 → 72 s in that run, 108 → 65 s in a separate pair. The cooked output at
O0 and O1 is the same bytes (`diff -r` of the template's `cooked/`, without
`.cook-times.ogt`).

## The test-quick step

`cargo nextest run --workspace --exclude og-tests` without the GPU crates
(`-E 'not (package(og-tests-cooked)|package(og-ui)|package(og-render))'`,
1552 tests, no GPU slot needed so both runs got the same machine):

| | wall | user CPU | load |
|---|---|---|---|
| before | 165.8 s | 892 s | ~33 |
| after | 105.9 s | 772 s | 17–37 |

The whole test-quick after the change: 311 s wall, 2293 passed, at load ~46
(the before run of the whole step never got a GPU slot in the window, so it
has no pair).

## `og polish` (GPU, topdown)

273 s wall at A, 151 s after, under a noisy load: read it as a direction,
not a number.

## Cost

An edit to og-ui or og-cook that rebuilds the native addon builds about 9 s
longer (og-cook 5–9 → 15–20 s, og-ui 9–10 → 18–20 s; og-runtime 13–22 →
11–28 s, within the noise). serde_json and the rest are dependencies: one
rebuild, not the loop.

## Tried and dropped

og-runtime's argument parse through `str::parse::<Value>()` instead of
`from_str`: 170 s against 155 s on the patterns playtest, no gain; reverted.
