<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EdgeClampDesc"></a>

### `EdgeClampDesc` (desc struct)

Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `margin` | `number` | `24` | ≥ 0 | `cssPx` | — | Inset from the safe area's edges. |
| `arrow` | `Paint \| null` | `null` | — | — | — | The arrow's ink; null draws none. |
| `arrowSize` | `number` | `14` | ≥ 0 | `cssPx` | — | The arrow's length. |

```ogt
struct EdgeClampDesc {
  doc "Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there."
  desc true
  field margin f32 "Inset from the safe area's edges." {
    default 24
    unit cssPx
    min 0
  }
  field arrow Option.Paint "The arrow's ink; null draws none."
  field arrowSize f32 "The arrow's length." {
    default 14
    unit cssPx
    min 0
  }
}
```

<sub>Module `ui` line 7415</sub>
