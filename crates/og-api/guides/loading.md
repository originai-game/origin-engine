# Loading assets

Loading what `og cook` made, and what a player or a generator makes at run time. This guide covers the import settings and asset files that each loader reads, what the cook does to them, streaming and residency, uncooked loads, and composed effects.

The API: `og api show load` (one function per asset kind), `og api show vfx`.
Asset queries, publish pruning and the cook cache are in assets.

## 1. The bundle

- The game's bundle (`cooked`) is open before the first tick, and
  GameDesc.scene is spawned then. Loads from an open bundle complete at
  once on both hosts, because the web host fetches the bundle's files
  before the engine starts.
- Only og.load.bundle of another bundle can be pending. It completes at
  a tick boundary.
- Every cooked loader answers the same handle for the same path. Load
  once in GameDesc.start and keep the handle.
- An asset in a streaming group's pack loads once its group is
  resident, and the refusal before then names the group. Texture
  arrays, flipbook sheets and environment maps come back under the same
  handle when their group returns.

## 2. Models

- A glTF's meshes, materials and textures join the renderer once. Every
  instance shares the materials, so tint, flash and variant are the
  per-instance controls.
- GltfDesc.castShadow and receiveShadow belong to the asset. Another
  og.load.gltf of the same path answers the same handle and sets them
  again, for the models instantiated after it. Model.castShadow turns
  shadows off for one instance. A scene entity with its own `Model`
  table takes the table's flags, which default to on.
- glTF punctual lights (KHR_lights_punctual) spawn as child entities of
  the model, at their nodes. glTF's candela becomes the radiance at one
  metre and its range the light's distance. A light of infinite range
  ends where its radiance falls to 0.01. A spot's outer cone becomes its
  angle, and 1 - inner / outer its penumbra.
- Material variants (KHR_materials_variants) are picked per instance, for
  example a car's paint or a chair's fabric. Each material is still one
  draw, as before.

### MagicaVoxel

A `.vox` cooks to a glTF: greedy-meshed and flat-shaded, with its
palette as vertex colours. Z-up is turned to y-up, and each model sits
about its centre (MagicaVoxel's pivot, with or without a scene graph).
Size and place it in its import settings (`models/hut.vox.ogt`):

```text
type import
version 1
---
model {
  voxScale 0.1
  voxOrigin bottom
  voxFps 8
  voxClip walk 0 3
}
```

- `voxScale` is metres per voxel (default 1). `voxOrigin bottom` puts
  the lowest voxel on y = 0; the default `center` is the editor's origin.
- A `.vox` with frames cooks to clips that swap the frames' models,
  played by og.AnimationPlayer as any skinned clip. `voxFps` defaults to
  10. Without `voxClip` there is one clip, `frames`, over the file's
  playback range, else over every frame. A node's `_loop` repeats its
  keys as the editor plays them.

### Level of detail

The cook simplifies every mesh into levels. The default is
`model { lods 1 0.5 0.25 }` in the import settings, each number a
level's triangle fraction; a glTF with MSFT_lod keeps its own levels.
The cook measures each level's error against the full mesh. The
renderer draws the coarsest level whose error covers less than a pixel
at the model's distance, with hysteresis so it does not flicker.
dump_entity shows the level drawn.

| Model.lodBias | Use |
|---|---|
| above 1 | a hero: detail kept farther |
| 1 | the default |
| below 1 | a crowd: detail dropped sooner |
| 0 | always the full mesh |

### Sway and baked lighting

- Model.sway moves each vertex above the model's origin downwind by its
  height, in the model's units, in gusts sweeping across the scene. Use
  1 for trees and bushes, less for stiffer trunks, 0 for buildings and
  rocks. The shadow and depth move with it, and a posed model sways
  too. A material with its own shader asset keeps its shader and stays
  still.
- Model.static marks a model for `og bake lightmaps`. The bake gives it
  lightmap UVs (xatlas, over its world-space triangles) and traces the
  light arriving at its surfaces into `<scene>.lightmap.ogt` and
  `.ktx2` beside the scene. A baked model takes its ambient light from
  the lightmap, while direct light, shadows and highlights are still
  drawn every frame. A scene entity says `static true` beside its
  `model` line. An unbaked or stale static model draws as any model, and
  `og doctor --project` names the scene.

## 3. Sounds

`.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` cook to 16-bit PCM encoded as
QOA, so every format plays, streams and loops the same. The asset id
keeps the source's extension (`audio/theme.mp3`). Procedural `.ogt`
presets load the same way.

A long sound (music, ambience) cooks in chunks of whole QOA frames:

```text
type import
version 1
---
sound {
  stream true
  chunk 4
}
```

- The import settings file is `<file>.ogt` (`theme.mp3.ogt`). `chunk` is
  in seconds, 1 to 60, default 4.
- A sound whose QOA file would pass 1 MiB is chunked unless it says
  `stream false`.
- The first chunk loads with the sound. The engine then reads chunks
  from about half a second behind each playhead to four seconds ahead,
  and drops the rest. On the web they are fetched like a streaming
  group's pack.
- A chunked sound plays sample for sample as the whole file would,
  loops and crossfades included. It cannot join a streaming group.

## 4. Environment maps

An `.ogt` of `type environment` names its source:

- `source`: an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the
  centre column faces +X, as three.js maps it), or a KTX2 cube map (six
  square faces +X -X +Y -Y +Z -Z, rows from the top; RGBA16F, RGB9E5,
  BC6H and the other HDR formats, decoded at cook time).
