<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-WheelInput"></a>

### `WheelInput` (struct)

A wheel or trackpad scroll at a point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Horizontal position. |
| `y` | `number` | `0` | — | `cssPx` | — | Vertical position. |
| `deltaX` | `number` | `0` | — | `cssPx` | — | Horizontal scroll amount, as the browser's deltaX: positive scrolls right. |
| `deltaY` | `number` | `0` | — | `cssPx` | — | Vertical scroll amount, as the browser's deltaY: positive scrolls down (the wheel rolled towards the player). |

```ogt
struct WheelInput {
  doc "A wheel or trackpad scroll at a point."
  field x f32 "Horizontal position." {
    unit cssPx
  }
  field y f32 "Vertical position." {
    unit cssPx
  }
  field deltaX f32 "Horizontal scroll amount, as the browser's deltaX: positive scrolls right." {
    unit cssPx
  }
  field deltaY f32 "Vertical scroll amount, as the browser's deltaY: positive scrolls down (the wheel rolled towards the player)." {
    unit cssPx
  }
}
```

<sub>Module `input` line 723</sub>
