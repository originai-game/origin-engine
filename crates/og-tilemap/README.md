# og-tilemap

Tilemaps for the engine (E4): tilesets with per-tile collision, animated
tiles and auto-tiling terrains (47-blob and 16-bitmask rules), layered maps
with parallax and repeat, and a placed map's state: deterministic runtime
edits that re-resolve the terrain neighbourhood, snapshots of only what
changed, collision rectangles merged into runs and 16×16 chunks of quads
for og-render's sprite batch.

```rust
let tileset = Arc::new(Tileset::parse(&text)?);                   // or og_cook: bundle.load_tileset(id)
let asset = TilemapAsset::parse(&map_text)?;                      // or bundle.load_tilemap("maps/room.tmj")
let mut state = MapState::new(&asset, &[tileset.clone()])?;       // tilesets in asset.tilesets order
state.set(walls, 4, 7, 1)?;                                       // fill a terrain cell; its 3×3 neighbourhood re-resolves
let rects = collision_rects_all(&state);                          // static colliders, map-local world units
for (cx, cy) in chunks_in(&state, layer, view_min, view_max) {    // culling
    if state.chunk_version(layer, cx, cy) != cached { rebuild(chunk_quads(&state, layer, cx, cy, tex_w, tex_h)); }
}
let mut bytes = Vec::new(); state.encode(&mut bytes);             // a state block: the cells that differ from the asset
let back = MapState::decode(&asset, &[tileset], &bytes)?;         // == state
```

Dependencies: og-core, glam. No serde, no C, builds for
`wasm32-unknown-unknown`. The Tiled and LDtk importers live in og-cook
(`src/cook/tilemap.rs`, feature `cook`): they never reach the wasm.

## Coordinates

