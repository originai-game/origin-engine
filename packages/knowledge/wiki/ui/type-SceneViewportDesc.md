<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SceneViewportDesc"></a>

### `SceneViewportDesc` (desc struct)

The viewport element (docs/ui.md §3.3): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). Draw it with an og.CameraView into a render target on its own layers, with its own lights; this element shows the target and passes input in.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The target's colour (RenderTargetInfo.texture). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 320 CSS px. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null is 320 CSS px. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height; 0 is free. |
| `cameraPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its camera's position, for rays. |
| `cameraTarget` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | What its camera looks at, for rays. |
| `fov` | `number` | `50` | ≥ 0 | `deg` | — | Its camera's vertical field of view; 0 gives no ray. |
| `background` | `Paint \| null` | `null` | — | — | — | Drawn under the view. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `label` | `string` | `""` | — | — | — | What the probe and the accessibility tree call it. |

```ogt
struct SceneViewportDesc {
  doc "The viewport element (docs/ui.md §3.3): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). Draw it with an og.CameraView into a render target on its own layers, with its own lights; this element shows the target and passes input in."
  desc true
  field texture Option.Texture "The target's colour (RenderTargetInfo.texture)."
  field width Option.Size "Its width; null is 320 CSS px."
  field height Option.Size "Its height; null is 320 CSS px."
  field aspect f32 "Width over height; 0 is free." {
    min 0
  }
  field cameraPosition Vector3 "Its camera's position, for rays."
  field cameraTarget Vector3 "What its camera looks at, for rays."
  field fov f32 "Its camera's vertical field of view; 0 gives no ray." {
    default 50
    unit deg
    min 0
  }
  field background Option.Paint "Drawn under the view."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field label String "What the probe and the accessibility tree call it."
}
```

<sub>Module `ui` line 3695</sub>
