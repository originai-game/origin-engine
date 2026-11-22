# Render and lighting choices

The engine's render options and what each costs: tiers and quality presets, antialiasing and transparency, lights and shadows, global illumination and reflection, post effects, materials, atmosphere and water, interpolation, and secondary cameras. looks §3 covers how a scene is lit and looks §5 how to measure a frame; this guide says which engine option to turn on, when, and what it costs.

The API: `og api show render`, `og api show atmosphere`. Every option here
is off or at its cheapest default until a game turns it on, and none of it
feeds back into the simulation.

## 1. Tiers and quality presets

- RenderCapabilities.backend names the backend (`RenderBackend`, the
  tier in the probe's hello and the reference frames' directory). On
  `vulkan-cpu` (a CPU rasterizer in a sandbox) frame times say nothing
  about a player's GPU: measure cost on a real device only.
- RenderCapabilities.tier is the feature set the host made the renderer
  for (`full`, `miniGame`, `gl`), and RenderCapabilities.caps what the
  device can do under it. A pass whose needs the set lacks draws its
  fallback; no pass is chosen by backend.
- `og.render.applyQuality` writes one row of RenderSettings' quality
  fields (its doc has the table). Medium is every default. Apply the
  player's saved preset at start; the renderer reads the fields, never
  `RenderSettings.quality`, so a field set after the preset is drawn as
  set.
- Low turns off the costly effects. A game that turns one back on under
  low gets it at half resolution or half its steps.
