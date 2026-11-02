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
`88dd847a5c002a2a` at tick 480 (`080f6d27304e1948` with the per-cell
colliders of 1e4dabf9; the hex map now collides as one outline per solid
region, 36 entities instead of 102). At tick 601 the orb and the Mac agree
on `d90d887dd2e5117d`.

- 0 to 4 s, an isometric island, 20 × 20 diamonds of 2 × 1 world units,
  banded from FBM simplex noise falling off from the centre. At 2 s
  `og.tilemap.resize` grows it to 28 × 28 with the old cells shifted 4 × 4
  in, and the whole map is filled again from the same noise field offset by
  the shift: the island stays where it was and the new ring is open water
  with two small reefs. The cell under the centre and its four
  `og.tilemap.neighbours` are orange, the same cell before and after.
- 4 s on, a hex map, 30 × 20 pointy-top hexagons: two rows of solid rock at
  the bottom and noise-placed rock outcrops. Each connected rock region is
  one collider, its outline (a 2D `SHAPE_TRIMESH` polyline of a generated
  mesh); before the merge each hexagon was its own (`SHAPE_HEXAGON`). A ball drops every 0.2 s (18 in all) at a column taken
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
  and fell out of the map. With the merged outlines (the same run, redrawn
  at the merge commit) ball-9 and ball-7 rest at the same points to the
  millimetre, (105.150, -7.434) and (116.35, -11.18), with no velocity.

## Clips

`iso-resize.mp4` (4 s) and `hex-colliders.mp4` (6 s), 30 fps, 1280 × 720,
are in the showcase archive (feature G2-6); the merged-collider redraws of
`hex-rest.png`, `hex-settled.png` and `hex-colliders.mp4` too.

## The merged colliders' cost (256 × 256, half the cells solid at random)

Measured on the Mac satellite with a scratch og-runtime test (a map made
with og.tilemap.create, filled, then one tick timed for the build and 60
for the steady tick; deleted after):

| Build | Map | Colliders | Build tick | Steady tick |
|---|---|---|---|---|
| release | 2D hex, merged | 1,264 | 21.6 to 22.9 ms | 0.26 to 0.29 ms |
| release | 2D orthogonal, merged rectangles | 15,074 | 23.0 to 23.8 ms | 1.73 to 1.77 ms |
| release | 2D isometric, merged | 4,494 | 15.4 to 16.2 ms | 0.85 ms |
| release | 2D hexFlat, merged | 1,374 | 21.1 ms | 0.28 to 0.30 ms |
| release | 3D hex, merged prisms | 1,264 | 75 ms | 0.26 ms |
| release | 3D orthogonal | 15,074 | 22.8 ms | 1.75 ms |
| dev | 2D hex, merged | 1,264 | 67 ms | 0.76 ms |
| dev | 2D orthogonal | 15,074 | 158 ms | 5.45 ms |
| dev | 3D hex, merged prisms | 1,264 | 148 ms | 0.87 ms |

Before the merge (per-cell hexagons, 32,631 colliders) the release steady
tick was 3.9 ms, nearly all of it og-physics' gather and sync touching
every static collider each tick (rapier's own step was 0.02 ms).

# Batch 3: formatting and time groups

## The time groups sample (`timegroups/`)

Two columns of five squares bob and pulse on looping tweens; the right
(orange) column has `og.TimeGroup { group: 1 }`. Group 1 runs at 1 for
0-2 s, 0.25 for 2-4 s, frozen 4-5.5 s, 1 again, then the global scale is
0.5 for 6-7.5 s (both columns slow: the group multiplies it). Under them
one locale per second (en-US, de-DE, fr-FR, es-ES) formats a compact
number, a euro price, a long date with a medium time, and a duration, from
a fixed moment plus the tick.

    og.darwin-arm64 run tests/evidence/g2-6/timegroups --headless --size 1280x720 --out /tmp/tgs \
      --commands "1.6:screenshot:a-together.png;3.3:screenshot:b-quarter.png;5:screenshot:c-frozen.png;7:screenshot:d-global-half.png"
    og.darwin-arm64 render tests/evidence/g2-6/timegroups --commands "0:get:show:Show:step" \
      --fps 30 --from 0 --to 8 --size 1280x720 --out /tmp/tgs/timegroups.mp4

Read on the Mac satellite (metal):

- `a-together.png` (tick 96): both columns in the same wave.
- `b-quarter.png` (tick 198): "group 1 x 0.25"; the orange wave lags the
  blue one; the es-ES lines read `1,4 M`, `1234,50 €` (es groups only from
  five digits), `2 de noviembre de 2026  15:08:09`.
- `c-frozen.png` (tick 300): "group 1 frozen"; the orange column holds a
  pose the blue one has left; en-US `1.5M`, `€1,234.50`.
- `d-global-half.png` (tick 420): "global x 0.5"; both columns slower.
- `timegroups.mp4` (8 s): the whole schedule.

Under each column a fountain of sparks (og.ParticleSystem, `fx/sparks2d.ogt`)
plays; the right one has og.TimeGroup { group: 1 }, so its sparks rise,
fall and fade at the group's pace. Measured on the clip (pixels changed
by more than 6 % between frames, each half of the frame below the title):

| interval | left (global pace) | right (group 1) |
|---|---|---|
| frames 60 to 66, both at 1 | 18,799 | 11,415 |
| frames 95 to 96, group 1 at 0.25 | 7,134 | 3,045 |
| frames 128 to 134, group 1 frozen | 18,372 | 211 |

`sparks-quarter-pair.png` crops the two quarter-pace frames: the left
sparks move, the right ones barely.

The U+202F gap is closed: the font cook now adds the Unicode spaces a
font lacks, so the frames read `3:05:10 PM` (en) and `1 234,50 €` (fr).

Hashes (orb, `OG_RENDERER=none`, 480 ticks): c0a9f858736f1ded twice, with the sparks (d60046cefb17078a before them).

## The CLDR subset's size

`crates/og-i18n/src/cldr.rs` (CLDR 49.0.0, 46 locales, 40 currencies) is
63,730 bytes of source. Its data in the wasm, estimated from the
wasm32 layout (a 676-byte LocaleData of (ptr, len) strings, 16 bytes per
symbol override, 20 per currency) plus every string's UTF-8 bytes, with no
deduplication counted (an upper bound):

| | bytes |
|---|---|
| per locale | 946 (en) to 1,660 (my); mean 1,308 |
| 46 locales | 60,151 |
| 40 currencies | 1,035 |
| total data | 61,186 |

Per locale: am 1297, ar 1554, be 1363, bg 1614, bn 1598, ca 1252, cs 1195,
da 1170, de 1065, el 1354, en 946, es 1363, et 1081, eu 1127, fa 1503,
fi 1371, fr 1324, gl 1098, gu 1617, he 1272, hi 1479, hu 1387, id 1108,
it 1209, ja 1042, km 1587, kn 1505, ko 1080, lo 1545, mr 1587, ms 1059,
my 1660, nb 1364, nl 1117, pl 1371, pt 1171, ru 1338, sk 1334, sv 1363,
th 1518, tr 1059, uk 1636, vi 1154, zh 1148, zh-Hant 1116, zu 1050. The
formatters' code adds a few KB; the wave's end measures the wasm itself.
