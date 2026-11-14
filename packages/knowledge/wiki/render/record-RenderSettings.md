<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RenderSettings"></a>

### `og.RenderSettings` (resource)

The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.

See `render §5`.

Token `og.RenderSettings`, registered under the schema name `og.RenderSettings`; its value type is `RenderSettings`.

#### Sky light

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `environmentIntensity` | `number` | `0.55` | ≥ 0 | — | tunable | The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set. |
| `sky` | `boolean` | `false` | — | — | — | Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). |

#### Bloom

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bloom` | `boolean` | `false` | — | — | — | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold` | `number` | `1` | ≥ 0 | — | tunable | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee` | `number` | `0.5` | ≥ 0 | — | — | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity` | `number` | `0.6` | ≥ 0 | — | tunable | How much of the blurred light is added back. |
| `bloomRadius` | `number` | `-1` | -1 … 1 | — | — | Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy. See `render §5`. |
| `bloomContrast` | `number` | `1` | 0.5 … 2 | — | — | With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. |

#### Exposure and tone mapping

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick (earlier frames meter at once), so every host draws the same exposures. See `render §5`. |
| `adaptSpeedUp` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. See `render §5`. |
| `toneMapped` | `boolean` | `true` | — | — | — | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping` | `ToneMapping` | `"aces"` | — | — | — | Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. |

#### Scene

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `decalAtlas` | `Texture` | `0` | — | — | — | The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour. |
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full. See `render §10`. |
| `bakedLighting` | `boolean` | `true` | — | — | — | Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map. See `render §4`. |

#### Quality

The fields og.render.applyQuality writes from a graphics preset.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quality` | `QualityPreset` | `"none"` | — | — | — | The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. |
| `rectShadows` | `boolean` | `true` | — | — | — | Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off. See `render §3`. |
| `shadowedLights` | `number` | `8` | 0 … 16 | — | — | At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.shadowedLights caps it, 16); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. |
| `shadowCascades` | `number` | `3` | 1 … 4 | — | — | The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. |
| `shadowMapScale` | `number` | `1` | 0.125 … 4 | — | — | The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame; RenderCapabilities.shadowMapSize is the cascades' side drawn. |
| `volumetricFog` | `boolean` | `true` | — | — | — | An og.VolumetricFog is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even. |
| `softParticles` | `boolean` | `true` | — | — | — | Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. |
| `screenSpaceReflections` | `boolean` | `true` | — | — | — | PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. |
| `ambientOcclusion` | `boolean` | `true` | — | — | — | PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. |
| `aoResolution` | `AoResolutionSetting` | `"postEffects"` | — | — | — | The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. See `render §5`. |
| `ssrStepScale` | `number` | `1` | 0 … 32 | — | — | PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. |
| `volumetricStepScale` | `number` | `1` | 0 … 4 | — | — | An og.VolumetricFog's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame. See `render §7`. |
| `msaaSamples` | `number` | `4` | 1 … 4 | — | — | Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame. See `render §2`. |
| `upscaler` | `Upscaler` | `"taau"` | — | — | — | Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: taau (the default; the low and medium presets) or fsr3 (the high preset). A device lacking one (RenderCapabilities.upscalers) draws taau. Switching restarts the history next frame. See `render §1`. |

#### Interpolation

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `interpolate` | `boolean` | `true` | — | — | — | Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only. See `render §9`. |
| `snapDistance` | `number` | `10` | ≥ 0 | `m` | tunable | An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. |

```ogt
resource RenderSettings {
  doc "The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure."
  guide "render §5"
  group "Sky light"
  field environmentIntensity f32 "The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set." {
    default 0.55
    min 0
    tunable true
  }
  field sky bool "Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map)."
  group "Bloom"
  field bloom bool "Light above the threshold bleeds into its surroundings."
  field bloomThreshold f32 "Brightness after exposure where bloom starts; 1 is display white." {
    default 1
    min 0
    tunable true
  }
  field bloomKnee f32 "Softens the threshold over threshold plus or minus knee; 0 is a hard cut." {
    default 0.5
    min 0
  }
  field bloomIntensity f32 "How much of the blurred light is added back." {
    default 0.6
    min 0
    tunable true
  }
  field bloomRadius f32 "Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy." {
    default -1
    min -1
    max 1
    guide "render §5"
  }
  field bloomContrast f32 "With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit." {
    default 1
    min 0.5
    max 2
  }
  group "Exposure and tone mapping"
  field autoExposure bool "Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick (earlier frames meter at once), so every host draws the same exposures." {
    guide "render §5"
  }
  field adaptSpeedUp f32 "How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure." {
    min 0
    max 100
    unit evPerSecond
    tunable true
  }
  field adaptSpeedDown f32 "How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant." {
    min 0
    max 100
    unit evPerSecond
    tunable true
    guide "render §5"
  }
  field toneMapped bool "Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1." {
    default true
  }
  field toneMapping ToneMapping "Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none."
  field exposure f32 "A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are." {
    default 1
    min 0
    max 16
    tunable true
  }
  group "Scene"
  field decalAtlas Texture "The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour."
  field lodBias f32 "A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full." {
    default 1
    min 0
    max 16
    tunable true
    guide "render §10"
  }
  field bakedLighting bool "Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map." {
    default true
    guide "render §4"
  }
  group "Quality" "The fields og.render.applyQuality writes from a graphics preset."
  field quality QualityPreset "The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set."
  field rectShadows bool "Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off." {
    default true
    guide "render §3"
  }
  field shadowedLights u32 "At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.shadowedLights caps it, 16); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted." {
    default 8
    min 0
    max 16
  }
  field shadowCascades u32 "The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame." {
    default 3
    min 1
    max 4
  }
  field shadowMapScale f32 "The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame; RenderCapabilities.shadowMapSize is the cascades' side drawn." {
    default 1
    min 0.125
    max 4
  }
  field volumetricFog bool "An og.VolumetricFog is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even." {
    default true
  }
  field softParticles bool "Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them." {
    default true
  }
  field screenSpaceReflections bool "PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way." {
    default true
  }
  field ambientOcclusion bool "PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way." {
    default true
  }
  field aoResolution AoResolutionSetting "The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again." {
    guide "render §5"
  }
  field ssrStepScale f32 "PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps." {
    default 1
    min 0
    max 32
  }
  field volumetricStepScale f32 "An og.VolumetricFog's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame." {
    default 1
    min 0
    max 4
    guide "render §7"
  }
  field msaaSamples u32 "Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame." {
    default 4
    min 1
    max 4
    guide "render §2"
  }
  field upscaler Upscaler "Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: taau (the default; the low and medium presets) or fsr3 (the high preset). A device lacking one (RenderCapabilities.upscalers) draws taau. Switching restarts the history next frame." {
    guide "render §1"
  }
  group "Interpolation"
  field interpolate bool "Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only." {
    default true
    guide "render §9"
  }
  field snapDistance f32 "An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance." {
    default 10
    min 0
    unit m
    tunable true
  }
}
```

<sub>Module `render` line 679</sub>
