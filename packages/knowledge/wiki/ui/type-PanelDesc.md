<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PanelDesc"></a>

### `PanelDesc` (desc struct)

A card: panel, the panel radius, padded and spaced by the style.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `row` | `boolean` | `false` | — | — | — | Lay children left to right (the default is top to bottom). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets \| null` | `null` | — | — | — | Space inside; null is space.lg on every side. |
| `gap` | `number \| null` | `null` | — | — | — | Space between children; null is space.md. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null is panel. |

```ogt
struct PanelDesc {
  doc "A card: panel, the panel radius, padded and spaced by the style."
  desc true
  field row bool "Lay children left to right (the default is top to bottom)."
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Option.Insets "Space inside; null is space.lg on every side."
  field gap Option.f32 "Space between children; null is space.md."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null is panel."
}
```

<sub>`api.ogt` line 15901</sub>
