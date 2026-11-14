<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GpuMemory"></a>

### `GpuMemory` (struct)

The GPU memory the renderer holds, counted where it creates each texture and buffer (sizes as allocated, without the driver's alignment and padding).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `textures` | `number` | `0` | — | `bytes` | — | The scene's textures (every mip level as uploaded), texture arrays, environment maps, palettes, colour LUTs and the image-based light. |
| `meshes` | `number` | `0` | — | `bytes` | — | The meshes' vertex and index buffers and their skins. |
| `renderTargets` | `number` | `0` | — | `bytes` | — | Textures drawn into: og.render.renderTarget's, the frame's targets at the scene's size (the render scale included), the output, TAA's history, the shadow maps, the UI's glass, the reflection probes' faces and the volumetric fog. |
| `buffers` | `number` | `0` | — | `bytes` | — | The renderer's own buffers: instances, uniforms, light tables. |
| `total` | `number` | `0` | — | `bytes` | — | All four. |

```ogt
struct GpuMemory {
  doc "The GPU memory the renderer holds, counted where it creates each texture and buffer (sizes as allocated, without the driver's alignment and padding)."
  field textures f64 "The scene's textures (every mip level as uploaded), texture arrays, environment maps, palettes, colour LUTs and the image-based light." {
    unit bytes
  }
  field meshes f64 "The meshes' vertex and index buffers and their skins." {
    unit bytes
  }
  field renderTargets f64 "Textures drawn into: og.render.renderTarget's, the frame's targets at the scene's size (the render scale included), the output, TAA's history, the shadow maps, the UI's glass, the reflection probes' faces and the volumetric fog." {
    unit bytes
  }
  field buffers f64 "The renderer's own buffers: instances, uniforms, light tables." {
    unit bytes
  }
  field total f64 "All four." {
    unit bytes
  }
}
```

<sub>Module `render` line 3042</sub>
