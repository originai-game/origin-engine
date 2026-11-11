<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tilemap` module

Tilemaps: layers of tiles from a tileset atlas (a `type tileset` `.ogt`), loaded from a cooked map (a `type tilemap` `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component; og.tilemap.create and og.tilemap.resize make and change maps in game code.

Layers draw through the sprite batch; solid tiles become static colliders. Edits and run-time maps are world state (the og.tilemap state block). Cells count from the top-left corner, x right and y down (Tiled's), at the entity's position.

See `world §6`.

- **TypeScript**: functions are the namespace `og.tilemap`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.Tilemap`, `TilemapInfo`, `SetTileDesc`, `og.tilemap.setTile`, `og.tilemap.getTile`; 16 more are advanced (`og api show tilemap --all`).
- **Index**: [every module and guide](index.md).

```ogt
module tilemap {
  doc "Tilemaps: layers of tiles from a tileset atlas (a `type tileset` `.ogt`), loaded from a cooked map (a `type tilemap` `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component; og.tilemap.create and og.tilemap.resize make and change maps in game code."
  doc "Layers draw through the sprite batch; solid tiles become static colliders. Edits and run-time maps are world state (the og.tilemap state block). Cells count from the top-left corner, x right and y down (Tiled's), at the entity's position."
  # declarations follow, each on its own page
```

<sub>Module `tilemap` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`TilemapAsset`](tilemap/type-TilemapAsset.md) | handle |  | A loaded map: its layers, tilesets and objects. |
| [`TilemapAsset()`](#column-TilemapAsset) | column kind |  | A TilemapAsset handle field. |
| [`og.Tilemap`](tilemap/record-Tilemap.md) | component |  | Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. |
| [`TileOrientation`](tilemap/type-TileOrientation.md) | enum |  | How a map's cells are laid out (Tiled's orientations). |
| [`TileLayerInfo`](tilemap/type-TileLayerInfo.md) | struct |  | One layer of a map. |
| [`TileObject`](tilemap/type-TileObject.md) | struct |  | An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers. |
| [`TilemapInfo`](tilemap/type-TilemapInfo.md) | struct |  | What og.load.tilemap loaded. |
| [`TileLayerDesc`](tilemap/type-TileLayerDesc.md) | desc struct |  | One layer of a map og.tilemap.create makes (empty). |
| [`TilemapDesc`](tilemap/type-TilemapDesc.md) | desc struct |  | A map to make at run time: empty layers on tilesets the bundle cooked. |
| [`ResizeDesc`](tilemap/type-ResizeDesc.md) | desc struct |  | A new size for a map, and where its cells go. |
| [`SetTileDesc`](tilemap/type-SetTileDesc.md) | desc struct |  | One cell to change. |
| [`TileCell`](tilemap/type-TileCell.md) | struct |  | A cell of a map. |
| [`og.tilemap.setTile`](tilemap/fn-setTile.md) | function | `(world: World, desc?: SetTileDesc): Result<void>` | Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). |
| [`FillDesc`](tilemap/type-FillDesc.md) | desc struct |  | A block of cells to write at once: a generated grid (og.procgen's BSP cells or WFC tiles mapped to tile ids). |
| [`og.tilemap.fill`](tilemap/fn-fill.md) | function | `(world: World, desc?: FillDesc): Result<void>` | Write a block of cells, as setTile on each (row by row) but re-tiling a terrain layer and rebuilding the colliders once. |
| [`og.tilemap.create`](tilemap/fn-create.md) | function | `(world: World, desc?: TilemapDesc): Result<TilemapInfo>` | Make an empty map in game code: draw it and edit it as a loaded one (an og.Tilemap with its `map`, then setTile and fill). |
| [`og.tilemap.resize`](tilemap/fn-resize.md) | function | `(world: World, desc?: ResizeDesc): Result<void>` | Change a map's size (a loaded one too: from then on it is a run-time map, saved whole). |
| [`og.tilemap.neighbours`](tilemap/fn-neighbours.md) | function | `(world: World, map: TilemapAsset, x: number, y: number): readonly TileCell[]` | The cells next to a cell, clockwise from the top (or the top right): 4 on an orthogonal or isometric grid (the sides), 4 on a staggered one (the diagonal sides), 6 on a hex one; cells off the map are… |
| [`og.tilemap.getTile`](tilemap/fn-getTile.md) | function | `(world: World, entity: Entity, layer: string, x: number, y: number): number` | A cell's tile id, -1 when empty: on a terrain layer the tile auto-tiling resolved from the neighbours (setTile writes 1 or 0 there; the id changes with them). |
| [`og.tilemap.cellAt`](tilemap/fn-cellAt.md) | function | `(world: World, entity: Entity, point: Vector3): TileCell` | The cell under a world point (its xy), for a map at its entity's position. |
| [`og.tilemap.cellCenter`](tilemap/fn-cellCenter.md) | function | `(world: World, entity: Entity, x: number, y: number): Vector3` | A cell's centre in world coordinates (z of the map). |

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

<sub>Module `tilemap` line 9</sub>
