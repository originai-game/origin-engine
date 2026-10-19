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
