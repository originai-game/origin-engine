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
}
```

<sub>Module `render` line 48</sub>
