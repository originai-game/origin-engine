<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RenderSettings"></a>

### `og.RenderSettings` (resource)

The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.

Token `og.RenderSettings`, registered under the schema name `og.RenderSettings`; its value type is `RenderSettings`.

#### Sky light

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `environmentIntensity` | `number` | `0.55` | ≥ 0 | — | tunable | The strength of the image-based ambient light from the sky on surfaces; og.Environment.intensity takes its place while an environment map is set. The sky and its clouds are drawn as they are whatever it is. |
| `sky` | `boolean` | `false` | — | — | — | Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). |

#### Bloom

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bloom` | `boolean` | `false` | — | — | — | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold` | `number` | `1` | ≥ 0 | — | tunable | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee` | `number` | `0.5` | ≥ 0 | — | — | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity` | `number` | `0.6` | ≥ 0 | — | tunable | How much of the blurred light is added back. |
| `bloomRadius` | `number` | `-1` | -1 … 1 | — | — | Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy. |
| `bloomContrast` | `number` | `1` | 0.5 … 2 | — | — | With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. |

#### Exposure and tone mapping

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to a key that follows its brightness (middle grey for a daylit view), then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick, so hosts draw alike. |
| `autoExposureMin` | `number` | `-10` | -20 … 20 | — | tunable | The least exposure auto exposure applies, in stops (EV: the image times 2^EV, Camera.exposure's compensation included): a very bright view is darkened no further. |
| `autoExposureMax` | `number` | `10` | -20 … 20 | — | tunable | The most exposure auto exposure applies, in stops (EV, compensation included): a dark view is brightened no further, so a night stays as dark as its light makes it. Below autoExposureMin it is taken as autoExposureMin. |
| `adaptSpeedUp` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. |
| `toneMapped` | `boolean` | `true` | — | — | — | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping` | `ToneMapping` | `"aces"` | — | — | — | Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. |

#### Scene

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full. |
| `bakedLighting` | `boolean` | `true` | — | — | — | Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map. |
| `staticLightReach` | `number` | `0` | ≥ 0 | `m` | tunable | How far from itself a stationary or static light (LightMobility) has its diffuse light shaded per pixel; past it, that light comes from a grid of probes round the camera and such a rect light skips rough surfaces. Glossy surfaces keep the highlights. 0 shades every light whole. |

#### Quality

The fields og.render.applyQuality writes from a graphics preset.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quality` | `QualityPreset` | `"none"` | — | — | — | The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. |
| `rectShadows` | `boolean` | `true` | — | — | — | Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off. |
| `shadowedLights` | `number` | `8` | 0 … 16 | — | — | At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.maxShadowedLights caps it); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. |
| `shadowCascades` | `number` | `3` | 1 … 4 | — | — | The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. |
| `shadowMapScale` | `number` | `1` | 0.125 … 4 | — | — | The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame. |
| `volumetricFog` | `boolean` | `true` | — | — | — | A volumetric og.Fog (Fog.volumetric) is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even. |
| `skyCaptureSliced` | `boolean` | `false` | — | — | — | Spread live sky capture and filtering over 13 rendered frames instead of completing each frame. Low sets true; medium, high and the default keep false so terrain ambient and reflections follow the current sky. Slicing lowers peak GPU cost but retains older lighting during fast sky changes. |
| `softParticles` | `boolean` | `true` | — | — | — | Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. |
| `particleEmission` | `number` | `1` | 0 … 1 | — | — | Every particle system's emission times this (0 to 1); an emitter left with fewer than 8 particles alive at once keeps more, so a flash or a ring stays whole. The low preset draws 0.5. Drawing only: particle events and collisions are the full emitter's. |
| `maxVfx` | `number` | `64` | — | — | — | At most this many composed effects (og.Vfx) drawn at once: past it the oldest one-shot effects are left out, then the oldest looping ones, and the probe's dump_render counts them (vfx.dropped). 64; the low preset 32, high 128. Drawing only: their events, shakes and sounds still come. |
| `maxVfxParticles` | `number` | `65536` | — | — | — | At most this many particle slots (the emitters' maxParticles, summed) among the og.Vfx drawn: past it the oldest one-shot effects are left out, then the oldest looping ones, as with maxVfx. 65536; the low preset 32768, high 131072. |
| `screenSpaceReflections` | `boolean` | `true` | — | — | — | PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. |
| `ambientOcclusion` | `boolean` | `true` | — | — | — | PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. |
| `aoResolution` | `AoResolutionSetting` | `"postEffects"` | — | — | — | The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. |
| `ssrStepScale` | `number` | `1` | 0 … 32 | — | — | PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. |
| `volumetricStepScale` | `number` | `1` | 0 … 4 | — | — | Fog.volumetric's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame. |
| `msaaSamples` | `number` | `4` | 1 … 4 | — | — | Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame. |
| `upscaler` | `Upscaler` | `"auto"` | — | — | — | Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: auto (the default and medium: fsr3 below scale 1, taau at it), taau (low) or fsr3 (high). A device lacking fsr3 (RenderCapabilities.upscalers) draws taau. Switching restarts the history. |

#### Interpolation

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `interpolate` | `boolean` | `true` | — | — | — | Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only. |
| `snapDistance` | `number` | `10` | ≥ 0 | `m` | tunable | An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. |

```ogt
resource RenderSettings {
  doc "The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure."
  group "Sky light"
  field environmentIntensity f32 "The strength of the image-based ambient light from the sky on surfaces; og.Environment.intensity takes its place while an environment map is set. The sky and its clouds are drawn as they are whatever it is." {
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
  }
  field bloomContrast f32 "With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit." {
    default 1
    min 0.5
    max 2
  }
  group "Exposure and tone mapping"
  field autoExposure bool "Meter every frame to a key that follows its brightness (middle grey for a daylit view), then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick, so hosts draw alike." {
  }
  field autoExposureMin f32 "The least exposure auto exposure applies, in stops (EV: the image times 2^EV, Camera.exposure's compensation included): a very bright view is darkened no further." {
    default -10
    min -20
    max 20
    tunable true
  }
  field autoExposureMax f32 "The most exposure auto exposure applies, in stops (EV, compensation included): a dark view is brightened no further, so a night stays as dark as its light makes it. Below autoExposureMin it is taken as autoExposureMin." {
    default 10
    min -20
    max 20
    tunable true
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
  field lodBias f32 "A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full." {
    default 1
    min 0
    max 16
    tunable true
  }
  field bakedLighting bool "Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map." {
    default true
  }
  field staticLightReach f32 "How far from itself a stationary or static light (LightMobility) has its diffuse light shaded per pixel; past it, that light comes from a grid of probes round the camera and such a rect light skips rough surfaces. Glossy surfaces keep the highlights. 0 shades every light whole." {
    default 0
    min 0
    unit m
    tunable true
  }
  group "Quality" "The fields og.render.applyQuality writes from a graphics preset."
  field quality QualityPreset "The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set."
  field rectShadows bool "Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off." {
    default true
  }
  field shadowedLights u32 "At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.maxShadowedLights caps it); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted." {
    default 8
    min 0
    max 16
  }
  field shadowCascades u32 "The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame." {
    default 3
    min 1
    max 4
  }
  field shadowMapScale f32 "The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame." {
    default 1
    min 0.125
    max 4
  }
  field volumetricFog bool "A volumetric og.Fog (Fog.volumetric) is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even." {
    default true
  }
  field skyCaptureSliced bool "Spread live sky capture and filtering over 13 rendered frames instead of completing each frame. Low sets true; medium, high and the default keep false so terrain ambient and reflections follow the current sky. Slicing lowers peak GPU cost but retains older lighting during fast sky changes."
  field softParticles bool "Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them." {
    default true
  }
  field particleEmission f32 "Every particle system's emission times this (0 to 1); an emitter left with fewer than 8 particles alive at once keeps more, so a flash or a ring stays whole. The low preset draws 0.5. Drawing only: particle events and collisions are the full emitter's." {
    default 1
    min 0
    max 1
  }
  field maxVfx u32 "At most this many composed effects (og.Vfx) drawn at once: past it the oldest one-shot effects are left out, then the oldest looping ones, and the probe's dump_render counts them (vfx.dropped). 64; the low preset 32, high 128. Drawing only: their events, shakes and sounds still come." {
    default 64
  }
  field maxVfxParticles u32 "At most this many particle slots (the emitters' maxParticles, summed) among the og.Vfx drawn: past it the oldest one-shot effects are left out, then the oldest looping ones, as with maxVfx. 65536; the low preset 32768, high 131072." {
    default 65536
  }
  field screenSpaceReflections bool "PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way." {
    default true
  }
  field ambientOcclusion bool "PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way." {
    default true
  }
  field aoResolution AoResolutionSetting "The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again." {
  }
  field ssrStepScale f32 "PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps." {
    default 1
    min 0
    max 32
  }
  field volumetricStepScale f32 "Fog.volumetric's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame." {
    default 1
    min 0
    max 4
  }
  field msaaSamples u32 "Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame." {
    default 4
    min 1
    max 4
  }
  field upscaler Upscaler "Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: auto (the default and medium: fsr3 below scale 1, taau at it), taau (low) or fsr3 (high). A device lacking fsr3 (RenderCapabilities.upscalers) draws taau. Switching restarts the history." {
    default auto
  }
  group "Interpolation"
  field interpolate bool "Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only." {
    default true
  }
  field snapDistance f32 "An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance." {
    default 10
    min 0
    unit m
    tunable true
  }
}
```

<sub>Module `render` line 734</sub>
