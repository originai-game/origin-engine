<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tilemap` module

Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component.

Layers draw through the sprite batch in 16x16-tile chunks culled to the view, each at the map's sprite layer plus its z, scrolled by its parallax; solid tiles become static og-physics colliders merged into rectangles. Runtime edits (og.tilemap.setTile, re-auto-tiling the neighbourhood) are world state (the og.tilemap state block), and the colliders follow them.

Cells count from the top-left corner, x right and y down (Tiled's); the map's top-left corner is at the entity's position.

- **TypeScript**: functions are the namespace `og.tilemap`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module tilemap {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 6090</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`TilemapAsset`](#type-TilemapAsset) | handle | A loaded map: its layers, tilesets and objects. |
| [`TilemapAsset()`](#column-TilemapAsset) | column kind | A TilemapAsset handle field. |
| [`og.Tilemap`](#record-Tilemap) | component | Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it. |
| [`TileLayerInfo`](#type-TileLayerInfo) | struct | One layer of a map. |
| [`TileObject`](#type-TileObject) | struct | An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers. |
| [`TilemapInfo`](#type-TilemapInfo) | struct | What og.load.tilemap loaded. |
| [`SetTileDesc`](#type-SetTileDesc) | desc struct | One cell to change. |
| [`TileCell`](#type-TileCell) | struct | A cell of a map. |
| [`og.tilemap.setTile`](#fn-setTile) | function | Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps. |
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

<sub>`api.ogt` line 6196</sub>

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

<sub>`api.ogt` line 6203</sub>

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

<sub>`api.ogt` line 6213</sub>

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

<sub>`api.ogt` line 6221</sub>

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
| `colliderLayers` | `number` | `1` | — | — | — | The collision layers of those colliders, as bits (Collider.layers). |
| `friction` | `number` | `0.5` | 0 … 2 | — | — | The colliders' friction. |

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
  field colliderLayers u32 "The collision layers of those colliders, as bits (Collider.layers)." {
    default 1
  }
  field friction f32 "The colliders' friction." {
    default 0.5
    min 0
    max 2
  }
}
```

<sub>`api.ogt` line 6105</sub>

## Types

<a id="type-TilemapAsset"></a>

### `TilemapAsset` (handle)

A loaded map: its layers, tilesets and objects.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TilemapAsset "A loaded map: its layers, tilesets and objects."
```

<sub>`api.ogt` line 6096</sub>

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
| `terrain` | `string` | `""` | — | — | — | The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer. |
| `visible` | `boolean` | `false` | — | — | — | Whether it is drawn. |

```ogt
struct TileLayerInfo {
  doc "One layer of a map."
  field name String "Its name."
  field index u32 "Its index (what drawing order and z start from)."
  field z i32 "Its sprite-layer offset from Tilemap.layer."
  field parallax Vector2 "Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera."
  field collision bool "Its solid tiles collide."
  field terrain String "The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer."
  field visible bool "Whether it is drawn."
}
```

<sub>`api.ogt` line 6135</sub>

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

<sub>`api.ogt` line 6146</sub>

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
}
```

<sub>`api.ogt` line 6162</sub>

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

<sub>`api.ogt` line 6177</sub>

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

<sub>`api.ogt` line 6189</sub>

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

<sub>`api.ogt` line 6098</sub>
