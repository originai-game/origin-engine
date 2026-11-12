<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderBackend"></a>

### `RenderBackend` (enum)

The backend the renderer runs on: RenderCapabilities.backend, and the probe's hello reports it as its tier. Reference frames are filed under it.

A string: one of the variants below. The zero value is `"webgpu"`.

| Variant | Doc |
|---|---|
| `"webgpu"` | WebGPU in a browser. |
| `"vulkan"` | Native Vulkan on a GPU. |
| `"vulkan-cpu"` | Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU. |
| `"metal"` | Native Metal. |
| `"dx12"` | Native Direct3D 12. |
| `"none"` | No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null. |

```ogt
enum RenderBackend {
  doc "The backend the renderer runs on: RenderCapabilities.backend, and the probe's hello reports it as its tier. Reference frames are filed under it."
  variant webgpu "WebGPU in a browser."
  variant vulkan "Native Vulkan on a GPU."
  variant vulkan-cpu "Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU."
  variant metal "Native Metal."
  variant dx12 "Native Direct3D 12."
  variant none "No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null."
}
```

<sub>Module `render` line 30</sub>
