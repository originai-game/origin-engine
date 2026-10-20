# Shadows and antialiasing by tier (P4, the shadows-aa round)

Branch `r-shadows-aa` from main 20a6d47, main 7696db4 merged at 847d2bd.
Apple M4 Pro (metal), the og-lavapipe container (vulkan-cpu: Mesa
llvmpipe, LLVM 19.1.7, og-gfx cross-built with `cargo zigbuild --release
--target x86_64-unknown-linux-gnu.2.31`), Chrome for Testing (webgpu,
webgl2). The design is `crates/og-render/README.md` §Shadows and
antialiasing by tier.

## Frames (each read)

| File | What it shows |
|---|---|
| `sun-hard-vs-soft.png` | top: `shadows-soft` without (left) and with (right) PCSS 0.03 and cascade blend 0.2; bottom: the same casters over a plain white ground lit by the sun alone (the test's frames). Hard: aliased stair-stepped edges on the plank's and cube's shadows. Soft: the cube's shadow sharp where it meets the cube and soft at its far end, the plank's (2.5 m up) and the floating ball's blurred all round, the thin far poles' shadows thinned to faint lines |
| `point-cube-vs-paraboloid.png`, `point-cube-vs-paraboloid-diff.png` | `point-spot-lights` with cube faces (left, the committed reference) and with paraboloid hemispheres (right): the same picture, RMSE 0.00225, 102 outlier pixels; the diff (×4) lights only shadow edges, the pillars' silhouettes and a ring where the hemispheres meet at the light's height |
| `fxaa-vs-taa-crop.png` | a 320×180 crop of the thin poles and bars, ×2 nearest: FXAA (top) against the committed TAA reference (bottom): both smooth the bars' edges; FXAA leaves the distant checker's moiré |
| `taa-bilinear-vs-catmull-sharpen-crop.png` | the same crop under a camera sliding 2 cm a frame after 16 frames: bilinear history (top, `taa-motion`), Catmull-Rom with 0.5 sharpening (bottom, `taa-sharp`): crisper bars, no ringing |

New og-tests references (metal recorded here, vulkan-cpu in the
og-lavapipe container; every one was opened):

| Scene | vulkan-cpu vs metal |
|---|---|
| `shadows-soft` | RMSE 0.00157, 0 outliers |
| `point-shadows-paraboloid` | RMSE 0.00135, 0 outliers |
| `shadows-local-8` (full tier) | RMSE 0.00150, 0 outliers |
| `fxaa` | RMSE 0.00178, 3 outlier pixels |
| `taa-sharp` (full tier) | RMSE 0.00136, 1 outlier pixel |

No existing native reference moved: `og-gfx --all --capture` at 20a6d47
and at f507334 (the renderer commit) wrote 51 PNGs byte for byte
identical on metal, with identical stats lines
(`og-gfx-metal-before-20a6d47.txt`, `og-gfx-metal-after-f507334.txt`).

## What `auto` picks, and the costs

Chrome for Testing on the M4 Pro (`reference-gfx.ts --scene … --time 60`:
best of 7 runs of 60 frames, each ending in a readback, so the GPU's work
is in; `browser-gfx-webgl2.txt`, `browser-gfx-webgpu.txt`), and natively
(`og-gfx` release, `--frames 300`, three interleaved runs at a load
average of 58 to 78 from other agents; `native-costs-metal.txt`):

| Frame (ms) | WebGL2 | WebGPU | metal |
|---|---|---|---|
| point-spot-lights, cube faces (7 passes) | 0.702 | 0.445 | 0.57–0.89 |
| point-spot-lights, paraboloids (3 passes) | 0.503 | 0.435 | 0.44–0.53 |
| shadows-local-8, cube (WebGL2: 4 lights, 18 passes; else 8, 38) | 1.085 | 1.332 | 2.19–2.70 |
| shadows-local-8, paraboloids (WebGL2: 4 lights, 8 passes; else 8, 14) | 0.602 | 0.600 | 0.82–1.01 |
| shadows-soft hard / PCSS 0.03 + blend 0.2 | 0.618 / 1.01 | 0.412 / 0.635 | 0.40–0.43 / 0.83–0.87 |
| thin geometry: no AA / FXAA / 4× MSAA | 0.668 / 0.75 / 1.113 | 0.412 / 0.495 / – | 0.42 / 0.52–0.54 / – |
| TAA bilinear / Catmull-Rom + 0.5 sharpen | – | 0.493 / 0.552 | 0.63–0.67 / 0.69–0.82 |

`PointShadows::Auto` resolves to **cube faces on the full tier and
paraboloids on WebGL2**. On WebGL2 the paraboloids save 0.2 ms with one
shadowed point light and 0.48 ms with four (each GL pass costs a draw
setup), and the shadow array holds four lights in 8 MB instead of 24 MB;
the frame stays within tolerance of metal's cube reference
(`point-spot-lights` on WebGL2: RMSE 0.00225 against metal, 0.011 %
outliers, so it needs no WebGL2 reference). On the full tier one point
light costs the same either way and cube texels are finer, so `Auto` keeps
cube faces; with six shadowed point lights paraboloids save 0.7 ms on
WebGPU and 1.2–1.7 ms natively (mostly the CPU encoding 38 passes instead
of 14): a game with many shadowed point lights should ask for
`pointShadows: "paraboloid"`.

## Ember Vale's beacon on WebGL2 (item 5)

The CPU particle path now bounces off `RenderScene::particle_colliders`,
which og-runtime fills from og.Collider (Ember Vale: the vale's height
field, the plinths' and bowls' boxes). Against main 7696db4's re-recorded
metal beacon (the ui-text round's kerning), with this branch merged with
main 376f3cd:

| Beacon | RMSE vs metal | outliers vs metal |
|---|---|---|
| webgl2, the old reference (sparks fall through) | 0.01852 | 0.919 % |
| webgl2, this branch (sparks bounce) | 0.01645 | 0.809 % |
| webgpu, this branch | 0.00475 | 0.060 % |

`beacon-metal-webgl2-new-old.png`: metal (left), this branch's WebGL2
(middle), the old WebGL2 reference (right): the bounced embers lie on the
grass round the plinth in the middle frame as on metal. The WebGL2 frame
stays over 0.01 by design: 4× MSAA against TAA (RMSE 0.0130 on its own in
the showcase round's per-spark read, docs/phase-1.md) and sparks that
bounce off the colliders' exact surfaces rather than the depth buffer's,
so their paths after a bounce differ. The WebGL2 beacon reference was
re-recorded (reproduced at RMSE 0: `third-person-webgl2-after-rerecord.json`);
title and dawn stay judged against metal on both tiers (WebGL2 0.00431 and
0.00180; WebGPU 0.00131 and 0.00067).

Not this round's: on the merged tree templates/fx-showcase's `fx` and
`2d` stations fail against their own webgpu and webgl2 references on both
tiers alike (RMSE 0.0140 and 0.0127; `fx-showcase-*-main-drift.json`):
the HUD text's kerning (main's ui-text round re-recorded metal only) and
the sparks there, the same on WebGPU, whose path this round did not change
(the fx station has no colliders). Against metal they pass (fx 0.00300 on
WebGL2, 0.00183 on WebGPU). Those browser references are main's to
re-record.

## vulkan-cpu

og-gfx cross-built at main 7696db4 and at this branch (6fb84fd) capture
all 51 earlier scenes byte for byte the same in the og-lavapipe container
(`og-gfx-vulkan-cpu-main-7696db4.txt`, `og-gfx-vulkan-cpu-branch-6fb84fd.txt`).
Both show the same pre-existing differences from the committed vulkan-cpu
references, among them `terrain-grass` at RMSE 0.01488 (over 0.01): main's,
from whichever container recorded those, not this branch's.

## Size (brotli -q 11, per download)

| Download | main 7696db4 | this branch merged with 7696db4 | Delta |
|---|---|---|---|
| 3D publish | 2,899,979 B | 2,912,532 B (69.4 % of 4 MiB) | +12,553 B |
| 2D publish | 2,735,444 B | 2,747,961 B | +12,517 B |
| lanes | 3,277,957 B | 3,285,201 B | +7,244 B |

(`wasm-size-after-merge-7696db4.json`.) After merging main 376f3cd (the
cook-anim round; its NOTES give 2,913,544 / 2,749,599 / 3,290,490 B), at
b6b447d: 3D publish 2,926,692 B (+13,148, 69.8 % of 4 MiB), 2D publish
2,759,931 B (+10,332), lanes 3,300,932 B (+10,442)
(`wasm-size-after-merge-376f3cd.json`). The new code is the paraboloid
and PCSS shader text, FXAA and the TAA entry points, the collider tests and
the API members.

## Tests

og-render `tests/webgl2.rs` translates the caster, lit, TAA and FXAA
modules to GLSL ES 3.0; og-render `shadows.rs` unit test (tier resolution,
capacity); og-tests `tests/shadows.rs` (13 tests: references, paraboloid
against cube, the brightest lights keep their shadows whatever the scene
order, eight against two lights, the array grows and frames stay the same,
PCSS penumbra 759 → 7,073 pixels, cascade blending opt-in, FXAA one pass,
Catmull-Rom and sharpening keep detail: mean |Laplacian| 7.07 → 7.38 →
8.61); `tests/gfx.rs` `antialiasing_smooths_edges` with FXAA (19,604
partial pixels); `tests/effects.rs`
`cpu_particles_bounce_off_the_scene_colliders` (plane, flat height field
and box: lowest particle 0.016 m, 32 rising again; none: below -1 m and
counted as skipped).
