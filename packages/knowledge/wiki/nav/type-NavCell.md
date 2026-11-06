<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavCell"></a>

### `NavCell` (struct)

A grid cell.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column. |
| `y` | `number` | `0` | — | — | — | The row (0 is the bottom). |
| `inside` | `boolean` | `false` | — | — | — | Whether the point was on the grid. |

```ogt
struct NavCell {
  doc "A grid cell."
  field x u32 "The column."
  field y u32 "The row (0 is the bottom)."
  field inside bool "Whether the point was on the grid."
}
```

<sub>`api.ogt` line 13918</sub>
