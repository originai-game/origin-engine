<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChartKind"></a>

### `ChartKind` (enum)

What a chart draws.

A string: one of the variants below. The zero value is `"line"`.

| Variant | Doc |
|---|---|
| `"line"` | Lines through the values (the default). |
| `"area"` | Lines with the area under them filled, stacked with stacked. |
| `"bar"` | Bars, grouped or stacked. |
| `"radar"` | A polygon a series over rings, one axis a category (at least three). |
| `"pie"` | The first series' values as wedges, with a legend. |
| `"donut"` | A pie with a hole that shows the total, or the hovered value. |

```ogt
enum ChartKind {
  doc "What a chart draws."
  variant line "Lines through the values (the default)."
  variant area "Lines with the area under them filled, stacked with stacked."
  variant bar "Bars, grouped or stacked."
  variant radar "A polygon a series over rings, one axis a category (at least three)."
  variant pie "The first series' values as wedges, with a legend."
  variant donut "A pie with a hole that shows the total, or the hovered value."
}
```

<sub>Module `ui` line 2731</sub>
