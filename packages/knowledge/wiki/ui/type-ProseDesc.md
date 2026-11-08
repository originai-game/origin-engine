<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ProseDesc"></a>

### `ProseDesc` (desc struct)

A passage of rich text: paragraphs at a readable measure.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `role` | `TextRole` | `"body"` | — | — | — | Each paragraph's role (body by default; caption for fine print). |
| `tone` | `TextTone` | `"normal"` | — | — | — | Its ink. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows, kept to a readable measure (about 66 characters) when measure is set. |
| `measure` | `boolean` | `true` | — | — | — | Keep lines to about 66 characters, as long reading wants. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Space between paragraphs; null is the style's md. |
| `align` | `TextAlign` | `"start"` | — | — | — | How its lines align. |

```ogt
struct ProseDesc {
  doc "A passage of rich text: paragraphs at a readable measure."
  desc true
  field role TextRole "Each paragraph's role (body by default; caption for fine print)."
  field tone TextTone "Its ink."
  field width Option.Size "Its width; null grows, kept to a readable measure (about 66 characters) when measure is set."
  field measure bool "Keep lines to about 66 characters, as long reading wants." {
    default true
  }
  field gap Option.f32 "Space between paragraphs; null is the style's md." {
    unit cssPx
  }
  field align TextAlign "How its lines align."
}
```

<sub>Module `ui` line 3203</sub>
