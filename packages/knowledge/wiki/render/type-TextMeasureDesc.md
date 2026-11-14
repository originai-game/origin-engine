<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TextMeasureDesc"></a>

### `TextMeasureDesc` (desc struct)

Text to measure with og.render.measureText, laid out as og.render.text lays it out.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | The font from og.load.font. |
| `text` | `string` | `""` | — | — | — | The text; `\n` starts a new line. |
| `size` | `number` | `1` | ≥ 0 | — | — | The em size in world units. |

```ogt
struct TextMeasureDesc {
  doc "Text to measure with og.render.measureText, laid out as og.render.text lays it out."
  desc true
  field font Option.Font "The font from og.load.font."
  field text String "The text; `\\n` starts a new line."
  field size f32 "The em size in world units." {
    default 1
    min 0
  }
}
```

<sub>Module `render` line 2403</sub>
