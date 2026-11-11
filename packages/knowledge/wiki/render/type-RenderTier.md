<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderTier"></a>

### `RenderTier` (enum)

The backend the renderer runs on; the probe's hello reports it.

A string: one of the variants below. The zero value is `"webgpu"`.

| Variant | Doc |
|---|---|
| `"webgpu"` | WebGPU in a browser: the full feature set. |
| `"vulkan"` | Native Vulkan on a GPU. |
| `"vulkan-cpu"` | Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU. |
| `"metal"` | Native Metal. |
| `"none"` | No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null. |

```ogt
enum RenderTier {
  doc "The backend the renderer runs on; the probe's hello reports it."
  variant webgpu "WebGPU in a browser: the full feature set."
  variant vulkan "Native Vulkan on a GPU."
  variant vulkan-cpu "Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU."
  variant metal "Native Metal."
  variant none "No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null."
}
```

<sub>Module `render` line 30</sub>
