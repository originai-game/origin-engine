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

@@BODY@@
