---
name: og-render
description: What the frame shows on Origin Engine and how it is lit — post effects (AO, outlines, depth of field, bloom, LUTs and colour sliders, screen blur), particle emitters with every line of their grammar, trails, decals, 2D lights, physical materials (clearcoat, sheen, transmission, iridescence), tone mapping, exposure, environment maps, shadows, antialiasing and quality presets, baked lightmaps and light probes, volumetric fog, area lights and cookies, levels of detail, reflection probes and screen-space reflections, fog, sky, day and night, weather and water, terrain (heightmaps, generated terrain, layers, scatter, wind), material shaders and full custom shaders in WGSL, custom post passes, render targets, second cameras, split screen, mirrors and layers, geometry made at run time, lines, points, picking, debug draw, and footage played as a texture. Use when setting up how a 3D or 2D scene looks, adding an effect, a light, a sky, water or terrain, writing a shader, or drawing what no asset holds.
---

Everything the renderer draws beyond the meshes is data: resources
(`og.PostEffects`, `og.RenderSettings`, `og.Fog`, `og.Sky`) and components
(`og.ParticleSystem`, `og.Water`, `og.Terrain`, `og.CameraView`), each off
until the game turns it on, plus a few assets written as text (emitters,
shaders, post passes, terrains). Choosing a look for the premise is
og-taste; a look sets most of these resources at once (og-taste §13).
Frame time is og-build §9.

- §1 Effects: post, particles, trails and beams, decals, 2D lighting,
  materials, texture repeat, the HUD.
- §2 Particle emitters: every line of a `type particles`.
- §3 Light, exposure, shadows, antialiasing, graphics quality and the display (frame cap, render scale, automatic quality).
- §4 Baked lighting, volumetric fog, area lights and cookies.
- §5 Levels of detail, reflection probes and screen-space reflections.
- §6 Atmosphere and water: fog, sky, day and night, weather, water.
- §7 Terrain: the `type terrain` file, layers, sampling, editing,
  scatter and wind, streaming.
- §8 A generated terrain: every line of the `generate` block.
- §9 Material shaders: WGSL hooks of the lit shader.
- §10 Full custom shaders and custom post passes.
- §11 Render targets, second cameras, split screen, mirrors and layers.
- §12 Geometry made at run time, lines and points, picking, debug draw.
- §13 Video: footage in a game.

## 1. Effects

Every effect is off until the game turns it on, as data:

- **Post** — the `og.PostEffects` resource: `ao`, `outline`,
  `depthOfField`, `motionBlur`, `godRays`, `glow`, `lut` (a handle from
  `og.load.lut(world, "luts/look.cube")`), the colour adjustments after it
  (`temperature`, `tint`, `brightness`, `contrast`, `balanceShadows`,
  `balanceMidtones`, `balanceHighlights`, `hue`, `saturation`,
  `vibrance`, `grayscale`, `invert`, `threshold`: a photo editor's sliders,
  0 is off; baked with the LUT into one table, so they cost no pass;
  each frame that changes one re-bakes it on the CPU, about half a
  millisecond to a few, so a fade such as `post.saturation = -t` is
  fine and a slider moved on every frame of play is not),
  `vignette`, `chromaticAberration`, `lensDistortion` (barrel above 0,
  pincushion below, `lensDistortionCenter`; fitted so no black edge
  shows, the UI not bent), `screenBlur` (`"zoom"` towards
  `screenBlurCenter` for a dash or a hit, `"spin"` round it, or
  `"directional"` along `screenBlurAngle` for a fast pan; off until
  `screenBlurStrength` is above 0, and fade that back to 0 rather than
  holding it: a frame blurred for long reads as a fault), `filmGrain`, `flash`, and a fixed shake of the
  frame through `shakeTrauma` (an earthquake that lasts; a hit's shake is
  `og.camera.shake`, which decays by itself: og-feel §4). A game with fast
  motion wants `render: { antialias: "taa" }` in `og.game`.
- **Particles** — an emitter is an `.ogt` asset of `type particles`
  (shape, rate, bursts, lifetime, speed, curves over life, texture or
  array, 3D or sprite rendering; every line is the grammar of §2)
  loaded with `og.load.emitter`; an entity with `og.ParticleSystem
  { emitter, seed, startTick }` emits it at its transform. Particles are
  drawn by the renderer from the tick, deterministic and never part of the
  simulation.
- **Trails and beams** — components `og.Trail` (follows the entity) and
  `og.Beam` (between two points); `og.render.ribbon(world, desc)` draws a
  polyline this tick.
- **Decals** — `og.Decal` projects an atlas rectangle
  (`og.RenderSettings.decalAtlas`) onto what its box covers.
- **2D lighting** — `og.Lighting2D` (ambient, the lit layer range,
  `resolution`, `shadowAngles`), `og.Light2D` and `og.Occluder2D`
  components, `castShadow` on sprites; sprites also take `flash`, `glow`
  and `blend`.
  - A `Light2D` is a `point`, a `spot` (a cone along the entity's local +x
    between `innerAngle` and `outerAngle`) or a `global` light (every lit
    pixel, unshadowed). It fades `(1 - d/r)^falloff` to zero at `radius`.
    Shadows soften with `sourceRadius`, and the penumbra widens away from
    the occluder.
  - An `Occluder2D` is a `width` × `height` box or a polygon of up to 16
    `points` (concave is fine), turned and scaled with its entity. A pixel
    inside an occluder is not shadowed by it.
  - Light layers: `Light2D.layers` and `Occluder2D.layers` are bit masks
    matched against a receiver's `og.LightMask2D` (layer 0 without one). A
    frame lights at most 4 different receiver masks apart.
  - Normal maps: a texture whose import settings say `normal auto` (a bevel
    raised from the art's silhouette) or `normal "hero_n.png"` (red towards
    +u, green up the image) is shaded by where its surface faces each
    light, which stands `Light2D.height` above the plane. This covers its
    sprites, tiles and 2D rig meshes; a flat area keeps exactly the light it
    had. The direction pass runs only while some texture has a normal map.
  - Game rules ask `og.render.light2dAt(world, point, layers)`: the same
    lights and hard shadows, as the same bits on every host (no ambient).
    Use it inside a system for a stealth check, never a pixel read.
