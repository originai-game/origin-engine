<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextOutlineDesc"></a>

### `TextOutlineDesc` (desc struct)

An outline around an element's glyphs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `2` | ≥ 0 | `cssPx` | — | Its width (up to the font's distance range). |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |
| `align` | `StrokeAlign` | `"center"` | — | — | — | Across the glyph's edge, inside it or outside it. |

```ogt
struct TextOutlineDesc {
  doc "An outline around an element's glyphs."
  desc true
  field width f32 "Its width (up to the font's distance range)." {
    default 2
    min 0
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
  field align StrokeAlign "Across the glyph's edge, inside it or outside it."
}
```

<sub>Module `ui` line 7200</sub>
