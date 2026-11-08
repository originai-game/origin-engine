<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GridPlace"></a>

### `GridPlace` (desc struct)

Where an item sits in its parent grid; the zero value is auto-placed in one cell.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `column` | `number` | `0` | — | — | — | Its first column, 1-based (0: auto). |
| `row` | `number` | `0` | — | — | — | Its first row, 1-based (0: auto). |
| `columnSpan` | `number` | `0` | — | — | — | Columns spanned (0: 1). |
| `rowSpan` | `number` | `0` | — | — | — | Rows spanned (0: 1). |
| `area` | `string` | `""` | — | — | — | A named area of the grid (wins over the numbers). |

```ogt
struct GridPlace {
  doc "Where an item sits in its parent grid; the zero value is auto-placed in one cell."
  desc true
  field column u32 "Its first column, 1-based (0: auto)."
  field row u32 "Its first row, 1-based (0: auto)."
  field columnSpan u32 "Columns spanned (0: 1)."
  field rowSpan u32 "Rows spanned (0: 1)."
  field area String "A named area of the grid (wins over the numbers)."
}
```

<sub>Module `ui` line 4516</sub>
