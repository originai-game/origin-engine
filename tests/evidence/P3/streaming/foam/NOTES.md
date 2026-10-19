# terrain-far's shore foam on vulkan-cpu: the cause

Phase 2 recorded that lavapipe draws no foam on `terrain-far`'s shore and
leaves the lake basin clear, and suspected the sea: one 24 km quad
(`segments` 1). This round tested that and did not change the renderer
(water is outside this round's render paths).

How: og-gfx cross-built for x86_64 Linux (`CARGO_TARGET_DIR=target/agent-render
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31 -p og-tests
--bin og-gfx`) with a temporary environment override of the sea's
`segments` in `island_view` (not committed), run in the local og-lavapipe
container (Mesa llvmpipe, LLVM 19.1.7): `og-gfx terrain-far --out` with 1,
64, 256 and 1024 segments. `terrain-far-sea-segments.png` crops the shore
and the lake of each beside the metal reference.

| Sea | vs metal RMSE | outliers (> 24 of 255) |
|---|---|---|
| 1 segment (the scene) | 0.01771 | 1.622 % |
| 64 segments | 0.01598 | 0.829 % |
| 256 segments | 0.01587 | 0.807 % |
| 1024 segments | 0.01587 | 0.807 % |

The one-segment frame matches the committed vulkan-cpu reference
(RMSE 0.00237, within og-tests' rule).

Cause: the huge triangle. The camera stands over the quad, so the near
plane clips it and its 1/w runs from the near plane's to 12 km's across one
triangle; llvmpipe evaluates its interpolants as plane equations from the
triangle's origin in f32, which leaves the interpolated world position at
the far shore metres off. The foam band (0.6 m of water depth) and the
lake's depth test against the depth prepass both need it to within
centimetres, so the shore loses its foam. The lake's own surface (a
360 m quad) vanishes with it and returns once the sea is tessellated, so
it is lost to the sea's wrong depth too, though the exact path was not
traced. Any tessellation brings both back; 256
and 1024 segments render identically, so the sea's interpolation is no
longer the error there.

What remains: the band is thinner on lavapipe than on metal at every
segment count. The likely reason is that the water's wave normals
(`wave_slope`) and foam noise (`value_hash`) take `sin`/`cos` of arguments
in the thousands at 1.5 km, where f32 results are implementation-defined;
the normal shifts the refracted depth sample that sets the band's
thickness. Not tested.

Not fixed, and why: the test scene's sea could be tessellated, but that
would only hide the renderer's limit, and the renderer's fixes (deriving
the water's own depth from the fragment's depth, which is exact in screen
space, or tessellating large water planes) belong to the water path,
outside this round's terrain, scatter and loading paths. Until then a game
whose water reaches kilometres should give it `segments` so a cell stays
within a few hundred metres.

## Fixed in the culling and device round (2026-10-19)

The water renderer now draws a surface as a grid whose cells are at most
256 m (`MAX_CELL` in `crates/og-render/src/water.rs`): a surface keeps its
own `segments` unless a cell would be longer, so the 24 km sea of
`terrain-far` draws as 94 x 94 cells and the 360 m lake as 2 x 2. The scene
is unchanged; any game's large water gets the same treatment without
setting `segments`. `water.wgsl` is unchanged.

The metal reference was the other half of the artifact. A temporary shader
that took the water's depth from an exact ray-plane intersection and
discarded on exact linear depth (not committed) drew the same thin band
with one segment as with many, on both tiers; the huge triangle's depth
error on Metal is the mirror image of llvmpipe's and widened the band to
several pixels. With the tessellated renderer both tiers draw that thin
band. `terrain-far-tessellated-renderer.png` crops the south-west shore
(pixels 240-560 x 400-640, doubled): metal before (thick white band),
vulkan-cpu before (no foam), metal after, vulkan-cpu after.

| terrain-far frames compared | RMSE | outliers (> 32 of 255) |
|---|---|---|
| metal before vs vulkan-cpu before | 0.01771 | 9,554 |
| metal after vs vulkan-cpu after | 0.00534 | 784 |
| metal before vs metal after | 0.01561 | 7,247 |
| vulkan-cpu before vs vulkan-cpu after | 0.00914 | 3,107 |

What is left between the tiers after the fix is the horizon row (y 103)
and one-pixel rims on the shore's and the lake's silhouettes, not the
band's width. So the `sin`/`cos` hypothesis above is not the cause: the
band was thinner on lavapipe only because it was compared with metal's
widened band.

References re-recorded, each on its own tier (og-gfx `--capture
crates/og-tests/references`; vulkan-cpu in the og-lavapipe container):

- `metal/terrain-far.png`: the shore's foam is now a thin line where it was
  a band several pixels wide (RMSE 0.01561 against the old reference, so
  the old one fails og-tests' rule).
- `vulkan-cpu/terrain-far.png`: the shore now has its foam line and the
  lake its surface edge (RMSE 0.00944 against the old reference; it
  passed, but it held the artifact).

No other reference moved past rounding: on vulkan-cpu, the comparisons of
water-planar, water-shadows, water-3d, water-underwater and the other
terrain scenes with their references change by at most 0.0012 RMSE
(water-shadows, 0.00059 to 0.00178), and on metal water-planar's moves
from 0.0047 to 0.0049. Break test: with the old `water.rs`,
`cargo test --release -p og-tests --test terrain` fails
`terrain_scenes_match_their_references` on metal.

Not this round's change: `vulkan-cpu/terrain-grass` reads RMSE 0.01488
(4,047 outliers) in the same container on main at cf6aeaf as on this
branch, with either culling path and either water renderer; it predates
both fixes.
