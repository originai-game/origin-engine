<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderOptions"></a>

### `RenderOptions` (desc struct)

How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.

See `render §2`.

A desc: every field may be omitted and takes its default; `{}` is valid.

#### Antialiasing

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing method; left out, tier (4x MSAA in a 2D project, TAA in a 3D one). |
| `taaHistory` | `TaaHistory` | `"bilinear"` | — | — | — | With TAA: how the history is resampled. |
| `taaSharpen` | `number` | `0` | 0 … 1 | `ratio` | — | With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range. |

#### Display

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hdr` | `boolean` | `false` | — | — | — | Request extended-range HDR surface output; false keeps SDR and its tone operator. Unsupported surfaces and headless renderers stay SDR; capabilities.hdrSupported reports the extended-sRGB path, capabilities.hdr the resolved mode. HDR maps with LPM, composes the UI linearly, transfers last. See `render §12`. |
| `paperWhiteNits` | `number` | `0` | — | — | — | HDR reference white; 0 means 203 nits. Clamped to 80..1000. On Apple EDR and web, absolute display nits are unavailable: white follows OS SDR white and peakNits/paperWhiteNits sets relative headroom. On DXGI/Vulkan scRGB this sets absolute paper white. Read once at renderer creation. |
| `peakNits` | `number` | `0` | — | — | — | HDR target peak; 0 means 1000 nits. Clamped to 80..10000 and at least paperWhiteNits. A display can limit actual output below this target. Read once at renderer creation. |

#### Shadows and reflections

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shadowMapSize` | `number` | `2048` | 256 … 4096 | — | — | The side of each of the directional light's cascade maps (RenderSettings.shadowCascades of them), in texels. See `render §3`. |
| `pointShadows` | `PointShadows` | `"auto"` | — | — | — | How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser). |
| `planarReflections` | `number` | `1` | 1 … 2 | — | — | How many planar reflections (og.Water.planarReflection) draw at once. 1: the first asking (a grid before a mesh); others reflect as if they did not ask, logged once. 2: a second in view mirrors too; shaders read the first's (`planar_reflection`). RenderCapabilities.planarReflections reports it. See `render §8`. |

#### Transparency

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `transparency` | `Transparency` | `"weighted"` | — | — | — | How blended surfaces are composited: weighted (the default) or the exact per-pixel lists where the device allows them. |
| `transparencyLayers` | `number` | `0` | 0 … 32 | — | — | With transparency linkedList: the fragments a pixel keeps and sorts exactly (0: 4), as far as the device's largest storage buffer holds them for every pixel at the render size; RenderCapabilities.transparency says whether the lists run at all. |

#### Shaders and debugging

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `without` | `readonly LitFeature[]` | `[]` | — | — | — | Optional lit features left out of every lit pipeline, on any host; empty (the default) builds them all. A scene using one left out draws as if it did not, logged once. Each name once, in any order. Read once, when the renderer is created; RenderCapabilities.without reports it. See `render §10`. |
| `debugView` | `DebugView` | `"none"` | — | — | — | A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds. |

```ogt
struct RenderOptions {
  doc "How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer."
  guide "render §2"
  desc true
  group "Antialiasing"
  field antialias Antialias "The antialiasing method; left out, tier (4x MSAA in a 2D project, TAA in a 3D one)."
  field taaHistory TaaHistory "With TAA: how the history is resampled."
  field taaSharpen f32 "With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range." {
    min 0
    max 1
    unit ratio
  }
  group "Display"
  field hdr bool "Request extended-range HDR surface output; false keeps SDR and its tone operator. Unsupported surfaces and headless renderers stay SDR; capabilities.hdrSupported reports the extended-sRGB path, capabilities.hdr the resolved mode. HDR maps with LPM, composes the UI linearly, transfers last." {
    guide "render §12"
  }
  field paperWhiteNits f32 "HDR reference white; 0 means 203 nits. Clamped to 80..1000. On Apple EDR and web, absolute display nits are unavailable: white follows OS SDR white and peakNits/paperWhiteNits sets relative headroom. On DXGI/Vulkan scRGB this sets absolute paper white. Read once at renderer creation."
  field peakNits f32 "HDR target peak; 0 means 1000 nits. Clamped to 80..10000 and at least paperWhiteNits. A display can limit actual output below this target. Read once at renderer creation."
  group "Shadows and reflections"
  field shadowMapSize u32 "The side of each of the directional light's cascade maps (RenderSettings.shadowCascades of them), in texels." {
    default 2048
    min 256
    max 4096
    guide "render §3"
  }
  field pointShadows PointShadows "How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser)."
  field planarReflections u32 "How many planar reflections (og.Water.planarReflection) draw at once. 1: the first asking (a grid before a mesh); others reflect as if they did not ask, logged once. 2: a second in view mirrors too; shaders read the first's (`planar_reflection`). RenderCapabilities.planarReflections reports it." {
    default 1
    min 1
    max 2
    guide "render §8"
  }
  group "Transparency"
  field transparency Transparency "How blended surfaces are composited: weighted (the default) or the exact per-pixel lists where the device allows them."
  field transparencyLayers u32 "With transparency linkedList: the fragments a pixel keeps and sorts exactly (0: 4), as far as the device's largest storage buffer holds them for every pixel at the render size; RenderCapabilities.transparency says whether the lists run at all." {
    default 0
    min 0
    max 32
  }
  group "Shaders and debugging"
  field without List.LitFeature "Optional lit features left out of every lit pipeline, on any host; empty (the default) builds them all. A scene using one left out draws as if it did not, logged once. Each name once, in any order. Read once, when the renderer is created; RenderCapabilities.without reports it." {
    guide "render §10"
  }
  field debugView DebugView "A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds."
}
```

<sub>Module `render` line 116</sub>
