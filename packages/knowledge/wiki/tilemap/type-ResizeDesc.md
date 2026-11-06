<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-ResizeDesc"></a>

### `ResizeDesc` (desc struct)

A new size for a map, and where its cells go.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `TilemapAsset` | `0` | — | — | — | The map: og.load.tilemap's or og.tilemap.create's. |
| `width` | `number` | `16` | ≥ 1 | — | — | Cells across. |
| `height` | `number` | `16` | ≥ 1 | — | — | Cells down. |
| `x` | `number` | `0` | — | — | — | Where the old cell 0 0 lands: columns added on the left (negative crops them). |
| `y` | `number` | `0` | — | — | — | Rows added at the top (negative crops them). |

```ogt
struct ResizeDesc {
  doc "A new size for a map, and where its cells go."
  desc true
  field map TilemapAsset "The map: og.load.tilemap's or og.tilemap.create's."
  field width u32 "Cells across." {
    default 16
    min 1
  }
  field height u32 "Cells down." {
    default 16
    min 1
  }
  field x i32 "Where the old cell 0 0 lands: columns added on the left (negative crops them)."
  field y i32 "Rows added at the top (negative crops them)."
}
```

<sub>`api.ogt` line 12693</sub>
