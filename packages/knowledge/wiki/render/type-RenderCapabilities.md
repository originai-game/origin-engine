<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RenderCapabilities"></a>

### `RenderCapabilities` (struct)

What the renderer can do on this device. Backend limits are reported here, never by missing members.

#### Device

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The backend. |
| `compute` | `boolean` | `false` | — | — | — | Whether compute passes run (GPU culling, GPU particles). |
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing the device resolved RenderOptions.antialias to. |
| `hdrSupported` | `boolean` | `false` | — | — | — | Whether this surface can be configured for float extended-sRGB HDR output. This is surface-path support, not a measurement of the monitor or current OS HDR headroom. False without a surface. |
| `hdr` | `boolean` | `false` | — | — | — | Whether HDR output was requested and the surface configured for it: then the LPM HDR display mapper replaces PostEffects.tonemap (including none); SDR fallback keeps the chosen operator. False without a surface. Paint, ink and quantization stay SDR; captures are clipped SDR previews. |

#### Lights and shadows

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxLights` | `number` | `0` | — | — | — | How many point and spot lights one frame draws (MAX_LIGHTS on every tier). |
| `maxShadowedLights` | `number` | `0` | — | — | — | How many point and spot lights cast shadows in one frame (8); past it, the shadowed lights brightest at the camera keep theirs. |

#### Antialiasing

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxMsaaSamples` | `number` | `0` | — | — | — | The samples per pixel the scene is multisampled at, which caps RenderSettings.msaaSamples: 4 where antialias resolved to msaa4 when the renderer was made, 1 where it resolved to taa, fxaa or off, 0 without a renderer. It stays 4 while RenderSettings.msaaSamples 1 draws FXAA. See `looks §18`. |
| `pointShadows` | `PointShadows` | `"auto"` | — | — | — | How point lights' shadows are stored on this device: cube or paraboloid (what RenderOptions.pointShadows resolved to). |

#### Target

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | — | — | The render target width in device pixels. |
| `height` | `number` | `0` | — | — | — | The render target height in device pixels. |

