<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="fn-cellAt"></a>

### `og.tilemap.cellAt`

The cell under a world point (its xy), for a map at its entity's position.

Also known as: world to cell, tile at position, pick a tile, Unity Tilemap.WorldToCell, Godot local_to_map

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
  aliases "world to cell, tile at position, pick a tile, Unity Tilemap.WorldToCell, Godot local_to_map"
  param world World "The world."
  param entity Entity "The entity with the og.Tilemap."
  param point Vector3 "A world point."
  returns TileCell
}
```

<sub>Module `tilemap` line 241</sub>
