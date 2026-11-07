<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ViewFrameResult"></a>

### `ViewFrameResult` (struct)

A view frame's turn, tilt and zoom after this tick's input, and its viewport's answer.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `yaw` | `number` | `0` | — | `deg` | — | The turn, to keep and pass back. |
| `pitch` | `number` | `0` | — | `deg` | — | The tilt. |
| `zoom` | `number` | `0` | — | — | — | The zoom. |
| `view` | `SceneViewportInfo` | `{ width: 0, height: 0, pixelWidth: 0, pixelHeight: 0, hovered: false, pressed: false, clicked: false, drag: { x: 0, y: 0 }, wheel: { x: 0, y: 0 }, at: null, ray: null }` | — | — | — | The viewport's answer: its pixel size for the target, the ray, the pointer. |

```ogt
struct ViewFrameResult {
  doc "A view frame's turn, tilt and zoom after this tick's input, and its viewport's answer."
  field yaw f32 "The turn, to keep and pass back." {
    unit deg
  }
  field pitch f32 "The tilt." {
    unit deg
  }
  field zoom f32 "The zoom."
  field view SceneViewportInfo "The viewport's answer: its pixel size for the target, the ray, the pointer."
}
```

<sub>Module `ui` line 3451</sub>
