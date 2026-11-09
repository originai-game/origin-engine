<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiTransform"></a>

### `UiTransform` (desc struct)

A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. Hit tests, the probe's rectangles and the accessibility tree's bounds follow it; layout does not.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Moves it across. |
| `y` | `number` | `0` | — | `cssPx` | — | Moves it down. |
| `scaleX` | `number` | `1` | — | — | — | Its horizontal scale. |
| `scaleY` | `number` | `1` | — | — | — | Its vertical scale. |
| `rotate` | `number` | `0` | — | — | — | Its turn, radians, clockwise on screen. |
| `skewX` | `number` | `0` | — | — | — | Its horizontal skew, radians. |
| `skewY` | `number` | `0` | — | — | — | Its vertical skew, radians. |
| `pivotX` | `number` | `0.5` | — | `ratio` | — | The pivot across its box (0 its start edge, 1 its end). |
| `pivotY` | `number` | `0.5` | — | `ratio` | — | The pivot down its box. |

```ogt
struct UiTransform {
  doc "A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. Hit tests, the probe's rectangles and the accessibility tree's bounds follow it; layout does not."
  desc true
  field x f32 "Moves it across." {
    unit cssPx
  }
  field y f32 "Moves it down." {
    unit cssPx
  }
  field scaleX f32 "Its horizontal scale." {
    default 1
  }
  field scaleY f32 "Its vertical scale." {
    default 1
  }
  field rotate f32 "Its turn, radians, clockwise on screen."
  field skewX f32 "Its horizontal skew, radians."
  field skewY f32 "Its vertical skew, radians."
  field pivotX f32 "The pivot across its box (0 its start edge, 1 its end)." {
    default 0.5
    unit ratio
  }
  field pivotY f32 "The pivot down its box." {
    default 0.5
    unit ratio
  }
}
```

<sub>Module `ui` line 1334</sub>
