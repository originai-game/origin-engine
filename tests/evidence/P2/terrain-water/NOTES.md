# Phase 2, terrain and water round: evidence

- `break-and-restore.md`: every new test broken once and what failed.
- `feature-costs.md`: the cost of each feature on metal and vulkan-cpu.
- `frame-times.md`: the scenes' frame times on metal and the fx-showcase
  terrain station's in Chrome on WebGPU and WebGL2, against the budget.
- `og-tests-metal-vs-vulkan-cpu.png`: the seven phase 2 og-tests scenes,
  metal left and vulkan-cpu right (terrain-near, -mid, -far, -grass,
  water-planar, -shadows, -underwater).
- `fx-showcase-terrain-tiers.png`: the fx-showcase terrain station on
  metal, vulkan-cpu (top), webgpu, webgl2 (bottom).

## References

How each tier was made:

- `metal`: `OG_GFX_CAPTURE=<scene> cargo test -p og-tests --test terrain`
  and `OG_FX_CAPTURE=terrain.png og test templates/fx-showcase` on the M4
  Pro.
- `vulkan-cpu`: og-gfx, `og` and the addon cross-built with `cargo zigbuild
  --release --target x86_64-unknown-linux-gnu.2.31`, run in the og-lavapipe
  container (Debian trixie, Mesa llvmpipe on LLVM 19.1.7): `og-gfx --all
  --capture` and `OG_FX_CAPTURE=terrain.png og test templates/fx-showcase`.
  The same run compared every earlier og-tests scene with its committed
  vulkan-cpu reference: 35 at RMSE 0, `water-3d` at RMSE 0.00062 (4
  outlier pixels, within og-tests' rule), `msaa` without one (as before);
  every earlier fx-showcase station passes there. A second og-gfx run of
  the seven new scenes matched the recorded frames at RMSE 0.
- `webgpu` and `webgl2`: `bun tests/web/reference.ts [--webgl] --record
  --script "0.5:station:terrain" --shots terrain@330` in Chrome for
  Testing; later runs (with `--time`) matched at RMSE 0.

The tiers against metal:

| Frame | tier | RMSE | outliers |
|---|---|---|---|
| `terrain-near` | vulkan-cpu | 0.00598 | 0.006 % |
| `terrain-mid` | vulkan-cpu | 0.00554 | 0.010 % |
| `terrain-far` | vulkan-cpu | 0.01757 | 1.026 % |
| `terrain-grass` | vulkan-cpu | 0.00537 | 0.014 % |
| `water-planar` | vulkan-cpu | 0.00355 | 0.008 % |
| `water-shadows` | vulkan-cpu | 0.00475 | 0.002 % |
| `water-underwater` | vulkan-cpu | 0.02253 | 2.256 % |
| fx-showcase `terrain` | vulkan-cpu | 0.00427 | 0.008 % |
| fx-showcase `terrain` | webgpu | 0.00136 | 0.002 % |
| fx-showcase `terrain` | webgl2 | 0.00774 | 0.040 % |

Two differ beyond og-tests' cross-tier tolerance, which is why each tier
has its own reference. `water-underwater`: the caustics' fine cells and
the surface's waves seen from below land on different pixels. `terrain-far`:
lavapipe draws no foam on the island's shore and leaves the shallow lake
basin clear. The sea there is one 24 km quad (`segments` 1), and
llvmpipe's interpolation of depth across a triangle that large is too
coarse for the foam's depth test 1.5 km away. This was not investigated
further; a finer sea mesh would test it.

## Cook determinism

- `crates/og-cook/tests/terrain.rs`: two cooks of a terrain are
  byte-identical, a third cooks nothing, an edited heightmap re-cooks only
  the terrain.
- templates/fx-showcase cooked twice from empty `cooked/` directories:
  every output byte-identical (28 outputs, 1,826,637 bytes; the valley's
  `terrain/valley.ogt.ogb` is 126,125 bytes, sha256 `71871f7c…4c2e0`).
- `tools/make-terrain.py` run twice: its six images are byte-identical.

## Wasm

The publish wasm (`og_host_web-publish`, the gate's size-gate step):
2,943,180 B brotli before the round (2dae5fa), 2,964,497 B after the
renderer (0ee78f1, +21,317 B), 2,975,446 B after the og.terrain module
(5be9281), 2,978,461 B at ad06e6e (+35,281 B over the round, 71.0 % of the
4,194,304 B budget).
