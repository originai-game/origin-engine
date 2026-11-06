<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12758</sub>
