# G2-6: procgen sample (batch 1: noise, Poisson, BSP, WFC, tilemap fill)

`procgen/` is one 2D world with three tilemaps. Each map is generated with
`og.procgen` and written with `og.tilemap.fill`. The `Generate` system
regenerates them live, and the camera visits one map every three seconds.
Every seed is a fork of og.Rng (`rng.fork(label).seed`), so the run replays
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
