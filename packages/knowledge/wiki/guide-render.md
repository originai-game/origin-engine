<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `render` guide: Render and lighting choices

The engine's render options and what each costs: tiers and quality presets, antialiasing and transparency, lights and shadows, global illumination and reflection, post effects, materials, atmosphere and water, interpolation, and secondary cameras. looks §3 covers how a scene is lit and looks §5 how to measure a frame; this guide says which engine option to turn on, when, and what it costs.

The API: `og api show render`, `og api show atmosphere`. Every option here
is off or at its cheapest default until a game turns it on, and none of it
feeds back into the simulation.

<a id="guide-render-1"></a>
### `render §1` Tiers and quality presets

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
- High is paid only by the effects the scene draws: FSR 3.1 (about 1 to
  2 ms more than TAAU at 1080p and 1440p on an M4 Pro, sharper and
  steadier in motion), doubled shadow maps, full-resolution AO, doubled
  SSR and volumetric steps. A scene without those effects differs from
  medium only by its shadow maps.
- Automatic quality keeps the chosen upscaler while it lowers the render
  scale, and draws `taau` once its preset steps cap quality at medium or
  low.

<a id="guide-render-2"></a>
### `render §2` Antialiasing, upscaling and transparency

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

<a id="guide-render-3"></a>
### `render §3` Lights and shadows

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
- Point lights: cube shadows take six passes per light, paraboloid two
  (coarser; large casters' edges stay straight).
- Rect lights: the soft shadow is the costliest light in a frame (two maps
  when two-sided); the low preset turns rect shadows off. The shadow is
  cast from the rectangle's centre, so a caster close to a large panel is
  shadowed as from its centre. The highlight on a glossy floor keeps the
  rectangle's shape (LTC with three.js's tables). Aim one with
  `transform.lookAt`.
