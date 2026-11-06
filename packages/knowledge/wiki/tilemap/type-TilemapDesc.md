<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12680</sub>
