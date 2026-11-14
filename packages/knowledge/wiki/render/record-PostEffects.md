<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-PostEffects"></a>

### `og.PostEffects` (resource)

The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then records no pass. Read after each tick; it never feeds back into the simulation.

AO, outlines, depth of field, motion blur and god rays turn on the depth-normal prepass. Grading, the colour adjustments, vignette, aberration, grain and the flash apply to the display colour after tone mapping, lens distortion bends the result, and the overlay comes after all of it.

See `render §5`.

Token `og.PostEffects`, registered under the schema name `og.PostEffects`; its value type is `PostEffects`.

#### Ambient occlusion

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ao` | `boolean` | `false` | — | — | — | Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. |
| `aoRadius` | `number` | `0.6` | ≥ 0 | `m` | tunable | The world-space radius searched for occluders. |
| `aoIntensity` | `number` | `1` | ≥ 0 | — | tunable | 0 none, 1 full; above 1 exaggerates. |
| `aoMethod` | `AoMethod` | `"auto"` | — | — | — | The algorithm. |
| `aoResolution` | `AoResolution` | `"full"` | — | — | — | Full resolution, or half for a fraction of the GPU time at softer detail. |
| `aoDirect` | `number` | `0` | 0 … 1 | — | tunable | How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it. |
| `aoRadiusScale` | `number` | `0` | ≥ 0 | — | tunable | Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere. See `render §5`. |

#### Outlines

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `outline` | `boolean` | `false` | — | — | — | Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The line colour, linear. |
| `outlineWidth` | `number` | `2` | 0 … 31 | — | tunable | Line width in pixels, drawn inside the silhouette. |
| `outlineDepthThreshold` | `number` | `0.04` | ≥ 0 | — | — | The relative depth step that counts as a silhouette (0.04 is 4 %). |
| `outlineNormalThreshold` | `number` | `0.4` | ≥ 0 | — | — | The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). |

#### Depth of field

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `depthOfField` | `boolean` | `false` | — | — | — | Bokeh depth of field: pixels away from the focus distance blur into discs. |
| `dofFocusDistance` | `number` | `10` | ≥ 0 | `m` | tunable | The distance from the camera that is sharp. |
| `dofFocusRange` | `number` | `2` | ≥ 0 | `m` | tunable | The half-width of the sharp band around it. |
| `dofMaxBlur` | `number` | `0.012` | 0 … 0.1 | — | — | The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). |
| `dofFalloff` | `number` | `4` | ≥ 0 | — | — | How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. |
| `dofQuality` | `DofQuality` | `"low"` | — | — | — | Taps per pixel. |

#### Motion blur

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `motionBlur` | `boolean` | `false` | — | — | — | Per-object and camera motion blur along the screen-space velocity. |
| `motionBlurShutter` | `number` | `0.5` | 0 … 2 | — | tunable | The shutter as a fraction of the frame (0.5 is a 180 degree shutter). |
| `motionBlurSamples` | `number` | `12` | 4 … 32 | — | — | Taps along the velocity. |
| `motionBlurMaxPixels` | `number` | `40` | ≥ 1 | — | — | The longest blur in pixels. |

#### Screen blur

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `screenBlur` | `ScreenBlur` | `"none"` | — | — | — | A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0. |
| `screenBlurStrength` | `number` | `0` | ≥ 0 | — | tunable | How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off. |
| `screenBlurCenter` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | tunable, worldspace `none` | Zoom's and spin's centre, in fractions of the frame from the top left. |
| `screenBlurAngle` | `number` | `0` | — | `deg` | tunable | Directional's direction, counter-clockwise from the right (90 is vertical). |
| `screenBlurSamples` | `number` | `16` | 4 … 64 | — | — | Taps per pixel: more is smoother on long blurs, each one a texture read. |

#### God rays

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `godRays` | `boolean` | `false` | — | — | — | Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). |
| `godRaysIntensity` | `number` | `0.5` | ≥ 0 | — | tunable | How bright the shafts are. |
| `godRaysDecay` | `number` | `0.965` | 0.5 … 1 | — | — | How fast the rays fade along their length. |
| `godRaysDensity` | `number` | `1` | ≥ 0 | — | — | How far towards the sun each ray samples (a fraction of the distance). |
| `godRaysTint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint on the light the rays carry. |
| `godRaysSamples` | `number` | `48` | 8 … 128 | — | — | Taps per ray. |
| `godRaysThreshold` | `number` | `2` | ≥ 0 | — | — | Sky brightness (linear, before exposure) below which the sky carries no rays. |

