# G2-6: procgen sample (batch 1: noise, Poisson, BSP, WFC, tilemap fill)

`procgen/` is one 2D world with three tilemaps. Each map is generated with
`og.procgen` and written with `og.tilemap.fill`. The `Generate` system
regenerates them live, and the camera visits one map every three seconds.
Every seed is a fork of og.Rng (`world.resource(og.Rng)`'s value `.fork(label).seed`), so the run replays
exactly: two logic runs in the orb (`OG_RENDERER=none`) ended on the same
hash, `b6ed545b41534979` at tick 511.

- 0 to 3 s, the terrain. A 64 × 40 slice of 3D FBM simplex noise (5
  octaves, frequency 0.04) is banded into deep water, water, shallows, sand,
  grass, rock and snow. The slice scrolls along x and moves in z every tick,
  so the land drifts and morphs. Woods (dark green) go on inland grass where
  Poisson-disk points (radius 2.2) fall: `Show.woods` is 79 at 0.5 s.
- 3 to 6 s, the dungeon. A 48 × 32 BSP layout (rooms 4 to 10 cells, depth
  5) is drawn with the roguelike archetype's tomb tiles. Floor slabs go under
  rooms and corridors, and the stairs sit at the centre of the last room. The
  walls are the tileset's bitmask terrain layer, written in one fill and
  re-tiled once. A new seed (`dungeon-<n>`) comes every half second; the
  first layout has 18 rooms.
- 6 to 9 s, the village. A 16 × 10 WFC of nine tiles (grass, house, field,
  trees, a road with its bend, junction and crossing, all three rotated, and
  a well at a road's end) is drawn as 3 × 3 cells per tile, with
  `assets/tiles/palette.png`. A new seed (`village-<n>`) comes every 0.75 s.

`assets/tiles/palette.png` (16 flat colour tiles) was written once by a
short Python script; `tomb.ogt` and `tomb.png` are copied from
`templates/archetypes/roguelike`.

## Run on the Mac satellite (`metal`, macos-27, a dev build of this branch)

```sh
og run tests/evidence/g2-6/procgen --headless --size 1280x720 --out <dir> --commands \
  "1:screenshot:terrain-a.png;2.5:screenshot:terrain-b.png;3.6:screenshot:dungeon-a.png;5.1:screenshot:dungeon-b.png;6.5:screenshot:village-a.png;8.4:screenshot:village-b.png"
og render tests/evidence/g2-6/procgen --commands "0:get:show:Show:s" --fps 30 --from 0 --to 3 --size 1280x720 --out noise-terrain.mp4
og render tests/evidence/g2-6/procgen --commands "0:get:show:Show:s" --fps 30 --from 3 --to 6 --size 1280x720 --out bsp-dungeon.mp4
og render tests/evidence/g2-6/procgen --commands "0:get:show:Show:s" --fps 30 --from 6 --to 9 --size 1280x720 --out wfc-village.mp4
```

## What each frame shows

- `terrain-a.png` (tick 60): two large deep-water basins ringed by water,
  shallows and sand. Green land carries scattered dark-green wood cells.
  Four mountains have grey rock with white snow at their peaks.
- `terrain-b.png` (tick 150): the same kind of land, shifted left and
  reshaped by the moving slice.
- `dungeon-a.png` (tick 216): about 18 purple-slab rooms joined by
  one-cell corridors on a dark wall field. The walls' top edges are drawn by
  the bitmask terrain. The stairs (a dark triangle) are in the bottom-right
  room. Every room is reachable.
- `dungeon-b.png` (tick 306): another seed's layout.
- `village-a.png` (tick 390): tan roads that join without a broken edge,
  red-roofed houses with doors, yellow fields, clumps of five trees, and blue
  wells where roads end.
- `village-b.png` (tick 504): another seed's village. `Show` reads
  `villages 4, attempt 1`: each WFC succeeded on its first try.

## Clips

`noise-terrain.mp4`, `bsp-dungeon.mp4` and `wfc-village.mp4` (3 s each, 30
fps, 1280 × 720) are kept in `.amp/in/artifacts/g2-6/` of the thread that
drew them, waiting for the showcase archive.

# G2-6: run-time tilemaps sample (batch 2: create, resize, iso and hex)

`tilemaps/` is one 2D world with two maps made in game code by
`og.tilemap.create`; the project has no map asset, only two tilesets of
eight flat diamonds (`iso.png`, 32 × 16) and eight pointy-top hexagons
(`hex.png`, 32 × 32), drawn once with ImageMagick, and `ball.png`. Two
logic runs in the orb (`OG_RENDERER=none`) ended on the same hash,
`080f6d27304e1948` at tick 480.

- 0 to 4 s, an isometric island, 20 × 20 diamonds of 2 × 1 world units,
  banded from FBM simplex noise falling off from the centre. At 2 s
  `og.tilemap.resize` grows it to 28 × 28 with the old cells shifted 4 × 4
  in, and the whole map is filled again from the same noise field offset by
  the shift: the island stays where it was and the new ring is open water
  with two small reefs. The cell under the centre and its four
  `og.tilemap.neighbours` are orange, the same cell before and after.
- 4 s on, a hex map, 30 × 20 pointy-top hexagons: two rows of solid rock at
  the bottom and noise-placed rock outcrops, each hexagon its own collider
  (`SHAPE_HEXAGON`). A ball drops every 0.2 s (18 in all) at a column taken
  from a fork of og.Rng. The centre hex and its six neighbours are orange
  (not solid).

## Run on the Mac satellite (`metal`, macos-27, a dev build of this branch)

```sh
og run tests/evidence/g2-6/tilemaps --headless --size 1280x720 --out <dir> --commands \
  "1:screenshot:iso-small.png;3:screenshot:iso-grown.png;4.6:screenshot:hex-drop.png;7.8:screenshot:hex-rest.png"
og run tests/evidence/g2-6/tilemaps --headless --size 1280x720 --out <dir> --commands "10:screenshot:hex-settled.png"
og render tests/evidence/g2-6/tilemaps --commands "0:get:show:Show:s" --fps 30 --from 0 --to 4 --size 1280x720 --out iso-resize.mp4
og render tests/evidence/g2-6/tilemaps --commands "0:get:show:Show:s" --fps 30 --from 4 --to 10 --size 1280x720 --out hex-colliders.mp4
```

## What each frame shows

- `iso-small.png` (tick 60): a 20 × 20 diamond of deep water around a
  green island with sand beaches, dark-green woods, grey rock and an orange
  plus of five cells at its middle.
- `iso-grown.png` (tick 180): the same island, unmoved, inside a 28 × 28
  diamond; the added ring is deep water with two light-blue reefs.
- `hex-drop.png` (tick 276): the hex map (blue rows above, green below,
  grey outcrops and floor, an orange flower of seven hexes); the first
  balls fall from the top.
- `hex-rest.png` (tick 468): balls falling, bouncing off outcrops and lying
  on the floor's zigzag top.
- `hex-settled.png` (tick 600): balls in the floor's notches and wedged
  between outcrops. Measured at 12 s in the orb, ball-9 rests at
  (105.150, -7.434) with no velocity, 0.35 (its radius) from both the left
  side of hex (5, 9) and the upper-right edge of hex (4, 10): the colliders
  are the hexagons drawn. Two balls rolled off the floor's open left end
  and fell out of the map.

## Clips

`iso-resize.mp4` (4 s) and `hex-colliders.mp4` (6 s), 30 fps, 1280 × 720,
are kept in `.amp/in/artifacts/g2-6/tilemaps/` of the thread that drew
them, waiting for the showcase archive.
