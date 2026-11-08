<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ViewFrameDesc"></a>

### `ViewFrameDesc` (desc struct)

A framed 3D view (docs/ui.md §3.3): a camera's render target in the layout, turned by a drag, the wheel, keys and a pad.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The render target's colour; null shows the placeholder (loading). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null follows the aspect. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height; 0 is 1. |
| `yaw` | `number` | `0` | — | `deg` | — | The turn shown, from game state (keep ViewFrameResult.yaw). |
| `pitch` | `number` | `0` | — | `deg` | — | The tilt shown. |
| `zoom` | `number` | `1` | — | — | — | The zoom shown. |
| `minZoom` | `number` | `0.6` | — | — | — | The least zoom. |
| `maxZoom` | `number` | `2.5` | — | — | — | The most zoom. |
| `minPitch` | `number` | `-30` | — | `deg` | — | The least tilt. |
| `maxPitch` | `number` | `60` | — | `deg` | — | The most tilt. |
| `orbit` | `boolean` | `true` | — | — | — | A drag turns and tilts it, the wheel zooms, Left and Right turn it while it has the focus. |
| `cameraPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its camera's position, for the pointer ray. |
| `cameraTarget` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | What its camera looks at. |
| `fov` | `number` | `50` | — | `deg` | — | Its camera's vertical field of view; 0 gives no ray. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it (`Character preview`). |
| `hint` | `string` | `""` | — | — | — | A line over its bottom while it has not been turned (`Drag to rotate`); empty for none. |

```ogt
struct ViewFrameDesc {
  doc "A framed 3D view (docs/ui.md §3.3): a camera's render target in the layout, turned by a drag, the wheel, keys and a pad."
  desc true
  field texture Option.Texture "The render target's colour; null shows the placeholder (loading)."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height; null follows the aspect."
  field aspect f32 "Width over height; 0 is 1." {
    min 0
  }
  field yaw f32 "The turn shown, from game state (keep ViewFrameResult.yaw)." {
    unit deg
  }
  field pitch f32 "The tilt shown." {
    unit deg
  }
  field zoom f32 "The zoom shown." {
    default 1
  }
  field minZoom f32 "The least zoom." {
    default 0.6
  }
  field maxZoom f32 "The most zoom." {
    default 2.5
  }
  field minPitch f32 "The least tilt." {
    default -30
    unit deg
  }
  field maxPitch f32 "The most tilt." {
    default 60
    unit deg
  }
  field orbit bool "A drag turns and tilts it, the wheel zooms, Left and Right turn it while it has the focus." {
    default true
  }
  field cameraPosition Vector3 "Its camera's position, for the pointer ray."
  field cameraTarget Vector3 "What its camera looks at."
  field fov f32 "Its camera's vertical field of view; 0 gives no ray." {
    default 50
    unit deg
  }
  field label String "What a screen reader calls it (`Character preview`)."
  field hint String "A line over its bottom while it has not been turned (`Drag to rotate`); empty for none."
}
```

<sub>Module `ui` line 1779</sub>
