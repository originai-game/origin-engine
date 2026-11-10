<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderOptions"></a>

### `RenderOptions` (desc struct)

How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing method; left out, tier (4x MSAA in a 2D project, TAA in a 3D one). |
| `hdr` | `boolean` | `false` | — | — | — | Request extended-range HDR surface output. Opt-in; not yet accepted on a real HDR display (acceptance belongs to the compatibility phase). False keeps SDR and its tone operator. Unsupported surfaces fall back to SDR; capabilities.hdrSupported reports the float extended-sRGB surface path, capabilities.hdr reports the resolved mode. Headless renderers remain SDR. HDR uses the LPM display mapper and linear UI composition, with the surface transfer applied last. |
| `paperWhiteNits` | `number` | `0` | — | — | — | HDR reference white; 0 means 203 nits. Clamped to 80..1000. On Apple EDR and web, absolute display nits are unavailable: white follows OS SDR white and peakNits/paperWhiteNits sets relative headroom. On DXGI/Vulkan scRGB this sets absolute paper white. Read once at renderer creation. |
| `peakNits` | `number` | `0` | — | — | — | HDR target peak; 0 means 1000 nits. Clamped to 80..10000 and at least paperWhiteNits. A display can limit actual output below this target. Read once at renderer creation. |
| `shadowMapSize` | `number` | `2048` | 256 … 4096 | — | — | The side of each of the directional light's cascade maps (RenderSettings.shadowCascades of them), in texels; smaller saves memory on phones (three 2048 maps are 48 MB, with their depth buffer 64 MB). |
| `pointShadows` | `PointShadows` | `"auto"` | — | — | — | How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser). |
| `taaHistory` | `TaaHistory` | `"bilinear"` | — | — | — | With TAA: how the history is resampled. |
| `taaSharpen` | `number` | `0` | 0 … 1 | `ratio` | — | With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range. |
| `planarReflections` | `number` | `1` | 1 … 2 | — | — | How many planar reflection surfaces draw at once: water surfaces (og.Water.planarReflection) that mirror the scene in a pass of their own in the same frame. 1: the first surface that asks (a grid surface before one over a mesh); a second surface reflects as if it did not ask (screen-space and sky reflections), never in the first's mirror, and the log says so once. 2: a second surface in view mirrors the scene too, and material shaders can read the first's mirror (`planar_reflection`), at the cost of one more texture in every lit pipeline and a second reflection pass while both are in view. RenderCapabilities.planarReflections reports the count the renderer was made with. |
| `transparency` | `Transparency` | `"weighted"` | — | — | — | How blended surfaces are composited: weighted (the default) or the exact per-pixel lists where the device allows them. |
| `transparencyLayers` | `number` | `0` | 0 … 32 | — | — | With transparency linkedList: the fragments a pixel keeps and sorts exactly (0: 4), as far as the device's largest storage buffer holds them for every pixel at the render size; RenderCapabilities.transparency says whether the lists run at all. |
| `without` | `readonly LitFeature[]` | `[]` | — | — | — | Optional lit features to build no lit shader with, for a game that never uses them: empty (the default) builds every one into every lit shader, as before. Each one left out is in no lit pipeline the renderer makes, on any host, so those pipelines compile faster and draw a little faster; it is not drawn: a scene that uses it draws as if it did not, and the log says so once. Each name once, in any order. Read once, when the renderer is created; RenderCapabilities.without reports it. |
| `debugView` | `DebugView` | `"none"` | — | — | — | A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds. |

```ogt
struct RenderOptions {
  doc "How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer."
  desc true
  field antialias Antialias "The antialiasing method; left out, tier (4x MSAA in a 2D project, TAA in a 3D one)."
  field hdr bool "Request extended-range HDR surface output. Opt-in; not yet accepted on a real HDR display (acceptance belongs to the compatibility phase). False keeps SDR and its tone operator. Unsupported surfaces fall back to SDR; capabilities.hdrSupported reports the float extended-sRGB surface path, capabilities.hdr reports the resolved mode. Headless renderers remain SDR. HDR uses the LPM display mapper and linear UI composition, with the surface transfer applied last."
  field paperWhiteNits f32 "HDR reference white; 0 means 203 nits. Clamped to 80..1000. On Apple EDR and web, absolute display nits are unavailable: white follows OS SDR white and peakNits/paperWhiteNits sets relative headroom. On DXGI/Vulkan scRGB this sets absolute paper white. Read once at renderer creation."
  field peakNits f32 "HDR target peak; 0 means 1000 nits. Clamped to 80..10000 and at least paperWhiteNits. A display can limit actual output below this target. Read once at renderer creation."
  field shadowMapSize u32 "The side of each of the directional light's cascade maps (RenderSettings.shadowCascades of them), in texels; smaller saves memory on phones (three 2048 maps are 48 MB, with their depth buffer 64 MB)." {
    default 2048
    min 256
    max 4096
  }
  field pointShadows PointShadows "How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser)."
  field taaHistory TaaHistory "With TAA: how the history is resampled."
  field taaSharpen f32 "With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range." {
    min 0
    max 1
    unit ratio
  }
  field planarReflections u32 "How many planar reflection surfaces draw at once: water surfaces (og.Water.planarReflection) that mirror the scene in a pass of their own in the same frame. 1: the first surface that asks (a grid surface before one over a mesh); a second surface reflects as if it did not ask (screen-space and sky reflections), never in the first's mirror, and the log says so once. 2: a second surface in view mirrors the scene too, and material shaders can read the first's mirror (`planar_reflection`), at the cost of one more texture in every lit pipeline and a second reflection pass while both are in view. RenderCapabilities.planarReflections reports the count the renderer was made with." {
    default 1
    min 1
    max 2
  }
  field transparency Transparency "How blended surfaces are composited: weighted (the default) or the exact per-pixel lists where the device allows them."
  field transparencyLayers u32 "With transparency linkedList: the fragments a pixel keeps and sorts exactly (0: 4), as far as the device's largest storage buffer holds them for every pixel at the render size; RenderCapabilities.transparency says whether the lists run at all." {
    default 0
    min 0
    max 32
  }
  field without List.LitFeature "Optional lit features to build no lit shader with, for a game that never uses them: empty (the default) builds every one into every lit shader, as before. Each one left out is in no lit pipeline the renderer makes, on any host, so those pipelines compile faster and draw a little faster; it is not drawn: a scene that uses it draws as if it did not, and the log says so once. Each name once, in any order. Read once, when the renderer is created; RenderCapabilities.without reports it."
  field debugView DebugView "A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds."
}
```

<sub>Module `render` line 112</sub>
