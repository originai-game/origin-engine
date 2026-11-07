<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiRect"></a>

### `UiRect` (struct)

A rectangle of the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its left edge. |
| `y` | `number` | `0` | — | `cssPx` | — | Its top edge. |
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |

```ogt
struct UiRect {
  doc "A rectangle of the play surface."
  field x f32 "Its left edge." {
    unit cssPx
  }
  field y f32 "Its top edge." {
    unit cssPx
  }
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 832</sub>
