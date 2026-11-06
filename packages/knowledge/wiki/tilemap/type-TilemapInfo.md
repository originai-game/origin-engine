<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12649</sub>
