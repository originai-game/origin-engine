<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

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

<sub>`api.ogt` line 12731</sub>