- `faces`: six `.hdr` or `.exr` images in that order.
- `size`: level 0's face size, a power of two from 16 to 2048, default
  128. Also `intensity`, `rotation` (degrees about +Y) and `samples`.

The cook prefilters it into a cube of five GGX levels (roughness 0 to 1)
in RGB9E5, plus the diffuse irradiance as nine harmonics, so loading
only uploads it. `prefiltered true` takes a KTX2 cube's own mip chain
as the levels, at the file's own size, without filtering it again: use
it for a cube already prefiltered by cmgen, IBLBaker or the glTF IBL
sampler. Level k of L is then at roughness k / (L - 1).

On the web the map is fetched with the bundle like any blob. Until a
streamed one's group is resident, the sky lights the scene.

## 5. Particle emitters

An `.ogt` of `type particles` describes an emitter: shape, rate and
bursts, lifetime, speed, size and colour over life, gravity, drag,
noise, a texture and sprite sheet, and blending.

| Line | Does |
|---|---|
| `collision <bounce> <friction> [depth \| colliders]` | bounce off the drawn depth, or off the scene's colliders on the GPU |
| `render mesh box \| sphere \| tetrahedron \| "<model id>"` | each particle a small lit shape that tumbles with its rotation; otherwise billboards or 2D sprites |
| `lit true` | billboards and meshes shaded by the sun, the sky and local lights |
| `distortion <0..1>` | billboards that bend the scene behind them: heat haze, shock rings |
| `subEmitter <birth \| death \| collision> "<id>" <count> [<inherit velocity>]` | another emitter's particles born where this one's are born, die or bounce; up to 4 |
| `subEmitter life "<id>" [<inherit velocity>]` | each particle runs the other emitter's own emission (rate, bursts, duration, looping) from its birth while it lives |
| `events <birth \| death \| collision>...` | og.ParticleEvent; its systems simulate in the tick |
| `array "<id>" [<layers>] [off \| life <cycles> \| fps <n> \| random]` | sprite mode: a layer of a texture array by age, at a rate, or one at random; one draw per array |
| `round true` | fade each particle to the soft dot's round edge |
| `effect "<id>"` | draw billboards or mesh particles with an effect material |
| `speedOverLife`, `spinOverLife`, `rateOverTime` | curves of `<t> <value>` pairs against the velocity, the angular velocity, and `rate` over each cycle |
| `attractor <x> <y> <z> <strength> [<radius>]` | up to 4 points in the emitter's space that pull, or push when negative |
| `field curl <strength> [<frequency> <scroll>]` | divergence-free curl noise |
| `field "<texture>" <strength> <size>` | a vector grid baked from an image |
| `shape spline <x> <y> <z> ... [closed]` | 2 to 8 Catmull-Rom points, emitted evenly along it and along its tangent |
| `inheritVelocity <fraction>` | a fraction of the emitter's own velocity, in world space |

