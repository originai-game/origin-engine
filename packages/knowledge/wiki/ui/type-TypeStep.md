<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TypeStep"></a>

### `TypeStep` (struct)

One step of the type ramp.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | — | `cssPx` | — | The font size. |
| `lineHeight` | `number` | `0` | — | `cssPx` | — | The leading of Latin text. |
| `trackingEm` | `number` | `0` | — | — | — | Latin tracking, in em. |
| `cjkLineHeight` | `number` | `0` | — | `cssPx` | — | The leading text with Chinese, Japanese or Korean characters takes. |
| `cjkTrackingEm` | `number` | `0` | — | — | — | Tracking of text containing CJK characters, in em. |
| `weight` | `number` | `0` | — | — | — | The face weight (100 to 900); zero takes the font role's own face. |
| `uppercase` | `boolean` | `false` | — | — | — | Style-driven locale-sensitive uppercase, applied before measurement without rewriting source text or editable values. |

```ogt
struct TypeStep {
  doc "One step of the type ramp."
  field size f32 "The font size." {
    unit cssPx
  }
  field lineHeight f32 "The leading of Latin text." {
    unit cssPx
  }
  field trackingEm f32 "Latin tracking, in em."
  field cjkLineHeight f32 "The leading text with Chinese, Japanese or Korean characters takes." {
    unit cssPx
  }
  field cjkTrackingEm f32 "Tracking of text containing CJK characters, in em."
  field weight u32 "The face weight (100 to 900); zero takes the font role's own face."
  field uppercase bool "Style-driven locale-sensitive uppercase, applied before measurement without rewriting source text or editable values."
}
```

<sub>Module `ui` line 242</sub>