- **Materials** — toon, rim, dissolve, hit flash and outline are
  `og.render.standardMaterial` fields; animate them with
  `og.render.updateMaterial(world, material, { dissolve: t })`. Beside
  three.js's standard fields (metalness, roughness, emissive, `normalMap` in
  glTF's tangent frame, `alphaTest`, `transparent`, `side: "double"`,
  `unlit`), three.js `MeshPhysicalMaterial`'s fields work on every tier:
  `clearcoat` (with `clearcoatMap`, `clearcoatNormalMap`) for car paint and
  varnish, `sheen` and `sheenColor` for cloth and velvet, `transmission`
  with `thickness`, `ior` and `attenuationColor` for glass and water that
  refract what is behind them, `iridescence` for soap film and oil,
  `anisotropy` for brushed metal; a glTF's `KHR_materials_*` extensions
  load into them (and `KHR_materials_unlit` into `unlit`). Each map slot has
  a uv transform, three.js's `offset`/`repeat`/`rotation`: `mapOffset`,
  `mapRepeat`, `mapRotation`, the same for `metalnessMap`, `normalMap`,
  `emissiveMap` and `aoMap` (tiling, atlas cells, a glTF's
  `KHR_texture_transform`). A glTF with material variants
  (`KHR_materials_variants`) draws one per instance through
  `og.Model.variant`, an index into `GltfInfo.variants`. A non-uniform `scale` shades correctly. Set `sheenColor`
  with `sheen` (black is no sheen). Cloth: velvet is a very dark,
  saturated base (about a third of the cloth's colour) under a bright,
  desaturated sheen (`sheenRoughness` about 0.5, `sheenColor` a pale tint
  of the colour, `roughness` 1): a dark face and a soft glowing rim where
  the folds turn away; cotton, wool and canvas a base near their colour
  with a faint, rough sheen (`sheenRoughness` 0.8 to 1, `sheenColor` dim,
  about a third of the base) for matte cloth with a pale edge at its
  silhouette. Keep `roughness` 1 on cloth: the sheen is its only gloss,
  and a lower one draws a plastic highlight. Folds wider than they are
  deep read as cloth; deep, creased ones streak; silk and satin are `anisotropy`, not
  sheen. The sheen conserves energy (what it reflects is taken from the
  cloth under it), so a white sheen never makes cloth glow. `thickness` is in the mesh's units and
  `attenuationDistance` 0 absorbs nothing. A transmissive material refracts
  only what was drawn before it (not other glass or water), and unlit and
  toon materials ignore the physical fields:

  ```ts
  const amber = og.render.standardMaterial(world, { color: { r: 1, g: 1, b: 1 }, roughness: 0.02, transmission: 1, thickness: 1, ior: 1.5, attenuationColor: { r: 0.95, g: 0.45, b: 0.08 }, attenuationDistance: 0.35 });
  ```
- **Texture repeat** — three.js's `texture.repeat` is the geometry's
  `uvRepeat` here: `og.render.planeGeometry`, `boxGeometry` and
  `sphereGeometry` scale their UVs from 0..1 to 0..`uvRepeat`, and material
  textures sample with repeat wrapping, so the texture tiles. A ground grid
  every 10 m on a 1,600 m plane (`references/metal/ground-grid.png` in
  og-tests):

  ```ts
  const ground = must(og.render.planeGeometry(world, { width: 1600, height: 1600, uvRepeat: { x: 160, y: 160 } }));
  const grid = og.render.standardMaterial(world, { map: must(og.load.texture(world, "textures/grid.png")), roughness: 0.9 });
  // turn it -pi/2 about x for a floor
  ```
- **HUD** — `og.render.sprite` / `og.render.text` with `screen: true` draw
  in CSS pixels from the top-left over the finished frame, untouched by
  post effects, with an optional `clip`. `og.ui` draws in the same
  overlay at `UI_LAYER` (1000): screen sprites and text at a layer up to
  1000 draw under every UI element, above 1000 over every one (modals and
  the screen transition too), so a HUD icon made of sprites sits on a UI
  panel at `layer: UI_LAYER + 1` while its screen shows.

`templates/fx-showcase` shows every one of these on its first five stations;
every field is in `og api show render`. Floating text,
`og.render.floatingText(world, { text: "-12", position, color })`, pops a
damage number or a pickup that rises and fades by itself (amounts: og-feel
§7).

## 2. Particle emitters: the grammar

An emitter is an `.ogt` of `type particles` under `assets/` (`fx/` by
custom). Every line is optional; what is left out keeps its default (a
small fountain of soft white points). A `<min> <max>` pair is a random
range per particle, one number is both:

```text
type particles
version 1
---
maxParticles 256          # the ring: the newest overwrite the oldest (1 to 1048576)
shape cone 15 0.05        # point | sphere <radius> [surface] | cone <angle°> <radius> | box <x> <y> <z> (half extents)
spread 0                  # degrees of random deviation from the shape's direction (180: every way)
rate 40                   # particles per second; 0 for bursts only
burst 0 30                # <time s> <count> [<cycles> <interval s>]: at that time of each cycle (repeatable)
duration 1                # seconds per emission cycle
looping true              # false: one cycle, then the particles live out
lifetime 1 1.5            # seconds
speed 1.5 2.5             # units per second, along the shape's direction
size 0.08 0.12            # the quad's side in world units
sizeOverLife 0 1 1 0      # <t> <value> pairs, t 0 (birth) to 1 (death), at most 8; multiplies size (default 1)
colorOverLife {           # keys <t> <r> <g> <b> <a>, linear, at most 8 (default: white, fading over the last quarter)
  key 0 1 1 1 1
  key 1 1 1 1 0
}
color 1 1 1 1             # a tint, <r> <g> <b> [<a>]; above 1 blooms
rotation 0 0              # start rotation, degrees
angularVelocity 0 0       # degrees per second
gravity 0 -2 0            # acceleration, units per second²
drag 0                    # velocity scales by 1 / (1 + drag × dt)
noise 0 0.5 0.3           # <strength> [<frequency> <scroll>]: a moving turbulence
texture "fx/spark.png"    # an asset id; left out: a soft round dot
sheet 4 4 16 life 1       # <columns> <rows> [<frames> [off | life <cycles> | fps <n> | random]]
blend additive            # additive (fire, sparks, magic) | alpha (smoke, dust); left out: alpha for lit meshes (solid chips), additive otherwise
space world               # world: born at the emitter, then free | local: carried with it
align camera              # camera | velocity <stretch>: stretched along its motion (sparks, rain)
softDistance 0            # fade where the quad nears the scene behind it (0: off)
collision 0.4 0.2 depth   # <bounce> <friction> [depth | colliders], world space: depth bounces off what is drawn; colliders off the game's og.Collider shapes, on or off screen (sprite mode always), both on the GPU
render billboard          # billboard (3D) | sprite <layer>: 2D, drawn in the sprite batch on that layer
glow 0                    # sprite mode: strength in the 2D glow
array "fx/smoke.ogt" random   # sprite mode: draw from a texture array, [<layers>] [off | life <cycles> | fps <n> | random]
round false               # sprite mode: fade each particle to a round edge over its texture or layer
render mesh box           # or sphere | tetrahedron | "models/shard.glb": each particle a small tumbling shape (3D)
lit true                  # billboards and meshes shaded by the sun, the sky and local lights (default false: unlit)
roughness 0.04            # a lit mesh's surface (default 1: matte): below 1 its highlight and the sky's reflection lie over the body at full strength (glass shards)
distortion 0.006          # billboards bend the scene behind them by up to this fraction of the screen height (heat haze 0.004 to 0.01)
events death collision    # birth | death | collision: send og.ParticleEvent for these (simulated in the tick)
subEmitter death "fx/pop.ogt" 12 0.3   # <birth | death | collision> "<emitter id>" <count> [<inherit velocity>], up to 4
```

- Directions: `point`, `cone` and `box` emit along the entity's +y
  (`spread 180` sends a point every way), `sphere` outwards. In a 2D game
  set `render sprite <layer>`: positions stay in the xy plane, sizes and
  speeds are in world units like a sprite's, and the particles take the
  sprite batch's order and glow like any sprite.
- A one-shot effect (a hit, a pickup, landing dust) is `rate 0`,
  `looping false`, one `burst 0 <count>` and a short `duration`; fire
  and restart it by writing the tick to its `og.ParticleSystem.startTick`
  (moving the entity first). Spawn it in `start` without a `startTick`:
  the default, `PARTICLES_ARMED`, means never written, so it waits
  armed and emits nothing until its first use (write `PARTICLES_ARMED`
  back to re-arm one without firing it). A lasting one (embers, motes,
  rain) keeps the defaults `looping true` and a `rate`, and emits from
  tick 0 without a `startTick`.
- `sheet` animates a sprite sheet texture: `life 1` plays its frames once
  over each particle's life, `fps 12` at a rate, `random` one frame per
  particle. `array` does the same over a texture array's layers (og-2d §1), one
  draw for the whole emitter; a billboard emitter with an array is refused.
- `collision` bounces 3D billboards off what the depth buffer shows on the
  full tier (sparks on a floor); sprite-mode emitters skip it. It is a
  look, never gameplay: particles never touch the simulation.
- `render mesh` draws each particle as a shape (debris, shards, leaves)
  scaled by its size and turned by its rotation; `lit true` shades it
  (and lit billboards: smoke the sun lights from one side). A lit mesh
  with `roughness` below 1 is smooth: the sun's and the lights' highlight
  and the sky's Fresnel reflection are added over its body at full
  strength, so with `blend alpha` at a low alpha a shard stays clear and
  glints only where it faces the light. `distortion`
  draws a billboard as a bend in the scene behind it rather than a colour:
  without a texture a rising waver (heat haze), with one along its red and
  green as a normal map (a shock ring), shaped by its alpha; not on a mesh
  emitter. Keep it small: 0.01 already bends a check by 7 px at 720p.
- `subEmitter` starts another emitter's particles where one of this
  emitter's is born, dies or hits something (a firework's shell bursting
  into sparks): `count` particles each, inheriting that fraction of its
  velocity. One level deep: a sub-emitter's own `subEmitter` lines are
  ignored. A look, like the rest, unless the emitter has an `events` line.
- `events` is the one way particles reach gameplay. Once the game names
  `og.ParticleEvent`, a system whose emitter has an `events` line is
  simulated in the tick (an engine system, with the engine's
  randomness, carried by snapshots) and drawn as simulated, so what the
  game reads is what the player sees. Its `collision` hits og.Collider
  boxes and spheres faster than 0.5 m/s. Read them the tick after:

  ```ts
  for (const e of world.events(og.ParticleEvent)) {
    if (e.kind === "collision") ring(e.position);
  }
  ```
- The cook refuses an unknown line, a missing texture, array, mesh model
  or sub-emitter, and a value of the wrong shape, naming the line.

A hit's sparks in a 2D game (`fx/hit.ogt`):

```text
type particles
version 1
---
maxParticles 24
shape point
spread 180
rate 0
burst 0 10
duration 0.1
looping false
lifetime 0.18 0.32
speed 3 6
size 0.12 0.22
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.95 0.75 1
  key 1 0.95 0.55 0.2 0
}
drag 4
texture "fx/spark.png"
blend additive
render sprite 12
```

`templates/archetypes/idle/assets/fx/` (taps, stars, a milestone),
`templates/fx-showcase/assets/fx/` (fire, smoke, sparks with collision,
2D embers) and `templates/archetypes/third-person/assets/fx/` hold more.

## 3. Light, exposure, shadows and antialiasing