- Sub-emitters go one level deep. A sub-emitter that fires its parent is
  refused.
- A vector-grid image is N² pixels wide and N high, N up to 16. Pixel x
  + z·N holds a cell, y counts from the bottom, and rgb 0..1 maps to
  -1..1.
- An effect material is an `.ogt` of `type effect`. It has a main
  texture, gradient ramp, mask, noise erosion, flow map, fresnel and soft
  depth fade, each texture with its own tile, scroll, turn and polar
  coordinates. Intensity, alpha, the uv offset, the erosion and eight
  custom numbers can each be a constant, a random range per particle or
  a curve over the particle's life. An optional material shader's
  surface_color hook runs last and can read effect_life(),
  effect_random(), effect_custom1() and effect_custom2().
- Every line runs the same in compute and on the CPU path.
- The textures, array, mesh model, effect material and sub-emitters an
  emitter names load with it. While the array's streaming group is not
  resident, its systems draw nothing.

## 6. Shaders, styles, documents and looks

- A shader is an `.ogt` of `type shader`; the cook compiles its WGSL for
  every back end. `mode hooks` (the default) defines hooks of the lit
  shader. `mode full lit` and `mode full unlit` define
  `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput`
  and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over
  the binding contract: camera(), the model with its skin, Params, and
  tex0 to tex7 for the material's eight texture slots. A full shader is
  lit by the frame's lights or drawn as returned. The prepass and every
  shadow pass draw through the same entries.
- A UI style is an `.ogt` of `type style`. Version 3 is the CSS dialect
  (docs/ui.md §3.2), which `og style upgrade`, `og style new` and
  `og ui migrate` write and every built-in style uses. It holds the
  tokens: rarity, team, damage and chart colours, each with its cue; the
  type scale per breakpoint; the space, size, radius and stroke scales;
  elevation; the shape language; motion springs and eases; and sounds and
  haptics per UI event. Colours and lengths are custom properties
  (`--panel-raised: #211a15`). Selector blocks dress the parts
  (`Button::base.primary:active { -og-slice: … }`). Versions 1 and 2
  still load. The grammar is og-ui's `parse_style`.
