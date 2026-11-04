<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tilemap` module

Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component.

Layers draw through the sprite batch in 16x16-tile chunks culled to the view, each at the map's sprite layer plus its z, scrolled by its parallax; solid tiles become static og-physics colliders merged into rectangles. Runtime edits (og.tilemap.setTile, re-auto-tiling the neighbourhood) are world state (the og.tilemap state block), and the colliders follow them.

With og.Lighting2D enabled, a layer whose sprite layer (the map's layer plus its z) lies in minLayer to maxLayer is lit as the sprites there are: multiplied by the ambient plus every Light2D reaching it, with the shadows of Occluder2Ds and castShadow sprites, so a dark cave lit by torches is the map, its lights and its occluders. A layer that should stay as authored in the lit range (a sky, a far parallax backdrop) opts out: a bool custom property `lit` set to false on the Tiled layer, or a `lit false` line in a tilemap `.ogt`'s layer block (TileLayerInfo.lit reports it).

Cells count from the top-left corner, x right and y down (Tiled's); the map's top-left corner is at the entity's position.

Orientations: a map is orthogonal, isometric (diamonds on a turned grid), staggered (diamonds in rows) or hexagonal (pointy- or flat-top), from its `orientation` line or its Tiled map. cellAt, cellCenter and neighbours follow it; an isometric or staggered tile's image stands up from its cell's bottom (taller tiles overlap the ones behind, drawn back to front); solid tiles off the orthogonal grid collide as one collider per connected solid region, its outline (a hollow polyline in 2D, a prism in 3D; a layer's cellColliders makes it one diamond or hexagon per cell). Building them is one-time per map edit or resize; in 3D a non-orthogonal map's prisms build in about 75 ms per 400,000 triangles (a 256 × 256 hex map, half solid at random, release), so keep large 3D hex and iso collision layers in large solid regions, or edit them rarely. Stepping costs per region, not per cell. Terrain layers need a square grid (orthogonal or isometric).

Run-time maps: og.tilemap.create makes a map from a desc and og.tilemap.resize changes a map's size, both in game code. A map made or resized is world state: the og.tilemap state block saves it whole (its tilesets by asset id), so a save restores it in a session that never made it.

- **TypeScript**: functions are the namespace `og.tilemap`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-2d`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module tilemap {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 12061</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`TilemapAsset`](#type-TilemapAsset) | handle | A loaded map: its layers, tilesets and objects. |
| [`TilemapAsset()`](#column-TilemapAsset) | column kind | A TilemapAsset handle field. |
| [`og.Tilemap`](#record-Tilemap) | component | Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it. |
| [`TileOrientation`](#type-TileOrientation) | enum | How a map's cells are laid out (Tiled's orientations). |
| [`TileLayerInfo`](#type-TileLayerInfo) | struct | One layer of a map. |
| [`TileObject`](#type-TileObject) | struct | An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers. |
| [`TilemapInfo`](#type-TilemapInfo) | struct | What og.load.tilemap loaded. |
| [`TileLayerDesc`](#type-TileLayerDesc) | desc struct | One layer of a map og.tilemap.create makes (empty). |
| [`TilemapDesc`](#type-TilemapDesc) | desc struct | A map to make at run time: empty layers on tilesets the bundle cooked. |
| [`ResizeDesc`](#type-ResizeDesc) | desc struct | A new size for a map, and where its cells go. |
| [`SetTileDesc`](#type-SetTileDesc) | desc struct | One cell to change. |
| [`TileCell`](#type-TileCell) | struct | A cell of a map. |
| [`og.tilemap.setTile`](#fn-setTile) | function | Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps. |
| [`FillDesc`](#type-FillDesc) | desc struct | A block of cells to write at once: a generated grid (og.procgen's BSP cells or WFC tiles mapped to tile ids). |
| [`og.tilemap.fill`](#fn-fill) | function | Write a block of cells, as setTile on each (row by row) but re-tiling a terrain layer and rebuilding the colliders once. Cells past the map's edge are skipped. |
| [`og.tilemap.create`](#fn-create) | function | Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). Its handle comes from the world's own counter, so equal calls in equal order give equal handles, in a replay and after a restore. A world that makes none saves and hashes as before. |
| [`og.tilemap.resize`](#fn-resize) | function | Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). Every og.Tilemap showing it keeps its edits, moved with the cells by (x, y); cells past the new edges are dropped, new ones are empty, terrain re-tiles, colliders rebuild at the og.tilemap system and its objects (TilemapInfo.objects) move with the cells. |
| [`og.tilemap.neighbours`](#fn-neighbours) | function | The cells next to a cell, clockwise from the top (or the top right): 4 on an orthogonal or isometric grid (the sides), 4 on a staggered one (the diagonal sides), 6 on a hex one; cells off the map are left out. |
| [`og.tilemap.getTile`](#fn-getTile) | function | A cell's tile id, -1 when empty: on a terrain layer the tile auto-tiling resolved from the neighbours (setTile writes 1 or 0 there; the id changes with them). -1 outside the map or before the map is adopted. |
| [`og.tilemap.cellAt`](#fn-cellAt) | function | The cell under a world point (its xy), for a map at its entity's position. |
| [`og.tilemap.cellCenter`](#fn-cellCenter) | function | A cell's centre in world coordinates (z of the map). |

## Functions

<a id="fn-setTile"></a>

### `og.tilemap.setTile`

Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps.

```ts
og.tilemap.setTile(world: World, desc?: SetTileDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SetTileDesc` | yes | The map, the layer, the cell and the tile. |

**Returns** `Result<void>`: Refused for an entity whose map is not adopted yet, an unknown layer, a cell outside the map or a tile the tileset lacks.

Binding symbol `og_tilemap_set_tile` · publish builds: kept

```ogt
fn setTile {
  doc "Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps."
  param world World "The world."
  param desc SetTileDesc "The map, the layer, the cell and the tile."
  returns Result.Void "Refused for an entity whose map is not adopted yet, an unknown layer, a cell outside the map or a tile the tileset lacks."
}
```

<sub>`api.ogt` line 12245</sub>

<a id="fn-fill"></a>

### `og.tilemap.fill`

Write a block of cells, as setTile on each (row by row) but re-tiling a terrain layer and rebuilding the colliders once. Cells past the map's edge are skipped.

```ts
og.tilemap.fill(world: World, desc?: FillDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FillDesc` | yes | The map, the layer, where and the cells. |

**Returns** `Result<void>`: Refused for an entity whose map is not adopted yet, an unknown layer, a width of 0 or not dividing the cells, or a tile the tileset lacks (nothing is written then).

Binding symbol `og_tilemap_fill` · publish builds: kept

```ogt
fn fill {
  doc "Write a block of cells, as setTile on each (row by row) but re-tiling a terrain layer and rebuilding the colliders once. Cells past the map's edge are skipped."
  param world World "The world."
  param desc FillDesc "The map, the layer, where and the cells."
  returns Result.Void "Refused for an entity whose map is not adopted yet, an unknown layer, a width of 0 or not dividing the cells, or a tile the tileset lacks (nothing is written then)."
}
```

<sub>`api.ogt` line 12263</sub>

<a id="fn-create"></a>

### `og.tilemap.create`

Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). Its handle comes from the world's own counter, so equal calls in equal order give equal handles, in a replay and after a restore. A world that makes none saves and hashes as before.

```ts
og.tilemap.create(world: World, desc?: TilemapDesc): Result<TilemapInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TilemapDesc` | yes | Its size, tiles, orientation, tilesets and layers. |

**Returns** `Result<TilemapInfo>`: Refused for a size of 0 or past 16,777,216 cells, a tileset the bundle lacks, a layer naming a tileset index or terrain that does not exist, two layers of one name, or a terrain layer on a staggered or hex map.

Binding symbol `og_tilemap_create` · publish builds: kept

```ogt
fn create {
  doc "Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). Its handle comes from the world's own counter, so equal calls in equal order give equal handles, in a replay and after a restore. A world that makes none saves and hashes as before."
  param world World "The world."
  param desc TilemapDesc "Its size, tiles, orientation, tilesets and layers."
  returns Result.TilemapInfo "Refused for a size of 0 or past 16,777,216 cells, a tileset the bundle lacks, a layer naming a tileset index or terrain that does not exist, two layers of one name, or a terrain layer on a staggered or hex map."
}
```

<sub>`api.ogt` line 12270</sub>

<a id="fn-resize"></a>

### `og.tilemap.resize`

Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). Every og.Tilemap showing it keeps its edits, moved with the cells by (x, y); cells past the new edges are dropped, new ones are empty, terrain re-tiles, colliders rebuild at the og.tilemap system and its objects (TilemapInfo.objects) move with the cells.

```ts
og.tilemap.resize(world: World, desc?: ResizeDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ResizeDesc` | yes | The map, its new size and the shift. |

**Returns** `Result<void>`: Refused for an unknown map, a size of 0 or past 16,777,216 cells, or an odd shift along a staggered or hex map's shifted axis (it would change which rows are shifted).

Binding symbol `og_tilemap_resize` · publish builds: kept

```ogt
fn resize {
  doc "Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). Every og.Tilemap showing it keeps its edits, moved with the cells by (x, y); cells past the new edges are dropped, new ones are empty, terrain re-tiles, colliders rebuild at the og.tilemap system and its objects (TilemapInfo.objects) move with the cells."
  param world World "The world."
  param desc ResizeDesc "The map, its new size and the shift."
  returns Result.Void "Refused for an unknown map, a size of 0 or past 16,777,216 cells, or an odd shift along a staggered or hex map's shifted axis (it would change which rows are shifted)."
}
```

<sub>`api.ogt` line 12277</sub>

<a id="fn-neighbours"></a>

### `og.tilemap.neighbours`

The cells next to a cell, clockwise from the top (or the top right): 4 on an orthogonal or isometric grid (the sides), 4 on a staggered one (the diagonal sides), 6 on a hex one; cells off the map are left out.

```ts
og.tilemap.neighbours(world: World, map: TilemapAsset, x: number, y: number): readonly TileCell[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `map` | `TilemapAsset` | no | The map. |
| `x` | `number` | no | The column. |
| `y` | `number` | no | The row. |

**Returns** `readonly TileCell[]`.

Binding symbol `og_tilemap_neighbours` · publish builds: kept

```ogt
fn neighbours {
  doc "The cells next to a cell, clockwise from the top (or the top right): 4 on an orthogonal or isometric grid (the sides), 4 on a staggered one (the diagonal sides), 6 on a hex one; cells off the map are left out."
  param world World "The world."
  param map TilemapAsset "The map."
  param x u32 "The column."
  param y u32 "The row."
  returns List.TileCell
}
```

<sub>`api.ogt` line 12284</sub>

<a id="fn-getTile"></a>

### `og.tilemap.getTile`

A cell's tile id, -1 when empty: on a terrain layer the tile auto-tiling resolved from the neighbours (setTile writes 1 or 0 there; the id changes with them). -1 outside the map or before the map is adopted.

```ts
og.tilemap.getTile(world: World, entity: Entity, layer: string, x: number, y: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Tilemap. |
| `layer` | `string` | no | The layer's name. |
| `x` | `number` | no | The column. |
| `y` | `number` | no | The row. |

**Returns** `number`.

Binding symbol `og_tilemap_get_tile` · publish builds: kept

```ogt
fn getTile {
  doc "A cell's tile id, -1 when empty: on a terrain layer the tile auto-tiling resolved from the neighbours (setTile writes 1 or 0 there; the id changes with them). -1 outside the map or before the map is adopted."
  param world World "The world."
  param entity Entity "The entity with the og.Tilemap."
  param layer String "The layer's name."
  param x u32 "The column."
  param y u32 "The row."
  returns i32
}
```

<sub>`api.ogt` line 12293</sub>

<a id="fn-cellAt"></a>

### `og.tilemap.cellAt`

The cell under a world point (its xy), for a map at its entity's position.

```ts
og.tilemap.cellAt(world: World, entity: Entity, point: Vector3): TileCell
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Tilemap. |
| `point` | `Vector3` | no | A world point. |

**Returns** `TileCell`.

Binding symbol `og_tilemap_cell_at` · publish builds: kept

```ogt
fn cellAt {
  doc "The cell under a world point (its xy), for a map at its entity's position."
  param world World "The world."
  param entity Entity "The entity with the og.Tilemap."
  param point Vector3 "A world point."
  returns TileCell
}
```

<sub>`api.ogt` line 12303</sub>

<a id="fn-cellCenter"></a>

### `og.tilemap.cellCenter`

A cell's centre in world coordinates (z of the map).

```ts
og.tilemap.cellCenter(world: World, entity: Entity, x: number, y: number): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity with the og.Tilemap. |
| `x` | `number` | no | The column. |
| `y` | `number` | no | The row. |

**Returns** `Vector3`.

Binding symbol `og_tilemap_cell_center` · publish builds: kept

```ogt
fn cellCenter {
  doc "A cell's centre in world coordinates (z of the map)."
  param world World "The world."
  param entity Entity "The entity with the og.Tilemap."
  param x u32 "The column."
  param y u32 "The row."
  returns Vector3
}
```

<sub>`api.ogt` line 12311</sub>

## Components, resources and events

<a id="record-Tilemap"></a>

### `og.Tilemap` (component)

Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it.

Token `og.Tilemap`, registered under the schema name `og.Tilemap`; its value type is `Tilemap`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `TilemapAsset` | `0` | — | — | — | The map (og.load.tilemap's `map`); 0 is none. |
| `layer` | `number` | `-10` | — | — | — | The sprite layer of the map's layer 0; every layer draws at this plus its z (default: its index). |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha, times each layer's. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `collision` | `boolean` | `true` | — | — | — | Give the layers marked collision static colliders (rectangles merged from the solid tiles). |
| `colliderLayers` | `number` | `1` | — | — | — | The collision layers of those colliders, as bits (Collider.layers), for every collision layer of the map that names none of its own (a map layer's `colliderLayers`, with its `colliderMask`, puts a layer of grates on a one-way layer while the walls stay here). |
| `friction` | `number` | `0.5` | 0 … 2 | — | — | The colliders' friction. |
| `palette` | `Texture` | `0` | — | — | — | A palette swap of every layer (og.Sprite.palette): the tile sets' index textures (the cook's `<id>#index`) drawn through row paletteRow of this texture of N rows; 0 is none. |
| `paletteRow` | `number` | `0` | — | — | — | The row of `palette` drawn (past the last, the last). |

```ogt
component Tilemap {
  doc "Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it."
  field map TilemapAsset "The map (og.load.tilemap's `map`); 0 is none."
  field layer i32 "The sprite layer of the map's layer 0; every layer draws at this plus its z (default: its index)." {
    default -10
  }
  field color Color "A linear tint." {
    default 1 1 1
  }
  field opacity f32 "The alpha, times each layer's." {
    default 1
    min 0
    max 1
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field collision bool "Give the layers marked collision static colliders (rectangles merged from the solid tiles)." {
    default true
  }
  field colliderLayers u32 "The collision layers of those colliders, as bits (Collider.layers), for every collision layer of the map that names none of its own (a map layer's `colliderLayers`, with its `colliderMask`, puts a layer of grates on a one-way layer while the walls stay here)." {
    default 1
  }
  field friction f32 "The colliders' friction." {
    default 0.5
    min 0
    max 2
  }
  field palette Texture "A palette swap of every layer (og.Sprite.palette): the tile sets' index textures (the cook's `<id>#index`) drawn through row paletteRow of this texture of N rows; 0 is none."
  field paletteRow u32 "The row of `palette` drawn (past the last, the last)."
}
```

<sub>`api.ogt` line 12079</sub>

## Types

<a id="type-TilemapAsset"></a>

### `TilemapAsset` (handle)

A loaded map: its layers, tilesets and objects.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TilemapAsset "A loaded map: its layers, tilesets and objects."
```

<sub>`api.ogt` line 12070</sub>

<a id="type-TileOrientation"></a>

### `TileOrientation` (enum)

How a map's cells are laid out (Tiled's orientations).

A string: one of the variants below. The zero value is `"orthogonal"`.

| Variant | Doc |
|---|---|
| `"orthogonal"` | Rectangles in rows and columns. |
| `"isometric"` | Diamonds on a turned grid: x runs down to the right, y down to the left, cell 0 0 at the top. |
| `"staggered"` | Diamonds in rows half a tile apart, every odd row shifted right half a tile (Tiled's staggered, odd rows). |
| `"hex"` | Pointy-top hexagons with corners at a quarter of the tile, rows 3/4 of a tile apart, every odd row shifted right half a tile (Tiled's hexagonal, stagger axis y, odd, side half the tile's height). |
| `"hexFlat"` | Flat-top hexagons, columns 3/4 of a tile apart, every odd column shifted down half a tile (stagger axis x). |

```ogt
enum TileOrientation {
  doc "How a map's cells are laid out (Tiled's orientations)."
  variant orthogonal "Rectangles in rows and columns."
  variant isometric "Diamonds on a turned grid: x runs down to the right, y down to the left, cell 0 0 at the top."
  variant staggered "Diamonds in rows half a tile apart, every odd row shifted right half a tile (Tiled's staggered, odd rows)."
  variant hex "Pointy-top hexagons with corners at a quarter of the tile, rows 3/4 of a tile apart, every odd row shifted right half a tile (Tiled's hexagonal, stagger axis y, odd, side half the tile's height)."
  variant hexFlat "Flat-top hexagons, columns 3/4 of a tile apart, every odd column shifted down half a tile (stagger axis x)."
}
```

<sub>`api.ogt` line 12111</sub>

<a id="type-TileLayerInfo"></a>

### `TileLayerInfo` (struct)

One layer of a map.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `index` | `number` | `0` | — | — | — | Its index (what drawing order and z start from). |
| `z` | `number` | `0` | — | — | — | Its sprite-layer offset from Tilemap.layer. |
| `parallax` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera. |
| `collision` | `boolean` | `false` | — | — | — | Its solid tiles collide. |
| `colliderLayers` | `number` | `0` | — | — | — | The collision layers (og.Collider.layers bits) of its colliders, from its `colliderLayers` line or Tiled int property; 0 (unset) takes Tilemap.colliderLayers. |
| `colliderMask` | `number` | `0` | — | — | — | The layers its colliders collide with (og.Collider.mask bits), from its `colliderMask` line or Tiled int property; 0 (unset) is every layer. |
| `cellColliders` | `boolean` | `false` | — | — | — | Off the orthogonal grid, whether each solid cell collides on its own (a diamond or hexagon per cell, from its `cellColliders` line or Tiled bool property) rather than each connected solid region as one collider, its outline (the default: a static map costs physics per region, not per cell). |
| `terrain` | `string` | `""` | — | — | — | The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer. |
| `visible` | `boolean` | `false` | — | — | — | Whether it is drawn. |
| `lit` | `boolean` | `false` | — | — | — | Whether og.Lighting2D lights it when its sprite layer is in the lit range (true unless the Tiled layer's bool property `lit` or the `.ogt` layer's `lit` line says false). |

```ogt
struct TileLayerInfo {
  doc "One layer of a map."
  field name String "Its name."
  field index u32 "Its index (what drawing order and z start from)."
  field z i32 "Its sprite-layer offset from Tilemap.layer."
  field parallax Vector2 "Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera."
  field collision bool "Its solid tiles collide."
  field colliderLayers u32 "The collision layers (og.Collider.layers bits) of its colliders, from its `colliderLayers` line or Tiled int property; 0 (unset) takes Tilemap.colliderLayers."
  field colliderMask u32 "The layers its colliders collide with (og.Collider.mask bits), from its `colliderMask` line or Tiled int property; 0 (unset) is every layer."
  field cellColliders bool "Off the orthogonal grid, whether each solid cell collides on its own (a diamond or hexagon per cell, from its `cellColliders` line or Tiled bool property) rather than each connected solid region as one collider, its outline (the default: a static map costs physics per region, not per cell)."
  field terrain String "The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer."
  field visible bool "Whether it is drawn."
  field lit bool "Whether og.Lighting2D lights it when its sprite layer is in the lit range (true unless the Tiled layer's bool property `lit` or the `.ogt` layer's `lit` line says false)."
}
```

<sub>`api.ogt` line 12120</sub>

<a id="type-TileObject"></a>

### `TileObject` (struct)

An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `string` | `""` | — | — | — | Its type or class. |
| `position` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its top-left corner, in world units relative to the map's top-left corner (y up, so below the corner is negative). |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its width and height (0 0 for a point). |
| `center` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its centre, relative to the map's corner. |
| `properties` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Its custom properties, as text. |

```ogt
struct TileObject {
  doc "An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers."
  field name String "Its name."
  field kind String "Its type or class."
  field position Vector2 "Its top-left corner, in world units relative to the map's top-left corner (y up, so below the corner is negative)." {
    unit m
  }
  field size Vector2 "Its width and height (0 0 for a point)." {
    unit m
  }
  field center Vector2 "Its centre, relative to the map's corner." {
    unit m
  }
  field properties Map.String "Its custom properties, as text."
}
```

<sub>`api.ogt` line 12135</sub>

<a id="type-TilemapInfo"></a>

### `TilemapInfo` (struct)

What og.load.tilemap loaded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `TilemapAsset` | `0` | — | — | — | What Tilemap.map takes. |
| `width` | `number` | `0` | — | — | — | Cells across. |
| `height` | `number` | `0` | — | — | — | Cells down. |
| `tileWidth` | `number` | `0` | — | `m` | — | A cell's width in world units. |
| `tileHeight` | `number` | `0` | — | `m` | — | A cell's height in world units. |
| `layers` | `readonly TileLayerInfo[]` | `[]` | — | — | — | Its layers, drawing order. |
| `objects` | `readonly TileObject[]` | `[]` | — | — | — | Its objects. |
| `orientation` | `TileOrientation` | `"orthogonal"` | — | — | — | How its cells are laid out. |

```ogt
struct TilemapInfo {
  doc "What og.load.tilemap loaded."
  field map TilemapAsset "What Tilemap.map takes."
  field width u32 "Cells across."
  field height u32 "Cells down."
  field tileWidth f32 "A cell's width in world units." {
    unit m
  }
  field tileHeight f32 "A cell's height in world units." {
    unit m
  }
  field layers List.TileLayerInfo "Its layers, drawing order."
  field objects List.TileObject "Its objects."
  field orientation TileOrientation "How its cells are laid out."
}
```

<sub>`api.ogt` line 12151</sub>

<a id="type-TileLayerDesc"></a>

### `TileLayerDesc` (desc struct)

One layer of a map og.tilemap.create makes (empty).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name (unique in the map). |
| `tileset` | `number` | `0` | — | — | — | The index of its tileset in TilemapDesc.tilesets. |
| `terrain` | `string` | `""` | — | — | — | An auto-tiling terrain of that tileset: a terrain layer (cells 1 or 0); empty for a tile layer. |
| `z` | `number` | `0` | — | — | — | Its sprite-layer offset from Tilemap.layer. |
| `collision` | `boolean` | `false` | — | — | — | Its solid tiles collide. |
| `colliderLayers` | `number` | `0` | — | — | — | Its colliders' collision layers (0: Tilemap.colliderLayers). |
| `colliderMask` | `number` | `0` | — | — | — | The layers its colliders collide with (0: every layer). |
| `cellColliders` | `boolean` | `false` | — | — | — | Off the orthogonal grid, one collider per solid cell (a diamond or hexagon) instead of one per connected solid region (its outline, the default). |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
struct TileLayerDesc {
  doc "One layer of a map og.tilemap.create makes (empty)."
  desc true
  field name String "Its name (unique in the map)."
  field tileset u32 "The index of its tileset in TilemapDesc.tilesets."
  field terrain String "An auto-tiling terrain of that tileset: a terrain layer (cells 1 or 0); empty for a tile layer."
  field z i32 "Its sprite-layer offset from Tilemap.layer."
  field collision bool "Its solid tiles collide."
  field colliderLayers u32 "Its colliders' collision layers (0: Tilemap.colliderLayers)."
  field colliderMask u32 "The layers its colliders collide with (0: every layer)."
  field cellColliders bool "Off the orthogonal grid, one collider per solid cell (a diamond or hexagon) instead of one per connected solid region (its outline, the default)."
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 12167</sub>

<a id="type-TilemapDesc"></a>

### `TilemapDesc` (desc struct)

A map to make at run time: empty layers on tilesets the bundle cooked.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `16` | ≥ 1 | — | — | Cells across. |
| `height` | `number` | `16` | ≥ 1 | — | — | Cells down. |
| `tileWidth` | `number` | `16` | ≥ 1 | — | — | A tile's width in pixels (its cell's). |
| `tileHeight` | `number` | `16` | ≥ 1 | — | — | A tile's height in pixels (its cell's; half the width for the usual isometric diamond). |
| `unit` | `number` | `1` | — | — | — | World units per tile width. |
| `orientation` | `TileOrientation` | `"orthogonal"` | — | — | — | How the cells are laid out. |
| `tilesets` | `readonly string[]` | `[]` | — | — | — | The tilesets' asset ids (tileset `.ogt` files, such as `tiles/dungeon.ogt`). |
| `layers` | `readonly TileLayerDesc[]` | `[]` | — | — | — | The layers, drawing order. |

```ogt
struct TilemapDesc {
  doc "A map to make at run time: empty layers on tilesets the bundle cooked."
  desc true
  field width u32 "Cells across." {
    default 16
    min 1
  }
  field height u32 "Cells down." {
    default 16
    min 1
  }
  field tileWidth u32 "A tile's width in pixels (its cell's)." {
    default 16
    min 1
  }
  field tileHeight u32 "A tile's height in pixels (its cell's; half the width for the usual isometric diamond)." {
    default 16
    min 1
  }
  field unit f32 "World units per tile width." {
    default 1
  }
  field orientation TileOrientation "How the cells are laid out."
  field tilesets List.String "The tilesets' asset ids (tileset `.ogt` files, such as `tiles/dungeon.ogt`)."
  field layers List.TileLayerDesc "The layers, drawing order."
}
```

<sub>`api.ogt` line 12183</sub>

<a id="type-ResizeDesc"></a>

### `ResizeDesc` (desc struct)

A new size for a map, and where its cells go.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `TilemapAsset` | `0` | — | — | — | The map: og.load.tilemap's or og.tilemap.create's. |
| `width` | `number` | `16` | ≥ 1 | — | — | Cells across. |
| `height` | `number` | `16` | ≥ 1 | — | — | Cells down. |
| `x` | `number` | `0` | — | — | — | Where the old cell 0 0 lands: columns added on the left (negative crops them). |
| `y` | `number` | `0` | — | — | — | Rows added at the top (negative crops them). |

```ogt
struct ResizeDesc {
  doc "A new size for a map, and where its cells go."
  desc true
  field map TilemapAsset "The map: og.load.tilemap's or og.tilemap.create's."
  field width u32 "Cells across." {
    default 16
    min 1
  }
  field height u32 "Cells down." {
    default 16
    min 1
  }
  field x i32 "Where the old cell 0 0 lands: columns added on the left (negative crops them)."
  field y i32 "Rows added at the top (negative crops them)."
}
```

<sub>`api.ogt` line 12210</sub>

<a id="type-SetTileDesc"></a>

### `SetTileDesc` (desc struct)

One cell to change.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Tilemap. |
| `layer` | `string` | `""` | — | — | — | The layer's name. |
| `x` | `number` | `0` | — | — | — | The column (0 is the left). |
| `y` | `number` | `0` | — | — | — | The row (0 is the top). |
| `tile` | `number` | `0` | ≥ -1 | — | — | A tile layer: the tile id, -1 empty. A terrain layer: 1 fills, 0 clears, and the neighbours re-tile. |

```ogt
struct SetTileDesc {
  doc "One cell to change."
  desc true
  field entity Entity "The entity with the og.Tilemap."
  field layer String "The layer's name."
  field x u32 "The column (0 is the left)."
  field y u32 "The row (0 is the top)."
  field tile i32 "A tile layer: the tile id, -1 empty. A terrain layer: 1 fills, 0 clears, and the neighbours re-tile." {
    min -1
  }
}
```

<sub>`api.ogt` line 12226</sub>

<a id="type-TileCell"></a>

### `TileCell` (struct)

A cell of a map.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column. |
| `y` | `number` | `0` | — | — | — | The row. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point was on the map. |

```ogt
struct TileCell {
  doc "A cell of a map."
  field x u32 "The column."
  field y u32 "The row."
  field inside bool "Whether the point was on the map."
}
```

<sub>`api.ogt` line 12238</sub>

<a id="type-FillDesc"></a>

### `FillDesc` (desc struct)

A block of cells to write at once: a generated grid (og.procgen's BSP cells or WFC tiles mapped to tile ids).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Tilemap. |
| `layer` | `string` | `""` | — | — | — | The layer's name. |
| `x` | `number` | `0` | — | — | — | The block's left column. |
| `y` | `number` | `0` | — | — | — | The block's top row. |
| `width` | `number` | `0` | — | — | — | The block's width in cells: `tiles` is row-major, `width` per row. |
| `tiles` | `readonly number[]` | `[]` | — | — | — | The cells, as og.tilemap.setTile's `tile`: a tile id or -1 on a tile layer, 1 or 0 on a terrain layer (re-tiled once, after every cell is written). |

```ogt
struct FillDesc {
  doc "A block of cells to write at once: a generated grid (og.procgen's BSP cells or WFC tiles mapped to tile ids)."
  desc true
  field entity Entity "The entity with the og.Tilemap."
  field layer String "The layer's name."
  field x u32 "The block's left column."
  field y u32 "The block's top row."
  field width u32 "The block's width in cells: `tiles` is row-major, `width` per row."
  field tiles List.i32 "The cells, as og.tilemap.setTile's `tile`: a tile id or -1 on a tile layer, 1 or 0 on a terrain layer (re-tiled once, after every cell is written)."
}
```

<sub>`api.ogt` line 12252</sub>

## Column kinds

Each is a factory, `TilemapAsset(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TilemapAsset"></a>`TilemapAsset` | `TilemapAsset` | `Uint32Array` | 2 | A TilemapAsset handle field. |

```ogt
column TilemapAsset {
  doc "A TilemapAsset handle field."
  array Uint32Array
  stride 2
  value TilemapAsset
}
```

<sub>`api.ogt` line 12072</sub>
