<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12777</sub>