- A UI document is an `.ogt` of `type screen` or `type component`.
  Version 2 uses CSS declarations for an element's look and layout
  (docs/ui.md §3.2, og-ui's README §Documents); `og ui migrate` writes a
  version 1 document as version 2. `og dev` reloads an edited document in
  place.
- A look is an `.ogt` of `type look`. Its top-level lines are `name`,
  `extends`, `ui` and `antialias`. `extends` names a built-in preset's
  slug or another look's asset id; the game's own
  `assets/looks/<slug>.ogt` wins over the preset of that slug. `ui` names
  a built-in style's slug or the game's style file. Then comes one block
  per resource it sets, with that resource's fields by name and an asset
  id where a field is a handle (`lut "luts/warm.cube"`). The cook checks
  every field against the reflected schema and flattens `extends`, so
  the blob is the whole look (looks §1).

## 7. Animation clips

Clips cook as f32 samples by default. With
`animation { compress true tolerance 0.0001 }` in the model's import
settings they cook as quantised keys: rotations as 48-bit
smallest-three, and translations, scales and weights as 16 bits over each
channel's range. Every frame that interpolation rebuilds within the
tolerance is dropped. Loading rebuilds every frame either way, so the
clip plays as f32. Morph targets cook sparse, holding only the vertices
each one moves.

## 8. Texture arrays and flipbooks

- A texture array is an `.ogt` of `type texture-array`: `kind
  color|normal|linear|pixel` and one `layer "<file>"` per layer, 1 to 256
  images of one size. Each path is beside the file or under assets/.
  Each layer is also a texture of its own, `<path>#layer<n>`.
- Sprites, og.Sprite and 2D particle emitters draw from an array with
  one draw per array, whatever layer each shows, with glow and the
  additive blend. Materials and terrain layers do not sample arrays yet.
- When an array's group unloads, the renderer lets it go: og.render.sprite
  of it is refused, and og.Sprite and particles of it draw nothing until
  the group is resident again.
- TextureArrayInfo and the frame's texture bytes report an array's GPU
  size.
- A flipbook sheet is an `.ogt` of `type flipbook`; the cook resolves its
  Aseprite or TexturePacker atlas. A sheet may name animated GIF, WebP
  or APNG files instead (`source "hero_run.gif"`, a clip per file). The
  cook turns their delays into clip durations, crops every frame to one
  size with the pivot at the anchor, shares identical frames and packs
  them into `<sheet>#sheet`. `key auto` keys out a flat background,
  `align feet` holds the figure still and `scale` resamples.
  `og sprite inspect` measures a source's frames, drift, loop seam and
  edge fringe.
- A sheet with `array true` cooks its frames into a texture array, so
  every sprite of the sheet is one draw, whichever frame each shows.

## 9. Video

An `.ogt` of `type video` names `source` (an .mp4, .webm or .mov beside
it or under assets/), `fps`, `maxSeconds`, `size W H`, `kind
color|pixel` and `audio`. A stream adds `stream true`, `keyframe` and
`crf`. The cook needs ffmpeg (`og doctor` checks it).

| Kind | Cooked as | Plays |
|---|---|---|
| clip | a texture array of its frames | uploaded whole |
| stream | AV1 units | fetched as it plays (on the web, not in the startup bundle) and decoded ahead of its players |

Load videos in GameDesc.start. Every process of an `og render --jobs`
run then loads them the same way, so their handles agree.

## 10. Uncooked loads

og.load.imageData, og.load.gltfData and og.load.environmentData load bytes
a player drew, uploaded or a generator wrote. The game fetches the file
itself and passes its bytes.

- The header is read and checked against the budget at the call, before
  anything is decoded.
- Decoding stays off the engine's thread. On the web, images decode with
  createImageBitmap in the engine's Worker, so the wasm carries no image
  decoder. glTF primitives are converted, and `.hdr` rows decoded, at the
  tick boundaries within LoadBudget.frameMs, never in one long stall.
  Natively a worker thread decodes.
- Completion is host input, applied at a tick boundary and journaled
  like a key press. Natively a load is in place before the tick after
  the call. On the web it lands at the first boundary after decoding,
  and og.load.completions records that tick. A replay
  (`og replay --compare`, og.load.replayCompletions) puts it in place at
  the same tick.
- In a lockstep or rollback room, content one player made is not on the
  other machines, and an asset's status is not world state. Send the
  bytes with og.platform.roomSend (at most 64 KiB a message) and let each
  peer load them. The shared simulation uses the content only from a
  tick the peers agree on through their inputs (og.net.submit), never
  from og.load.status.

### What gltfData reads

It reads the metallic-roughness materials (base colour,
metallic-roughness, normal, occlusion and emissive maps, alpha mask,
double-sided, KHR_materials_unlit, KHR_materials_emissive_strength), the
node tree and scenes, and vertex colours and tangents. A colour texture
magnified nearest is drawn as pixel art.

- Skipped and counted: skins and animations. Skipped: points and lines.
- Not made: levels of detail or a raycast BVH; a raycast builds one when
  first needed.
- Not read: the other KHR_materials extensions, KHR_texture_transform and
  KHR_lights_punctual.
- Refused by name: Draco and meshopt compression, Basis, WebP and AVIF
  textures, sparse accessors, a buffer or image in another file (pack the
  model as one .glb) and any other required extension.
- A system that instantiates writes og.Transform (and og.Parent for a
  parented root): the root is its spawn. What is under the root is in the
  world when the system returns, not before: read it from a later system.

### environmentData

The image is twice as wide as it is tall, and its centre column faces
+X. The diffuse irradiance (nine harmonics) is gathered as it decodes.
When the renderer first draws with it, the GPU resamples it into a cube
and GGX-prefilters the five levels a cooked map has, with the passes the
sky's own light runs. The cook's `bake` does the same on the CPU. A web
build cannot read .exr (RenderCapabilities.exrDecode): convert the image
to .hdr, or cook it.

### Release

og.load.release frees an uncooked asset's memory on the GPU and its copy
on the CPU; the probe's memory report and `dump_memory` show both. What
still names its handles draws nothing and nothing breaks. Physics shapes
built from its geometries are world state and stay.

## 11. Composed effects

A composed effect (a fireball's impact, a heal, a level-up) is an
`.ogt` of `type vfx`: a timeline of elements, Effekseer's and Niagara's
systems as text. An Effekseer effect in assets/ (`.efkproj`, `.efkefc`)
loads too: the cook converts its nodes into the engine's vfx, emitters,
effect materials and meshes and warns of what it could not carry.
`og vfx import` writes the same conversion as `.ogt` text to edit.

```text
type vfx
version 1
---
duration 1.2
loop false
impact 0.3
event 0.6 "linger"
emitter "fx/sparks.ogt" {
  at 0.3
  until 0.8
  seed 7
}
mesh ring {
  effect "fx/shock.ogt"
  at 0.3
  life 0.4
  inner 0.6
  scale curve 0 0.2 1 3
  blend additive
}
shake {
  at 0.3
  trauma 0.4
}
```

The header lines are `duration <seconds>`, `loop true|false`,
`event <seconds> "<name>"`, and `impact <seconds>` (the same as
`event <seconds> "impact"`, at most one). They mean the same in a
`type vfx2d` pixel effect (pixel-vfx §2).

Every element takes:

- `at <seconds>` from the effect's start;
- `offset <x> <y> <z>` and `rotate <x> <y> <z>` (degrees) in the
  effect's space;
- `move <x> <y> <z>` (metres per second) and `accelerate <x> <y> <z>`
  (per second squared), carrying it from its offset; an emitter spawns
  along the path;
- `life <seconds>` or `until <seconds>`, for elements that last.

A value is a number, `random <min> <max>` (fixed per instance) or
`curve <t> <v> ...` over the element's life.

| Element | Lines |
|---|---|
| `emitter "<id>"` | an og.load.emitter asset; `until` stops its emission and its particles live out; `seed` |
| `mesh sphere \| quad \| ring \| box \| cylinder \| "<model id>"` | drawn with `effect "<id>"`, an effect material; `scale` (a value), `spin` (degrees per second), `color`, `blend additive \| alpha` |
| `trail` | `effect`, `width <head> <tail>`, `lifetime`, `color`, `orbit <radius> <turns per second>`, `rise <metres per second>` about and up the element's y |
| `beam` | `to <x> <y> <z>` from the element, `width`, `effect`, `segments`, `jitter`, `color` |
| `light` | a point light: `color`, `intensity` (a value), `distance` |
| `decal` | projected down the element's -y: `uv <u0> <v0> <u1> <v1>` (a region of RenderSettings.decalAtlas), `size <x> <y> <z>` with y the depth it reaches, `fade <seconds>` at its end, `color` |
| `flash` | a soft glow facing the eye: `size` (a value), `color` |
| `shake` | `trauma` (og.camera.shake) |
| `flipbook "<id>"` | a camera-facing og.Sprite with og.Flipbook playing the sheet's first clip; `size` its height, the width by the frame's shape; `color`, `blend additive \| alpha`; for `life` or the clip's length |
| `sound "<id>"` | `volume`, `playbackRate <min> <max>` picked by the seed; a child entity with og.AudioSource, cut when the effect despawns |

- A `ring` takes its own shape: `inner <radius>` (outer 1), `band <0..1>`
  (its middle row), `height <outer> <inner>` (its edges raised along its
  axis: a cone, a tube), `arc <start> <length>` (degrees from +x),
  `segments <n>` and `colors <outer rgba> <band rgba> <inner rgba>`. Its
  u runs along the arc and v from the outer edge to the inner. It is
  built at load.
- A `flipbook` takes a pixel effect (`type vfx2d`) or any og.load.flipbook
  sheet. Its og.FlipbookEvent events, the impact frame among them, reach
  the game as this effect's og.VfxEvent of the same name.
- `og cook` checks the file and that everything it names is in the
  project.

### Pitfalls

- Spawn an effect with a parent (og.vfx.spawn) to make it follow an
  entity. Without one it stays where it was spawned.
- Visuals never feed gameplay: read og.VfxEvent for the frame something
  lands, never the drawn elements.
- The og.vfx system is scheduled when a game first names og.Vfx or
  og.VfxEvent, after the systems defined so far. An effect started after
  it ran in a tick catches up on the next tick.
- An `event` line fires on startTick plus its time rounded to ticks.
- Element times run on the tick, never on the wall clock.
