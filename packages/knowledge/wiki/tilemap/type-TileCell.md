<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-TileCell"></a>

### `TileCell` (struct)

A cell of a map.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column. |
| `y` | `number` | `0` | — | — | — | The row. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point was on the map. |

```ogt
struct TileCell {
  doc "A cell of a map."
  field x u32 "The column."
  field y u32 "The row."
  field inside bool "Whether the point was on the map."
}
```

<sub>`api.ogt` line 12677</sub>
