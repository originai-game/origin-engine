<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiPatch"></a>

### `UiPatch` (desc struct)

What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Direction \| null` | `null` | — | — | — | Its direction. |
| `width` | `Size \| null` | `null` | — | — | — | Its width. |
| `height` | `Size \| null` | `null` | — | — | — | Its height. |
| `padding` | `Insets \| null` | `null` | — | — | — | Its padding. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Its gap. |
| `align` | `Align \| null` | `null` | — | — | — | Its main-axis alignment. |
| `cross` | `Cross \| null` | `null` | — | — | — | Its cross-axis alignment. |
| `wrap` | `boolean \| null` | `null` | — | — | — | Its wrapping. |
| `layout` | `LayoutMode \| null` | `null` | — | — | — | Its layout mode. |
| `columns` | `readonly Track[]` | `[]` | — | — | — | Its grid columns (empty: unchanged). |
| `hidden` | `boolean \| null` | `null` | — | — | — | Hide it and its subtree where the variant applies. |
| `values` | `readonly PropValue[]` | `[]` | — | — | — | Animatable values it sets. |

```ogt
struct UiPatch {
  doc "What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged."
  desc true
  field direction Option.Direction "Its direction."
  field width Option.Size "Its width."
  field height Option.Size "Its height."
  field padding Option.Insets "Its padding."
  field gap Option.f32 "Its gap." {
    unit cssPx
  }
  field align Option.Align "Its main-axis alignment."
  field cross Option.Cross "Its cross-axis alignment."
  field wrap Option.bool "Its wrapping."
  field layout Option.LayoutMode "Its layout mode."
  field columns List.Track "Its grid columns (empty: unchanged)."
  field hidden Option.bool "Hide it and its subtree where the variant applies."
  field values List.PropValue "Animatable values it sets."
}
```

<sub>Module `ui` line 1722</sub>