Cells are `(x, y)`, x right, y **down**, row 0 the top row (Tiled's). A map
is placed with its top-left corner at its owner's position; world
coordinates here are **map-local, y up** from that corner, so cell `(x, y)`
covers x in `[x·tw, (x+1)·tw)` and y in `(−(y+1)·th, −y·th]`, where
`tw = unit` (world units per tile width, default 1) and
`th = unit · tile_height_px / tile_width_px`. Objects' `x, y` are their
top-left corner in the same coordinates (y ≤ 0 inside the map).

## API map

| Item | What it is |
|---|---|
| `Tileset { texture, tile_width, tile_height, columns, rows, margin, spacing, filter, tiles: BTreeMap<u32, TileDef>, terrains }` | `parse(text)`, `to_text()` (canonical; parses back equal), `validate()`, `count()`, `terrain(name)`, `collision(tile)`, `uv(tile, texture_w, texture_h) -> [u0, v0, u1, v1]` (v down, inset 1/512 texel) |
| `TileDef { collision: Option<Collision>, anim: Option<TileAnim { frames, durations }> }`, `Collision::{Full, Rect { x, y, w, h }}` (px from the tile's top-left), `Filter::{Nearest, Linear}` | per-tile data |
| `Terrain { name, mode: TerrainMode::{Blob47, Bitmask16}, rules: BTreeMap<u8, u32>, default, edge_filled }`, `resolve(neighbours)`, `reduce_blob(mask)`, `mask::{N, NE, E, SE, S, SW, W, NW}` | auto-tiling |
| `TilemapAsset { width, height, tile_width_px, tile_height_px, unit, tilesets: Vec<TilesetRef>, layers, objects }` | `parse(text)`, `validate()`, `tile_size()`, `world_size()`, `layer(name)` |
| `TilesetRef::{Asset(id), Embedded(Tileset)}` | an asset id of a tileset `.ogt`, or one an importer made |
| `Layer { name, tileset, cells, terrain: Option<String>, parallax, offset, opacity, visible, collision, collider_layers, collider_mask, repeat_x, repeat_y, z, lit }`, `Layer::new(name, w, h, z)` | cells: tile layers 0 empty else `(tile + 1) | FLIP_X | FLIP_Y`; terrain layers 0/1 |
| `FLIP_X` (bit 31), `FLIP_Y` (bit 30), `FLIP_DIAGONAL` (bit 29, refused), `TILE_MASK` | the cell encoding (Tiled's gid flip bits) |
| `MapObject { name, kind, x, y, width, height, properties }` | spawns, doors, triggers |
| `MapState` | `new(asset, tilesets)`, `get(layer, x, y) -> i32` (tile id or −1; terrain 1/0), `tile(layer, x, y) -> Option<(tile, flip_x, flip_y)>` (resolved), `set(layer, x, y, value)`, `layer_version`, `chunk_version(layer, cx, cy)`, `chunks()`, `encode(&mut Vec<u8>)`, `decode(asset, tilesets, bytes)`, `asset()`, `tileset(layer)`; `==` compares cells |
| `collision_rects(&state, layer)`, `collision_rects_all(&state)` → `Vec<Rect { min, max }>`; `map_colliders(&state, default_layers)` → `Vec<MapCollider { rect, layers, mask }>` (each layer's `collider_layers`, 0 the owner's, and `collider_mask`, 0 every layer) | merged static colliders |
| `chunk_quads(&state, layer, cx, cy, texture_w, texture_h) -> ChunkQuads { quads: Vec<TileQuad { min, size, uv }>, animated: Vec<AnimatedQuad { quad, frames, durations }> }`, `chunks_in(&state, layer, view_min, view_max)`, `animated_frame(durations, seconds)`, `CHUNK` = 16 | drawing |
| `world_to_cell(asset, local) -> Option<(x, y)>`, `cell_center(asset, x, y)` | coordinates |

## `.ogt` of `type tileset`

```text
type tileset
version 1
---
texture "tiles/dungeon.png"   # asset id of the atlas image
grid 16 16 8 8                # tile width, height (px), columns, rows
margin 0                      # optional, px around the grid
spacing 0                     # optional, px between tiles
filter nearest                # nearest (default) | linear
solid 1 2 3 4                 # full-tile colliders (any number of lines)
tile 9 {
  collide 0 8 16 8            # a rect in px from the tile's top-left (one covering the tile is full)
  frames 9 10 11 10           # animated: these tile ids in turn
  fps 6                       # or `durations 0.1 0.2 ...`, seconds per frame
}
terrain wall {
  mode blob47                 # blob47 | bitmask16
  rule 255 12                 # neighbour mask -> tile id
  default 12                  # tile for a mask no rule lists
  edge filled                 # filled (default): off-map neighbours count as filled | empty
}
```

**Masks.** bitmask16: N = 1, E = 2, S = 4, W = 8 — the edge neighbours
filled with the same terrain (16 masks). blob47: N = 1, NE = 2, E = 4,
SE = 8, S = 16, SW = 32, W = 64, NW = 128, with a corner bit kept only when
both of its edge bits are set (`reduce_blob`), so 47 distinct masks occur; a
`rule` naming any other mask is refused with its line. Filled cells resolve
to `rules[mask]` or `default`; empty cells draw nothing.

## `.ogt` of `type tilemap`

```text
type tilemap
version 1
---
size 32 18                 # cells
tile 16 16                 # px
unit 1                     # world units per tile width (optional)
tileset "tiles/dungeon.ogt"
layer floor {
  tileset 0                # optional, default 0
  z -10                    # sprite layer offset (default: the layer's index)
  parallax 0.5 1           # optional
  offset 0 0               # optional, world units
  opacity 1                # optional
  visible true             # optional
  collision true           # optional: its tiles' shapes become colliders
  colliderLayers 4         # optional: those colliders' layer bits (default: the owner's)
  colliderMask 3           # optional: the layers they collide with (default: every layer)
  repeat x                 # optional: x, y or both
  terrain wall             # optional: a terrain layer (cells 1 filled)
  legend "#" 1             # a character and the tile id (terrain: 1 filled); "." is empty unless given
  row "################"   # one per map row, top first, exactly `size` wide
  # numeric rows: row 3 3 4 -1 5   (tile ids, -1 empty; terrain 1/0)
}
object door {
  kind "door"
  rect 15 8 1 2            # x y w h in cells (y down) -> world units; or `at x y` for a point
  prop target "room2"
}
```

## Invariants

- **Deterministic.** Terrain resolution, edits, collision merging, chunk
  building and encoding are functions of the asset and the edit sequence;
  no hash map is iterated. Two equal edit sequences give equal states and
  equal `encode` bytes.
- **Snapshots** hold only cells that differ from the asset's (16 bytes for an
  untouched map of any size, 12 per edited cell): width, height, layer count
  and edit count, then `(layer, index, value)` ascending, little-endian.
  `decode` refuses bytes saved for another size or layer count, and
  re-resolves terrain from the cells.
- **Edits** (`set`): tile layers take a tile id of their tileset or −1;
  terrain layers 1 or 0, and re-resolve the 3×3 neighbourhood; `get`
  answers the tile drawn there (a terrain cell's resolved tile). Every change
  to a drawn tile bumps the layer's and its chunk's version (setting a cell
  to its current value bumps nothing).
- **Collision.** Full-tile solids (`solid`, or a `collide` rect covering the
  tile) merge greedily: runs along each row, then runs with the same columns
  on consecutive rows into one rectangle. Partial `collide` rects are one per
  tile (flips applied), except identical full-width ones next to each other
  on a row, which merge. Rectangles include the layer's offset and come top
  to bottom, then left to right. Only layers with `collision` count in
  `collision_rects_all`.
- **Chunks** are `CHUNK` = 16 cells square. `chunk_quads` excludes the
  layer's offset and parallax (the drawer adds them per frame); `chunks_in`
  takes a map-local rectangle and subtracts the layer's offset itself.
  Animated quads carry every frame's region; `animated_frame` loops.
- **Validation**: tile ids inside the grid, animations with frames and
  positive durations, cells of the map's size, layers naming existing
  tilesets and (in `MapState::new`) existing terrains, no diagonal flips.

## Tests

`cargo test -p og-tilemap`: `tests/autotile.rs` (47 reduced masks; an
isolated cell and a 3×3 block's centre, edges and corners resolve to their
rule tiles; a fill then clear restores the neighbourhood; edge filled vs
empty; bitmask16; refused masks and terrains), `tests/collision.rs` (an L
shape is two rectangles, a room outline four and a full block one; partial
rects per tile and merged full-width ones; only collision layers, with
their offsets), `tests/state.rs` (equal edits give equal bytes, decode
restores them; an untouched 256×256 four-layer map snapshots in 16 bytes;
edit checks and chunk versions), `tests/chunks.rs` (quads with flips and
animations, inset regions with margin and spacing, culling),
`tests/text.rs` (tileset round trip and refusals by line; a hand-written map
with legends, numeric rows, objects and coordinates). The cook side:
`crates/og-cook/tests/tilemap.rs`. Each was checked by breaking the code it
covers.