#### 2D glow

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `glow` | `boolean` | `false` | — | — | — | Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. |
| `glowIntensity` | `number` | `1` | ≥ 0 | — | tunable | How much of the blurred glow is added back. |

#### Grading

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lut` | `Lut` | `0` | — | — | — | Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). |
| `lutIntensity` | `number` | `1` | 0 … 1 | — | tunable | 0 leaves the image as tone mapped, 1 applies the table fully. |

#### Colour adjustments

Applied in field order to the display colour after the tone curve and the LUT, each clamping it to 0..1 (white balance in linear light, the rest on the sRGB-encoded colour), baked with the LUT into one 32³ table.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `temperature` | `number` | `0` | -1 … 1 | — | tunable | White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off. See `render §5`. |
| `tint` | `number` | `0` | -1 … 1 | — | tunable | White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off. |
| `brightness` | `number` | `0` | -1 … 1 | — | tunable | Added to every channel of the display colour; 0 is off. |
| `contrast` | `number` | `0` | -1 … 1 | — | tunable | Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off. |
| `balanceShadows` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | tunable | The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off. |
| `balanceMidtones` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | tunable | The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows. |
| `balanceHighlights` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | tunable | The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows. |
| `preserveLuminosity` | `boolean` | `true` | — | — | — | The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity). |
| `hue` | `number` | `0` | -180 … 180 | `deg` | tunable | Turns every hue about the grey axis (red towards green at 120); 0 is off. |
| `saturation` | `number` | `0` | -1 … 2 | — | tunable | -1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation). |
| `vibrance` | `number` | `0` | -1 … 1 | — | tunable | Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off. |
| `grayscale` | `number` | `0` | 0 … 1 | — | tunable | How far each colour goes towards its luminance (grey); 0 is off. |
| `invert` | `number` | `0` | 0 … 1 | — | tunable | How far each colour goes towards its complement; 0 is off. |
| `threshold` | `number` | `0` | 0 … 1 | — | tunable | Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off. |

#### Vignette

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vignette` | `boolean` | `false` | — | — | — | Darkened (or tinted) edges of the frame. |
| `vignetteIntensity` | `number` | `0.35` | 0 … 1 | — | tunable | 0 none, 1 black corners. |
| `vignetteRadius` | `number` | `0.55` | 0 … 1 | — | — | Where darkening starts, as a fraction of the half-diagonal. |
| `vignetteSmoothness` | `number` | `0.45` | 0 … 1 | — | — | The width of the fade. |
| `vignetteColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The linear colour the edges move towards. |

#### Lens

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chromaticAberration` | `number` | `0` | 0 … 20 | — | tunable | Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. |
| `lensDistortion` | `number` | `0` | -1 … 1 | — | tunable | Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame. See `render §5`. |
| `lensDistortionCenter` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | tunable, worldspace `none` | The lens's centre, in fractions of the frame from the top left. |

#### Grain and sharpening

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `filmGrain` | `number` | `0` | 0 … 1 | — | tunable | Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. |
| `filmGrainSize` | `number` | `1.5` | ≥ 0.5 | — | — | The grain size in pixels. |
| `sharpen` | `number` | `-1` | -1 … 1 | — | tunable | Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off. |

#### Flash

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear display colour. |

#### Camera shake

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shakeTrauma` | `number` | `0` | 0 … 1 | — | tunable | Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only. |
| `shakeMaxOffset` | `number` | `0.05` | ≥ 0 | — | — | The largest offset at trauma 1, as a fraction of the view height. |
| `shakeMaxRoll` | `number` | `3` | ≥ 0 | `deg` | — | The largest roll at trauma 1. |
| `shakeFrequency` | `number` | `12` | ≥ 0 | `hz` | — | How fast the shake moves. |
| `shakeSeed` | `number` | `0` | — | — | — | The shake's noise seed. |

#### Screen-space reflections

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ssr` | `boolean` | `false` | — | — | — | Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass. See `render §5`. |
| `ssrMaxDistance` | `number` | `100` | ≥ 0 | `m` | tunable | How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance. |
| `ssrThickness` | `number` | `0.3` | ≥ 0 | `m` | tunable | How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. |
| `ssrSteps` | `number` | `64` | 4 … 128 | — | — | The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them. See `render §5`. |
| `ssrFade` | `number` | `0.15` | 0 … 1 | `ratio` | — | The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too). |
| `ssrIntensity` | `number` | `1` | 0 … 1 | — | tunable | How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are). |
| `ssrMaxRoughness` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them. See `render §5`. |
| `ssrResolution` | `AoResolution` | `"half"` | — | — | — | Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution. See `render §5`. |

