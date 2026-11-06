<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12811</sub>