- Cookies: a spot projects one texture (a window's bars, stained glass); a
  point light takes a six-face strip, so a lamp behind a pierced shade
  throws its pattern on the walls.
- Mesh flags: `castShadow` is off by default (a lamp's glass, a glow card);
  `receiveShadow` is on (every mesh received shadows before the flag
  existed), and false keeps a glowing sign or a hero readable in a dark
  alley. Shadow maps ignore `layers`, so a body hidden from a first-person
  camera still casts.

<a id="guide-render-4"></a>
### `render §4` Global illumination and reflection

| Need | Use | Cost |
|---|---|---|
| static level, best quality | `og bake lightmaps` + LightVolume | CPU bake; free at run time |
| moving lights in a room | RealtimeLightVolume | a few probe captures a frame |
| contact and fine bounce | GlobalIllumination `screen` | rays per pixel over the prepass |
| bounce from off screen, sky occlusion | GlobalIllumination `probes` | probe cascades, coarse |
| both | `both` | the sum |

- Baked: static models use lightmaps; everything else inside a LightVolume
  (meshes, models, skinned characters, lit particles) takes the probes, so
  a ball rolling past a red wall turns red on that side. A volume added,
  moved or resized is unbaked until the next bake. `bakedLighting false`
  compares the bake against the realtime lights alone.
- RealtimeLightVolume: each capture draws every light, the sky and
  lights' shadows, emissive surfaces and lightmapped models, and sees the
  probes, so a lamp in a red room fills it with red light after a few
  sweeps. Its probes are used once all are relit (a few frames after it is
  added or resized), and the low preset falls back to the bake. Up to
  eight volumes draw: baked first, then realtime ones nearest the camera.
  Keep the box's faces a little inside the walls: its light reaches one
  spacing beyond them, and a probe on a wall sees into it (so does the far
  side of a wall thinner than a spacing). Its reflections are a soft image
  of the room, too soft for a mirror: put a ReflectionProbe there. It gives
  no contact shadow: pair it with PostEffects.ao or GlobalIllumination.
- Screen GI: rays over each pixel's hemisphere take the light the previous
  frame showed where they hit, so a few frames give many bounces (a red
  wall tints the floor, a sunlit patch lights the ceiling), but nothing
  off screen. Under TAA the rays change each frame and average; without
  it a still frame keeps one blurred pattern.
- Probe GI: cascades traced in software around the camera light every lit
  surface (meshes, terrain, grass, scattered copies); light from behind
  walls bounces and the sky is occluded under roofs and between
  buildings. Coarse: pair it with PostEffects.ao, whose GTAO bent normals
  pick the probes' light from the direction a crease opens to.
- Reflections, best first: SSR, then reflection probes and probe GI, then
  the environment or sky. A grounded environment (`groundHeight`) shows
  the same ground in mirrors, water and clear coats.

<a id="guide-render-5"></a>
### `render §5` Post effects

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
  a natural pair.
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

<a id="guide-render-6"></a>
### `render §6` Materials

- One StandardMaterialDesc covers PBR, stylised and physical shading; share
  materials, since instancing is per (geometry, material) (looks §5).
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
- Texture arrays put variants of one look (tiles, skins, damage states) in
  one material and one draw.
- Wireframe is drawn as lines, not a polygon mode (WebGPU has none).

<a id="guide-render-7"></a>
### `render §7` Fog, sky, clouds and weather

- Fog: plain distance and height fog is nearly free. Volumetric fog (a
  froxel grid, RenderCapabilities.volumetricFog names the method) makes
  shafts through windows and trees that move with the sun; opaque surfaces
  and the sky are seen through it. Its cost and memory grow with its
  slices (`volumetricStepScale`); thinner slices resolve a shaft's edge in
  depth and fog just in front of surfaces. Froxel cells take the mean of
  eight samples over their depth; the raymarch path lights each step
  through shadows and cookies too.
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
  `lightIntensity`; `get:<sun>.og.DirectionalLight` gives the drawn
  `color` and `intensity` with `toLight` and `drivenBy`.
- PhysicalSky gives blue overhead, red sunsets, a white glow round the sun
  and a blue twilight zenith; the sun's colour turns white, then orange,
  then red, gone below the horizon, and `get:og.DayNight` answers those.
- Clouds: bright tops, dark bases, silver edges against the sun. A broken
  sky hides the sun a share of the time; an overcast deck leaves flat grey
  light on every surface. `steps` is their cost.
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
  weather, so nothing is sunlit under heavy rain. Ramp `intensity` over
  seconds to build or clear it: the particles in the air stay. The deck
  costs what og.Clouds costs; `dim: 0` keeps the sky and the light as
  they are.
- Aurora: hard electrons give a low aurora with a pink-violet border, soft
  ones a high red-topped one; the air dims it near the horizon.
- Weather on surfaces: porous surfaces darken when wet and smooth ones
  gloss, drying in patches from high spots; rain streaks down walls;
  puddles fill level hollows (a terrain's layer heights first) and ripple;
  snow settles on what faces up, deeper snow reaching steeper faces,
  lifting the drawn terrain; dust settles up and low; a strike flashes the
  scene with a bolt. Meshes (the standard and physical materials, hooks
  and `mode full lit` shaders), terrain, grass and scattered copies take
  all of it. Water takes the rain alone: rings where the sky is open over
  it, and from afar a duller mirror; no snow or dust lies on it. The 3D
  fluid, particles and unlit shaders take none. Shelter is a weather occlusion map looking along the
  rain as the wind slants it (og.Wind and og.Weather's own); a game that needs to know whether a
  place is sheltered casts a ray up against the colliders.

<a id="guide-render-8"></a>
### `render §8` Wind and water

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
  2.1 m. Buoyancy rides the drawn long waves; the shorter ones are the
  GPU's alone and never read back. Detail waves fade where a pixel cannot
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

<a id="guide-render-9"></a>
### `render §9` Interpolation

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

<a id="guide-render-10"></a>
### `render §10` Cost

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
| CameraView | a draw of all its layers admit | `layers`, `every` |
| render target depth | a second draw of opaque meshes | leave off |

- `RenderOptions.without` leaves features out of every lit pipeline, which
  compiles faster and draws a little faster: for a game that never uses
  them.
- `lodBias` 0.5 simplifies models sooner.

<a id="guide-render-11"></a>
### `render §11` Cameras, render targets and billboards

- A CameraView draws everything its layers admit: skinned models, material
  hooks and shaders, terrain, sky, particles, lines, points, world sprites
  and text, lit and shadowed by the frame's lights. Keep it cheap with
  `layers` and `every` (a monitor at 4 costs a quarter).
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

<a id="guide-render-12"></a>
### `render §12` Pitfalls

- HDR output is opt-in and not yet accepted on a real HDR display; ship
  SDR.
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
