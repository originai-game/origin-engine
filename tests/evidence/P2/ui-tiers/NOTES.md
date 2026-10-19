# Phase 2, og-ui tiers round: evidence (owed row 20)

Branch `p2-ui-tiers` from main 44d12a8, 2026-10-19, Apple M4 Pro (shared
with other agents' builds and gates), Chrome for Testing 150.

| File | What it shows |
|---|---|
| `four-tiers.png` | from the committed references, columns metal, vulkan-cpu, webgpu, webgl2; rows: the "Settings" title of `kit-desktop-settings` at 2x (the largest gap to metal), `glass` (identical on all four), `kit-locale-mainMenu-ar` at 1/4, `widget-set` at 1/4, the first cell of `kit-skin-neon` at 1/2, `kit-phone-shop` at 1/6 |
| `break-and-restore.md` | each new tier broken once with a wrong reference and seen to fail, then restored |

## What exists now

`crates/og-ui/references/<tier>/` holds the same 74 frames for `metal`,
`vulkan-cpu`, `webgpu` and `webgl2`:

| Group | Frames | Native harness | Browser harness |
|---|---|---|---|
| og-ui scenes: sample-screen, text-glow, glass, widget-set | 4 | `cargo test -p og-ui --test gfx --test kit_gfx` | `tests/web/reference-gfx.ts` (scenes `og-ui/<name>` in og-tests-web) |
| ui-kit desktop screens (`frames-desktop` 21, `players` 1) | 22 | `og test templates/ui-kit` | `tests/web/reference-ui.ts` |
| ui-kit phone screens (`frames-phone`) | 21 | same | same |
| locale screens (mainMenu, pause, confirm in en, zh-CN, ja, ko, ar) | 15 | same | same |
| skin sheets (six packs, desktop and phone, 20 cells each) | 12 | same | same, cell by cell (`--sheet`) |

Nothing was left out: the browser runs the full set, since a tier takes
100 to 140 s.

## Tiers against metal

The rule: a channel off by more than 32/255 is an outlier; a frame fails
above 0.5 % outliers or RMSE 0.01. Figures are outliers / RMSE of the
committed reference against the metal one (skin sheets as whole sheets;
the tests judge each cell, and every cell passed against metal before the
tiers were recorded).

| Frame | vulkan-cpu | webgpu | webgl2 |
|---|---|---|---|
| glass | 0.000 % / 0.00000 | 0.000 % / 0.00000 | 0.000 % / 0.00000 |
| sample-screen | 0.000 % / 0.00062 | 0.000 % / 0.00000 | 0.000 % / 0.00000 |
| text-glow | 0.000 % / 0.00045 | 0.000 % / 0.00000 | 0.000 % / 0.00000 |
| widget-set | 0.000 % / 0.00071 | 0.000 % / 0.00046 | 0.000 % / 0.00046 |
| kit-desktop-settings | 0.038 % / 0.00890 | 0.038 % / 0.00881 | 0.038 % / 0.00881 |
| kit-phone-shop | 0.028 % / 0.00881 | 0.029 % / 0.00871 | 0.029 % / 0.00871 |
| kit-desktop-pause | 0.026 % / 0.00643 | 0.026 % / 0.00624 | 0.026 % / 0.00623 |
| kit-locale-pause-en | 0.026 % / 0.00643 | 0.026 % / 0.00624 | 0.026 % / 0.00623 |
| kit-desktop-puzzle | 0.027 % / 0.00531 | 0.027 % / 0.00530 | 0.027 % / 0.00530 |
| kit-desktop-levels | 0.024 % / 0.00541 | 0.024 % / 0.00528 | 0.024 % / 0.00528 |
| kit-desktop-toasts | 0.020 % / 0.00571 | 0.020 % / 0.00571 | 0.020 % / 0.00571 |
| kit-desktop-shop | 0.014 % / 0.00426 | 0.014 % / 0.00398 | 0.014 % / 0.00398 |
| kit-desktop-idle | 0.001 % / 0.00181 | 0.001 % / 0.00176 | 0.001 % / 0.00175 |
| kit-desktop-gallery | 0.004 % / 0.00125 | 0.004 % / 0.00119 | 0.004 % / 0.00119 |
| kit-phone-gallery | 0.019 % / 0.00284 | 0.019 % / 0.00273 | 0.019 % / 0.00273 |
| kit-locale-mainMenu-ar | 0.000 % / 0.00101 | 0.000 % / 0.00058 | 0.000 % / 0.00056 |
| kit-skin-neon | 0.000 % / 0.00104 | 0.000 % / 0.00023 | 0.000 % / 0.00020 |
| kit-skin-paper-phone (the largest RMSE without outliers) | 0.000 % / 0.00401 | 0.000 % / 0.00386 | 0.000 % / 0.00385 |
| every other frame | no outliers, RMSE ≤ 0.0025 | same | same |

Identical to metal: 1 frame on vulkan-cpu (glass), 6 on webgpu, 11 on
webgl2. The eleven frames with outliers are the same on every tier: the
edges of the large MSDF titles ("Settings", "Paused", "Shop", "Levels",
"Toasts", the puzzle's score), eleven edge pixels on idle, and the
gallery's scrollbar thumb, about a quarter pixel over. Each is edge
coverage a fraction of a pixel apart, at most 0.04 % of the pixels. The
three non-metal tiers agree with one another on those edges: vulkan-cpu
against webgpu and webgl2 has no outlier on any frame (RMSE ≤ 0.0019), and
webgpu against webgl2 is within RMSE 0.0002. The odd one out is metal's
rasterisation, so recording each tier's own frames absorbs no defect.

## How they were made

- `vulkan-cpu`: `cargo zigbuild --release --target
  x86_64-unknown-linux-gnu.2.31 -p og-host-node -p og-cli`, then `-p og-ui
  --test gfx --test kit_gfx`; the addon and og copied to
  `packages/engine/native/origin-engine.linux-x64.node` and
  `og.linux-x64` (ignored). In the `og-lavapipe` image (linux/amd64,
  `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`,
  `WGPU_BACKEND=vulkan`) with `OG_UI_CAPTURE=1`: the two test binaries
  from `crates/og-ui`, then `og.linux-x64 test templates/ui-kit` (29
  suites passed while recording; the phone skin playtests take 300 to
  540 s each on lavapipe). Then without `OG_UI_CAPTURE`: gfx 5 of 5 and
  kit_gfx 1 of 1 at RMSE 0 against `vulkan-cpu`, and the playtest run without capture (`og.linux-x64 test templates/ui-kit`)
  passed 17 of 29 suites before it was stopped for time: a11y, drag,
  frames-desktop (45 checks), frames-phone (45), gamepad, hit, juice, keys,
  locales (33), multitouch, pad, phone, players, refuse, skin-cartoon,
  skin-cartoon-phone and skin-fantasy-phone, all against `vulkan-cpu` with
  no "comparing with metal" note. skin-fantasy, the neon, paper, pixel and
  scifi skin suites, sounds, touch and tutorial were not rerun after
  capture (they passed in the recording run).
- `webgpu`, `webgl2`: from packages/engine, `bun tests/web/reference-gfx.ts
  --record [--webgl] --scene og-ui/<name>` for the four scenes and `bun
  tests/web/reference-ui.ts --record [--webgl]` for the 70 kit frames,
  after a run of each against metal passed (16 of 16 playtests, 298 shots,
  on each tier). A rerun without `--record` reproduces every frame at
  RMSE 0 on both tiers (webgpu 100 s, webgl2 112 s).

## Gate

`OG_GATE_JOBS=2 scripts/check.sh` on 8f1d3af (the docs and evidence of
the next commit were uncommitted and touch no check): fmt, clippy (75 s),
wasm-build, wasm-opt (212 s), size-gate (3,176,490 B brotli, 75 % of
4,194,304; no crate in the wasm build changed this round), drift (186 s)
and test (722 s) passed. bun-test failed after 506 s on five timeouts
and no assertion, while a lavapipe container ran beside it (104 pass):
render.test.ts "trails, beams and ribbons" and "material and sprite
effects" at 5 s and web-export.test.ts "the same inputs give the same
bytes" at 120 s (the known load flakes), plus asset-stream.test.ts
"what is resident never changes the simulation" at 5 s and "og build
--publish ships each group as its pack" at 120 s (not on the known list;
this round changes nothing they run). This stopped determinism. Neither the `--only bun-test`
rerun nor determinism nor `scripts/check.sh --only web-reference` ran:
the coordinator now runs the gate on main after the merge. The browser
kit and scene frames were checked directly instead: `reference-ui.ts`
and `reference-gfx.ts` without `--record` reproduce every frame on
webgpu and webgl2 at RMSE 0.