- **Tone mapping and exposure** (`og.RenderSettings`): `toneMapping` is
  `aces` (the default), `agx` (bright saturated lights go to white instead
  of shifting hue: neon, fire, magic), `neutral` (Khronos PBR Neutral:
  colours up to about 0.8 come out as picked and only highlights roll
  off; product shots, toy and cartoon colour that ACES darkens) or
  `none`; `toneMapped false` clamps
  so 2D colours come out as authored. `exposure` multiplies every camera's
  `Camera.exposure`. `autoExposure` meters each frame to middle grey;
  `adaptSpeedUp` and `adaptSpeedDown` (EV per second of game time, so a
  replay adapts the same) make it adapt like an eye: 1 up and 3 down,
  walking out of a cave into sun.
- **Environment maps** light PBR materials from a photographed sky: an
  `.ogt` of `type environment` beside an equirectangular `.hdr` or `.exr`
  (`source "studio.hdr"`, `size 128`, `intensity`, `rotation` in degrees,
  `samples`), prefiltered at cook time. Set it on `og.Environment`:

  ```ts
  const env = must(world.resource(og.Environment));
  env.map = must(og.load.environment(world, "env/studio.ogt"));
  env.rotation = 40;          // degrees about +Y; light and background turn together
  env.background = true;      // draw it behind the scene (Camera.background and og.Sky give way)
  ```

  `intensity` replaces `RenderSettings.environmentIntensity` while a map is
  set; `backgroundIntensity` dims the background alone;
  `StandardMaterialDesc.envMapIntensity` scales it per material. A look's
  `Environment` block sets it too.
- **Shadows**: the sun (`og.DirectionalLight`) casts three cascades;
  `shadowSoftness` (0.01–0.05) widens the penumbra with the distance from
  caster to receiver, as a real sun's does, and `cascadeBlend` (0.1–0.2)
  hides the seams between cascades. Point and spot lights with
  `castShadow` cast cube shadows (`pointShadows: "paraboloid"` is the
  cheaper, coarser choice).
- **Antialiasing and renderer options** are `GameDesc.render`, read once
  when the renderer attaches:

  ```ts
  og.game({ render: { antialias: "taa", taaHistory: "catmullRom", taaSharpen: 0.3, pointShadows: "auto", shadowMapSize: 2048 }, ... });
  ```

  `antialias`: `tier` (TAA, the default),
  `taa` (converges over about 8 frames: fast motion, thin
  geometry), `msaa4`, `fxaa` (one cheap pass after tone mapping; pixel and
  low-resolution looks), `off` (pixel art). `taaHistory: "catmullRom"`
  stays sharper under camera motion and `taaSharpen` (0–1) restores
  detail TAA softens. `RenderCapabilities.antialias` says what the device
  resolved it to.
- **Graphics quality** is RenderSettings' quality fields, read every
  frame: `rectShadows`, `shadowedLights`, `shadowMapScale`,
  `volumetricFog`, `softParticles`, `screenSpaceReflections`,
  `ambientOcclusion`, `aoResolution` (`postEffects`, or `full`/`half`
  in place of `PostEffects.aoResolution`), `ssrStepScale` and
  `volumetricStepScale` (multiples of the SSR steps and the volumetric
  fog's slices), `msaaSamples` (4, or 1 for FXAA where the
  antialiasing multisamples), `upscaler` (under TAA, `taau` or `fsr3`:
  AMD FSR 3.1, sharper and steadier in motion for more GPU time).
  `og.render.applyQuality(world, preset)`
  writes a preset's row of them and records it in
  `RenderSettings.quality` (`none` before any):

  | Field | low | medium | high |
  |---|---|---|---|
  | rectShadows | false | true | true |
  | shadowedLights | 2 | 8 | 8 |
  | shadowMapScale | 0.5 | 1 | 2 |
  | volumetricFog, softParticles, screenSpaceReflections, ambientOcclusion | false | true | true |
  | aoResolution | half | postEffects | full |
  | ssrStepScale, volumetricStepScale | 0.5 | 1 | 2 |
  | msaaSamples | 1 | 4 | 4 |
  | upscaler | taau | taau | fsr3 |

  Medium is the defaults, the frame drawn before any preset. The renderer
  reads the fields and nothing else, so `og run --commands
  "0.5:get:og.RenderSettings"` shows what is drawn, and a field set after
  the preset wins. The switches only take away and the scales only
  scale: the scene's PostEffects and lights keep their settings. High
  costs GPU time where the scene draws AO, SSR or volumetric fog (about
  twice each). The device caps two fields, which keep the row's value:
  `RenderCapabilities.maxShadowedLights` and `maxMsaaSamples` (1 where
  the antialiasing resolved to TAA, FXAA or off, so 4 draws one sample). A settings page
  (`og.ui.settingsTree`) offers the presets and the game applies the
  player's pick with `og.render.applyQuality`; apply a saved preset
  yourself at start:

  ```ts
  start(world) { og.render.applyQuality(world, save.settings.quality ?? "medium"); }
  ```

  Inside a system, declare `og.RenderSettings` among its writes.

