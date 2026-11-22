<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderTier"></a>

### `RenderTier` (enum)

The feature set the renderer was made for, as og-render's Tier: it caps what the device may use (RenderCapabilities.caps), and a pass whose needs it lacks draws its fallback or not at all. Chosen by the host, never by the backend.

A string: one of the variants below. The zero value is `"full"`.

| Variant | Doc |
|---|---|
| `"full"` | Whatever the device can do (WebGPU, Vulkan, Metal, DX12). Also without a renderer. |
| `"miniGame"` | The mini-game WebGPU set: no multiple render targets, asynchronous pipelines or timestamps. |
| `"gl"` | The GL tier: as miniGame, and no compute, nor the storage and indirect execution WebGL2 lacks with it. |
| `"compat"` | WebGPU's compatibility mode (GLES and D3D11 devices): no vertex-stage storage, per-target blending or rg32 storage; 4096-texel textures, 128-invocation workgroups, no MSAA (msaa4 resolves to fxaa) and no order-independent transparency (blended surfaces sorted back to front). |

```ogt
enum RenderTier {
  doc "The feature set the renderer was made for, as og-render's Tier: it caps what the device may use (RenderCapabilities.caps), and a pass whose needs it lacks draws its fallback or not at all. Chosen by the host, never by the backend."
  variant full "Whatever the device can do (WebGPU, Vulkan, Metal, DX12). Also without a renderer."
  variant miniGame "The mini-game WebGPU set: no multiple render targets, asynchronous pipelines or timestamps."
  variant gl "The GL tier: as miniGame, and no compute, nor the storage and indirect execution WebGL2 lacks with it."
  variant compat "WebGPU's compatibility mode (GLES and D3D11 devices): no vertex-stage storage, per-target blending or rg32 storage; 4096-texel textures, 128-invocation workgroups, no MSAA (msaa4 resolves to fxaa) and no order-independent transparency (blended surfaces sorted back to front)."
}
```

<sub>Module `render` line 40</sub>
