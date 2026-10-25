# r4/ao-cost: the ambient-occlusion pair's GPU cost

Base `96e89601` (origin/main). Branch `r4/ao-cost`.

The finding this answers (Apple M4 Pro, WebGPU, the §7 reference scene at
1280×720, non-publish build with GPU timestamps): `ao` 0.789 ms (44 % of a
1.79 ms frame) and `ao-blur` 0.114 ms (6 %), more than the lit main pass
(0.356 ms).

## Commits

| Commit | Tier | What | Frames |
|---|---|---|---|
| `87ba7958` | (a) | `fs_ao_blur` returns 1 on the cleared sky (view depth 1e9) before its 32 loads and 16 exps; `fs_gtao` reads the depth's size once and each slice's jitter once, not per tap. | bit-identical |
| `c49d6875` | (a) | The full tier's blur, `fs_ao_blur_gather` (fxpost_gather.wgsl, appended to the effect module on the full tier only: GLSL ES 3.0 has no gather), reads the 16 depth and 16 raw-AO texels as 4 + 4 `textureGather`s through a new nearest clamp-to-edge sampler (fx post binding 6), then weighs and sums them in `fs_ao_blur`'s loop order. WebGL2 keeps `fs_ao_blur`. | bit-identical |
| `6e3a1f48` | (b) | `AmbientOcclusion.resolution: AoResolution { Full (default), Half }`, IDL `PostEffects.aoResolution` (`full`/`half`), `AoResolution` column factory. Half: `ao` and `ao-blur` run at ⌈w/2⌉×⌈h/2⌉ (texel q searched at full-res pixel 2q, its depth read there), then a new `ao-upsample` pass writes the full-res `ao` (2×2 half texels, bilinear × the blur's depth weight, nearest-in-depth fallback). | move only when opted in |

### Why the sky early-out is exact

On a cleared-sky pixel every sky tap weighs `exp(0) = 1` and holds raw AO
1 (GTAO and SSAO write 1 on sky). A surface tap is nearer than 1e8 m
(guarded: `fxp.cam.x < 1e8`, the far plane), so its weight is below
`exp(-(1e9 - 1e8) / 4e7) = exp(-22.5) ≈ 1.7e-10`; sixteen of them sum below
3e-9, under half an ulp of 1 (6e-8), so both the sum and the weight round
to the count of sky taps and the blur returned exactly 1.0 already.

### Why the gather blur is exact

A gather returns the stored texels (R32 float depth, R8 unorm AO) exactly
as `textureLoad` does; the sampler's clamp-to-edge is the loads'
`clamp(q, 0, size - 1)`; the weights are the same expression and the 16
taps are accumulated in the loop's order (row −2 to 1, x −2 to 1), each
`acc + a * w` as the loop's `sum += a * w`.

### What was not done in tier (a), and why

- **The AO pass itself.** Its cost is ALU, not fetches. Per pixel it does
  36 `view_pos` reconstructions (two mat4×vec4 and two perspective
  divides each), 36 normalisations and 3 slices of acos/cos/sin: about
  2,700–3,000 lane-cycles a pixel by count, against 0.789 ms × 2,560
  lanes × ~1.5 GHz / 921,600 px ≈ 3,300 measured on the M4 Pro. Every
  cheaper reconstruction (closed-form view position, dot before divide)
  changes rounding, and a matrix product's summation order is the
  driver's, so none can be proven bit-identical: they belong with the
  frame-moving mode, not tier (a).
- **An R32Float depth for the taps** (lindepth is Rg32Float, 8 B a texel;
  its `.g` is SSR's reflectivity, written by the prepass, terrain,
  scatter and fluid shaders). Halving the tap footprint would help a
  cache-bound pass; this one is ALU-bound by the count above, and moving
  reflectivity touches five writers. Left as a candidate if the Mac lane
  shows `ao` texture-bound.
- **The blur's graph read of `gbuffer`.** `fs_ao_blur` never reads it; the
  declaration only orders passes and does not cost GPU time, so it was
  left alone.

## Tap and bandwidth counts (1280×720 = 921,600 px; half 640×360 = 230,400)

Texture instructions are per pixel of the pass; "texels" the bytes they
address before caching (lindepth 8 B, gbuffer 8 B, AO 1 B).

| Pass | Base | Tier (a) | Half (tier b) |
|---|---|---|---|
| `ao` (GTAO) | 921,600 px × (1 depth + 1 normal + 36 depth taps = 38 loads, 304 B) = 35.0 M loads, 280 MB addressed, 0.92 MB written; 36 `view_pos` | same (hoists only) | 230,400 px × 38 = 8.76 M loads (−75 %), 70 MB, 0.23 MB written; 36 `view_pos` on a quarter of the pixels |
| `ao` (SSAO, WebGL2) | 921,600 × (1 + 1 + 8 = 10 loads) = 9.2 M | same | 230,400 × 10 = 2.3 M (−75 %) |
| `ao-blur` | 921,600 × 33 loads (1 + 16 depth + 16 AO; 152 B) = 30.4 M loads, 140 MB; 16 exp | full tier: 1 load + 8 gathers = 9 instructions (−73 %), same texels; sky pixels: 1 load, 0 exp. WebGL2: 33 loads, sky 1 | 230,400 × 33 = 7.6 M loads, 35 MB; 16 exp at a quarter of the pixels |
| `ao-upsample` | — | — | 921,600 × 9 loads (1 + 4 depth + 4 AO; 68 B) = 8.3 M, 63 MB; 4 exp; 0.92 MB written |
| pair, per full-res pixel | 71 texture instructions | 47 (full tier) | 9.5 + 8.25 + 9 = 26.75 |

The reference scene is about a fifth sky (the band past the 80 m floor);
the sky early-out removes the blur's work there on both tiers (the AO
pass already returned on sky).