- **Display: frame cap, battery saver, render scale, automatic
  quality** (`og.render.setDisplay(world, settings)`, read back with
  `og.render.display(world)`). Presentation only: the settings are not in
  the world, so hashes, saves and replays never see them, and a native run
  (`og run`, `og test`, reference frames) draws at scale 1 whatever they
  say; they take effect in the browser.
  - Each call changes only the fields it names; the rest keep their
    values (the defaults before any call), so a settings page sets the
    one field the player moved. `og.render.resetDisplay(world)` puts every
    field back to its default (a Reset to defaults button).
  - `quality`: the player's preset (`low` … `ultra`), drawn in place of
    the game's RenderSettings without writing them; null (the default) is
    the game's own.
  - `frameCap`: `auto` (60 on phones, the display's rate on desktops),
    `display`, `fps144`, `fps120`, `fps60`, `fps30`. Skipped display frames still read input
    and the fixed tick is unchanged.
  - `batterySaver`: 30 frames a second, render scale at most 0.75,
    automatic quality on.
  - `renderScale` 0.5–2: the 3D scene at that fraction of the display,
    scaled up bilinearly; the UI and screen text stay sharp.
  - `autoQuality` (on by default), `autoMinScale` (0.5), `autoPresetSteps`
    (1): while frames run over the cap's interval the scale drops a tenth
    at a time, then the quality is capped at medium (and low with 2),
    without writing RenderSettings; it comes back up when there is room,
    and never oscillates. Turn it off for a game whose look must not move
    (pixel art draws on its own grid and ignores the scale anyway). A rung
    above that falls back within a few frames waits twice as long before
    it is tried again and is given up after three falls.

  - `fullscreen`: the web shell enters fullscreen (on the next press when
    the browser wants a gesture) and leaves it; the player's Escape turns
    it off in `display().settings`.
  - `powerPreference`: `highPerformance` (default) or `lowPower`, the GPU
    a laptop with two uses. The adapter is chosen at the boot, so it
    applies on the next load: `display().restartNeeded` is true until
    then, and a settings page says "Applies after a restart".
  - `display()` also reports `vsync` (always on in a browser, which has no
    switch: show it fixed), `fullscreen` and the `powerPreference` the page
    loaded with.

  The player's graphics page is a `og.ui.settingsTree` page over
  `display().settings` (og-ui §2), confirmed by its countdown: keep
  `display().settings` in the game's settings (`og.platform.saveSettings`)
  and pass them back to `setDisplay` at start; they are never in the world.

  ```ts
  og.render.setDisplay(world, { frameCap: "fps30", autoQuality: false, renderScale: 0.8 });
  const d = og.render.display(world); // d.renderScale, d.presetSteps, d.maxFps, d.gpu.total, d.deviceLosses
  ```

  A lost GPU device (iOS Safari after the page was in the background) is
  recovered by the engine: the next frames draw the world as it is, the
  game never sees it (`display(world).deviceLosses` counts them). A new
  device that takes other texture encodings gets the cooked textures
  transcoded again from the bundle; in the browser they draw as a flat
  placeholder for the few frames that takes.

## 4. Baked lighting, volumetric fog, area lights and cookies

**Baked lighting.** Use it for light that bounces around a static space,
such as a red wall tinting the floor beside it or a room lit through its
door. Mark the scene's unmoving models `static true` and put an
`og.LightVolume` box over the space where dynamic objects move:

```
entity "hall" {
  model "models/hall.glb"
  static true
  Transform {
    position 0 0 0
    quaternion 0 0 0 1
    scale 1 1 1
  }
}
entity "hall-probes" {
  Transform {
    position 0 2 0
    quaternion 0 0 0 1
    scale 1 1 1
  }
  LightVolume {
    size 12 4 12
    spacing 1
  }
}
```

Then run `og bake lightmaps` in the project. Its options are
`--samples 256`, `--bounces 3`, `--texels-per-metre 16`, `--max-size 1024`,
`--sky r,g,b`, `--seconds S` and `--scene <id>`; the fx-showcase Cornell
box takes 1.4 s at 512 samples. The bake is a CPU path tracer and needs
no GPU. It writes `<scene>.lightmap.ogt` and `.ktx2` beside the scene;
commit both.

A static entity needs a name. The bake holds the light that bounces, the
sky and emissive surfaces; lights still shine and cast shadows live on
top. Static models draw with their lightmap, and everything else inside a
LightVolume takes its ambient light from the probes. After you change the
scene's text or a static model's geometry, the bake is stale and the scene
renders unbaked until you bake again; a re-cook that keeps the geometry
(a cooker update, a texture or raycast option) keeps the bake. `og doctor
--project` warns `unbaked`, and
`og.render.bakedLighting(world)` lists each scene with `baked` and a
`reason`. `RenderSettings.bakedLighting = false` draws everything unbaked,
which is useful to compare. Every instantiated scene draws its own bake,
so baked scenes loaded together or streamed in side by side all keep
their light.

**Volumetric fog.** Set `Fog.volumetric = true` to light the fog as a
medium, which gives shafts through windows and the cones of spots. Tune it
with `density`, `heightDensity` and `falloff` (thicker low down),
`scattering` (how bright) and `anisotropy` (towards 1, bright looking into
the light), up to `volumetricDistance`. Fog of 0.03 to 0.1 per metre reads
well indoors. With `og.DayNight`, the shafts move with the sun.
`RenderCapabilities.volumetricFog` is `froxel` (the default) or
`raymarch` (the cheaper method, when the renderer was asked for it). In
both, the sun, spots, points and rect lights light the fog, so a lantern
glows in it.
`RenderCapabilities.volumetricLights` is how many local lights light one
sample (64 for the froxel grid, 8 for the raymarch, brightest first).

**Area lights.** `og.RectLight { width, height, color, intensity,
twoSided }` emits from a rectangle facing the entity's -Z. Aim it with
`og.transform.lookAt`, as you would a spot. A glossy floor shows the
rectangle's shape in its highlight. Pair one with an emissive slab for
the fixture. Set `castShadow` for a soft shadow: sharp where a caster
touches the floor, widening with the panel's size away from it. It takes
one shadow layer (two when `twoSided`) from the same budget as shadowed
spots and points, and is drawn from the panel's centre, so a caster right
against a large panel is shadowed as from its middle.

**Cookies.** `SpotLight.cookie` is a texture projected along the cone,
like a window's lattice or stained glass. `PointLight.cookie` is six
square faces stacked top to bottom (+X, -X, +Y, -Y, +Z, -Z), like a
pierced lantern. A cookie multiplies the light and its colour. A frame
draws at most `og.render.capabilities(world).maxLightCookies` (24) cookie
layers: a spot cookie takes one, a point cookie six, and a texture shared
by lights of one kind counts once. Past that, the later lights shine
without their cookies and the log warns, naming them; share cookie
textures between lanterns to stay under it.

## 5. Levels of detail, reflection probes and screen-space reflections

**Levels of detail.** Every model the cook reads is simplified into
levels: by default `lods 1 0.5 0.25`, each level a fraction of the full
mesh's triangles. A glTF's own `MSFT_lod` levels are kept instead. Import
settings change them per model, and `lods 1` keeps only the full mesh:

```
type import
version 1
---
model {
  lods 1 0.4 0.15
  lodError 0.08
}
```

The cook measures each level's error, the largest distance between the
level and the full mesh in the model's units, and `og cook --report`
lists each model's triangles (`lods`) and error (`lodErrors`) per level.
The renderer draws each `og.Model` entity at the coarsest level whose
error covers less than a pixel at the entity's distance, with
hysteresis, so nothing flickers at a boundary: a small prop keeps its
full mesh until it is far away, a large tree drops levels sooner. A
level whose error is too large for its purpose (a hard-surface model
melting at a distance) is cooked tighter with `lodError`, or the model
keeps more detail with `lodBias`. `og.Model.lodBias` (per entity) and
`RenderSettings.lodBias` (the frame) scale that choice: 2 keeps the full
mesh twice as far, and 0 always draws it. Check the choice with
`debug:lod`, which tints models green, yellow, orange and red by level,
with `dump_entity:<name>` (`render.lod`), and with the triangles in
`ProbeTimings.triangles` or `dump_render`.

**Reflection probes.** Smooth surfaces reflect the environment map, which
is the sky. Indoors that is wrong: a chrome sphere in a room would show
the sky. Put an `og.ReflectionProbe` in the room, and lit surfaces inside
its box reflect a cube captured from its position instead, box-projected
so the walls reflect where they are:

```ts
const probe = must(world.spawn({ name: "hall-probe", position: { x: 0, y: 1.6, z: 0 } }));
world.insert(probe, og.ReflectionProbe, {
  size: { x: 10, y: 4.2, z: 10 }, // the room's extents, centred on the entity
  resolution: 128,                // texels per cube face
  mode: "baked",                  // or "realtime", every `interval` ticks
  blendDistance: 0.5,             // fade into the environment at the box's faces
});
```

- Choose the probe's mode by what moves in the room:
  - A `baked` probe is captured once, on its first frame. For a static
    room, run `og bake probes [project] --commands "0:station:hall"` (the
    commands reach the scene). It writes the captures into
    `cooked/probes/`, and runs then load them instead of drawing them.
    A bake is keyed by the probe's place and size and by its room's
    content: moving the probe or changing what is in its box (a mesh,
    a material, a texture, a light) needs a new bake, and until then it
    draws its own capture. `og doctor --project` lists stale bakes
    (`stale-probe`).
  - A `realtime` probe redraws six faces every `interval` ticks. Keep
    the interval high.
- Size the box to the room. Each object takes the smallest box holding
  its centre, so two rooms that share a model each reflect their own.
  The capture shows everything in the room: characters in their pose
  (and each one takes the probe of the room its pose stands in), terrain,
  grass, water (refracting what lies below it and, on the full tier,
  reflecting the capture in screen space), 3D fluids, glass, particles
  (soft ones fading as in the frame), lines and world sprites and text,
  lit by the sun and every local light the camera's frame has (a light
  whose range the camera does not see lights nothing in the capture
  either). As in the frame, water and fluids show the opaque scene
  through them, not glass behind them. Terrain, grass and water inside
  the box are lit by it too. At most 8 probes (`MAX_REFLECTION_PROBES`) are used,
  nearest the camera first.
- `dump_render` lists the frame's probes and whether each was loaded from
  the cache. In a playtest, assert what the sphere shows with
  `expect:region:x,y,w,h:color:#rrggbb:tol`, and a model's level of
  detail and the frame's triangles with `expect:entity:<name>:render.lod==1`
  and `expect:timings:triangles<=1500000`.

**Screen-space reflections.** On the full tier, `PostEffects.ssr` adds the
frame's own reflections to smooth surfaces: a wet street, a polished
floor. They show what is on screen, including moving things:

```ts
const post = must(world.resource(og.PostEffects));
post.ssr = true;
post.ssrMaxDistance = 12; // metres a ray travels
post.ssrSteps = 48;       // more for long, thin reflections
post.ssrThickness = 0.3;  // how far behind a surface a ray still hits it
post.ssrFade = 0.15;      // fade at the screen's edges
```

SSR is weighted by roughness as the surface is shaded, its roughness map
and a terrain layer's roughness included (`roughness` 0.05 reflects
sharply; above 0.75 it reflects nothing), and by Fresnel. What it misses (off screen,
behind objects) falls back to the probes and the environment, so use SSR
together with a probe.

## 6. Atmosphere and water

`og.atmosphere` is data too, and off until set (`og api show atmosphere`):

- **Fog** — the `og.Fog` resource: `enabled`, distance fog counted from
  the camera (`density` per metre past `start` and, when `end > start`, a
  linear ramp to full fog at `end`: both apply together, and `density`
  defaults to 0.02, so setting only `end` still adds exponential fog; set
  `density: 0` for the ramp alone), height fog
  (`heightDensity` below `height`, thinning by `falloff`), `sunColor` for
  the glow looking into the sun, `sky` for how much of the sky it covers.
- **Sky** — `og.Sky { enabled, zenith, horizon, ground, sunDirection,
  sunColor, sunSize, glow, stars }` draws the sky per pixel and lights the
  scene by it; `moonSize` (degrees, 0 none), `moonPhase` (0 new, 0.25 first
  quarter, 0.5 full, 0.75 last quarter), `moonIntensity`, `moonColor` and
  `moonDirection` (zero: opposite the sun) add a moon disc over the stars.
- **Day and night** — `og.DayNight { enabled, start, speed, tilt, azimuth,
  moonSize, moonPhase }`: the clock is `start + seconds × speed` hours from
  the tick, so it replays; it drives the first `og.DirectionalLight` (the
  sun by day, a dim moon by night, where the moon disc hangs), the sky, the
  ambient light and the fog's tint. Read the clock with
  `og.atmosphere.timeOfDay(world)` (declare `og.DayNight` among the system's
  reads) to light lamps at dusk.