#### Stylisation

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stylise` | `readonly string[]` | `[]` | — | — | — | The Stylise and MaterialStyle primitive groups this renderer draws, by name (pixel, posterize, dither, paint, watercolor, surfaceSpace, toon, outline, wireframe, ...). A look that uses one this list lacks still applies; that primitive is not drawn. See `looks §18`. |

#### Reflections and targets

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ssr` | `boolean` | `false` | — | — | — | Whether PostEffects.ssr draws: true with a renderer, false without one. |
| `reflectionProbes` | `number` | `0` | — | — | — | How many og.ReflectionProbe components one frame samples (MAX_REFLECTION_PROBES on every tier). |
| `renderTargetDepth` | `boolean` | `false` | — | — | — | Whether a render target made with `depth` gets its view's depth drawn: true with a renderer, false without one. The depth comes from the view's own depth prepass of its opaque meshes into a linear-depth colour target, never from its depth buffer. |
| `maxRenderTargetSize` | `number` | `0` | — | — | — | The largest side of a render target (the device's largest 2D texture). |

#### Post and fog

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxCustomPosts` | `number` | `0` | — | — | — | The most custom post passes a frame runs: 8 on every tier, PostEffects.custom's capacity. |
| `volumetricFog` | `VolumetricFog` | `"froxel"` | — | — | — | How Fog.volumetric is drawn on this device: a froxel grid or a raymarch along each pixel's view ray, each lit by the sun through its shadows and by the point, spot and rect lights around each sample (up to volumetricLights of them) through their shadows and cookies. |
| `volumetricLights` | `number` | `0` | — | — | — | The most point, spot and rect lights that light one sample of Fog.volumetric: those of its screen cluster, MAX_CLUSTER_LIGHTS (64) in the froxel grid, 8 in the raymarch, the brightest at the camera first. |

#### Cookies

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxLightCookies` | `number` | `0` | — | — | — | The most light cookie layers one frame draws (24 on every tier): a SpotLight.cookie takes one, a PointLight.cookie six, and a texture lights of one kind share takes its layers once. Cookies past it, in the order of their lights' entities, are not drawn, and the log warns. |

#### Timing

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `gpuTimestamps` | `boolean` | `false` | — | — | — | Whether the device times each render pass on the GPU (timestamp queries: Vulkan, Metal and D3D12 adapters that have them, WebGPU with `timestamp-query`; never in a publish build, which leaves the queries out): ProbeTimings.gpuPasses and the trace's gpu thread are filled only where it is true. |

#### Filtering

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `spriteAnisotropy` | `number` | `0` | — | — | — | The anisotropic filtering of sprites and og.ui's images (never text): 16 where the device filters anisotropically (WebGPU, Vulkan, Metal, DX12), 1 where it has none and on a software rasterizer, 0 without a renderer. Above 1 an image squeezed along one axis keeps the other's detail; at 1 it blurs. See `looks §18`. |

#### Water

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `planarReflections` | `number` | `0` | — | — | — | How many water surfaces draw a planar reflection in one frame: RenderOptions.planarReflections (1 by default, at most 2), 0 without a renderer or where world3d is false. A surface past it that asks reflects by the screen-space and sky reflections instead. |

#### Optional features

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `without` | `readonly LitFeature[]` | `[]` | — | — | — | The optional lit features the renderer was made without (RenderOptions.without), in LitFeature's order: none of its lit shaders carries them and a scene that uses one draws without it. Empty by default and without a renderer. |
| `upscalers` | `readonly string[]` | `[]` | — | — | — | The Upscaler variants this device draws under TAA: taau and fsr3 on every host and tier with a renderer (FSR 3.1 is ported in fp32 without subgroups, float atomics or device-wide coherence, so it needs neither shader-f16 nor subgroups, which Safari and Firefox lack), empty without one. |

#### Transparency

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `transparency` | `Transparency` | `"weighted"` | — | — | — | What RenderOptions.transparency resolved to on this device: linkedList only where it was asked for and the device writes storage from fragment shaders with room for one more storage buffer in its lit pipelines; weighted otherwise and without a renderer. |

#### Crowds

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `gpuCrowds` | `boolean` | `false` | — | — | — | Whether og.CrowdAnimation crowds are culled (frustum and occlusion), given their level of detail and drawn on the GPU, tens of thousands at a time. Where false, and without a renderer, the CPU culls them: the same members, levels and frame at a higher CPU cost. See `looks §18`. |

#### Decoding

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `exrDecode` | `boolean` | `false` | — | — | — | Whether og.load.environmentData reads OpenEXR (.exr): true on the native host, false on the web, whose module carries no EXR decoder (a Radiance .hdr loads on every host, and a cooked environment's .exr is converted by the cook, so a shipped game never needs this). Known without a renderer. |

#### Visibility buffer

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visibilityBuffer` | `boolean` | `false` | — | — | — | Whether the visibility buffer draws the camera's small clusters of clustered meshes (rasterised in compute). False without a renderer and once the device refused the resolve (visibilityBufferRefused): those clusters then draw on the hardware path. True until the first frame with a clustered mesh. See `looks §18`. |
| `visibilityBufferRefused` | `string \| null` | `null` | — | — | — | What the device said when it refused the visibility buffer's resolve pipelines (Chrome on the Pixel 10's PowerVR D-Series answers VK_ERROR_UNKNOWN), or null. |

#### Visual simulation

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visualSimulation` | `boolean` | `false` | — | — | — | Whether visual bodies (og.Cloth, og.SoftBody and og.Garment with SIMULATION_VISUAL) simulate here: the device runs compute and reads storage in the vertex stage. Where false they draw their rest shape, as their points read in the tick; false without a renderer. See `looks §18`. |

#### 3D world

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `world3d` | `boolean` | `false` | — | — | — | Whether the renderer draws the 3D world: terrain, planar water (og.Water), 3D fluids, scattered copies and volumetric fog. False in the 2D profile's publish and lanes modules, where og.terrain's calls and a SHAPE_HEIGHTFIELD collider are refused naming this field; true elsewhere. See `looks §18`. |

```ogt
struct RenderCapabilities {
  doc "What the renderer can do on this device. Backend limits are reported here, never by missing members."
  group "Device"
  field tier RenderTier "The backend."
  field compute bool "Whether compute passes run (GPU culling, GPU particles)."
  field antialias Antialias "The antialiasing the device resolved RenderOptions.antialias to."
  field hdrSupported bool "Whether this surface can be configured for float extended-sRGB HDR output. This is surface-path support, not a measurement of the monitor or current OS HDR headroom. False without a surface."
  field hdr bool "Whether HDR output was requested and the surface configured for it: then the LPM HDR display mapper replaces PostEffects.tonemap (including none); SDR fallback keeps the chosen operator. False without a surface. Paint, ink and quantization stay SDR; captures are clipped SDR previews."
  group "Lights and shadows"
  field maxLights u32 "How many point and spot lights one frame draws (MAX_LIGHTS on every tier)."
  field maxShadowedLights u32 "How many point and spot lights cast shadows in one frame (8); past it, the shadowed lights brightest at the camera keep theirs."
  group "Antialiasing"
  field maxMsaaSamples u32 "The samples per pixel the scene is multisampled at, which caps RenderSettings.msaaSamples: 4 where antialias resolved to msaa4 when the renderer was made, 1 where it resolved to taa, fxaa or off, 0 without a renderer. It stays 4 while RenderSettings.msaaSamples 1 draws FXAA." {
    guide "looks §18"
  }
  field pointShadows PointShadows "How point lights' shadows are stored on this device: cube or paraboloid (what RenderOptions.pointShadows resolved to)."
  group "Target"
  field width u32 "The render target width in device pixels."
  field height u32 "The render target height in device pixels."
  group "Stylisation"
  field stylise List.String "The Stylise and MaterialStyle primitive groups this renderer draws, by name (pixel, posterize, dither, paint, watercolor, surfaceSpace, toon, outline, wireframe, ...). A look that uses one this list lacks still applies; that primitive is not drawn." {
    guide "looks §18"
  }
  group "Reflections and targets"
  field ssr bool "Whether PostEffects.ssr draws: true with a renderer, false without one."
  field reflectionProbes u32 "How many og.ReflectionProbe components one frame samples (MAX_REFLECTION_PROBES on every tier)."
  field renderTargetDepth bool "Whether a render target made with `depth` gets its view's depth drawn: true with a renderer, false without one. The depth comes from the view's own depth prepass of its opaque meshes into a linear-depth colour target, never from its depth buffer."
  field maxRenderTargetSize u32 "The largest side of a render target (the device's largest 2D texture)."
  group "Post and fog"
  field maxCustomPosts u32 "The most custom post passes a frame runs: 8 on every tier, PostEffects.custom's capacity."
  field volumetricFog VolumetricFog "How Fog.volumetric is drawn on this device: a froxel grid or a raymarch along each pixel's view ray, each lit by the sun through its shadows and by the point, spot and rect lights around each sample (up to volumetricLights of them) through their shadows and cookies."
  field volumetricLights u32 "The most point, spot and rect lights that light one sample of Fog.volumetric: those of its screen cluster, MAX_CLUSTER_LIGHTS (64) in the froxel grid, 8 in the raymarch, the brightest at the camera first."
  group "Cookies"
  field maxLightCookies u32 "The most light cookie layers one frame draws (24 on every tier): a SpotLight.cookie takes one, a PointLight.cookie six, and a texture lights of one kind share takes its layers once. Cookies past it, in the order of their lights' entities, are not drawn, and the log warns."
  group "Timing"
  field gpuTimestamps bool "Whether the device times each render pass on the GPU (timestamp queries: Vulkan, Metal and D3D12 adapters that have them, WebGPU with `timestamp-query`; never in a publish build, which leaves the queries out): ProbeTimings.gpuPasses and the trace's gpu thread are filled only where it is true."
  group "Filtering"
  field spriteAnisotropy u32 "The anisotropic filtering of sprites and og.ui's images (never text): 16 where the device filters anisotropically (WebGPU, Vulkan, Metal, DX12), 1 where it has none and on a software rasterizer, 0 without a renderer. Above 1 an image squeezed along one axis keeps the other's detail; at 1 it blurs." {
    guide "looks §18"
  }
  group "Water"
  field planarReflections u32 "How many water surfaces draw a planar reflection in one frame: RenderOptions.planarReflections (1 by default, at most 2), 0 without a renderer or where world3d is false. A surface past it that asks reflects by the screen-space and sky reflections instead."
  group "Optional features"
  field without List.LitFeature "The optional lit features the renderer was made without (RenderOptions.without), in LitFeature's order: none of its lit shaders carries them and a scene that uses one draws without it. Empty by default and without a renderer."
  field upscalers List.String "The Upscaler variants this device draws under TAA: taau and fsr3 on every host and tier with a renderer (FSR 3.1 is ported in fp32 without subgroups, float atomics or device-wide coherence, so it needs neither shader-f16 nor subgroups, which Safari and Firefox lack), empty without one."
  group "Transparency"
  field transparency Transparency "What RenderOptions.transparency resolved to on this device: linkedList only where it was asked for and the device writes storage from fragment shaders with room for one more storage buffer in its lit pipelines; weighted otherwise and without a renderer."
  group "Crowds"
  field gpuCrowds bool "Whether og.CrowdAnimation crowds are culled (frustum and occlusion), given their level of detail and drawn on the GPU, tens of thousands at a time. Where false, and without a renderer, the CPU culls them: the same members, levels and frame at a higher CPU cost." {
    guide "looks §18"
  }
  group "Decoding"
  field exrDecode bool "Whether og.load.environmentData reads OpenEXR (.exr): true on the native host, false on the web, whose module carries no EXR decoder (a Radiance .hdr loads on every host, and a cooked environment's .exr is converted by the cook, so a shipped game never needs this). Known without a renderer."
  group "Visibility buffer"
  field visibilityBuffer bool "Whether the visibility buffer draws the camera's small clusters of clustered meshes (rasterised in compute). False without a renderer and once the device refused the resolve (visibilityBufferRefused): those clusters then draw on the hardware path. True until the first frame with a clustered mesh." {
    guide "looks §18"
  }
  field visibilityBufferRefused Option.String "What the device said when it refused the visibility buffer's resolve pipelines (Chrome on the Pixel 10's PowerVR D-Series answers VK_ERROR_UNKNOWN), or null."
  group "Visual simulation"
  field visualSimulation bool "Whether visual bodies (og.Cloth, og.SoftBody and og.Garment with SIMULATION_VISUAL) simulate here: the device runs compute and reads storage in the vertex stage. Where false they draw their rest shape, as their points read in the tick; false without a renderer." {
    guide "looks §18"
  }
  group "3D world"
  field world3d bool "Whether the renderer draws the 3D world: terrain, planar water (og.Water), 3D fluids, scattered copies and volumetric fog. False in the 2D profile's publish and lanes modules, where og.terrain's calls and a SHAPE_HEIGHTFIELD collider are refused naming this field; true elsewhere." {
    guide "looks §18"
  }
}
```

<sub>Module `render` line 2761</sub>
