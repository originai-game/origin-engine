# r3/meridian-webgl2: why Meridian Loop's frames differ on WebGL2

Base de11947c (origin/main). Branch `r3/meridian-webgl2`. Orb (Linux x64,
lavapipe, no GPU, no Chrome run): the WebGL2 frames are the coordinator's,
from the measure lane at a7f10c57 (Chrome for Testing, ANGLE Metal, M4 Pro).

## The question

`web-reference` judged Meridian Loop's ten frames against
`examples/meridian-loop/references/metal/`: WebGPU 10/10 (RMSE ≤ 0.0092),
WebGL2 0/10 (RMSE 0.0138 to 0.0509; outliers title 4.79 %, jump 2.88 %,
slide 4.80 %, race 5.50 %, ghost 3.99 %, board 1.0 %, the rest ≤ 0.3 %),
the differences on the neon grid lines, the horizon, the windows and the
car's rear light.

## What decides it: the antialiasing the bloom reads

The game leaves `GameDesc.render.antialias` at `tier`: TAA on the full tier
(metal, WebGPU), 4× MSAA on WebGL2 (og-render README, §Backend tiers).
Bloom's prefilter reads the antialiased lit image (`lit_names` in
`src/post.rs`: with TAA the TAA output, `history-write`). TAA's resolve
(`shaders/taa.wgsl`) weights each colour by 1 / (1 + luma) and clamps the
history to the 3×3 neighbourhood, so a sub-pixel emissive line keeps far
less energy than under the MSAA resolve's linear average, and blooms
weaker.

The same game on one adapter (lavapipe, `og run examples/meridian-loop
--commands "1.2:screenshot:title.png;1.3:quit"`, dev og at de11947c with
a temporary `render: { antialias: "<m>" }` in `src/main.ts`, not
committed), compared per pixel (og-tests' tolerance: a channel off by more
than 32/255 is an outlier):

| title frame | against metal's reference | against the WebGL2 frame |
|---|---|---|
| WebGL2 (Chrome, `tier` = 4× MSAA) | RMSE 0.0495, 4.790 % | — |
| lavapipe `msaa4` | RMSE 0.0499, 4.862 % | **RMSE 0.0039, 0.010 %** |
| lavapipe `off` | RMSE 0.0514, 5.138 % | RMSE 0.0128, 0.170 % |
| lavapipe `taa` (= `tier`) | RMSE 0.0191, 0.635 % | RMSE 0.0599, 8.043 % |

The moving frame agrees: `race` (tick 1140, the camera chasing the car at
144 km/h; the playtest's commands up to `19:screenshot:race.png`, `msaa4`):

| race frame | against metal's reference | against the WebGL2 frame |
|---|---|---|
| WebGL2 (Chrome, `tier` = 4× MSAA) | RMSE 0.0509, 5.498 % | — |
| lavapipe `msaa4` | RMSE 0.0511, 5.477 % | **RMSE 0.0050, 0.081 %** |

The WebGL2 frame is the native MSAA frame within og-tests' tolerance; the
full tier's TAA is what moves metal's away. Sky and fog away from emitters
match to a level (race: the sky at (100, 40) is 134,66,73 on both, the
road at (900, 600) 18,16,30 against 17,16,29), so neither the fog band nor
the sky diverges on GLSL ES; the horizon's brighter band on WebGL2 is the
bloom of the track's emissive edge along it.

- `title-bloom-by-antialias.png`: a 560×300 crop of the title frame
  (x 0, y 290): metal's reference (TAA) and the WebGL2 render on top,
  lavapipe `taa` and `msaa4` below. The left column is TAA, the right
  MSAA, whatever the backend.
- `race-bloom-by-antialias.png`: a 420×300 crop of the race frame
  (x 300, y 150): metal's reference, the WebGL2 render, lavapipe `msaa4`.
  Besides the halos, MSAA keeps lines thinner than a pixel that TAA's
  clamp removes: the gantry's cyan edges and the dashed edge along the
  horizon.

## Verdict per difference

- (a) by design, every one of them: the grid lines' halos, the horizon's
  band, the windows, the rear light and the sub-pixel edges are the tier's
  antialiasing (4× MSAA against TAA) feeding the look's bloom.
- (b) none in og-render's GLSL ES path: the same frames on a native
  backend with the same antialiasing match WebGL2 within tolerance, so no
  format, mip, filtering or sRGB difference is left to account for.
- (c) none in the game's assets. (Aside: `bun tools/make-assets.ts` on
  this Linux orb rewrote the committed `assets/models/track.glb` with the
  same size and different bytes; it was restored before the `msaa4` runs
  and does not move the title frame.)

## Coordinator acceptance, 2026-10-24

On the Mac measure lane at ad677e36 with the published-input correction
in 98cf9a82, recorded the ten `tests/frames.og.ts` stations with:

    bun tests/web/reference.ts --project examples/meridian-loop --playtest tests/frames.og.ts --webgl --record

Inspected all ten frames: title, jump, slide, countdown, race, ghost,
pause, results, board and German settings render intact. A second run
without `--record` passed 10/10, all RMSE 0.

The published harness had separately delivered input one tick before
`Probe::pump` does. Delivering at tick T instead of T−1 fixes WebGPU's
`pub-straight`: RMSE 0.02575 → 0.00792 against the unchanged Metal
reference; 22/22 published checks pass. No renderer or tolerance changed.

After that correction, WebGL2's title, countdown and straight still
exceed Metal's tolerance (RMSE 0.04949, 0.01994, 0.04848). Their stronger
neon halos agree with the AA investigation above; their menu, countdown,
race timer and HUD match the native states. Kept these three inspected
captures from `published.ts --project examples/meridian-loop --webgl`
as `references/webgl2/pub-*.png`. Kept no copy of `pub-pause`, which
passes against Metal (RMSE 0.00985). A fresh published run passed 22/22;
the three new reference comparisons have RMSE 0. The German setting
persists through a reload, and the page and Workers report no warnings
or errors. `device.ts --project examples/meridian-loop` passes 27/27.