- The default upscaler, `auto`, draws FSR 3.1 while the render scale is
  below 1 (automatic quality's too) and TAAU at 1: below 1 FSR keeps the
  thin geometry and moving detail TAAU's one sample a pixel loses, at
  about 1 to 2 ms more at 1080p and 1440p on an M4 Pro.
- High is paid only by the effects the scene draws: FSR 3.1 at every
  scale, doubled shadow maps, full-resolution AO, doubled SSR and
  volumetric steps. A scene without those effects differs from medium
  only by its shadow maps and its resolve at render scale 1.
- Automatic quality lowers the render scale first, so `auto` switches to
  FSR then; a preset step to medium keeps `auto`, one to low draws
  `taau`.

## 2. Antialiasing, upscaling and transparency

| Choice | When | Cost and catch |
|---|---|---|
| `tier` | default | MSAA 4x in 2D (TAA's jitter would soften nearest-sampled pixel art), TAA in 3D |
| `msaa4` | 2D, sprites and text | sharp, no blur; FXAA where the HDR format cannot be multisampled |
| `taa` | 3D | converges over about 8 frames; with render scale below 1 it upscales |
| `fxaa` | cheapest edge smoothing | one pass, softer on fine texture |
| `smaa` | TAA's blur unwanted | no history; not combined with TAAU |
| `smaaT2x` | temporal without TAA | resolves in linear display colour before stylisation and the overlay; the first frame and every camera cut use the current frame alone |

- `msaaSamples` 1 draws FXAA. Where the device resolved no MSAA (TAA,
  FXAA, off, or FXAA for want of a multisampled HDR format) it reads 1;
  RenderCapabilities.antialias is what is drawn now.
- Upscalers under TAA: `taau` is the cheapest, softer on thin geometry and
  ghosts a little more on disocclusion. `fsr3` costs more GPU time
  (docs/design.md §11 has the measurements); its reactive mask, fed by
  the transparency coverage, keeps particles and glass from smearing.
- Alpha-tested foliage under MSAA: `alphaToCoverage` lets minified cards
  cover part of a pixel instead of drawing as solid sheets.
- Transparency on the main camera is order independent, never sorted.
  `weighted` (the default) keeps the nearest layer exact, marked by a
  depth pass of its own, so a billboard or pane in front keeps its
  colour; only layers behind it blend as a mean. Two submission orders
  differ by at most one 8-bit step per channel for a few layers.
  `linkedList` is exact up to `transparencyLayers` fragments; it falls
  back to weighted on devices without fragment storage writes or when the
  scene is too large for one node per pixel in the largest storage
  buffer. Secondary cameras, reflections and probe faces still sort
  transparent surfaces back to front.

## 3. Lights and shadows

- Lights are clustered (Forward+): hundreds of point, spot and rect lights
  are cheap, and lights out of view are skipped. Shadows are what cost:
  `RenderSettings.shadowedLights` (8 by default, capped by
  RenderCapabilities.maxShadowedLights, 16) picks the brightest at the
  camera. One shadowed sun plus a few shadowed locals where the player
  looks (looks §3).
- The sun: 3 cascades by default, each a 2048² map. Three 2048 maps are
  48 MB, 64 MB with their depth buffer: lower `RenderOptions.shadowMapSize`
  or `shadowMapScale` on phones. `cascadeBlend` 0.1 to 0.2 hides seams at
  the cost of two cascade reads on those pixels. `shadowBox` draws the
  casters once instead of once per cascade, as three.js games that fit the
  shadow camera around the player do. `shadowSoftness` (PCSS) costs a
  blocker search and a 16-tap filter per shadowed pixel.
- Several DirectionalLights: one is drawn, the brightest by intensity
  times its colour's luminance (of equals, the one spawned first), and its
  cascades, the sky's sun, the clouds and the weather follow it. Two within
  1.5x of each other hand over: the drawn light turns from one to the other
  and dims through the crossing when they point apart, so a sun rising as
  the moon fades lights the scene without a jump. Fade one out as the
  other comes up; a fill or rim light is a point, spot or rect light. A
  development build warns, naming them, when another shines with a quarter
  of the drawn one's light or more. With og.DayNight running, its sun and
  moon take the place of the light that would be drawn without it (with
  that light's shadow settings) and hand over to each other by the same
  rule; `get:<light>.og.DirectionalLight` answers `keyShare`, the light's
  share of the one drawn. `og bake lightmaps` bounces the same one light.
- Point lights: cube shadows take six passes per light, paraboloid two
  (coarser; large casters' edges stay straight).
- Rect lights: the soft shadow is the costliest light in a frame (two maps
  when two-sided); the low preset turns rect shadows off. The shadow is
  cast from the rectangle's centre, so a caster close to a large panel is
  shadowed as from its centre. The highlight on a glossy floor keeps the
  rectangle's shape (LTC with three.js's tables). Aim one with
  `transform.lookAt`.
- Light mobility (`mobility`, Unreal's): `stationary` (the default) stays
  where it is; its colour and intensity may change. `movable` moves.
  `static` never moves or changes. `og bake lightmaps` bakes the bounce of
  stationary and static lights, and a static light's direct light too;
  while its scene's bake is drawn a static light is not drawn at all (no
  highlight, no shadow of a moving body), so keep `static` for fill lights
  of a baked level. A movable light is never baked. A DirectionalLight
  under a running og.DayNight is movable whatever it says.
- The far field: `RenderSettings.staticLightReach` is 0 by default, and
  then every light draws whole. Above 0, a stationary light's diffuse
  light past the reach comes from a 32 × 12 × 32 grid of probes round the
  camera, half the reach apart (0.6 MB of GPU memory; recomputed on the
  CPU when the camera moves a cell or such a light changes), and such a
  rect light draws nothing on a rough surface (roughness 0.5 or more)
  there. Specular is kept: glossy surfaces keep every light's highlight
  over its whole distance, so a wet street still mirrors the signs. Use it
  in light-heavy scenes, where many unmoving lights (large rect lights
  especially) reach many rough walls: it saves about 5% of the frame
  there, and trades the far diffuse detail of those lights (probe-sized,
  soft in direction) for it. A scene with a few lights gains nothing.
  Movable lights and lights that cast a shadow or have a cookie are never
  cached. 4 to 8 m suits a street.
- Cookies: a spot projects one texture (a window's bars, stained glass); a
  point light takes a six-face strip, so a lamp behind a pierced shade
  throws its pattern on the walls.
- Mesh flags: `castShadow` is off by default (a lamp's glass, a glow card);
  `receiveShadow` is on (every mesh received shadows before the flag
  existed), and false keeps a glowing sign or a hero readable in a dark
  alley. Shadow maps ignore `layers`, so a body hidden from a first-person
  camera still casts.

## 4. Global illumination and reflection

### Environment images

An environment is both a camera-relative spherical background and image-based
lighting, not a flat camera plate. Cook a `type environment` file and use
`og.load.environment` with `og.Environment`. Rotation, intensity, blurriness,
grounded projection (`groundHeight`, `groundRadius`) and blending are the same
for HDR and LDR sources. `loading §4` covers loading and aiming a DirectionalLight.

```text
type environment
version 1
---
source "ai-sky.png"
size 256
samples 256
expand 0.25
sunBoost 1
```

- `source` accepts a full equirectangular `.hdr`/`.exr` (linear), sRGB
  `.png`/`.jpg`/`.jpeg`/`.webp`, or a KTX2 cube. `faces` accepts six square
  HDR or LDR images in +X -X +Y -Y +Z -Z order. Keep all six in one range.
  LDR uses the texture decoder, then the sRGB transfer function into linear
  float RGB before the usual GGX prefilter and diffuse harmonics.
- A full panorama must be within 5% of 2:1. The cook resamples in linear
  light to exact 2:1, without cropping; a distant aspect ratio is refused
  with its dimensions. 1774 × 887 is already exactly 2:1 and needs no repair.
- `hemisphere upper` instead means the image runs from zenith at its top to
  horizon at its bottom, across the full 360° width. Its aspect ratio need
  not be 2:1. The lower half is filled with the mean linear colour of the
  last four resampled horizon rows; `ground 0.12 0.10 0.08` overrides it
  with linear RGB in 0..1. Each column starts at its last sky colour and
  smoothly eases to that ground colour over six degrees below the horizon,
  avoiding a hard step. This supplies a ground to the existing dome; it
  cannot turn a full-sphere image's mirrored lower half into terrain.
- A lone LDR image has an authored background: the image's own colours,
  without exposure or the tone curve, like a camera background texture,
  but spherical and camera-relative. `backgroundIntensity` still scales
  it; `intensity` scales only lighting. Its cube face size is the power of
  two nearest source width / 4 (ties round up), capped at 2048, independent
  of lighting `size`. Ordinary box mips handle minification and blur, not
  GGX filtering or the expanded light. HDR backgrounds use normal HDR exposure.
- `expand` is the bounded inverse-Reinhard amount for LDR lighting only,
  default 0.25, range 0..0.9 (0 disables). It preserves RGB ratios and
  mildly opens highlights rather than trying to recover the sun with a
  steep curve. It has no effect on HDR.
- LDR defaults to `sun extract`; `sun keep` opts out. The cook finds a
  compact bright disc above the horizon, measures its solid-angle-equivalent
  diameter, and fits the unclipped upper sky's luminance gradient to the
  Perez/Preetham daylight model (turbidity and exposure), following the
  analytic sky-fitting approach of Lalonde and Hold-Geoffroy. Atmospheric
  transmittance gives relative sun intensity and colour. `sunBoost` scales
  that estimate (default 1, range 0..100). These are estimated facts, not
  measured lux: clipped colours and arbitrary generated skies are ambiguous.
  Overcast images may yield no compact disc; insufficient sky samples use
  the clipping lower bound rather than a fabricated fit. The original disc
  remains visible in the authored background, but is replaced with its
  surrounding sky before the lighting prefilter. `og.load.environmentSun`
  takes the map, Environment.rotation and Environment.intensity and reports
  world direction, colour, DirectionalLight intensity and `angularSize` (radians). Give
  those to a DirectionalLight and aim its -Z away from that direction for
  actual shadows; the engine never adds a second light for you.

When the cook finds no sun, its message and `environment.sun.missing` in
the report name the cause: none bright enough (no pixel above the horizon
with every channel near white), a clipped area (white clouds or haze
touching the sun joined its disc), or a disc too large (over 0.03 sr,
about 11° across). Draw one small, unclipped white disc with the clouds
around it below white.

#### Turning a sky by its sun

`og.load.environmentSunRotation(world, map, view, azimuth)` answers the
Environment.rotation that stands the map's sun at `azimuth` around a view
direction: 0 straight ahead (the subject backlit), π behind the viewer,
−π/2 to its left, −3π/4 behind its left shoulder ("light from the upper
left" over a camera looking at the subject). Set the same rotation on
og.Environment and pass it to og.load.environmentSun for the light. A turn
about +Y never changes the sun's height: `elevation` reports the map's,
and a different height needs a different sky.

```ts
const forward = og.transform.cameraForward(camera.quaternion); // the direction the view looks
const turn = og.load.environmentSunRotation(world, map, forward, -0.75 * Math.PI);
if (!turn.ok) throw new Error(turn.reason);
env.rotation = turn.value.rotation;
const sun = og.load.environmentSun(world, map, turn.value.rotation, env.intensity);
```

#### Matching an authored sky's exposure

An authored LDR background is drawn as its source, without exposure or
the tone curve, and the auto-exposure meter leaves it out. So:

- `autoExposure` (on in most looks) meters the lit models alone to middle
  grey, whatever the sky: a dark whale against a pale sky comes out as
  bright as a pale one, and neither a darker tint nor a lower
  Environment.intensity moves it. Use it when the camera moves between
  places whose light differs and no source frame is to be matched.
- To match a source image, turn it off and keep `exposure 1`. With
  `toneMapping none` and Environment.intensity 1, a diffuse surface of
  albedo a in the sky's light is drawn at a × the sky's mean linear
  radiance, as a photograph of it would be: the source's brightness.
  Lighten or darken the models by their albedo, not by the intensity.
  `neutral` adds a highlight shoulder and takes 0.04 off every channel
  (a mid-grey wall about 10 % darker). ACES lifts these mid-tones about
  1.5 times and AgX (the cinematic-realism look's curve) reshapes them
  too, so under either the intensity that matches is no longer 1: measure
  it against the source frame.
- Add the sun the cook took out with og.load.environmentSun at the same
  intensity; the default `expand 0.25` opens the sky's highlights a little.

```text
type look
version 1
---
RenderSettings {
  autoExposure false
  exposure 1
  toneMapping none
}
```

`og cook --report` stores each environment's source facts under
`environment.facts`: wrap edge-column mean/max absolute channel difference
in linear RGB, clipped pixel fraction (any channel at white), lower-half
mirror error and blue fraction (observable sky/ground cues, not a semantic
classification). With `hemisphere upper`, `lowerMirrorMean` and
`lowerBlueFraction` are null: the source has no lower hemisphere to measure,
as `lowerHemisphere` explains. Wrap and clipping still measure the source.
`environment.sun` holds the extracted sun and whether it
is estimated. Facts persist in the blob, so an incremental report retains
them. Six faces report cues on their spherical projection; KTX2 chains have
no source-image cues. No threshold scores the image and no seam is repaired:
fix a visible wrap in the source, for example with a masked image edit.

The report's sun intensity is in DirectionalLight units including the
cook's `intensity`. The runtime answer additionally multiplies it by the
Environment.intensity argument. Both use the sky's linear-radiance scale;
LDR sunlight is estimated, not a calibrated illuminance measurement.

| Need | Use | Cost |
|---|---|---|
| static level, best quality | `og bake lightmaps` + LightVolume | CPU bake; free at run time |
| moving lights in a room | RealtimeLightVolume | a few probe captures a frame |
| contact and fine bounce | GlobalIllumination `screen` | rays per pixel over the prepass |
| bounce from off screen, sky occlusion | GlobalIllumination `probes` | probe cascades, coarse |
| both | `both` | the sum |

- Baked: static models use lightmaps; everything else inside a LightVolume
  (meshes, models, skinned characters, lit particles) takes the probes, so
  a ball rolling past a red wall turns red on that side. Its box is the
  og.Volume on its entity, as the world's axes bound it: a turned box
  takes the box around it, a radius the cube around the sphere, and the
  entity's scale stretches it. Without an og.Volume it has no box and
  `og bake lightmaps` names the entity. A volume added, moved or resized
  is unbaked until the next bake. `bakedLighting false`
  compares the bake against the realtime lights alone.
- RealtimeLightVolume: each capture draws every light, the sky and
  lights' shadows, emissive surfaces and lightmapped models, and sees the
  probes, so a lamp in a red room fills it with red light after a few
  sweeps. Its probes are used once all are relit (a few frames after it is
  added or resized), and the low preset falls back to the bake. Up to
  eight volumes draw: baked first, then realtime ones nearest the camera.
  Keep the box's faces a little inside the walls: a probe on a wall sees
  into it (so does the far side of a wall thinner than a spacing). Its
  light is whole up to the faces and reaches og.Volume.fade beyond them;
  give it a spacing of fade so the walls, floor and ceiling take it. Its reflections are a soft image
  of the room, too soft for a mirror: put a ReflectionProbe there. It gives
  no contact shadow: pair it with PostEffects.ao or GlobalIllumination.
- Screen GI: rays over each pixel's hemisphere take the light the previous
  frame showed where they hit, so a few frames give many bounces (a red
  wall tints the floor, a sunlit patch lights the ceiling), but nothing
  off screen beyond a floor or wall running a little on past the frame's
  edge. Each ray gathers every surface along it, so the default two give
  the whole bounce; more rays and steps only smooth it. Under TAA the
  rays change each frame and average; without it a still frame keeps one
  blurred pattern.
- Probe GI: cascades traced in software around the camera light every lit
  surface (meshes, terrain, grass, scattered copies); light from behind
  walls bounces and the sky is occluded under roofs and between
  buildings. Coarse: pair it with PostEffects.ao, whose GTAO bent normals
  pick the probes' light from the direction a crease opens to.
- Reflections, best first: SSR, then reflection probes and probe GI, then
  the environment or sky. A grounded environment (`groundHeight`) shows
  the same ground in mirrors, water and clear coats.
- A grounded environment's ground takes the shadows of what stands on
  it (`groundShadows`, 1 by default): the map's light the props hide from
  it, within 32 m of `groundCenter`, and the sun's shadow from a
  DirectionalLight that casts shadows, as a real floor would (three.js's
  ShadowMaterial, Unity's HDRI backplate and Unreal's HDRI Backdrop do
  the same). For a hard sun shadow on an
  outdoor photograph, cook the map with `sun extract` and let the
  DirectionalLight cast the sun the cook took out. Its limits: mirrors
  show the ground without these shadows, point and spot lights cast none
  on it, and the map's light is occluded as by a height field, so what
  overhangs the ground (a sphere's lower half, a table top) occludes it
  as if solid down to the ground, which widens the dark contact a little.

## 5. Post effects

Every PostEffects switch off costs nothing. AO, outlines, depth of field,
motion blur, god rays, lens flare in 3D, SSR and custom passes share one
depth-normal prepass: the first one pays for it.

- Bloom: Origin's by default; `bloomRadius` 0 to 1 matches three.js
  UnrealBloomPass exactly (smoothstep high pass keeping the colour whole
  above it, Gaussian kernels 3, 5, 7, 9, 11, weights for f = 1, 0.8, 0.6,
  0.4, 0.2, and EffectComposer's additive copy weighted again by its
  alpha), for ports that must match.
- Exposure: auto exposure adapts like an eye by ticks, never the wall
  clock, so replays and hosts adapt alike whatever their frame rates. A
  frame drawn before the first tick (the web shell's frame behind the play
  gate) meters at once and is never adapted from. 3 EV/s down and 1 up is
  a natural pair. `autoExposureMin` and `autoExposureMax` bound the
  exposure it applies (default -10 to 10 EV, which lets a moonlit night
  meter up to a grey day). For a day-night cycle, set the maximum to about
  the exposure a dusk view meters to (read it with the probe, then add a
  stop or two): the night then draws darker than the day by what the light
  lost past that point, with no per-tick Camera.exposure from the game.
- AO: `gtao` looks better, `ssao` is cheaper, `screen` imitates three.js-era
  post SSAO (ground seen at a low angle darkens a little, as theirs did).
  Half resolution is a fraction of the GPU time at softer contacts.
  `aoRadiusScale` pays off with ssao, whose open ground stays unshaded;
  gtao darkens open ground a little as the radius shrinks.
  RenderSettings.aoResolution is read before PostEffects', so the probe's
  RenderSettings and PostEffects together say what is drawn.
- Colour adjustments (temperature to threshold) cost no pass: they bake
  with the LUT into one 32³ table on the CPU when one changes, about half
  a millisecond with one adjustment on and a few with all of them. Hold
  them still; do not animate many at once.
- Lens distortion is one pass; a barrel never shows black edges wherever
  its centre is, and paint, hatching, halftone and ink lines bend with the
  image.
- SSR: neighbours' rays are reused and accumulated, so mirrors stay sharp
  and rough metal blurs; a frame after a cut has no history yet. Reach is
  cheap (cost follows `ssrSteps`, not distance: the default lets a wet
  street mirror lights 40 m away); 64 steps let rays pass behind lamp
  posts down a street. Rough surfaces cost the same per ray and
  accumulate over more frames. Half resolution is under half of full's
  cost (about 1.6 ms at 1080p on an M4 Pro) and keeps edges sharp, slightly
  softer near mirrors; full is the sharpest.
- Lens flare: Filament's screen-space ghosts and halo from bloom's first
  level, plus aperture-shaped ghosts of the sun (8 % of the frame height)
  and of the seven brightest point and spot lights (three times their
  on-screen size up to that, magnified up to 1.5 times), at 0.56 % (sun)
  or 0.28 % (light) of the source's exposed radiance, so a dim or small
  lamp flares faintly. A source behind a roof or pillar fades by the depth;
  a ghost on its own glare is dropped, and each source flares once. With
  bloom on its thresholds apply above bloomThreshold; with bloom off the
  first bloom level runs for the flare alone. Clouds and fog dim the sun's
  flare, a single cloud crossing the disc does not. Each ghost takes an
  amber, teal or violet coating tint in turn.
- Custom passes: write the whole list (`post.custom = [grade, crt]`,
  `post.custom = []`; a look writes `custom "post/tint.ogt" "post/crt.ogt"`),
  set parameters with `og.render.setPostParam`; each also reads the lit
  image. The cap is og-render's MAX_CUSTOM_POSTS.
- Display output (RenderOptions.output, left out `best`): on a surface
  that offers them the frame goes out in extended range (HDR) and Display
  P3; `capabilities.hdr` and `capabilities.wideGamut` say what resolved.
  On HDR the tone curve keeps middle grey and the SDR image's mid-tones,
  and ends at the headroom (peakNits / paperWhiteNits, 1000 / 203 left
  out, no more than the display reports) instead of SDR white: emissive,
  glow, bloom and sun keep rising past white. Author them as for SDR; a
  value meant to read as white is still 1. Captures, `og test` frames and
  references are offscreen, sRGB at standard range, whatever the output.
  Choose `wide` where a scene's brightness must match SDR players', and
  `srgb` to show exactly what a capture shows.

## 6. Materials

- One StandardMaterialDesc covers PBR, stylised and physical shading; share
  materials, since instancing is per (geometry, material) (looks §5).
- Under og.DayNight and og.PhysicalSky the daylight is physical: at 0 EV
  the noon sun lights a grey card (0.18) to middle grey, so a base colour
  (and a terrain layer's `color`) is the surface's albedo, linear. Grass is
  about (0.1, 0.15, 0.045), soil 0.1 to 0.25, plaster and pale stone 0.5
  to 0.75, fresh snow 0.8 to 0.9. A colour picked by eye on a screen is
  often two or three times that: a meadow of (0.3, 0.42, 0.2) reads
  washed out, pale and grey-green, under a summer noon.
- Give metalnessMap and roughnessMap the same glTF texture. The
  metalness transform also places a physical material's other packed maps
  and aoMap.
- Opaque with `alphaTest` beats `transparent` for foliage and fences.
- Stylised fields are filled by a look's MaterialStyle (including its
  outline distance) unless `styled` is false: a lantern's lacquer, a
  UI-like prop, a hero kept photographic in a toon scene. Physical lobes
  stay physical under the look's toon, which would drop them.
- Hull outlines line silhouettes and contact creases, never a model's own
  faces, so `.vox` and flat-shaded meshes stay clean; a box side nearly
  edge-on or a large ground plane is lined only where it ends. Creases
  inside an object are Stylise.inkDepth's.
- Physical layers: transmission (glass, water, thin plastic) keeps
  reflections, unlike opacity. Sheen never glows: velvet is a very dark
  base, a bright desaturated sheenColor and sheenRoughness about 0.5;
  cotton and wool a dim sheenColor at 0.8 to 1; roughness 1 on both.
- Subsurface: `subsurface: 1` is skin, wax, marble, jade or milk, by
  `subsurfaceRadius`, how far each channel travels in metres. The shading
  softens past the terminator where the surface curves: a face barely
  (a head's radius is 30 times red's 3.7 mm), a nose, an ear or a finger
  clearly, red first. The default `subsurfaceRadius` is skin's, for a
  body in metres; a small or stylised character wants it a few times
  larger, and wax or jade one distance in every channel (0.005 to 0.02). Flat faces and flat-shaded
  meshes do not curve and stay as they are. With `thickness` (and a
  `thicknessMap`, thin where the ear is) the sun and the lamps behind
  come through the subsurface material, red most: 1 cm of skin passes a
  third of red and under a hundredth of blue. Rect lights and the sky's light are not spread.
- Hair: `hair: 1` shades the surface as strands, in place of the glossy
  and the diffuse lobe: a white highlight in a band across them, a second
  in the hair's `color` nearer the tips, and `color` in the light they
  pass between them. The strands run along v with their roots at the top
  of the image, the way hair cards and hair textures are laid out;
  `anisotropyRotation` turns them (π/2: along u) and an `anisotropyMap`
  combs them. `roughness` is how wide the highlights are along the
  strand: 0.3 to 0.5, lower for wet or straight hair. With a `thickness`
  the light behind hair comes through it, once through every strand on
  its way: a card or a lock drawn from both sides (`side: "double"`) is a
  layer `thickness` deep, darker and redder the deeper (1 cm is a strand
  or two); a closed head of hair glows at its rim only, where the light's
  way stays within `thickness` of the surface. Rect lights and the sky's
  light shade hair as a plain surface of its `color`.
- Eyes: a sphere per eye, its texture painted flat with the iris round
  the centre of the uv square. `irisRadius`, the iris's radius in uv, puts
  what is painted within it under the cornea: it is seen refracted by
  `ior` (1.34 for an eye), `irisDepth` radii down at its middle and level
  with the white at its rim, so the iris has depth, shifts against the
  white as the eye turns and stays in view from the side. The cornea's
  wet reflection is the clear coat (`clearcoat: 1`,
  `clearcoatRoughness: 0.03`, `roughness` 0.4 or more under it), and the
  white of the eye takes `subsurface`.
- Texture arrays put variants of one look (tiles, skins, damage states) in
  one material and one draw.
- Wireframe is drawn as lines, not a polygon mode (WebGPU has none).

## 7. Fog, sky, clouds and weather

- Fog: plain distance and height fog is nearly free. Volumetric fog (a
  froxel grid, RenderCapabilities.volumetricFog names the method) makes
  shafts through windows and trees that move with the sun; opaque surfaces
  and the sky are seen through it. Its cost and memory grow with its
  slices (`volumetricStepScale`); thinner slices resolve a shaft's edge in
  depth and fog just in front of surfaces. Froxel cells take the mean of
  eight samples over their depth; the raymarch path lights each step
  through shadows and cookies too.
- `Fog.color` is the air's colour as the light around it leaves it, in
  both fogs: the plain fog draws it, and volumetric fog tends to it where
  no sun or lamp lights it, then adds the sun's shafts and the lamps'
  glow, tinted by it and scaled by `scattering`. So `volumetric` never
  makes a haze darker than the plain fog's, and a device that draws no
  volumetric fog shows the same haze without the shafts. Set the colour
  for the scene's light as for the plain fog (a night's haze dark, a
  city's glow in it); og.DayNight's `tintFog` and a cloud cover do that
  for a cycle. This is authored ambient, not geometrically shadowed sky
  light: at the same density it stays just as bright under a canopy or
  inside a closed room as outside. Scaling the colour uniformly scales
  both ambient and shafts, leaving their air contrast unchanged. Raising
  global fog's scattering brightens the sun and lamps, not this ambient.
- For an og.FogVolume, probe GI replaces environment light where it
  covers the air; scattering scales both direct and environment light.
  The hemisphere light and global fog's ambient have no geometric
  occlusion. Do not assume that adding a volume and GI makes indoor fog
  as dark as its surrounding surfaces.
- For a visible shaft, the view ray must pass through sunlit air below
  an opening, beside shadowed air, with a dim enough ambient and
  background for their difference to show. Real canopy contrast also
  relies on blocking sky light there; global fog does not infer that
  darkness. A sun outside the frame works; ground dapples alone do not
  establish a long sunlit path through the visible air. Arrange openings
  on the sun-facing side of a canopy. Darkening ambient cannot create
  missing direct light. Choose the opening and view before spending
  more slices or density on the fog.
  Strong forward scattering favours looking towards the light; weak
  side-on shafts are not evidence that shadows failed.
- Past `volumetricDistance` the fog goes on to the far scene and, by
  `Fog.sky`, over the sky, in its colour with the unshadowed sun's light
  in it: far towers fade into the haze as in the plain fog. Under the
  horizon a procedural sky stands for land, and both fogs hide it whole
  whatever `Fog.sky` is, so a gap in the world's ground reads as land
  lost in the haze, not as a band of sky. Above a cumulus base the lower
  sky is a cloud sea instead: it keeps the sky's fog share, with the
  physical sky's aerial perspective, not that extra land fog.
  It has no shafts and no lamps'
  glow there, so set the distance past the last lamp whose glow should
  show (the froxels grow with it).
- Local fog: og.FogVolume beside an og.Volume fills that box or sphere
  with fog (mist in a hollow, smoke in a room, haze for a lamp's shaft),
  lit as volumetric fog is. It needs no og.Fog: alone it lies over the
  plain fog, out to the farthest volume; with `Fog.volumetric` it adds to
  that medium, within `volumetricDistance`. Give the og.Volume a `fade`
  of a metre or more: a hard edge shows the froxels. The 16 nearest the
  camera are drawn; nothing is drawn for an orthographic camera, with
  RenderQuality.volumetricFog off, or in the 2D profile. Blended surfaces
  and sprites inside one are not fogged by it.
- Sky: reflections and the ambient light follow the drawn sky within
  sixteen frames, about a quarter of a second at 60 fps, so a drifting
  cloud, lightning or a fast day shows in them.
- DayNight: the image-based light follows the sky as drawn, captured a
  slice a frame. The probe answers a pure function of the tick:
  `get:og.DayNight` adds `hours`, `toSun`, `toLight`, `lightColor` and
  `lightIntensity` (the light drawn, the sun's or the moon's or the two
  handing over); `get:<sun>.og.DirectionalLight` of the light it drives
  gives the drawn `color` and `intensity` with `toLight` and `drivenBy`.
- Seasons: an enabled og.Season sets the sun's path and the day's length
  by its date and latitude in place of `DayNight.tilt` (`seasons §3`).
- PhysicalSky gives blue overhead, red sunsets, a white glow round the sun
  and a blue twilight zenith; the sun's colour turns white, then orange,
  then red, gone below the horizon, and `get:og.DayNight` answers those.
- By night the moon's light scatters through the same air: a full moon
  high up gives a dim blue sky, brightest round the moon, and the land
  under it the light of a dim day; a quarter moon about 0.38 of that, a
  new moon none, leaving the black, starlit sky. Stars show by their
  contrast with the sky: a full moon leaves only the brighter, none near it. `moonPhase` sets the
  moonlight on the land under og.DayNight with either sky. A moonlit
  night is a few stops under noon: open the camera's exposure for it as
  for twilight.
- og.DayNight's light is the sun's or the moon's, whichever is brighter.
  The moon stands opposite the sun, so it rises as the sun sets and takes
  the light over soon after: under og.PhysicalSky within a few minutes,
  low and orange through the air, nearly half its night light half an
  hour after sunset at the default tilt and full once it stands high;
  under the gradient sky once the sun's afterglow fades, about 4° under
  the horizon. At dawn it hands back. Where the two are about equal the
  light dims for a minute or two of game time, so shadows never jump
  across at full strength. A new moon gives none: the twilight then has
  only the sky's light.
- Clouds: bright tops, dark bases, silver edges against the sun. A broken
  sky hides the sun a share of the time; an overcast deck leaves flat grey
  light on every surface. They also draw over an HDR or authored LDR
  og.Environment background, lit by its SH ambient and the DirectionalLight
  (set it from og.load.environmentSun). For a cloud sea, put `base` and
  `base + thickness` below the camera: the same march draws the sunlit tops
  and fades into the distant background without cutting at the horizon.
  `steps` is their cost. Under a procedural sky, clouds are part of the
  sky: they are lit as bright as the sky behind them is drawn, so
  `environmentIntensity` (and og.DayNight's twilight lift) brightens or
  darkens the land under them, never the deck against the clear sky.
  For a cloud sea below a flying route, keep gameplay near world y 0:
  place the world's origin at a physical altitude and the cloud base
  below it (for example altitude 4000 m, base -2500 m, thickness 1800 m).
  Cloud heights are world-relative; the atmosphere and clouds share the
  same sea level. Do not lift every game object just to stand above clouds.
- Cloud shadows: the cumulus shades the world under it, along the sun's
  ray: the ground, the water, the grass, particles and the volumetric
  fog's shafts take it, in every camera, out to the horizon. A cloud is
  `scale` across, so a place stands in the sun or in a shadow for a
  minute or more as the wind carries them over: in a shadow the sun's
  light is gone and the sky's stays, and the exposure opens for it with
  `autoExposure`. `shadows` below 1 lets that much of the sun
  through a cloud; 0 casts none and dims the sun a little everywhere, as
  a level lit the same from start to end wants. They fall where the sun
  casts shadows and fade as the cover closes past 0.6; the shadow's edge
  is tens of metres soft, and the high layer casts none.
- Weather: og.Weather is one state for the air. Rain and snow close the
  sky as they build (og.Clouds' cover raised to theirs over the first
  half of `intensity`, or a deck of the weather's own under a procedural
  sky), and the light is dimmed once, by the darker of clouds and
  weather, so nothing is sunlit under heavy rain. Write `kind` and
  `intensity` and the weather drawn follows them over `change` (20 s by
  default): a new kind builds while the one before thins out, the sky,
  light, fog and particles blending through, and the ground (wetness,
  puddles, snow, dust) follows what falls. 10 to 30 s reads as weather
  turning in front of the player; 0 switches at once, right for loading a
  level. The deck costs what og.Clouds costs; `dim: 0` keeps the sky and
  the light as they are.
- This is the one ramp a weather takes: write the weather wanted and
  never step `kind` or `intensity` towards it. `change` is seconds per
  unit of intensity, so for every switch to arrive within T seconds set
  it to T over the heaviest intensity the game writes (a storm of rain at
  2 in 6 s: 3). What else a game draws with the weather (its own sky,
  fog, sea or wind) follows `drawnKind`, `drawnIntensity`, `fadingKind`
  and `fadingIntensity`, a kind's amount being the sum of the two that
  name it. They are the last tick's: the weather is drawn after the
  game's systems. To switch at once in play, set `change` to 0 for that
  tick and back once the drawn weather is the one written.
- Aurora: hard electrons give a low aurora with a pink-violet border, soft
  ones a high red-topped one; the air dims it near the horizon.
- Weather on surfaces: porous surfaces darken when wet and smooth ones
  gloss, drying in patches from high spots; rain streaks down walls;
  puddles fill level hollows (a terrain's layer heights first) and ripple,
  up to og.WeatherState.puddleLimit (a draining street: 0.3 to 0.5 keeps
  the water in the joints and hollows; raise it for a storm);
  on a terrain the joints and cracks of its layers' relief fill first, to
  just under the relief's middle (a layer's height 0.5) by half `puddles`,
  the drawn ground rising flat to the water so it mirrors the sky between
  the stones, which stand out of it (a height map should put its joints
  well under 0.5 and its crowns over it);
  snow settles on what faces up, deeper snow reaching steeper faces,
  lifting the drawn terrain; dust settles up and low; a strike flashes the
  scene with a bolt. An og.GrassPusher walking through snow or sand lying
  on a terrain leaves a track (packed darker, its sides shaded), which
  fills in over four minutes; it is a trail at the map's 0.25 m cells,
  not a shoe print (stamp those as og.Decal), and meshes keep none. Meshes (the standard and physical materials, hooks
  and `mode full lit` shaders), terrain, grass and scattered copies take
  all of it. Water takes the rain alone: rings where the sky is open over
  it, and from afar a duller mirror; no snow or dust lies on it. The 3D
  fluid, particles and unlit shaders take none. Shelter is a weather occlusion map looking along the
  rain as the wind slants it (og.Wind and og.Weather's own); a game that needs to know whether a
  place is sheltered casts a ray up against the colliders.

## 8. Wind and water

- og.Wind is the world's one wind: set its direction and speed and the
  vegetation leans (5 cm per metre of height for each m/s), the clouds
  drift (twice as fast aloft), rain and snow slant, ropes, cloths and
  garments are pushed and a body with `RigidBody.airDrag` is carried, all
  the same way. A `wind` on og.Clouds,
  og.Weather, og.Rope, og.Cloth or og.Garment adds to it: leave it at 0
  to follow the world, and set it only for air that differs there (a fan
  on one flag, a high cloud layer crossing the ground wind). A sea keeps
  its own (og.Water's `wind`): give it the same vector for waves that run
  with the air.
- Roots stay put, tips move furthest. The engine's vegetation sways
  unless its `sway` is 0. Where the field is simulated a gust front
  crosses grass, copies and trees together, eddies, and vegetation springs
  into a gust and settles; og.Model sway keeps the wind's own gusts.
  Motors (16 act) ripple out and settle.
- An og.WindMotor is one push on the air for everything: the tick blows
  the ropes, cloths and garments in its reach and carries bodies with an
  `airDrag` (a fan lifting a balloon, a blast scattering leaves), and the
  vegetation leans in the same place the same way.
- Water cost, cheapest first: a grid with detail waves; refraction and
  screen-space reflection; caustics (one surface a frame); a planar
  reflection, about a second scene pass at its resolution per surface;
  a sea (`wind`): up to three 256² tiles by the graphics quality.
- Planar reflections: one by default. With RenderOptions.planarReflections
  2 every lit pipeline carries one more texture and a second pass runs
  while both are in view; with 1 a second surface is never drawn in the
  first's mirror.
- Physical water (`absorption`, `scattering`): shallows over sand turn
  turquoise and deep water dark blue-green by themselves; red is gone in a
  few metres. A silty coast scatters ten times clear ocean.
- Underwater (`underwaterDensity` above 0, the camera below the surface):
  the frame's fog is the water's, darker with depth, and the sun falls in
  shafts as strong as `caustics`. Looking up, the sky and the sun show
  inside Snell's window (48.6 degrees from straight up), bent as the
  surface bends them, the horizon at its rim and broken by the ripples;
  outside it the surface mirrors the water.
- Mesh water (og.Water's `geometry`): the same colour by depth,
  refraction, foam and reflections as the grid, over the mesh's triangles
  alone and at their height. It has no wave heights: it is still water
  for og.physics.water, waterHeight, og.Buoyancy and og.Splash. Use it for
  a lake of any outline, not a square that covers the low ground around
  it.
- Rivers: a waterfall's pool churns white for about three widths. Use
  `flowMap` for a stream that bends round rocks.
- Normal maps: the lake mode is three.js Water's getNoise. The flow mode
  slides over the mesh's uv (or the grid's metres), cross-fades every 0.15
  of a cycle, and then, as three.js Water2, refraction 1 shifts the scene
  by N.xz · 0.05 of the screen, the reflection is tinted by `shallow`, and
  there is no sun highlight. `waveSpeed` is the lake's clock.
- Glint mode shades the body as three.js's water example:
  sunColor · max(L·N, 0) · sunColor · 0.15 + waterColor · max(N·V, 0),
  shaded by ripples and shadows (spec × sunColor over the light's
  intensity in three.js).
- Sea: at 10 m/s waves are about 80 m long with a significant height of
  2.1 m. Buoyancy rides every wave the surface rises with, down to a
  metre; the ripples under that only shade it. Detail waves fade where a pixel cannot
  hold them; whitecaps foam where crests fold. A seabed is re-read when
  the water, the terrain's placement or its edits change. Ageing foam: a
  whitecap's dense foam bursts within seconds and leaves lace for some ten
  seconds, gathered by currents of about 1 % of the wind.
- Wakes (`dynamicWaves`): every body that cuts an og.Water raises them, a
  hull by its shape and any other body by its box, on one 64 m square
  round the camera: the water being cut holds it (the first of several),
  and it rests two minutes after the last cut. Kelvin's 19.5 degree wedge
  at deep water's speeds, on a shallow lake too. A sea's surface rises
  with them; a lake or a river stays flat and shows them in its
  reflections and foam. An og.WaterSurface on the same entity draws its
  own ripples instead.

## 9. Interpolation

On a 120 or 144 Hz screen with a 60 Hz tick, a frame between ticks blends
from the tick before the last to the last by the time past it: positions
and scales lerped, rotations slerped; skinned joints the same and
blendshape weights lerped. Root motion and every pose the simulation reads
stay at the tick. UI anchored to an entity follows its drawn place;
object motion blur measures from the frame drawn before, never across a
snap or a cut. Immediate-mode draws (og.render.sprite, og.render.text,
og.debug) and shader time stay at the tick.

- Hosts that step whole ticks (`og run` without `--realtime`, `og render`,
  `og test`) and the probe's screenshots draw ticks exactly, so reference
  frames are the ticks'.
- The world_hash, snapshots, replays, lockstep, the probe and dumps read
  ticks, never the blend.
- Call `og.render.snap` on the tick of a teleport or respawn, or rely on
  `snapDistance` or the probe's teleport; opt out with og.Interpolation (a
  marker already moved every frame, a clock hand that jumps).

## 10. Cost

| Option | Cost | Cheaper |
|---|---|---|
| shadowed local light | its own maps each frame (cube: 6 passes) | paraboloid, fewer `shadowedLights` |
| rect light shadow | the costliest light | `rectShadows` false |
| sun cascades | one pass and map each | fewer cascades, `shadowBox` |
| SSR | ~1.6 ms half res at 1080p (M4 Pro) | half res, fewer steps |
| FSR 3.1 | +1 to 2 ms over TAAU | `taau` |
| AO, GI screen | prepass + rays | half resolution |
| volumetric fog | grows with slices | plain fog |
| planar reflection | a second scene pass | screen-space reflection |
| colour adjustments | ~0.5 ms CPU per change | hold still |
| CameraView | a draw of all its layers admit | `layers`, `everyTicks` |
| render target depth | a second draw of opaque meshes | leave off |

- `RenderOptions.without` leaves features out of every lit pipeline, which
  compiles faster and draws a little faster: for a game that never uses
  them.
- `lodBias` 0.5 simplifies models sooner.

Measuring a cost. The GPU runs passes at once, so no pass's timestamps
say what it cost the frame. Three numbers, each for its question:

- **What a feature costs the frame:** `og cost . --off "<options>" --
  <options for both>` runs the game with `og run --trace` at least 4
  times a side, on and off alternating, and prints each side's median
  GPU frame span, its range and the machine's load, and their
  difference. The off side's options turn the feature off (a `--setup`
  action or `--commands` the game answers); hold the camera with the
  common ones, give the run's length with og run's `--ticks N` among
  them and read a window with `--ticks A..B`. A difference
  inside the runs' range is no difference; rerun on a quiet machine.
- **Where a frame went:** `ProbeTimings.gpuPasses` and the trace's gpu
  thread give each pass's time alone on the GPU (nothing else in
  flight). The passes' alone time, the frame's `overlapped` and its
  `idle` sum to the frame's span (`args` of `gpu frame N`). Pick a
  window's frames by their `ts` against the ticks' `ts`: every span's
  `ts` is on the trace's one clock. A pass's
  `(in flight)` span on the `gpu in flight` threads includes waits:
  never its cost.
- **What a pass costs by itself:** `og run --gpu-serial --trace` makes
  every pass wait for the one before it, so its alone time is its whole
  cost; the frame is slower than a real one. Not in a publish build.

## 11. Cameras, render targets and billboards

- A CameraView draws everything its layers admit: skinned models, material
  hooks and shaders, terrain, sky, particles, lines, points, world sprites
  and text, lit and shadowed by the frame's lights. Keep it cheap with
  `layers` and `everyTicks` (a monitor at 4 costs a quarter).
- An hdr target is exposed by the frame that shows it (a mirror, a portal);
  an ldr one is finished (a monitor, a minimap). No view runs bloom, AO,
  SSR, depth of field, grading or custom passes, whose inputs are the main
  camera's; sprites in a view are not 2D-lit and do not glow.
- Billboards (SpriteFacing) are a mode of a sprite, not a component: the
  same quad, batch and shader, turned on the GPU per camera (the main
  camera, every CameraView and Mirror, each reflection probe face), so a
  snapshot holds only the mode. Its drop shadow shows the whole shape from
  every camera. A faced sprite sorted with the transparent meshes draws its
  shadow before them and shows through glass behind it; its layer no
  longer orders it against other sprites. With sortY or toneMapped false it
  keeps the sprite order after transparent meshes and particles. It is
  culled by a sphere round its anchor, depth tested, and writes no depth.
  World text (TextDesc.facing, FloatingTextDesc.facing) turns as one panel.

## 12. Pitfalls

- Judging an HDR look from a capture: captures clip at SDR white. Look at
  the game on an HDR display (`capabilities.hdr` true).
- Judging cost on `vulkan-cpu` or with an effect the scene does not draw.
- A preset applied, then the game sets a quality field: the field wins, by
  design. A game that names GlobalIllumination after applying a preset
  applies the preset again.
- Gliding teleports: snap the entity or the camera on that tick.
- An unbaked or stale LightVolume lights nothing; `og doctor --project`
  names the scene.
- A stealth rule reading 2D light: `og.render.light2dAt` is deterministic,
  with hard shadows through the same occluders the light map draws, so a
  system may store it.
- A gameplay rule reading weather shelter, wind sway, wakes or spray: they
  are drawn only.
