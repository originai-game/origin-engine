<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderCaps"></a>

### `RenderCaps` (struct)

What the device can do under RenderCapabilities.tier, as og-render's Caps: every decision the renderer makes about the device reads this set. All false without a renderer.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `compute` | `boolean` | `false` | — | — | — | Compute passes. |
| `mrt` | `boolean` | `false` | — | — | — | More than one colour attachment in a render pass. |
| `asyncPipelines` | `boolean` | `false` | — | — | — | Pipelines compiled off the frame: on worker threads natively, asked for asynchronously on the web. |
| `timestamps` | `boolean` | `false` | — | — | — | Timestamp queries granted by the device (gpuTimestamps also needs a build that keeps them). |
| `indirect` | `boolean` | `false` | — | — | — | Indirect draws and dispatches. |
| `indirectFirstInstance` | `boolean` | `false` | — | — | — | Indirect draws with a first instance. |
| `fragmentStorage` | `boolean` | `false` | — | — | — | Storage written from fragment shaders. |
| `vertexStorage` | `boolean` | `false` | — | — | — | Storage read in the vertex stage. |
| `independentBlend` | `boolean` | `false` | — | — | — | Colour targets of one pipeline blended each their own way: order-independent transparency's targets and the 2D lights' directions need it. |
| `rg32Storage` | `boolean` | `false` | — | — | — | Storage bindings of two-channel 32-bit textures: scattered impostor cards take the depth prepass's surface with it, and search their frames again without it. |
| `rg32Renderable` | `boolean` | `false` | — | — | — | Render-attachment support for rg32float textures (2D shadow maps and dual-channel float targets); falls back to rgba16float without it. |
| `highBindingSlots` | `boolean` | `false` | — | — | — | Extended per-stage binding budgets (at least 24 sampled textures and 8 storage buffers per stage) beyond WebGPU compatibility limits. |
| `rg11b10Renderable` | `boolean` | `false` | — | — | — | Render-attachment support for packed 32-bit HDR RGB (rg11b10ufloat) post-processing targets. |
| `depthClipControl` | `boolean` | `false` | — | — | — | Unclipped depth (depth-clip-control), preventing near-plane shadow pancaking clips on directional shadow cascades. |
| `shaderF16` | `boolean` | `false` | — | — | — | Native 16-bit floating-point WGSL arithmetic (shader-f16) for half-precision post-processing passes. |
| `subgroup` | `boolean` | `false` | — | — | — | Subgroup / warp wave intrinsics (subgroups) for 256-lane prefix scans and reductions. |
| `multiDrawIndirectCount` | `boolean` | `false` | — | — | — | Hardware multi-draw indirect coalescing across contiguous compatible draw batches. |
| `bindlessTextures` | `boolean` | `false` | — | — | — | Native bindless sampled texture arrays with non-uniform indexing (1024-slot shared material table). |

```ogt
struct RenderCaps {
  doc "What the device can do under RenderCapabilities.tier, as og-render's Caps: every decision the renderer makes about the device reads this set. All false without a renderer."
  field compute bool "Compute passes."
  field mrt bool "More than one colour attachment in a render pass."
  field asyncPipelines bool "Pipelines compiled off the frame: on worker threads natively, asked for asynchronously on the web."
  field timestamps bool "Timestamp queries granted by the device (gpuTimestamps also needs a build that keeps them)."
  field indirect bool "Indirect draws and dispatches."
  field indirectFirstInstance bool "Indirect draws with a first instance."
  field fragmentStorage bool "Storage written from fragment shaders."
  field vertexStorage bool "Storage read in the vertex stage."
  field independentBlend bool "Colour targets of one pipeline blended each their own way: order-independent transparency's targets and the 2D lights' directions need it."
  field rg32Storage bool "Storage bindings of two-channel 32-bit textures: scattered impostor cards take the depth prepass's surface with it, and search their frames again without it."
  field rg32Renderable bool "Render-attachment support for rg32float textures (2D shadow maps and dual-channel float targets); falls back to rgba16float without it."
  field highBindingSlots bool "Extended per-stage binding budgets (at least 24 sampled textures and 8 storage buffers per stage) beyond WebGPU compatibility limits."
  field rg11b10Renderable bool "Render-attachment support for packed 32-bit HDR RGB (rg11b10ufloat) post-processing targets."
  field depthClipControl bool "Unclipped depth (depth-clip-control), preventing near-plane shadow pancaking clips on directional shadow cascades."
  field shaderF16 bool "Native 16-bit floating-point WGSL arithmetic (shader-f16) for half-precision post-processing passes."
  field subgroup bool "Subgroup / warp wave intrinsics (subgroups) for 256-lane prefix scans and reductions."
  field multiDrawIndirectCount bool "Hardware multi-draw indirect coalescing across contiguous compatible draw batches."
  field bindlessTextures bool "Native bindless sampled texture arrays with non-uniform indexing (1024-slot shared material table)."
}
```

<sub>Module `render` line 48</sub>