## Measured here (lavapipe, llvmpipe 256-bit, relative only)

GPU timestamps per pass (`Renderer::set_clock` + `gpu_timings`), mean of
10 frames after 2, the `ao-gtao` gfx scene, ms:

| | `ao` | `ao-blur` | `ao-upsample` | pair |
|---|---|---|---|---|
| base `96e89601` | 18.3–19.0 | 6.2 | — | 24.5 |
| tier (a) `c49d6875` | 18.2 | 3.7 | — | 21.9 |
| half (tier b) | 6.7 | 2.7 | 3.3 | 12.7 |
| (rejected) full res, 2 slices × 4 steps | 11.0 | 3.9 | — | 14.9 |
| SSAO, full / half | 6.9 / 3.7 | 3.9 / 2.8 | — / 3.4 | 10.7 / 9.9 |

llvmpipe has a large fixed cost per pass (a half-size blur costs 2.7 ms,
not a quarter of 3.7), so the half mode's ratio on a GPU should be better
than here. Half resolution beat 2 slices × 4 steps (16 taps instead of
36) here and keeps all 36 taps per searched pixel; the slices mode was
measured with the constants patched, not committed.

## Frames

Captured with `og-gfx … --capture` on lavapipe, base and branch, compared
byte for byte (`cmp`) and with ImageMagick (`compare -metric AE`, RMSE
normalised):

- **Full resolution (the default), tier (a) and (b):** `ao-gtao`,
  `ao-ssao`, `showcase`, `overlay`, `skinned-hooks` and the AO debug view
  (`OG_GFX_DEBUG_AO=1`) of `ao-gtao`, `ao-ssao`, `showcase`,
  `skinned-hooks`: **byte-identical** to base after each commit. The
  gather path was confirmed to run (forcing its output to 0 changed
  `ao-gtao`).
- **Half resolution (`OG_GFX_AO_HALF=1`), against base full:**

| scene | AE (any diff) | AE (fuzz 2 %) | RMSE |
|---|---|---|---|
| ao-gtao | 311,552 | 3,277 | 0.0034 |
| ao-ssao | 177,484 | 2,230 | 0.0029 |
| showcase | 304,495 | 270 | 0.0018 |
| overlay | 277,255 | 209 | 0.0017 |
| skinned-hooks | 273,909 | 344 | 0.0021 |
| ao-gtao, AO debug view | 183,105 | 2,287 | 0.0036 |
| showcase, AO debug view | 582,975 | 62,133 | 0.0089 |

Half resolution keeps contacts and silhouettes (the upsample is depth
aware); its noise pattern is twice as coarse, which the showcase floor's
AO debug view shows most.

No committed reference frame was re-recorded. The reference scene
(`crates/og-tests/reference-scene`) does not opt in: three.js's side of
the §7 comparison runs a full-resolution GTAOPass, and its frame could not
be rendered here (no `og run` of the TS project in this lane).

## Tests run

- og-render: `cargo nextest run -p og-render` (96 passed: the full-tier
  MSL/SPIR-V and GL-tier GLSL ES 3.0 translation of every module, the
  gather module included).
- og-tests, one at a time: `ao_gtao_matches_its_reference`,
  `ao_ssao_matches_its_reference`, `ambient_occlusion_darkens_contacts`,
  `binary(hooks)` (3, `skinned_hooks_match_their_reference` with AO),
  `the_showcase_frame_is_recorded_from_its_graph`, and the new
  `half_resolution_ao_adds_its_upsample_and_halves_its_targets`.
- og-runtime, og-api, og-idl: `cargo nextest run` (90 passed), with the
  new `ao_resolution_reaches_the_renderer_and_defaults_to_full` (fails
  with the runtime's mapping of the field dropped).
- `og api gen`, then `og api check`: the generated files match.
- `cargo fmt --all --check`; `cargo clippy -p og-render -p og-api
  -p og-runtime -p og-tests --all-targets -D warnings`.

## For the Mac lane

Per-pass GPU times of the reference scene on this branch (as the finding
was measured), then the same with `post.aoResolution = "half";` added
after `post.ao = true;` in `crates/og-tests/reference-scene/src/main.ts`
(locally, not committed). Expected, from the counts above:

- tier (a), default: `ao` unchanged (~0.79 ms); `ao-blur` down from
  0.114 ms (gathers: 9 texture instructions instead of 33; the fifth of
  the frame that is sky skips it). WebGL2: `ao-blur` only the sky saving.
- half: `ao` about a quarter (~0.2 ms), `ao-blur` a quarter of its new
  cost, `ao-upsample` new (9 loads, 4 exp a pixel, ~ half today's blur).
  The pair should land near 0.3 ms against 0.90 ms.
- `og-gfx --costs` natively on Metal now has "E1: AO (GTAO, half
  resolution)" and "(SSAO, half resolution)" rows.

## What moves

- **Frames:** none at the default. Only scenes that set `half` move.
- **World hashes:** `og.PostEffects` gains `aoResolution` (a u32), so every
  world's type table and resource bytes hash differently once, as with any
  new resource field. Simulation is untouched.
- **wasm bytes:** shader text +1.9 kB (fxpost.wgsl, comments stripped:
  14,536 → 16,417 B) and +2.0 kB (fxpost_gather.wgsl), plus the new enum
  and pass; a few kB raw, less after brotli. The wave's size gate
  measures it.
- **The WebGL2 trace fixture** (`packages/engine/tests/fixtures/webgl2-frame`)
  is a recorded input to the GL state filter's test, not compared with a
  live frame; the GL path's calls are unchanged except that the fx bind
  groups carry one more (unused there) sampler.
