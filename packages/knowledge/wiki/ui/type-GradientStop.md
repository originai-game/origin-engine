<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GradientStop"></a>

### `GradientStop` (desc struct)

A gradient's colour at an offset.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `number` | `0` | 0 … 1 | `ratio` | — | Where along the gradient, 0 to 1. |
| `color` | `Paint` | `{}` | — | — | — | Its colour: a theme token, resolved when the frame is drawn, or a literal. |

```ogt
struct GradientStop {
  doc "A gradient's colour at an offset."
  desc true
  field offset f32 "Where along the gradient, 0 to 1." {
    min 0
    max 1
    unit ratio
  }
  field color Paint "Its colour: a theme token, resolved when the frame is drawn, or a literal."
}
```

<sub>Module `ui` line 1326</sub>
