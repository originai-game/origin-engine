# macrunner validation of 95104f3e — PASS

2026-10-27 (NZDT), macrunner, thread
T-01a17a1e-0262-7465-929b-874936a5f559. Base `95104f3e` (origin/main:
0.6.0, PR #3 wasm heap, PR #4 reflection slot and the five RedGPU render
ports of `tests/evidence/r5/post-redgpu/`). Branch
`mac-validate-95104f3e` on lane-3; the wave's end ran in the measure lane
pinned at `ccc676d4` (the branch with both commits below, clean tree).

## Machine

| | |
|---|---|
| Mac | Mac16,8, Apple M4 Pro, 24 GB |
| GPU | Apple M4 Pro, 16-core GPU, Metal 4 (wgpu adapter "Apple M4 Pro", tier `metal`) |
| macOS | 27.2 (26B5091g) |
| Chrome (web-reference) | Chrome for Testing 150.0.7871.115, headless, the harness's default (`tests/web/cdp.ts`; `CHROME` unset). Google Chrome 156.0.8078.47 is installed but not used by the harness |
| Safari | 27.2 (22625.2.5.11.1), **not run**: web-reference drives Chrome over CDP only; Safari is reachable only through `og run --device iphone` (safaridriver), which is not a reference tier |
| Toolchain | rustc 1.99.1, bun 1.4.4, node v26.3.0, tsgo 7.0.2 (tsrs 0.18.0), docker 27.5.1 |

## 1. Metal references of the five new scenes (commit `29b2afcd`)

`OG_GFX_CAPTURE=post-adjust,post-neutral,post-lens,terrain-grass-wind,post-blur
cargo test -p og-tests --test tests -- …` (`capture.log`). Each frame
was read beside its vulkan-cpu (lavapipe) reference; whole-frame numbers
(1280×720, metal vs vulkan-cpu, per-channel 0–255):

