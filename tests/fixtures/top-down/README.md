# top-down

A farm at dusk in the manner of Stardew Valley: a farmer walks down the path
between fenced plots of crops while the lamps come on. It exercises the 2D
stack together on the real engine:

- **A tilemap.** `assets/tiles/farm.ogt` is a generated tileset (`type
  tileset`, 16 texels a tile: grass, flowers, soil, watered soil, path),
  its texture cooked as pixel art (`farm.png.ogt`, `kind pixel`); the map
  `farm-map.ogt` is 48×30 cells. `till` waters a cell with
  og.tilemap.setTile.
- **A flipbook from a GIF.** `assets/farmer/farmer.ogt` keys the generated
  walk (512×512, magenta ground, an art pixel about 11 of its pixels) and
  brings it to its own grid with `scale 0.0909`; `align none` keeps the
  walk's own motion (its centroid drifts under 2 px). Steps are frame
  events 1 and 5.
- **Loose PNGs packed into one texture.** The 29 crop and prop PNGs in
  `assets/props/` are the still sources of one sheet, `props.ogt` with
  `crop each`: the cook cuts each to its bounds and shelf-packs them into
  `props/props.ogt#sheet`. Every prop, crop, fence and tree is an og.Sprite
  with an og.Flipbook on its still clip, sorted by y (`sortY`).
- **2D light at dusk.** og.Lighting2D with a violet ambient and five warm
  lamps (og.Light2D, `castShadows`, `sourceRadius` 0.25); fences and crates
  cast with their sprite rectangle (`castShadow`), trees through an
  og.Occluder2D at the trunk. Blob shadows, glow, a warm grade and a
  vignette.
- **Camera.** og.Follow2D, clamped to the map.

`art.py` generates the tiles, the map, the props, their sheet and
`src/art.ts` (cell sizes and the map's layout); rerun it to change them.

**vfx2d.** `vfx2d()` in `src/main.ts` is where the dust puff goes once
`type vfx2d` lands: the `Walk` system calls it at the farmer's feet on each
step event. Adding it is the commented `og.vfx.spawn` line and an
`assets/fx/dust.ogt`.

What it checks (`og test tests/fixtures/top-down`, `tests/walk.og.ts`): the
farmer's step events fire (`the farmer steps`, `steps>=3`), she walks
3.5 tiles or more and ends on a path tile (`onPath==1`, read with
og.tilemap.getTile), `till:12:9` changes the map (`watered==7`), and the
frame at 2.2 s matches `references/metal/farm.png` (rmse ≤ 3).

Budget: under 10 s warm. Measured 1.5 s of run, 1.9 s with its cook and
check.

## Numbers

Measured on macrunner (Apple M-series, Metal) at 960×540. `crowd`,
`lamps` and `penumbra` are commands of the fixture for this; the playtest
never calls them.

**a. Packing loose PNGs.** 29 PNGs (4 to 44 texels wide) pack into a
112×143 texture: 16,016 texels, 6,437 of them drawn (alpha ≥ 16), a fill of
40.2 % (the cells' one texel of padding and the gap between them are most
of the rest). The default `crop all` cuts every source to the union of
all, 44×61, and packs them 314×309: 6.6 %. Building the pack takes about
10 ms (`og sprite inspect props.ogt --out`, best of 5: 42 ms, less 32 ms
for the process and one still); the cold cook of the whole fixture is
417 ms. Method: `og sprite inspect assets/props/props.ogt --out sheet.png`,
then count the texels of `sheet.png` with alpha ≥ 16.

**b. 2D soft shadows.** `penumbra:R` puts one light (radius 30, falloff
0.1) on a white floor, a 0.3×3 wall 3 units from it, and takes ambient,
grade and vignette off; the screenshot's column through the wall's lower
shadow edge is read from black to lit, 10 % to 90 %:

| sourceRadius | 6 units behind the wall | 10 units behind |
|---|---|---|
| 0.1 | 13 px | 19 px |
| 1.0 | 105 px | 172 px |

The penumbra grows with the source and with the distance, as the geometry
says, but at sourceRadius 1.0 it is drawn as 4 to 5 flat steps of 21 to
29 px rather than a ramp.

The light pass's GPU time (`lights-2d`, timestamp queries) with N shadowed
lights in view, `lamps:N` (which replaces the farm's lamps), median of the
frames from 1 s after the command:

| shadowed lights | 0 | 4 | 16 |
|---|---|---|---|
| lights-2d, GPU ms | 0.016 | 0.558 | 1.937 |

About 0.12 ms a shadowed light. Method: `og run tests/fixtures/top-down
--size 960x540 --trace t.json --commands "0.2:lamps:N;4.2:quit"`; the trace's
`gpu` spans named `lights-2d` (og-render's `gpu_timings`), median from tick
72.

**c. Sprite count.** `crowd:N` adds N crop sprites (flipbooks on still
clips, sorted by y) spread over the view, beside the farm's 410 entities.
Same trace method: CPU is the tick's span plus the render's `frame` span,
GPU the `gpu frame` span.

Release build (`OG_RELEASE=1 sh scripts/build.sh --native`, `target/release/og`):

| sprites added | 0 | 1,000 | 10,000 | 50,000 |
|---|---|---|---|---|
| CPU ms (tick + render) | 1.28 | 2.31 | 10.24 | 47.88 |
| GPU ms (frame) | 1.57 | 2.05 | 6.45 | 15.55 |
| of which `main` (the sprites), GPU ms | 0.19 | 0.66 | 4.82 | 14.03 |

At 50,000 the CPU's 47.9 ms is 33.4 ms of render (extraction and
encoding) and a 14.5 ms tick, 14.1 ms of it og.flipbook stepping flipbooks
that do not play (about 0.28 µs each): props that never animate would be
cheaper as plain sprites with the sheet's region. The gate's dev build
(workspace crates unoptimised) is about ten times slower on the CPU
(14.2, 98.7 and 494 ms) and the same on the GPU.

The light-pass numbers in b are the same on either build (release: 0.016,
0.581, 1.946 ms).
