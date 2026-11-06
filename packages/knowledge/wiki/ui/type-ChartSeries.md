<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChartSeries"></a>

### `ChartSeries` (desc struct)

A series of a chart: its name and a value a category.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name, in the legend and the tooltip. |
| `values` | `readonly number[]` | `[]` | — | — | — | A value for each category. |
| `color` | `string` | `""` | — | — | — | Its colour as `#rrggbb`; empty is the style's series colour for its place. |

```ogt
struct ChartSeries {
  doc "A series of a chart: its name and a value a category."
  desc true
  field label String "Its name, in the legend and the tooltip."
  field values List.f32 "A value for each category."
  field color String "Its colour as `#rrggbb`; empty is the style's series colour for its place."
}
```

<sub>`api.ogt` line 18417</sub>