#### Custom passes

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `custom` | `readonly Post[]` | `[]` | — | — | a list of at most 8 `Post` (`List.Post.8`): reads a frozen copy, writes a whole array | Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors). See `render §5`. |

```ogt
resource PostEffects {
  doc "The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then records no pass. Read after each tick; it never feeds back into the simulation."
  doc "AO, outlines, depth of field, motion blur and god rays turn on the depth-normal prepass. Grading, the colour adjustments, vignette, aberration, grain and the flash apply to the display colour after tone mapping, lens distortion bends the result, and the overlay comes after all of it."
  guide "render §5"
  group "Ambient occlusion"
  field ao bool "Screen-space ambient occlusion: darkens the ambient light in creases and contact areas."
  field aoRadius f32 "The world-space radius searched for occluders." {
    default 0.6
    min 0
    unit m
    tunable true
  }
  field aoIntensity f32 "0 none, 1 full; above 1 exaggerates." {
    default 1
    min 0
    tunable true
  }
  field aoMethod AoMethod "The algorithm."
  field aoResolution AoResolution "Full resolution, or half for a fraction of the GPU time at softer detail."
  field aoDirect f32 "How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it." {
    default 0
    min 0
    max 1
    tunable true
  }
  field aoRadiusScale f32 "Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere." {
    guide "render §5"
    min 0
    tunable true
  }
  group "Outlines"
  field outline bool "Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead)."
  field outlineColor Color "The line colour, linear." {
    default 0.02 0.02 0.03
  }
  field outlineWidth f32 "Line width in pixels, drawn inside the silhouette." {
    default 2
    min 0
    max 31
    tunable true
  }
  field outlineDepthThreshold f32 "The relative depth step that counts as a silhouette (0.04 is 4 %)." {
    default 0.04
    min 0
  }
  field outlineNormalThreshold f32 "The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees)." {
    default 0.4
    min 0
  }
  group "Depth of field"
  field depthOfField bool "Bokeh depth of field: pixels away from the focus distance blur into discs."
  field dofFocusDistance f32 "The distance from the camera that is sharp." {
    default 10
    min 0
    unit m
    tunable true
  }
  field dofFocusRange f32 "The half-width of the sharp band around it." {
    default 2
    min 0
    unit m
    tunable true
  }
  field dofMaxBlur f32 "The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p)." {
    default 0.012
    min 0
    max 0.1
  }
  field dofFalloff f32 "How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away." {
    default 4
    min 0
  }
  field dofQuality DofQuality "Taps per pixel."
  group "Motion blur"
  field motionBlur bool "Per-object and camera motion blur along the screen-space velocity."
  field motionBlurShutter f32 "The shutter as a fraction of the frame (0.5 is a 180 degree shutter)." {
    default 0.5
    min 0
    max 2
    tunable true
  }
  field motionBlurSamples u32 "Taps along the velocity." {
    default 12
    min 4
    max 32
  }
  field motionBlurMaxPixels f32 "The longest blur in pixels." {
    default 40
    min 1
  }
  group "Screen blur"
  field screenBlur ScreenBlur "A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0."
  field screenBlurStrength f32 "How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off." {
    min 0
    tunable true
  }
  field screenBlurCenter Vector2 "Zoom's and spin's centre, in fractions of the frame from the top left." {
    worldspace none
    default 0.5 0.5
    tunable true
  }
  field screenBlurAngle f32 "Directional's direction, counter-clockwise from the right (90 is vertical)." {
    unit deg
    tunable true
  }
  field screenBlurSamples u32 "Taps per pixel: more is smoother on long blurs, each one a texture read." {
    default 16
    min 4
    max 64
  }
  group "God rays"
  field godRays bool "Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera)."
  field godRaysIntensity f32 "How bright the shafts are." {
    default 0.5
    min 0
    tunable true
  }
  field godRaysDecay f32 "How fast the rays fade along their length." {
    default 0.965
    min 0.5
    max 1
  }
  field godRaysDensity f32 "How far towards the sun each ray samples (a fraction of the distance)." {
    default 1
    min 0
  }
  field godRaysTint Color "A linear tint on the light the rays carry." {
    default 1 1 1
  }
  field godRaysSamples u32 "Taps per ray." {
    default 48
    min 8
    max 128
  }
  field godRaysThreshold f32 "Sky brightness (linear, before exposure) below which the sky carries no rays." {
    default 2
    min 0
  }
  group "2D glow"
  field glow bool "Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom."
  field glowIntensity f32 "How much of the blurred glow is added back." {
    default 1
    min 0
    tunable true
  }
  group "Grading"
  field lut Lut "Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident)."
  field lutIntensity f32 "0 leaves the image as tone mapped, 1 applies the table fully." {
    default 1
    min 0
    max 1
    tunable true
  }
  group "Colour adjustments" "Applied in field order to the display colour after the tone curve and the LUT, each clamping it to 0..1 (white balance in linear light, the rest on the sRGB-encoded colour), baked with the LUT into one 32³ table."
  field temperature f32 "White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off." {
    guide "render §5"
    min -1
    max 1
    tunable true
  }
  field tint f32 "White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off." {
    min -1
    max 1
    tunable true
  }
  field brightness f32 "Added to every channel of the display colour; 0 is off." {
    min -1
    max 1
    tunable true
  }
  field contrast f32 "Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off." {
    min -1
    max 1
    tunable true
  }
  field balanceShadows Vector3 "The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off." {
    tunable true
  }
  field balanceMidtones Vector3 "The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows." {
    tunable true
  }
  field balanceHighlights Vector3 "The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows." {
    tunable true
  }
  field preserveLuminosity bool "The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity)." {
    default true
  }
  field hue f32 "Turns every hue about the grey axis (red towards green at 120); 0 is off." {
    min -180
    max 180
    unit deg
    tunable true
  }
  field saturation f32 "-1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation)." {
    min -1
    max 2
    tunable true
  }
  field vibrance f32 "Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off." {
    min -1
    max 1
    tunable true
  }
  field grayscale f32 "How far each colour goes towards its luminance (grey); 0 is off." {
    min 0
    max 1
    tunable true
  }
  field invert f32 "How far each colour goes towards its complement; 0 is off." {
    min 0
    max 1
    tunable true
  }
  field threshold f32 "Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off." {
    min 0
    max 1
    tunable true
  }
  group "Vignette"
  field vignette bool "Darkened (or tinted) edges of the frame."
  field vignetteIntensity f32 "0 none, 1 black corners." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field vignetteRadius f32 "Where darkening starts, as a fraction of the half-diagonal." {
    default 0.55
    min 0
    max 1
  }
  field vignetteSmoothness f32 "The width of the fade." {
    default 0.45
    min 0
    max 1
  }
  field vignetteColor Color "The linear colour the edges move towards."
  group "Lens"
  field chromaticAberration f32 "Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off." {
    min 0
    max 20
    tunable true
  }
  field lensDistortion f32 "Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame." {
    guide "render §5"
    min -1
    max 1
    tunable true
  }
  field lensDistortionCenter Vector2 "The lens's centre, in fractions of the frame from the top left." {
    worldspace none
    default 0.5 0.5
    tunable true
  }
  group "Grain and sharpening"
  field filmGrain f32 "Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off." {
    min 0
    max 1
    tunable true
  }
  field filmGrainSize f32 "The grain size in pixels." {
    default 1.5
    min 0.5
  }
  field sharpen f32 "Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off." {
    default -1
    min -1
    max 1
    tunable true
  }
  group "Flash"
  field flash f32 "A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear display colour." {
    default 1 1 1
  }
  group "Camera shake"
  field shakeTrauma f32 "Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only." {
    min 0
    max 1
    tunable true
  }
  field shakeMaxOffset f32 "The largest offset at trauma 1, as a fraction of the view height." {
    default 0.05
    min 0
  }
  field shakeMaxRoll f32 "The largest roll at trauma 1." {
    default 3
    min 0
    unit deg
  }
  field shakeFrequency f32 "How fast the shake moves." {
    default 12
    min 0
    unit hz
  }
  field shakeSeed u32 "The shake's noise seed."
  group "Screen-space reflections"
  field ssr bool "Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass." {
    guide "render §5"
  }
  field ssrMaxDistance f32 "How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance." {
    default 100
    min 0
    unit m
    tunable true
  }
  field ssrThickness f32 "How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it." {
    default 0.3
    min 0
    unit m
    tunable true
  }
  field ssrSteps u32 "The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them." {
    guide "render §5"
    default 64
    min 4
    max 128
  }
  field ssrFade f32 "The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too)." {
    default 0.15
    min 0
    max 1
    unit ratio
  }
  field ssrIntensity f32 "How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field ssrMaxRoughness f32 "The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them." {
    guide "render §5"
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
  field ssrResolution AoResolution "Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution." {
    guide "render §5"
    default half
  }
  group "Custom passes"
  field custom List.Post.8 "Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors)." {
    guide "render §5"
  }
}
```

<sub>Module `render` line 1017</sub>
