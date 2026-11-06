<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-CellRect"></a>

### `CellRect` (struct)

A rectangle of grid cells: column x, row y (from the top), width and height in cells.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The left column. |
| `y` | `number` | `0` | — | — | — | The top row. |
| `width` | `number` | `0` | — | — | — | Columns. |
| `height` | `number` | `0` | — | — | — | Rows. |

```ogt
struct CellRect {
  doc "A rectangle of grid cells: column x, row y (from the top), width and height in cells."
  field x u32 "The left column."
  field y u32 "The top row."
  field width u32 "Columns."
  field height u32 "Rows."
}
```

<sub>`api.ogt` line 1898</sub>
