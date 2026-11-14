<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TextMetrics"></a>

### `TextMetrics` (struct)

The extent of a laid-out text, in world units.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | — | — | The widest line's advance. |
| `height` | `number` | `0` | — | — | — | Lines times the font's line height. |
| `lines` | `number` | `0` | — | — | — | How many lines. |

```ogt
struct TextMetrics {
  doc "The extent of a laid-out text, in world units."
  field width f32 "The widest line's advance."
  field height f32 "Lines times the font's line height."
  field lines u32 "How many lines."
}
```

<sub>Module `render` line 2410</sub>
