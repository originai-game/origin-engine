# og-tilemap

Tilemaps for the engine (E4): tilesets with per-tile collision, animated
tiles and auto-tiling terrains (47-blob and 16-bitmask rules), layered maps
with parallax and repeat, and a placed map's state: deterministic runtime
edits that re-resolve the terrain neighbourhood, snapshots of only what
changed, collision rectangles merged into runs and 16×16 chunks of quads
for og-render's sprite batch.

```rust
let tileset = Arc::new(bundle.load_tileset("base/tiles/dungeon-set")?);
let asset = bundle.load_tilemap("base/maps/room")?;               // checked file.tilemap TS
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
(`src/cook/systems/tilemap.rs`, feature `cook`): they never reach the wasm.

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
| `Tileset { texture, tile_width, tile_height, columns, rows, margin, spacing, filter, tiles: BTreeMap<u32, TileDef>, terrains }` | `encode(out)`, `decode(bytes)`, `validate()`, `count()`, `terrain(name)`, `collision(tile)`, `uv(tile, texture_w, texture_h) -> [u0, v0, u1, v1]` (v down, inset 1/512 texel) |
| `TileDef { collision: Option<Collision>, anim: Option<TileAnim { frames, durations }> }`, `Collision::{Full, Rect { x, y, w, h }}` (px from the tile's top-left), `Filter::{Nearest, Linear}` | per-tile data |
| `Terrain { name, mode: TerrainMode::{Blob47, Bitmask16}, rules: BTreeMap<u8, u32>, default, edge_filled }`, `resolve(neighbours)`, `reduce_blob(mask)`, `mask::{N, NE, E, SE, S, SW, W, NW}` | auto-tiling |
| `TilemapAsset { width, height, tile_width_px, tile_height_px, unit, tilesets: Vec<TilesetRef>, layers, objects }` | `validate()`, `tile_size()`, `world_size()`, `layer(name)` |
| `TilesetRef::{Asset(id), Embedded(Tileset)}` | a cooked tileset ID, or one an importer made |
| `Layer { name, tileset, cells, terrain: Option<String>, parallax, offset, opacity, visible, collision, collider_layers, collider_mask, repeat_x, repeat_y, z, lit }`, `Layer::new(name, w, h, z)` | cells: tile layers 0 empty else `(tile + 1) | FLIP_X | FLIP_Y`; terrain layers 0/1 |
| `FLIP_X` (bit 31), `FLIP_Y` (bit 30), `FLIP_DIAGONAL` (bit 29, refused), `TILE_MASK` | the cell encoding (Tiled's gid flip bits) |
| `MapObject { name, kind, x, y, width, height, properties }` | spawns, doors, triggers |
| `MapState` | `new(asset, tilesets)`, `get(layer, x, y) -> i32` (tile id or −1; terrain 1/0), `tile(layer, x, y) -> Option<(tile, flip_x, flip_y)>` (resolved), `set(layer, x, y, value)`, `layer_version`, `chunk_version(layer, cx, cy)`, `chunks()`, `encode(&mut Vec<u8>)`, `decode(asset, tilesets, bytes)`, `asset()`, `tileset(layer)`; `==` compares cells |
| `collision_rects(&state, layer)`, `collision_rects_all(&state)` → `Vec<Rect { min, max }>`; `map_colliders(&state, default_layers)` → `Vec<MapCollider { rect, layers, mask }>` (each layer's `collider_layers`, 0 the owner's, and `collider_mask`, 0 every layer) | merged static colliders |
| `chunk_quads(&state, layer, cx, cy, texture_w, texture_h) -> ChunkQuads { quads: Vec<TileQuad { min, size, uv }>, animated: Vec<AnimatedQuad { quad, frames, durations }> }`, `chunks_in(&state, layer, view_min, view_max)`, `animated_frame(durations, seconds)`, `CHUNK` = 16 | drawing |
| `world_to_cell(asset, local) -> Option<(x, y)>`, `cell_center(asset, x, y)` | coordinates |

## Checked TypeScript tilesets

`content/tiles/dungeon-set.ts` is the sole author entry. The cook checks
canonical references, then writes native fields used by the bundle loader,
embedded Tiled/LDtk tilesets and world snapshots. No text parser remains.

```ts
import { file, ref } from "@origingame/engine/author";
export default file.tileset({
  texture: ref("base/tiles/dungeon"), // omit for collision-only grids
  tileWidth: 16, tileHeight: 16, columns: 8, rows: 8,
  margin: 0, spacing: 0, filter: "nearest",
  tiles: {
    1: { collision: { kind: "full" } },
    9: {
      collision: { kind: "rect", x: 0, y: 8, w: 16, h: 8 },
      animation: { frames: [9, 10, 11, 10], durations: [1/6, 1/6, 1/6, 1/6] },
    },
  },
  terrains: [{ name: "wall", mode: "blob47", rules: { 255: 12 }, default: 12, edgeFilled: true }],
});
```

**Masks.** bitmask16: N = 1, E = 2, S = 4, W = 8 — the edge neighbours
filled with the same terrain (16 masks). blob47: N = 1, NE = 2, E = 4,
SE = 8, S = 16, SW = 32, W = 64, NW = 128, with a corner bit kept only when
both of its edge bits are set (`reduce_blob`), so 47 distinct masks occur; a
rule naming any other mask is refused at the TS declaration. Filled cells resolve
to `rules[mask]` or `default`; empty cells draw nothing.

## Checked TypeScript maps

`content/maps/room.ts` exports the complete declaration. Imports and explicit
`input.bytes/text` reads enter the shared deterministic cook cache. No map
author text parser or generation wrapper remains. Tiled and LDtk are external
import formats and feed the same native writer.

```ts
import { file, ref } from "@origingame/engine/author";
export default file.tilemap({
  width: 32, height: 18, tileWidth: 16, tileHeight: 16, unit: 1,
  orientation: "orthogonal", // also isometric, staggered, hex, hexFlat
  tilesets: [ref("base/tiles/dungeon-set")],
  layers: [{
    name: "floor", tileset: 0, z: -10,
    parallax: [0.5, 1], offset: [0, 0], opacity: 1, visible: true,
    collision: true, colliderLayers: 4, colliderMask: 3,
    repeatX: true, terrain: "wall",
    cells: new Uint32Array(32 * 18).fill(1), // terrain occupancy, not tile IDs
  }],
  objects: [{ name: "door", kind: "door", x: 15, y: -8,
    width: 1, height: 2, properties: { target: "room2" } }],
});
```

Tile layers use packed u32 cells, zero empty and otherwise tile ID + 1 with
optional `FLIP_X` / `FLIP_Y`; terrain layers use 0/1. Every layer has exactly
width × height cells and a unique name. Maps allow 1..16,777,216 cells.
Objects and offsets are map-local world units, **not** cell coordinates.

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

`cargo test -p og-tilemap` checks deterministic edits, snapshots and native
map round trips. The public cook/load contract in
`crates/og-cli/tests/cli/generating.rs` covers checked TS maps, packed cells,
orientations, metadata, object coordinates, tracked dependency invalidation
and rejected grids. `crates/og-cook/tests/template.rs` covers tilesets,
external-map bindings and embedded tilesets through bundles and snapshots.