- **Weather** — `og.Weather { kind: WEATHER_RAIN | WEATHER_SNOW |
  WEATHER_DUST | WEATHER_MIST, intensity, wind, radius }`: particles around
  the camera, the preset's fog (an enabled `og.Fog` wins) and dimmed light.
  With an orthographic camera the particles are 2D sprites on `layer`.
- **Weather on surfaces** — name `og.WeatherState` (add it as a resource)
  and the weather changes what it falls on, simulated in the tick so it
  replays and gameplay can read it (`wetness`, `puddles`, `snowDepth`,
  `dust`): rain soaks porous ground darker and glosses sealed surfaces
  (paint, metal, glass), runs down walls in streaks and fills puddles in
  the low parts that ripple and splash; when it stops the ground dries in
  patches from the high spots (`dryTime`, `drainTime`). Snow settles on
  what faces up, reaching steeper faces as it deepens (`snowfall`,
  `snowCover`), and melts (`melt`). A `WEATHER_DUST` storm leaves dust and
  sand on what faces up (`dustTime`, `dustColor`) until rain washes it off
  (`washTime`). Roofs and canopies keep what is under them dry
  (`shelterSize`). Meshes, terrain and grass all take it; a material's
  relief comes from its `aoMap` and a terrain's from its layers' height
  maps (water, snow and dust fill the dark crevices and the mortar
  joints first). Lying snow lifts the drawn terrain by `snowDepth`, at
  most 10 cm (drawn only: what stands on the ground does not rise with
  it), buries the grass's roots and bows its blades. Set `weathering: 0` on a material the weather must not touch
  (an interior seen through a door, a screen, a lamp's glass), or between
  0 and 1 for one that sheds some. For a scene that starts wet, write the
  state's fields once at setup rather than waiting for rain.
- **Water** — `og.Water` on an entity draws a surface on its xz plane
  (`size`, `shallow`/`deep` colours over `depthFade`, `foamWidth` where it
  meets geometry, `waveStrength`, `reflectivity`); with an
  `og.WaterSurface` on the same entity its waves move the surface and bodies
  splash it (`og.Splash` events) and float. In 2D, `og.WaterLine` with
  `og.Water2D` is a side-on pool drawn in the sprite order. Three more
  cost frame time and are off until set: `planarReflection` (0.25 to 1, the
  share of the screen's resolution) mirrors the scene in a still lake or a
  pool you look across, where the sky reflection alone looks fake;
  `shadows` (0 to 1) darkens it under the cascaded shadow (a pier, a cliff);
  `caustics` throws rippling light on what lies below it. With the camera
  below the surface the frame's fog becomes `underwaterColor` at
  `underwaterDensity` and the surface from below shows the sky only
  straight up (on by default; set the density to 0 to see through).

Which look for which mood is og-taste §8.

## 7. Terrain

**Terrain** (`og api show terrain`, 3D projects only: a `2d` project's
publish build has none, og-game §2) is a height field cooked from a
`type terrain` .ogt beside a 16-bit grey PNG (or an EXR in metres), or
generated by the cook (below):

```text
type terrain
version 1
---
heightmap "island.png"     # 16-bit grey: black and white are `height`'s ends
height -20 120             # metres
size 2048 2048             # metres along x and z, centred on the entity
control "island-control.png"  # optional RGBA: layer 0 in R, 1 in G, 2 in B, 3 in A
heightBlend 0.15           # sharpen the blends by each albedo's alpha (its height)
layer {                    # a block's '}' stands alone on its line
  albedo "textures/grass.png"
  normal "textures/grass-n.png"
  tile 6
  slope 0 30
  antiTiling 1             # hex-tiling: no visible 6 m repeat across the meadow
  macroVariation 0.5       # brightness and hue drift, a coarser repeat far off
}
layer {
  albedo "textures/rock.png"
  tile 12
  projectSlope 45          # past 45°: the two projections facing the face (no streaks)
  slope 28 90
}
layer {
  albedo "textures/pebbles.png"  # its alpha is the pebbles' height
  normal "textures/pebbles-n.png"  # its alpha names each pebble (any value per pebble)
  tile 2
  displacement 0.08        # metres the drawn ground rises and falls by that height, near the camera
  cellVariation 0.8        # each pebble its own brightness and warmth
}
layer {
  albedo "textures/snow.png"
  altitude 90 1000
  altitudeFade 10
}
```

A layer shows where its control channel (1 without a map) times its slope
band (degrees) times its altitude band (metres above the terrain's entity)
is high. Give cliff layers a `projectSlope` (planar below it, biplanar
past it: the two projections facing the face, so an 85° wall shows its
texture square instead of in vertical streaks; `triplanar true` samples
three projections everywhere instead). `antiTiling` (0 to 1) breaks a
layer's repeat by hex-tiling, `macroVariation` (0 to 1) keeps a large field
of one layer from reading as one texture, and `heightContrast` (0 to 1)
sharpens the `heightBlend` rim at that layer's edges. `cellVariation`
(0 to 1) gives each cell of a laid pattern (setts, flags, tiles, pebbles)
its own tone: write a value per cell into the normal map's alpha (the
same value over the whole cell, its joints included), and each cell's
brightness and warmth move by it, so one texture reads as many stones;
an opaque normal map changes nothing. `displacement`
(metres, 0 to 4) moves the drawn ground up and down by the layer's
height (its albedo's alpha, about its middle) within og.Terrain's
`displacementDistance` of the camera (40 m by default) where the ground
is not steep, so pebbles and cobbles stand out at the silhouette and
cast their own shadows. Near the camera the clipmap then draws finer
than the samples (about 32 vertices across the layer's `tile`), so the
relief shows within a few metres and smooths out with distance; a larger
og.Terrain `lodRing` carries it further. It costs vertices, mostly in the
depth prepass: on an M4 Pro, island-near with every layer displaced
8 cm draws 83,210 vertices instead of 49,926, and its frame takes
6.3 ms instead of 4.6 ms. The prepass goes from 0.9 to 2.8 ms, and the
main pass is unchanged. So displace only the layers that need it, and
lower `displacementDistance` before raising `lodRing`. Collision, `og.terrain.sample` and picking keep the
undisplaced field, so keep it to centimetres where bodies walk.
In the game, load it in `start` and place it:

```ts
const info = must(og.terrain.load(world, "terrain/island.ogt"));
const ground = must(world.spawn({ name: "island", position: { x: 0, y: 0, z: 0 } }));
world.insert(ground, og.Terrain, { data: info.data });
world.insert(ground, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry });
```

It is drawn around the camera at finer detail nearby (casting and
receiving shadows, under the fog), placed by its entity's world transform
(turn or scale the entity and the terrain turns and scales with it), and
`og.CharacterController` walks on it up to `maxSlope`.
`og.terrain.sample(world, { data, origin, quaternion, scale, x, z })`
(the entity's position, rotation and scale; the last two may be left out
for an unturned, unscaled terrain) gives the height, normal and slope
anywhere (to place trees, spawn on the ground, or check a slope); `og.terrain.edit` raises, lowers, flattens,
smooths or paints under a round brush (craters, building pads, a paint
tool) and `og.terrain.layer` / `setLayer` retune a layer live. Edits are
world state: a snapshot keeps them and restoring it brings the ground
back as it was then. Grass, flowers and stones are an `og.TerrainScatter` on the
terrain's entity: `{ data, geometry, material, layer, density, distance }`
grows copies of the mesh where that layer shows, within `distance` of the
camera (thinning out over its outer `fade`, never shrinking), each fixed to its spot
as the camera moves. Keep `density × distance²` modest: 4 a square metre
within 60 m is about 36,000 copies. The resource `og.Wind` sways them:
`strength` (0 still; 0.2 leans a 1 m stem's tip 20 cm downwind),
`direction`, `speed` (how fast gusts cross the field) and `gust` (0 a
steady lean, 1 from upright to twice it); set `sway: 0` on a stone
scatter, and model a copy with its root at the mesh's origin, since each
vertex moves by its height above it. It is drawn from the tick, so it
needs no system and costs nothing in the simulation.

A meadow is one `og.Grass` on the terrain's entity: `world.insert(e,
og.Grass, { data })` grows blades over layer 0 to 150 m, generated on the
GPU each frame (no mesh to make, nothing per blade stored), thinning with
distance and handing over to the ground's own grass colour past them, so
the field reaches the horizon. Pick the layer with `layer`, the look with
`height`, `width`, `curve`, `clumpSize`, `baseColor`/`tipColor`/`dryColor`
and `dryness`, the light with `translucency` (sun through blades lit from
behind) and `sheen`, the wind's hold with `sway` and `stiffness`;
`castShadows` adds the blades' shadows near the camera. Lower `density`
or `distance` to save frame time; prefer it to a hand-made grass
og.TerrainScatter, which suits flowers, stones and bushes. A navmesh .ogt's `terrain "terrain/island.ogt" { step 4 }` bakes
the walkable slopes for og-ai. The clipmap keeps a finer level out as
far as the next coarser one's height error would show more than
og.Terrain's `lodError` pixels (default 1), so far ridges keep their
peaks; the levels then grow to 128 quads (6.0 ms instead of 4.8 for the
island seen from 150 m on an M4 Pro). Raise `lodError` or lower `lodRing`
on og.Terrain to trade detail for frame time.
A cliff, arch or overhang the height field cannot make (a sheer 300 m drop
on a half-metre grid is a sawtooth wall) is a mesh stamped into the
terrain. Draw it with `og.TerrainStamp { data, geometry, blend }` on its
own entity: it is shaded by the terrain's layers by its own normal (rock
on its walls where the rock layer's slope rule says so, grass on its top)
and its lowest `blend` metres take the ground's look, so it meets the
terrain with no seam. Then, once, carve the ground under it in the tick:
`og.terrain.stamp(world, { data, geometry, position, rotation, size,
mode: "carve" })` (the stamp entity's world transform; `origin`,
`quaternion` and `scale` for the terrain's, as for `sample`) lowers the
height field to the mesh's underside, or `mode: "hole"` cuts the cells the
mesh covers (a cave mouth). The carve is world state like an edit, and the
id it answers in `stamp` takes it back with `og.terrain.unstamp(world,
data, id)`. Give the stamp entity an `og.Collider` naming the same
geometry to stand on it.
```ts
const rock = world.spawn({ name: "cliff", position: { x: 0, y: 151, z: 0 } });
world.insert(rock, og.TerrainStamp, { data: info.data, geometry: cliffMesh, blend: 1 });
og.terrain.stamp(world, { data: info.data, geometry: cliffMesh, position: { x: 0, y: 151, z: 0 }, mode: "carve" });
```
Its tiles stream: only the tiles the camera needs are on the GPU (16 MiB a
terrain by default), decoded off the frame, the coarser level standing in
until each arrives, and nothing a tick reads changes with it. In a probe
script `terrain_stream:resident` decodes every wanted tile each frame (for
captures to compare), `terrain_stream:stream:<MiB>` streams within that
budget, and `dump_stream` gives the terrain's tiles.

## 8. A generated terrain

**A generated terrain.** Instead of `heightmap`, a `generate` block makes
the field in the cook (never both: the cook refuses the pair and names
the lines). `height` and `size` mean what they mean for a heightmap: the
field is stretched so its lowest sample is `height`'s low and its highest
its high, and no sample leaves that range. Every line, with its default
(leave out what you do not need; erosion, thermal, terraces, rivers and
the sea are off until given):

```text
type terrain
version 1
---
height -40 160             # metres
size 1024 1024             # metres along x and z
generate {
  samples 257 257          # columns and rows (2 to 4097)
  seed 1                   # another seed, another terrain of the same kind
  noise {
    octaves 6              # 1 to 12: more is finer detail
    frequency 3            # features across the longer side
    lacunarity 2           # each octave's frequency over the last's (1 to 8)
    gain 0.5               # each octave's amplitude over the last's (0 to 1): higher is rougher
    warp 0                 # domain warp in features: 0.3 to 1 bends ridges and coasts
  }
  mask island 0.4          # land fades to `height`'s low over this share of the half size
                           # or `mask ridge 1`: sharp ridged mountains (0 to 1); default no mask
  erosion {                # hydraulic erosion: seeded drops carve gullies and fill valleys
    drops 40000            # ~0.5 to 1 per sample; more is deeper erosion and a slower cook
    lifetime 30
    radius 3               # samples each drop erodes around it
    inertia 0.05
    capacity 4
    deposition 0.3
    erosion 0.3
    evaporation 0.01
    gravity 4
  }
  thermal {                # slopes steeper than talus slump into scree
    talus 35               # degrees
    iterations 20
    rate 0.5
  }
  terraces {
    step 10                # metres between steps
    blend 0.4              # 0 hard steps, 1 smooth ramps
  }
  sea 0                    # metres above the entity: og.terrain.load's seaLevel
  shore 2                  # metres either side of the sea flattened into beaches (needs sea)
  rivers {                 # carved where the flow gathers, down to the sea or the edge
    threshold 400          # samples draining through before it is a river: lower, more rivers
    depth 2                # metres
    width 6                # metres
  }
  moisture {               # what biomes read, 0 to 1
    flow 1                 # weight of the gathered flow (wet in valleys and by rivers)
    scale 64               # samples draining through for half of it
    noise 0.5              # weight of a noise
    frequency 6
  }
  biome {                  # any number; each writes its layer's control channel
    layer 0                # required: the layer (0 to 3, one the terrain has)
    weight 1
    altitude -1000000 1000000   # metres above the entity
    altitudeFade 2
    slope 0 90             # degrees
    slopeFade 4
    moisture 0 1
    moistureFade 0.1
  }
}
layer {
  albedo "textures/sand.png"
  tile 4
}
layer {
  albedo "textures/grass.png"
  tile 6
}
layer {
  albedo "textures/rock.png"
  tile 12
  triplanar true
}
```

An island with beaches, rivers and rock on its cliffs, for that file's
three layers (a block's `{` ends its line):

```text
generate {
  seed 7
  noise {
    warp 0.5
  }
  mask island 0.45
  erosion {
    drops 40000
  }
  sea 6
  shore 3
  rivers {
    threshold 300
    depth 2.5
    width 8
  }
  biome {
    layer 0
    altitude -1000 9
  }
  biome {
    layer 1
    altitude 9 1000
    slope 0 32
  }
  biome {
    layer 2
    slope 32 90
  }
}
```

With `biome` rules the generator writes the control map (a terrain with
biomes names no `control`); a channel is the most any of its layer's
biomes gives a sample, its `weight` times its three bands. Without biomes
there is no control map and the layers' own `slope` and `altitude`
place them, as for a heightmap. The same file makes the same terrain on
every machine (the generator uses only exact arithmetic and its own
random numbers). It is a credible terrain of a kind, never a terrain from
a picture: move the seed, the mask and the erosion until it reads right.
`og cook --report` gives its height histogram, the share of land above
the sea and the rivers' length, and writes its height and control maps
as PNGs to `og-out/terrain-maps/` to look at.

Its sea is the water's level; the water is the terrain's size:

```ts
if (info.seaLevel !== null) {
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: info.seaLevel, z: 0 } }));
  world.insert(sea, og.Water, { size: info.size, segments: 128 });
}
```

(on the terrain's entity's x and z, under its parent if it has one; an
`og.WaterSurface` on the same entity makes the waves move and bodies float).

## 9. Material shaders

A material can run your own WGSL in four hooks of the lit shader. Write the
hooks in a `.wgsl` file and declare it in an `.ogt` of `type shader`:

```text
type shader
version 1
---
source "shaders/dissolve.wgsl"
param amount float 0
param edgeColor color 4 1.6 0.4
```

```wgsl
// Any of the four; the others stay the identity.
fn vertex_offset(h: Hook, p: Params) -> vec3<f32>                  // world-space offset
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32>    // base colour and alpha
fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32>   // world normal, unit length
fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> // emitted light
// h: world, local (object-space) position, normal, uv, color, time (s).
// p: the declared parameters by name (p.amount). value_noise3(p, seed) is
// the engine's 3D value noise (0..1).
```

The cook compiles every shader for every back end (WebGPU, Metal, Vulkan,
DX12) and refuses one that does not compile with naga's
message, its lines counted in your hooks. In the game:
`og.render.shaderMaterial(world, must(og.load.shader(world,
"shaders/dissolve.ogt")), { color, alphaTest: 0.5 })` makes the material,
`og.render.setShaderParam(world, material, "amount", [0.4])` sets a
parameter, and `og.render.setMaterialShader(world, material, shader)` gives
an existing material (a loaded model's, from `GltfInfo.materials`) the
hooks. Alpha below `alphaTest` discards the fragment (a dissolve). Skinned
meshes run the hooks too, and shadows and the depth prepass see the mesh
where `vertex_offset` moved it. `templates/fx-showcase/assets/shaders/` has a
dissolve, wind-swayed grass and a ripple on the walking walker.

## 10. Full custom shaders and custom post passes

**Full custom shaders.** `mode full lit` (or `mode full unlit`) in a `type
shader` asset replaces the lit shader's stages with your own entries:
```wgsl
fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput   // v: position normal uv color tangent (object space; skin and morphs applied)
fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput                  // f: world local normal uv color data view pixel time
// VertexOutput: world (projected by the engine), normal, uv, color, data (free, interpolated)
// SurfaceOutput: color (albedo + alpha; unlit: the colour), normal, metallic, roughness, emissive
// camera(): view_proj, position, forward, time. tex0(uv)..tex7(uv): the material's map, metalnessMap,
// roughnessMap, normalMap, emissiveMap, aoMap, gradientMap, dissolveMap. Helpers: standard_vertex(v, model),
// standard_surface(f), noise3(p).
```
An entry you leave out is the standard one.
- `lit` surfaces are lit by the sun's cascades, the point, spot and rect
  lights with their shadows and cookies (a rect light as an area, as on a
  standard material), the environment and the fog. `unlit` surfaces draw
  as returned.
- The depth prepass, every shadow pass and every view run the same
  entries, and alpha under the material's `alphaTest` is cut in each of
  them. A dissolve's shadow therefore has its holes.
- Make the material with `og.render.shaderMaterial` and set parameters with
  `og.render.setShaderParam`, as for hooks (§9).
- `templates/fx-showcase/assets/shaders/dissolve-full.*` is the example.

**Custom post passes.** A `type post` asset declares
`fn post(f: PostInput, p: Params) -> vec4<f32>` in its `code` or `source`,
with up to eight `param` lines. The function reads:
- `previous(uv)`: the pass before it, linear HDR;
- `scene_color(uv)`: the lit image;
- `depth(uv)`: view depth in metres;
- `normal(uv)`: the world normal;
- `f.uv`, `f.pixel`, `f.size` and `f.time`.

It returns linear HDR. To use one:
1. Load it with `og.load.post(world, "post/crt.ogt")`.
2. Put the handles in the list `PostEffects.custom`, at most
   `RenderCapabilities.maxCustomPosts` (8) of them. They run in its order,
   each over the one before it, after depth of field and motion blur and
   before exposure, bloom and tone mapping. Write the list whole:
   `post.custom = [grade, crt]`, `post.custom = [...post.custom, h]`,
   `post.custom = []`. Reading it answers a frozen copy, so
   `post.custom.push(h)` throws instead of changing nothing; nine passes
   are refused, never cut to eight. A look gives it on one line:
   `custom "post/tint.ogt" "post/crt.ogt"`.
3. Set its parameters with `og.render.setPostParam(world, post, "curve",
   [0.1])`.

The cook compiles it for every back end. A pass that a device still fails
to compile is skipped and listed by `og.render.postErrors(world)`.
`templates/fx-showcase/assets/post/crt.*` is a CRT example.

## 11. Render targets, second cameras, mirrors and layers

**Render to a texture.** `og.render.renderTarget(world, { width, height,
format, depth })` makes a target, as three.js `WebGLRenderTarget`. It
answers `{ target, texture, depthTexture, width, height }`. `texture` is
an ordinary Texture: a material's `map` or `emissiveMap`, an og.Sprite, an
`og.render.sprite` or an og.ui image shows it. `format` is `ldr` (the
default) or `hdr`:
- `ldr`: 8-bit and finished (exposed with the view's `exposure`, tone
  mapped). Use it for a monitor, a minimap or a portrait, shown through an
  unlit material's `map`.
- `hdr`: linear radiance, exposed once with the frame that shows it. Use
  it for a mirror or a portal, shown as `emissiveMap` on a material whose
  `color` is black.

`depth: true` also keeps the view's linear depth, from near (0) to far
(1), of its opaque meshes, on every backend. It costs a
second draw of the view's opaque meshes, so ask for it only when a
material reads it. (A scene with soft particles, `softDistance` above 0,
pays that draw in every view anyway: its particles fade against it.)
`RenderCapabilities.maxRenderTargetSize` is the largest side.

**A second camera.** `og.CameraView` on an entity is a camera with
og.Camera's fields (`fov`, `near`, `far`, `zoom`, `orthographic`,
`height`, `exposure`). It looks down the entity's local -Z:
```ts
const screen = must(og.render.renderTarget(world, { width: 480, height: 270 }));
world.insert(monitorMesh, og.Mesh, { geometry: plane, material: must(og.render.standardMaterial(world, { map: screen.texture, unlit: true })) });
const cam = must(world.spawn({ name: "cctv", position: eye, quaternion: og.transform.lookAt(eye, target) }));
world.insert(cam, og.CameraView, { target: screen.target, fov: 60, every: 2 });
// Split-screen: no target, a rect of the frame (fractions from the top left).
world.insert(p2, og.CameraView, { viewportX: 0.5, viewportWidth: 0.5 });
```
- `every: n` draws it on every n-th tick, and the target keeps its last
  image in between.
- `order` sorts the views. Every view draws before the frame, so a
  material shows this frame's image.
- `clear` with `clearColor` replaces the sky or background colour.
- `active: false` stops it.

A view draws what the main camera draws, from its own eye and on its own
`layers`: meshes and models (skinned meshes and custom shaders included),
opaque then transparent back to front, the terrain and the sky, particle
systems, trails, og.Lines and og.Points, and world sprites and text, lit and
shadowed by the frame's lights. Each view is a second draw of everything
its layers admit, so keep it cheap with `layers` and `every`.
- An `ldr` target and a viewport are exposed with the view's `exposure`
  and tone mapped with the frame's curve. An `hdr` target holds linear
  radiance and is exposed by the frame that shows it.
- Bloom, AO, SSR, depth of field, colour grading and custom post passes
  run on the main frame only (the 2D glow, `PostEffects.glow`, is the
  exception: a view adds its own).
- In a view, sprites are 2D-lit and glow and 2D fluids and soft
  particles draw as in the main frame. Screen sprites, the UI and debug
  draw stay on the main frame.

A viewport view draws over the finished image and under the UI.

**A mirror.** Put `og.Mirror` on the glass's entity. It reflects the main
camera in the entity's xy plane (its +Z faces the room) and fits the image
to a `width` × `height` rect:
```ts
const rt = must(og.render.renderTarget(world, { width: 512, height: 384, format: "hdr" }));
world.insert(glass, og.Mesh, { geometry: must(og.render.planeGeometry(world, { width: 4, height: 2.4 })),
  material: mat({ color: { r: 0, g: 0, b: 0 }, emissive: { r: 1, g: 1, b: 1 }, emissiveMap: rt.texture }) });
world.insert(glass, og.Mirror, { target: rt.target, width: 4, height: 2.4 });
```

**Layers.** `layers` on og.Mesh, og.Model, og.Sprite, og.ParticleSystem,
og.Lines, og.Points and og.Camera (and on og.CameraView and og.Mirror) is a
three.js bit mask:
bit n is layer n, and the default 1 is layer 0. A camera draws an object
when `object.layers & camera.layers` is not 0. `og.render.raycast`'s
`layers` uses the same test. Some uses:
- A first-person body on layer 1 (mask 2) is hidden from the player's
  camera (mask 1) and shown on a third-person monitor (mask 3).
- A minimap marker only the minimap camera draws.

Shadow maps take every caster whatever its layers, so a hidden object
still casts a shadow. Give it `castShadow: false` if it should not.

The probe's `describe` lists what the main camera shows, and each
og.CameraView and og.Mirror under `views` with what its own image shows,
on its layers. An entity on a layer only a view sees is found under that
view, not in `entities`. A main-camera entity names the views that also
show it in its `views`. `pick` and `expect:entity:...:within` read the
main camera.

## 12. Geometry made at run time, lines, points, picking and debug draw

Meshes need not come from a file. `og.render.bufferGeometry(world, desc)`
takes three.js BufferGeometry's attributes: `positions` (a Float32Array of
x y z), and optionally `normals`, `uvs` (v down the image), `colors`
(r g b or r g b a, multiplied into the material), `tangents` and `indices`
(a Uint32Array, counter-clockwise from the front). Missing normals are
computed as `computeVertexNormals` does. The geometry draws as any other
through `og.Mesh`, and it is a `Collider.mesh` at once (`physics` is true by
default), so a procedural level collides with what is drawn:

```ts
const terrain = must(og.render.bufferGeometry(world, { positions, colors, indices, dynamic: true }));
world.insert(e, og.Mesh, { geometry: terrain, material });
world.insert(e, og.RigidBody, { kind: BODY_STATIC });
world.insert(e, og.Collider, { shape: 5 /* SHAPE_MESH */, mesh: terrain });
// Each tick (water, a deforming surface): every attribute again.
must(og.render.updateGeometry(world, terrain, { positions, colors, indices }));
```

Only a geometry made with `dynamic: true` can be updated. The same vertex
and index counts rewrite it in place, and other counts reallocate it. Its
physics mesh is re-registered, and its colliders are rebuilt at the next
step. `og.render.geometryInfo` answers the counts and bounds of any
geometry.

Beside `boxGeometry`, `planeGeometry` and `sphereGeometry`, three.js's
primitives keep their parameters and defaults: `cylinderGeometry`,
`coneGeometry`, `capsuleGeometry` (an og.Collider capsule of the same
radius and `height` matches it), `torusGeometry`, `ringGeometry`,
`circleGeometry` and `tubeGeometry`. A tube follows a centripetal
Catmull-Rom curve through `points`, for pipes, rails and cables; give it
`path` instead (an og.path from `og.path.create`, or a scene's `path`
block found by `og.path.find`) and it follows that path's own curve,
Bezier or linear included, closed when the path is. Its rings are evenly
spaced along the length, as three.js spaces them. Rings and circles are one-sided, so use
`side: "double"` to see them from behind. `wireframe: true` on a
standard material draws the triangle edges instead of the faces, one pixel
wide on every tier.

**Lines and points.** `og.render.lines(world, desc)` and
`og.render.points(world, desc)` are immediate mode, like
`og.render.sprite`: submit them every tick they should show.
- Lines: `mode` is `strip`, `segments` or `loop`. `width` is in CSS
  pixels, or in world units with `worldUnits`. They also take `colors` per
  point, `depthTest` (false draws over the scene), `dashed` with `dashSize`
  and `gapSize`, and `screen: true`. Screen lines use CSS pixels from the
  top left of the play surface and draw over the finished frame, under
  the UI.
- Points: `sizeAttenuation` makes `size` a size in world units; without it
  `size` is a fixed size in CSS pixels.
- A static set is data: the `og.Lines` and `og.Points` components draw a
  geometry's vertices in order at the entity's transform. Make the
  geometry with `bufferGeometry({ positions, colors, physics: false })`,
  unindexed, any count.

**What is under the cursor.** `og.render.raycast(world, ray, desc)` tests
what is drawn (every og.Mesh and og.Model at its world transform), not the
colliders. It goes through a BVH per geometry and answers every hit,
nearest first; `first: true` answers only the nearest. Each hit has
`entity`, `geometry`, `distance`, `point`, `normal` (the face's, in world
space), `uv` and `faceIndex`. Inside a system it reads the meshes as the
tick started, as `og.physics.raycast` does. Use it for picking, decals on
detailed meshes, and line of sight against geometry without colliders.

A model's BVH is cooked: `og cook` builds each primitive's into the
model's `.ogb` (about 6 to 9 bytes per triangle compressed, listed as
`bvhBytes` in `og cook --report`), and it loads with the mesh, so the
first raycast never stalls. A model no ray is cast at can leave it out
with import settings; its first raycast then builds it:

```
type import
version 1
---
model {
  raycast false
}
```

Leave it on for anything a player can click, aim at or stand a decal on.
Turn it off for a large model that is only scenery (templates/game's
16,512-triangle dropship gains 108 KB, 64 % of its blob). Geometries made
at run time (`bufferGeometry`, the primitives) build theirs when made or
updated.

A model whose geometries are triangle-mesh colliders (a level, a cell's
ground) can have the collider shapes baked too, with `collider true` in the
same `model { }` block: the cook bakes each primitive's (the proxy's for a
streamed model) and it is decoded as the model loads, so the physics step
that first meets an `og.Collider { shape: SHAPE_TRIMESH, mesh }` at scale 1
builds nothing (a 131,072-triangle mesh: 49 ms in that step, 9 ms baked,
plus 18 ms decoding at load). It costs about 90 bytes a triangle, so it is
off by default; a scaled collider still builds its own shape.

A model seen far away (trees, rocks, buildings on the horizon) can be
cooked with an octahedral impostor: `impostor true` in its `model { }`
block (with `impostorGrid 8` frames per side and `impostorSize 128` pixels
per frame, the defaults) bakes two atlases, `<model>#impostor` and
`<model>#impostor-normal`, and the model draws as one camera-facing card
once an atlas texel covers less than a pixel: the last level after its
meshes', picked as they are (lodBias moves it; dump_entity shows it).
A posed (animated) model keeps its meshes. Give an og.TerrainScatter the
same model as `impostor` and an `impostorDistance` below its `distance`,
and its copies past that distance draw as the card.

Trees and bushes placed as models sway in og.Wind with `og.Model.sway`
(0 still, the default; 1 as grass sways): each vertex above the model's
origin moves downwind by its height, so model it with its root at the
origin. It is drawing only, the same on every run for the same tick. A
material with its own shader asset keeps it and stays still, and a
model at its impostor's level draws a still card.

**Debug draw.** `og.debug.line`, `box`, `sphere`, `arrow`, `axes` and
`text` (a label above a world point, in the UI's body font; every field is
in `og api show debug`) each take
`{ color, width, duration, rotation }`. `duration` is in ticks; 0 shows the
shape this frame only, so draw it every tick. Debug draws show over the
finished frame and never hide behind geometry, so they read in every
capture. The command channel switches the views:
- `debug:physics` shows the colliders, joints and contacts.
- `debug:nav` shows the grids' blocked cells, the navmesh polygons and
  every agent's path.
- `debug:bounds` shows every drawn mesh's box.
- `debug:none` shows only the draws; `debug:off` and `debug:on` hide and
  show everything.

`og.debug.setView` does the same from code, and `RenderOptions.debugView`
picks a view from the first frame. A publish build drops all of it, so
debug draws may stay in the code. Debug draws are not world state:
snapshots and replays do not carry them.

## 13. Video: footage in

**Footage in.** An `.ogt` of `type video` names an `.mp4`, `.webm` or
`.mov`; `og cook` turns it with ffmpeg (`og doctor` says whether ffmpeg
and ffprobe are there) into a texture array of its frames and its sound:

```text
type video
version 1
---
source "clips/intro.mp4"   # beside this file, or under assets/
fps 30                     # frames kept per second (default 30)
maxSeconds 8               # what is kept from the start (default 8)
size 320 180               # each frame's size (default the source's, at most 1024 wide)
kind color                 # or pixel: nearest, lossless
audio true                 # its sound track (default true)
```

At most 256 frames (fps × seconds): the cook refuses more, naming the
`maxSeconds` that fits, and warns past 64 MiB of cooked frames. Keep
footage small: a 320×180 clip of 8 s at 30 fps is about 2 MiB cooked.
Load it in `start`, then play it with `og.VideoPlayer`; its frame is a
plain `Texture`, so a sprite, a material map or a UI image shows it:

```ts
const info = must(og.load.video(world, "clips/intro.ogt"));
const screen = must(world.spawn({ name: "screen", position: { x: 0, y: 0, z: 0 } }));
must(world.insert(screen, og.Sprite, { texture: info.texture, width: 16, height: 9 }));
must(world.insert(screen, og.VideoPlayer, { video: info.video, loop: true }));
must(world.insert(screen, og.AudioSource, {})); // its sound, kept in step
```

The `og.video` system plays it by the tick (not the time scale, so the
picture stays with its sound): with a 30 Hz tick, tick 90 shows the
frame 3 s in, the same in every run and after a restore. Write
`VideoPlayer.time` to seek (the sound starts again there on the next
tick), `playing` to pause, `speed` for slow motion (the pitch follows).
`frame` and `finished` are written for you, and so is `texture`: each
player of a video has a texture of its own (the first takes
`info.texture`, each further one a texture made for it over the same
frames), so two players at different times show different frames. With
more than one player, draw each player's `VideoPlayer.texture`. Every field and the loaders are in `og api show video`.

**Long footage: streams.** A clip is for short loops. For anything
longer (a cutscene, a trailer, a TV in the level) add `stream true`: the
cook encodes it to AV1 (ffmpeg's libsvtav1 or libaom-av1) and it is
decoded as it plays, with no frame limit, the sound in Opus:

```text
type video
version 1
---
source "clips/intro.mp4"
stream true                # decoded as it plays
size 1280 720              # default the source's, at most 1920 wide; even, at least 64x64
keyframe 1                 # seconds between key frames: what a seek decodes at most (default 1)
crf 32                     # quality, 0 best to 63 (default 32)
crf720 38                  # the 720p rendition's (default crf + 6)
```

The API is the same: `og.load.video`, `og.VideoPlayer`, its texture on a
sprite, material or UI image. `info.streamed` is true and `info.poster`
is the first frame as an ordinary texture, ready at once (a menu's
thumbnail, the picture while the stream loads on the web, where it is
fetched by `og.load.video` and not with the startup bundle).
`og.video.capabilities(world)` names the decoder (`rav1d` natively,
`webcodecs` or `rav1d-wasm` in a browser) and the largest size it plays
in real time. `rav1d-wasm` (Safari without AV1 hardware, for one) holds
720p, so the cook also makes a 720p rendition of a larger stream and that
browser plays it instead: author footage at any size. The player's time is the tick's, as for a clip; decoding
never changes the world, and `og run`, `og render` and `og test` wait for
each frame, so captures are exact.

Put an `og.AudioSource` on the player's entity for the footage's sound:
og.video keeps it on the picture through seeks, `playing = false` pauses
(both hold, and resume together), `loop` wraps both at the picture's end,
and `speed` pitches it. Footage ignores the time scale and hit stop, as
its sound does; stop the player's `playing` to freeze it with a pause
menu. A TV wall of one stream is one `og.load.video` and a player per
screen, each showing its own `VideoPlayer.texture`; every player decodes
on its own, so a player off screen should stop (`playing = false`) and
simultaneous streams stay few.
