<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12782</sub>