| Scene | metal sha256 | RMSE | max | px >8 | px >32 | What it shows |
|---|---|---:|---:|---:|---:|---|
| post-adjust | 7942d493f140aa19 | 0.00580 | 255 | 0.45 % | 0.03 % | the robots a deep blue-violet, sand-warm far floor and haze, neutral near floor, rich yellow crates, as on lavapipe; differences only on robot and crate silhouettes (thin detail, AA) |
| post-neutral | d16d92d064e9ea6e | 0.00122 | 1 | 0 | 0 | the bloom spheres keep their hue (orange stays orange-red, blue stays blue), white core rolls to white, navy backdrop; identical within one level |
| post-lens | 61b633e9544e069d | 0.00471 | 246 | 0.10 % | 0.01 % | the same barrel fit to the corners (the floor's edges bow, no black corners); differences only on model edges |
| terrain-grass-wind | ede3a1160c026b7a | 0.00540 | 132 | 0.22 % | 0.01 % | the blades leaning the same way at the fixed tick, rocks and mountains in place; differences on blade tips and the far ridge silhouette; frame mean 0.4 lower in R and G on metal |
| post-blur | ba5c19dadf6cabb9 | 0.00136 | 1 | 0 | 0 | the zoom blur streaking the spheres and floor towards the centre, bloomed after; identical within one level |

`five-scenes-vk-metal-diff.jpg`: per scene, vulkan-cpu | metal | |diff|×4,
at a third of the size. No colour, layout or effect differs; the
differences are rasterisation (edge coverage) between lavapipe and Metal.
The browser tiers then compared against these metal references (wave's
end, below):

| Scene | webgpu RMSE / outliers | webgl2 RMSE / outliers |
|---|---|---|
| post-adjust | 0.00376 / 0.004 % | 0.00365 / 0.004 % |
| post-neutral | 0.00000 / 0 | 0.00002 / 0 |
| post-lens | 0.00352 / 0.007 % | 0.00334 / 0.006 % |
| terrain-grass-wind | 0.00085 / 0.002 % | 0.00285 / 0.014 % |
| post-blur | 0.00000 / 0 | 0.00003 / 0 |

(tolerance ≤ 0.01 RMSE, ≤ 0.5 % outliers, unchanged).

## 2. Functional lane on metal

First run at `29b2afcd`: **FAIL bun-functional** (test cancelled with it),
one test: packages/create `what a scaffold ships never calls a UI style a
skin`:

```text
+   "templates/archetypes/idle/cooked/.cook-times.ogt: type cook-times version 1 --- asset "audio/buy.wav" 78.488 asset "audio/milestone.wav" 115.731 asset",
+   "templates/archetypes/rhythm/cooked/.cook-times.ogt: type cook-times version 1 --- asset "audio/bass0.wav" 133.929 asset "audio/bass1.wav" 115.782 asset ",
(fail) what a scaffold ships never calls a UI style a skin
 15 pass / 1 fail
```

Not a regression since 0.6.0 (the test is unchanged since 0.4.0, and no
source file says skin). The cause is ignored cook output in lane-3 dated
2026-10-21, from before the skins → styles rename (`cooked/skins/...`).
The test scanned `templates/archetypes` as one root, so its `copied()`
skipped `cooked/` only at that root and not at each archetype's root,
which is where create.ts and pack-template.ts skip it. Fix `ccc676d4`:
each archetype is now its own root. With the stale cooks still present
the test passes (16/16). A sentence "The og.ui skin of the shop." added
to the idle archetype's README still fails it, naming the file. No
tolerance or reference changed.

Second run at `ccc676d4`: **gate PASSED (5/5 steps, 95 s)**: fmt 3 s,
clippy 1 s, test 94 s (nextest 1392 run, 1392 passed, 19 skipped; every
og-tests gfx scene against the metal references, the five new ones
included), drift skipped (it passed in the first run; nothing changed
since), bun-functional 16 s. `functional.log`.

## 3. Wave's end (measure lane, `scripts/wave-end.sh`, default stages)

**wave-end PASSED (1775 s)**: gate 617 s, web 512 s, conformance 5 s,
release 641 s. `wave-end.log` (every step's log printed whole),
`summary.txt`.

- **Gate `--all`: PASSED (9/9 steps, 616 s).** fmt 2 s, clippy 18 s,
  drift 16 s, determinism 217 s, wasm-build 263 s, wasm-opt 188 s,
  test 205 s (nextest 1393 run, 1393 passed, 18 skipped), size-gate 23 s,
  bun-test 145 s.
- **Size gate** (`size-gate.log`, the `wasm` profile, brotli q11, budget
  4,194,304 B per download):

  | Download | bytes | brotli-q11 | budget | vs 8b2128b5 wave |
  |---|---:|---:|---:|---:|
  | 3D lanes | 11,430,326 | 3,251,741 | 77 % | +21,470 |
  | 2D lanes | 10,537,630 | 3,004,353 | 71 % | +14,867 |
  | 3D publish | 11,140,489 | 3,222,901 | 76 % | +20,462 |
  | 2D publish | 10,245,675 | 2,973,790 | 70 % | +14,730 |

- **Determinism: PASS**, every build agrees (`determinism.log`): native
  debug, release lanes off, release lanes on (4 threads, 1067 parallel
  calls), wasm32 scalar, simd128, lanes 3d, lanes 2d (node v26.3.0).
  math_digest 6117185439544271997, bouncing_boxes_hash
  15551159556835976588, physics3d_hash 10053948282410406339,
  physics2d_hash 3002410719564870113, anim_hash 12806465523503570878,
  the same values as the 8b2128b5 and 8715fd27 waves.
- **web-reference: PASSED (510 s)**, webgpu and webgl2 in headless
  Chrome for Testing 150.0.7871.115: fx-showcase stations, E5 stations,
  37 looks, og-tests' gfx scenes, og-ui's kit frames (desktop, phone,
  locales, six styles each way), Ember Vale 3, Brasswick 9, Meridian Loop
  10, lane-panic 23/23, a11y 26/26 per tier, GPU timing. 1167 `ok` lines,
  no FAIL. Safari: not supported by the harness (above).
- **conformance: PASS** (tsrs 0.18.0-ts7.1.0-dev.20260929 against tsgo
  7.0.2).
- **release: PASS.** `release.sh pack` at `ccc676d4` (clean) in the
  measure lane's `dist/release/0.6.0/` (the version is not bumped; these
  tarballs are a measurement and not for publishing; the main checkout's
  published 0.6.0 tarballs are untouched): engine 25,029,069 B
  (aab133a7…), engine-darwin-arm64 17,266,025 B (b69cc6ce…),
  engine-linux-x64 18,664,926 B (fc8bfadd…), create-game 16,320,543 B
  (95700a73…). `release.sh verify` install test PASSED in 108.5 s:
  darwin-arm64 59.8 s, linux-x64 container 43.9 s.

## Not done / left alone

- Safari: no reference tier or harness for it (see above).
- The vulkan-cpu (lavapipe) stages of the wave's end stay set aside
  (2026-10-22); not run.
- Leftovers on this runner that are not this thread's, untouched:
  untracked `tests/evidence/r4/bench-complex/mac/` and `mac-reference/`
  in the main checkout; lanes 1, 2 and 5 on the WIP branches dogfood-7,
  platformer-2d and kit-words (2026-10-22, each pushed, 0 uncommitted);
  the measure lane was on the merged `r5/web-8a40f5ca` (clean) and is now
  detached at `ccc676d4`.
