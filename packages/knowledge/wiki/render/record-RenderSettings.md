<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RenderSettings"></a>

### `og.RenderSettings` (resource)

The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.

Token `og.RenderSettings`, registered under the schema name `og.RenderSettings`; its value type is `RenderSettings`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `environmentIntensity` | `number` | `0.55` | ≥ 0 | — | tunable | The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set. |
| `sky` | `boolean` | `false` | — | — | — | Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). |
| `bloom` | `boolean` | `false` | — | — | — | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold` | `number` | `1` | ≥ 0 | — | tunable | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee` | `number` | `0.5` | ≥ 0 | — | — | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity` | `number` | `0.6` | ≥ 0 | — | tunable | How much of the blurred light is added back. |
| `bloomRadius` | `number` | `-1` | -1 … 1 | — | — | Below 0 (the default) keeps Origin's bloom. 0 to 1 switches to three.js UnrealBloomPass with this radius: a luminance high pass (smoothstep from bloomThreshold to bloomThreshold + bloomKnee, the colour kept whole above it) at half resolution, five Gaussian mips (kernels 3, 5, 7, 9, 11), mixed with weights mix(f, 1.2 - f, radius) for f = 1, 0.8, 0.6, 0.4, 0.2; bloomIntensity is its strength, and the mix is added as EffectComposer's additive copy does, weighted again by its alpha (strength times the weights' sum). |
| `bloomContrast` | `number` | `1` | 0.5 … 2 | — | — | With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. |
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye, starting at the metered exposure of the first frame drawn after the first tick (a frame drawn before it, such as the web shell's first frame behind the play gate, meters at once and is never adapted from), so every host draws the same exposures for the same ticks. |
| `adaptSpeedUp` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame (never the wall clock, so a replay and every host adapt the same, however many frames each drew); 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair. |
| `toneMapped` | `boolean` | `true` | — | — | — | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping` | `ToneMapping` | `"aces"` | — | — | — | Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. |
| `decalAtlas` | `Texture` | `0` | — | — | — | The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour. |
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every model's projected level-of-detail error when its level is picked (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers less than a pixel on the screen. 1 is the engine's choice, 2 keeps full meshes twice as far, 0.5 simplifies sooner; 0 draws every model at its full mesh. |
| `bakedLighting` | `boolean` | `true` | — | — | — | Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map: to compare, or to see the realtime lights alone. |
| `quality` | `QualityPreset` | `"none"` | — | — | — | The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. |
| `rectShadows` | `boolean` | `true` | — | — | — | Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off: a rect light's shadow is the costliest light in the frame. |
| `shadowedLights` | `number` | `8` | 0 … 16 | — | — | At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.shadowedLights caps it, 16); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. |
| `shadowCascades` | `number` | `3` | 1 … 4 | — | — | The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. |
| `shadowMapScale` | `number` | `1` | 0.125 … 4 | — | — | The shadow maps at this fraction of their full side: the directional light's cascades (2048 texels by default) and the local lights' layers (512), each kept within 64 texels and the device's limit (4096 for the cascades). Changing it makes the maps again at the next frame; RenderCapabilities.shadowMapSize is the cascades' side drawn. |
| `volumetricFog` | `boolean` | `true` | — | — | — | An og.VolumetricFog is drawn lit, with its light shafts; false draws the same medium as plain distance and height fog. |
| `softParticles` | `boolean` | `true` | — | — | — | Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. |
| `screenSpaceReflections` | `boolean` | `true` | — | — | — | PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. |
| `ambientOcclusion` | `boolean` | `true` | — | — | — | PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. |
| `aoResolution` | `AoResolutionSetting` | `"postEffects"` | — | — | — | The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. The renderer reads this first, so the probe's RenderSettings and PostEffects together say what is drawn. |
| `ssrStepScale` | `number` | `1` | 0 … 32 | — | — | PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. |
| `volumetricStepScale` | `number` | `1` | 0 … 4 | — | — | An og.VolumetricFog's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), each thinner, so a shaft's edge in depth and the fog just in front of a surface resolve more finely, or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). The froxel grid's GPU time and memory grow with its slices; changing it makes the grid again at the next frame. |
| `msaaSamples` | `number` | `4` | 1 … 4 | — | — | Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it, as maxShadowedLights caps shadowedLights: where the device resolved no MSAA (TAA, FXAA, off, or FXAA for want of a multisampled HDR format) it is 1, and one sample is drawn whatever this says; RenderCapabilities.antialias is what is drawn now. Changing it makes the multisampled targets and pipelines again at the next frame. |
| `upscaler` | `Upscaler` | `"taau"` | — | — | — | Under TAA, the temporal resolve that takes the frame from the render scale (DisplaySettings.renderScale, automatic quality's) to the output: taau (the default, the low and medium presets) or fsr3 (the high preset). Automatic quality keeps it while it lowers the render scale and draws taau once its preset steps cap the quality at medium or low. A device that draws only some (RenderCapabilities.upscalers) draws taau for one it lacks. Switching starts the history again at the next frame. |
| `interpolate` | `boolean` | `true` | — | — | — | Draw what moves between the last two ticks. A display frame that falls between ticks (a 60 Hz tick on a 120 or 144 Hz screen) draws every entity's world transform and og.Camera's position and rotation blended from the tick before the last to the last, by the time past the last tick over the tick's length (positions and scales lerped, rotations slerped), so camera pans, cars and bullets glide instead of judder; skinned models blend their joints too (each joint's translation and scale lerped, rotation slerped, blendshape weights lerped; root motion and every pose the simulation reads stay at the tick), UI anchored to an entity (a document's `<anchor entity>`) stays on its drawn place, and object motion blur moves from the frame drawn before, never across a snap or a camera cut. Immediate-mode draws (og.render.sprite, og.render.text, og.debug.line and og.debug.text, placed by a system on its tick) and shader time stay at the tick. The frame then shows the world up to one tick late. A frame on a tick boundary (a host that steps whole ticks: og run without --realtime, og render, og test) draws the last tick exactly, as do the probe's screenshots and captures, so reference frames are the ticks'. Presentation only: the simulation, world_hash, snapshots, replays, lockstep, the probe and dump_* read the ticks, never the blend. og.Interpolation opts an entity out; og.render.snap, the probe's teleport and snapDistance keep a teleport from gliding. |
| `snapDistance` | `number` | `10` | ≥ 0 | `m` | tunable | An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. |

```ogt
resource RenderSettings {
  doc "The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuality sets from a graphics preset. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure."
  field environmentIntensity f32 "The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set." {
    default 0.55
    min 0
    tunable true
  }
  field sky bool "Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map)."
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
  field bloomRadius f32 "Below 0 (the default) keeps Origin's bloom. 0 to 1 switches to three.js UnrealBloomPass with this radius: a luminance high pass (smoothstep from bloomThreshold to bloomThreshold + bloomKnee, the colour kept whole above it) at half resolution, five Gaussian mips (kernels 3, 5, 7, 9, 11), mixed with weights mix(f, 1.2 - f, radius) for f = 1, 0.8, 0.6, 0.4, 0.2; bloomIntensity is its strength, and the mix is added as EffectComposer's additive copy does, weighted again by its alpha (strength times the weights' sum)." {
    default -1
    min -1
    max 1
  }
  field bloomContrast f32 "With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit." {
    default 1
    min 0.5
    max 2
  }
  field autoExposure bool "Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye, starting at the metered exposure of the first frame drawn after the first tick (a frame drawn before it, such as the web shell's first frame behind the play gate, meters at once and is never adapted from), so every host draws the same exposures for the same ticks."
  field adaptSpeedUp f32 "How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame (never the wall clock, so a replay and every host adapt the same, however many frames each drew); 0 is instant. Snapshots and probe sweeps carry the adapted exposure." {
    min 0
    max 100
    unit evPerSecond
    tunable true
  }
  field adaptSpeedDown f32 "How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair." {
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
  field decalAtlas Texture "The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour."
  field lodBias f32 "A multiplier on every model's projected level-of-detail error when its level is picked (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers less than a pixel on the screen. 1 is the engine's choice, 2 keeps full meshes twice as far, 0.5 simplifies sooner; 0 draws every model at its full mesh." {
    default 1
    min 0
    max 16
    tunable true
  }
  field bakedLighting bool "Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map: to compare, or to see the realtime lights alone." {
    default true
  }
  field quality QualityPreset "The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set."
  field rectShadows bool "Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off: a rect light's shadow is the costliest light in the frame." {
    default true
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
  field shadowMapScale f32 "The shadow maps at this fraction of their full side: the directional light's cascades (2048 texels by default) and the local lights' layers (512), each kept within 64 texels and the device's limit (4096 for the cascades). Changing it makes the maps again at the next frame; RenderCapabilities.shadowMapSize is the cascades' side drawn." {
    default 1
    min 0.125
    max 4
  }
  field volumetricFog bool "An og.VolumetricFog is drawn lit, with its light shafts; false draws the same medium as plain distance and height fog." {
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
  field aoResolution AoResolutionSetting "The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. The renderer reads this first, so the probe's RenderSettings and PostEffects together say what is drawn."
  field ssrStepScale f32 "PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps." {
    default 1
    min 0
    max 32
  }
  field volumetricStepScale f32 "An og.VolumetricFog's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), each thinner, so a shaft's edge in depth and the fog just in front of a surface resolve more finely, or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). The froxel grid's GPU time and memory grow with its slices; changing it makes the grid again at the next frame." {
    default 1
    min 0
    max 4
  }
  field msaaSamples u32 "Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it, as maxShadowedLights caps shadowedLights: where the device resolved no MSAA (TAA, FXAA, off, or FXAA for want of a multisampled HDR format) it is 1, and one sample is drawn whatever this says; RenderCapabilities.antialias is what is drawn now. Changing it makes the multisampled targets and pipelines again at the next frame." {
    default 4
    min 1
    max 4
  }
  field upscaler Upscaler "Under TAA, the temporal resolve that takes the frame from the render scale (DisplaySettings.renderScale, automatic quality's) to the output: taau (the default, the low and medium presets) or fsr3 (the high preset). Automatic quality keeps it while it lowers the render scale and draws taau once its preset steps cap the quality at medium or low. A device that draws only some (RenderCapabilities.upscalers) draws taau for one it lacks. Switching starts the history again at the next frame."
  field interpolate bool "Draw what moves between the last two ticks. A display frame that falls between ticks (a 60 Hz tick on a 120 or 144 Hz screen) draws every entity's world transform and og.Camera's position and rotation blended from the tick before the last to the last, by the time past the last tick over the tick's length (positions and scales lerped, rotations slerped), so camera pans, cars and bullets glide instead of judder; skinned models blend their joints too (each joint's translation and scale lerped, rotation slerped, blendshape weights lerped; root motion and every pose the simulation reads stay at the tick), UI anchored to an entity (a document's `<anchor entity>`) stays on its drawn place, and object motion blur moves from the frame drawn before, never across a snap or a camera cut. Immediate-mode draws (og.render.sprite, og.render.text, og.debug.line and og.debug.text, placed by a system on its tick) and shader time stay at the tick. The frame then shows the world up to one tick late. A frame on a tick boundary (a host that steps whole ticks: og run without --realtime, og render, og test) draws the last tick exactly, as do the probe's screenshots and captures, so reference frames are the ticks'. Presentation only: the simulation, world_hash, snapshots, replays, lockstep, the probe and dump_* read the ticks, never the blend. og.Interpolation opts an entity out; og.render.snap, the probe's teleport and snapDistance keep a teleport from gliding." {
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

<sub>Module `render` line 583</sub>
