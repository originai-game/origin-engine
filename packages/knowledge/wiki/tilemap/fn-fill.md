<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12702</sub>
