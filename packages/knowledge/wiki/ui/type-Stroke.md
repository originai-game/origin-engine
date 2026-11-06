<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Stroke"></a>

### `Stroke` (desc struct)

A stroke along the element's outline with a paint of its own (a gradient border is one stroke).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `cssPx` | — | Its width. |
| `align` | `StrokeAlign` | `"center"` | — | — | — | Which side of the outline. |
| `dash` | `number` | `0` | ≥ 0 | `cssPx` | — | A dash's length; 0 is solid. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | The gap between dashes; 0 is the dash's length. |
| `dashOffset` | `number` | `0` | — | `cssPx` | — | Where the dashes start along the outline (move it by the tick for marching ants). |
| `dashCap` | `LineCap` | `"butt"` | — | — | — | The dashes' ends. |
| `paint` | `Fill \| null` | `null` | — | — | — | Its paint; null is the theme's line colour. |

```ogt
struct Stroke {
  doc "A stroke along the element's outline with a paint of its own (a gradient border is one stroke)."
  desc true
  field width f32 "Its width." {
    default 1
    min 0
    unit cssPx
  }
  field align StrokeAlign "Which side of the outline."
  field dash f32 "A dash's length; 0 is solid." {
    min 0
    unit cssPx
  }
  field gap f32 "The gap between dashes; 0 is the dash's length." {
    min 0
    unit cssPx
  }
  field dashOffset f32 "Where the dashes start along the outline (move it by the tick for marching ants)." {
    unit cssPx
  }
  field dashCap LineCap "The dashes' ends."
  field paint Option.Fill "Its paint; null is the theme's line colour."
}
```

<sub>Module `ui` line 7242</sub>
