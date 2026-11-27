<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `loading` guide: Loading assets

Loading what `og cook` made, and what a player or a generator makes at run time. This guide covers the import settings and asset files that each loader reads, what the cook does to them, streaming and residency, uncooked loads, and composed effects.

The API: `og api show load` (one function per asset kind), `og api show vfx`.
Asset queries, publish pruning and the cook cache are in assets.

<a id="guide-loading-1"></a>
### `loading §1` The bundle

- The game's bundle (`cooked`) is open and GameDesc.scene is spawned
  before GameDesc.start and the first tick. The web host fetches the
  ungrouped boot files before start; streaming packs, sound and video
  chunks, texture mip chains and optional font-pack faces remain on demand.
- Fonts explicitly referenced by game code or its asset dependencies are
  boot files, even when they belong to a font pack: og.load.font in
  GameDesc.start returns a ready font on both hosts. The cook conservatively
  includes every font under a computed load's static prefix; an escaped
  loader or a path with no static prefix includes every candidate font.
  This is the publish-pruning reference graph, not a second analyser:
  string literals in the TypeScript/JavaScript the bundle reaches outside
  the engine, and resolved asset dependencies, are roots, as are
  project.ts's publish.keep patterns. An unimported scratch script is not.
  `og.load.font(world, data.path)` includes all candidates;
  `og.load.font(world, "fonts/latin/" + name)` includes that prefix.
  Aliased og.load functions/namespaces also include all candidates. There
  is no silent byte cap: a broad candidate set can include whole CJK packs.
  og build reports each boot font's path, bytes and compressed bytes; narrow
  the prefix or use explicit paths to reduce it. Recook after changing code.
  Paths hidden from the graph (for example a loader supplied as runtime
  code, with its path read from remote data) are not
  automatically declared. Name each required font in publish.keep and
  recook. An ungrouped pack face absent from the boot set is refused by
  og.load.font on both hosts, even when its bytes are resident, so og test
  catches a missed declaration before a browser does. Faces reachable
  only through a font-pack index remain on demand: og.i18n fetches the locale's chain and additional
  script coverage as needed. The existing first-frame readiness gate waits
  for the faces and glyphs that frame needs; no tick is chosen by a font fetch.
- A runtime bundle's manifest or scene blob can still be fetching.
  og.load.scene answers a stable pending handle once its bundle is open;
  keep that handle and query og.load.status before instantiating. Pending
  loads complete at tick boundaries; og.load.error gives a failed load's reason.
- og.load.bundle of another bundle opens at the call when the host has its
  manifest (natively always), else at a tick boundary once the web host has
  fetched it. Either way the open is host input: og.load.completions
  journals its tick (`kind` `bundle`), and a replay opens it at the same
  tick, so a system may act on its og.load.status.
- Every cooked loader answers the same handle for the same path. Load
  boot assets once in GameDesc.start and keep the handle. Other bundles
  and streaming groups must have the requested asset's bytes resident
  before calling a synchronous loader such as og.load.font.
- An asset in a streaming group's pack loads once its group is
  resident, and the refusal before then names the group. Texture
  arrays, flipbook sheets and environment maps come back under the same
  handle when their group returns.

### Import settings

A file's import settings are a `type import` document named after it
(`theme.mp3.ogt` beside `theme.mp3`). When many files in a directory
take the same settings, the directory's `import.ogt` says them once:

```text
type import
version 1
---
match "*.png" {
  texture {
    kind pixel
  }
}
match "music/*.ogg" "ambience/*.ogg" {
  sound {
    stream true
  }
}
```

- A `match` block holds what a file's own settings hold, for every file
  under the directory that one of its patterns names. A pattern is a
  path from the directory, read as `project.ts`'s `conform` patterns
  are: `*` is any run of characters within a name, `**` any run across
  directories, a trailing `/` everything under that folder; case counts.
  `*.png` is the directory's own files, `**.png` those at any depth.
- A file takes the matching blocks of every `import.ogt` from `assets/`
  down to its own directory, in that order and in each file's order, then
  its own `<file>.ogt`. A later entry replaces an earlier one of the same
  key; two blocks of one key (`texture { … }` twice) join the same way.
- A file's own settings need only say what differs. Editing a block
  recooks the files whose settings it changes and no others.
- An error in a block is reported on each file it reaches, with its line
  in the `import.ogt`.
- A texture a model references is cooked with the model. It also cooks
  on its own only when it has a `<file>.ogt`, never for a directory's
  block alone.
- `import.ogt` holds `match` blocks only. The engine's built-in assets
  (`@…`) do not read it.

<a id="guide-loading-2"></a>
### `loading §2` Models

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

### Orientation

The cooked model faces +Z with +Y up (the facing guide). A model from a
generator or a store often faces another way: state its axes in the
import settings and the cook turns it, so the game places it with no turn
of its own:

```text
type import
version 1
---
model {
  front "-x"   # the file's axis its nose points along
  up "+y"      # the file's axis that is up
}
```

Either may be left out: `up` is +y unless `front` lies along y, and with
`up` alone the front is the smallest turn's. The turn wraps each scene's
roots in one node, so meshes, skins, clips and fracture chunks turn
together. Check the nose with `og model preview <file> --axes`: it draws
the engine's `+Z FRONT`, `+Y UP` and `+X` at the origin as the cook will
place them (preview the source with the same `--up`/`--front` the import
settings state, or the cooked result).

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

A scene's distant static models can also be merged into proxies by the
cook (HLOD): the world guide, §5.

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
  the lightmap, while the direct light, shadows and highlights of its
  movable and stationary lights are still drawn every frame (a static
  light's direct light is in the lightmap: render §3). A scene entity says `static true` beside its
  `model` line. An unbaked or stale static model draws as any model, and
  `og doctor --project` names the scene.

<a id="guide-loading-3"></a>
### `loading §3` Sounds

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

<a id="guide-loading-4"></a>
### `loading §4` Environment maps

Export a `file.environment({...})` declaration from `content/**/*.ts`, importing
`file` from `@origingame/engine/author`. It names exactly one source:

- `source: {equirect: "assets/env/day.hdr"}`: an equirectangular sRGB PNG, JPG or WebP, Radiance `.hdr` or OpenEXR `.exr` (the
  centre column faces +X, as three.js maps it).
- `source: {ktx2: "assets/env/studio.ktx2"}`: a KTX2 cube map (six
  square faces +X -X +Y -Y +Z -Z, rows from the top; RGBA16F, RGB9E5,
  BC6H and the other HDR formats, decoded at cook time).
- `source: {faces: [px, nx, py, ny, pz, nz]}`: six HDR/EXR or PNG/JPG/WebP paths in that order.
- `size`: level 0's face size, a power of two from 16 to 2048, default
  128. Also `intensity`, `rotation` (degrees about +Y) and `samples`.
- `sun`: `extract` (the LDR default) or `keep` (the HDR default), below.

Every path is canonical and relative to its owning module, under `assets/` or
`sources/`. There is no declaration-sibling or project asset-root fallback.
The cook captures the image bytes before cache lookup and bakes that snapshot.
Load `<module>/<content-relative-stem>`, not the source filename. OGT environment
authoring is removed; `og api show file:environment` gives the generated shape.

`render §4` documents LDR skies, upper-hemisphere ground fill, expansion,
sun estimation and source diagnostics. They all produce
the same EnvironmentMap; `environmentData` remains an uncooked HDR loader.

The cook prefilters it into a cube of five GGX levels (roughness 0 to 1)
in RGB9E5, plus the diffuse irradiance as nine harmonics, so loading
only uploads it. `prefiltered: true` takes a KTX2 cube's own mip chain
as the levels, at the file's own size, without filtering it again: use
it for a cube already prefiltered by cmgen, IBLBaker or the glTF IBL
sampler. Level k of L is then at roughness k / (L - 1).

`sun: "extract"` is for an outdoor image with the sun in it. Left in the
map, a sun lights without casting a shadow and leaves bright speckles in
rough reflections; an `og.DirectionalLight` added beside it lights the
scene with the sun a second time. With `sun: "extract"` the cook takes the
sun's light out of the image (everything within 5° of the brightest
point that is over sixteen times the rest's mean for HDR; a detected disc
and fitted daylight estimate for LDR) and stores it:

```ts
const map = og.load.environment(world, "base/env/day");
if (!map.ok) throw new Error(map.reason);
const rotation = 0.4, intensity = 1.2; // the same values as og.Environment
const result = og.load.environmentSun(world, map.value, rotation, intensity);
if (!result.ok) throw new Error(result.reason);
const sun = result.value;
// sun.direction points towards the sun; a light shines along its -Z.
const light = world.spawn({ name: "sun" });
if (!light.ok) throw new Error(light.reason);
world.insert(light.value, og.DirectionalLight, { color: sun.color, intensity: sun.intensity });
world.set(light.value, og.Transform, { quaternion: og.transform.lookAt(
  { x: 0, y: 0, z: 0 },
  { x: -sun.direction.x, y: -sun.direction.y, z: -sun.direction.z },
) });
```

Aim the light's entity so its -Z points along `-sun.direction`: the answer
already applies runtime rotation and intensity (default 0 and 1). Its
intensity is in DirectionalLight units, not lux; assign it directly. The sun's disc
stays as authored in an LDR background (HDR keeps the extraction cap). `intensity` 0 means
the image had no sun; the cook's report says what it took. A scene lit
by a map alone (a studio, or an image whose sun is kept) sets a
DirectionalLight of intensity 0: without one the default sun still
lights the scene beside the map.

On the web the map is fetched with the bundle like any blob. Until a
streamed one's group is resident, the sky lights the scene.

<a id="guide-loading-5"></a>
### `loading §5` Particle emitters

An `.ogt` of `type particles` describes an emitter: shape, rate and
bursts, lifetime, speed, size and colour over life, gravity, drag,
noise, a texture and sprite sheet, and blending.

Every line, with its default, is in `effect-files §1`; the parser
refuses any other and names the ones it takes.

- Sub-emitters go one level deep. A sub-emitter that fires its parent is
  refused.
- An effect material is an `.ogt` of `type effect` (its lines are
  `effect-files §2`). It has a main texture, gradient ramp, mask, noise
  erosion, flow map, fresnel and soft depth fade, each texture with its own tile, scroll, turn and polar
  coordinates. Intensity, alpha, the uv offset, the erosion and eight
  custom numbers can each be a constant, a random range per particle or
  a curve over the particle's life. An optional material shader's
  surface_color hook runs last and can read effect_life(),
  effect_random(), effect_custom1() and effect_custom2().
- Every line runs the same in compute and on the CPU path.
- The textures, array, mesh model, effect material and sub-emitters an
  emitter names load with it. While the array's streaming group is not
  resident, its systems draw nothing.

<a id="guide-loading-6"></a>
### `loading §6` Shaders, styles, documents and looks

- A shader is an `.ogt` of `type shader`; the cook compiles its WGSL for
  every back end. `mode hooks` (the default) defines hooks of the lit
  shader. `mode full lit` and `mode full unlit` define
  `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput`
  and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over
  the binding contract: camera(), the model with its skin, Params, and
  tex0 to tex7 for the material's eight texture slots. A full shader is
  lit by the frame's lights or drawn as returned. A lit one takes what
  the standard material takes around its own surface: decals, rain,
  puddles and snow, and screen-space reflections by its roughness and
  metalness. The prepass and every shadow pass draw through the same
  entries.
- A UI style is a directory under `assets/styles/<name>/` holding a theme
  recipe (`theme.json`), a style pack (`pack.json`) laid over it, or both
  (screens §3, themes §1). Start one with `og style init <name> [--from <theme>]`,
  install a directory with `og style add <directory>`, and check it with
  `og style validate <directory>`. Legacy `style.ogt` CSS packs are removed
  and refused by the cook and runtime.
- A look is an `.ogt` of `type look`. Its top-level lines are `name`,
  `extends`, `variant`, `mood`, `ui`, `uiSeeds` and `antialias`. `extends`
  names a built-in preset's slug or another look's asset id; the game's own
  `assets/looks/<slug>.ogt` wins over the preset of that slug. `ui` names
  a built-in theme slug or the game's style directory (`styles/<name>`).
  Then comes one block per resource it sets, with that resource's fields by
  name and an asset id where a field is a handle (`lut "luts/warm.cube"`).
  The cook checks every field against the reflected schema and flattens
  `extends`, so the blob is the whole look (looks §1).

<a id="guide-loading-7"></a>
### `loading §7` Animation clips

Clips cook as f32 samples by default. With
`animation { compress true tolerance 0.0001 }` in the model's import
settings they cook as quantised keys: rotations as 48-bit
smallest-three, and translations, scales and weights as 16 bits over each
channel's range. Every frame that interpolation rebuilds within the
tolerance is dropped. Loading rebuilds every frame either way, so the
clip plays as f32. Morph targets cook sparse, holding only the vertices
each one moves.

<a id="guide-loading-8"></a>
### `loading §8` Texture arrays and flipbooks

- A texture array exports `file["texture-array"]({kind:"color",
  layers:["assets/tiles/grass.png","assets/tiles/dirt.png"]})` from
  `content/tiles.ts`, importing `file` from `@origingame/engine/author`.
  Give 1..256 images of one size, in layer order; kinds are `color`,
  `normal`, `linear`, `single` and `pixel`. Paths are explicit and relative
  to the owning module, never chosen by a sibling/assets fallback.
  Load `base/tiles` (or the owning module's ID); each actual layer is also
  a texture of its own, `base/tiles#layer<n>`. OGT array sources are refused.
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

<a id="guide-loading-9"></a>
### `loading §9` Video

Export `file.video({...})` from content/clips/intro.ts and load it as
`base/clips/intro`. Its `source` is a canonical .mp4, .webm or .mov path
under the owning module's assets/ or sources/, without sibling fallback.
A stream uses `stream: true`. Inspect the complete typed input with
`og api show file:video`; `og api gen` provides project-local completion.
`renditions: ['hevc', 'h264']` selects browser codecs; `[]` disables them,
and omission enables HEVC when available. Actual frames, streams, posters,
audio tracks and chunks are referenceable subresources of the canonical ID.
The cook needs ffmpeg (`og doctor` checks it and its encoders).

| Kind | Cooked as | Plays |
|---|---|---|
| clip | a texture array of its frames | uploaded whole |
| stream | an index and chunks of AV1 units, and HEVC or H.264 renditions of the same frames and chunks | never in the startup bundle; each chunk fetched ahead of the players that need it and let go after (one copy: the AV1, or the rendition a browser without AV1 decodes), and decoded ahead of its players |

Load videos in GameDesc.start, all of them: a stream's load fetches
nothing, so a game with hundreds of clips pays only for the ones that play.
Every process of an `og render --jobs` run then loads them the same way, so
their handles agree.

<a id="guide-loading-10"></a>
### `loading §10` Uncooked loads

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

For runtime bundles, release closes the bundle only when no live component,
resource, persisted cell or HLOD placement references it and its groups are
unloaded. Inside a system the check runs at the next tick boundary (`pending`
until then); between ticks it can finish immediately. Poll `og.load.error`:
`bundle release refused:` means the bundle remains open and can be retried
after removing the named references; an error naming the release means it
closed. Both outcomes use status `failed`, without events or hash changes.
The boot bundle and individual cooked assets cannot be released this way.
See **world §4** for parcel lifetime and streaming.

A runtime bundle's terrains (og.terrain.load) go with it. An og.Terrain
does not keep the bundle open: the close goes ahead, and each of its
terrains is freed at the first tick boundary where nothing names it any
more: no og.Terrain, og.TerrainScatter, og.Grass or other TerrainData
field, the game's records' included, and no Geometry field holding its
height field (TerrainInfo.geometry: a collider's, or a game's), on a live
entity, a resource or a persisted cell's kept entity. Freed means
whole: the cooked tiles, the field the CPU holds for collision and
sampling, the physics height field, the tiles on the GPU, and the layer
textures and arrays that came from that bundle and no other terrain
uses. A layer array from a bundle still open (a shared material pack)
stays with that bundle. So a parcel's terrain is released by despawning
its og.Terrain and releasing its pack, in either order; a terrain still
drawn outlives the release until its last og.Terrain despawns. A freed
terrain's handle is refused from then on, and loading its path again
from the bundle opened anew loads it again. The boot bundle's terrains
stay loaded.

<a id="guide-loading-11"></a>
### `loading §11` Composed effects

A composed effect (a fireball's impact, a heal, a level-up) is an
`.ogt` of `type vfx`: a timeline of elements, Effekseer's and Niagara's
systems as text. An Effekseer effect in assets/ (`.efkproj`, `.efkefc`)
loads too: the cook converts its nodes into the engine's vfx, emitters,
effect materials and meshes and warns of what it could not carry.
`og vfx import` writes the same conversion as `.ogt` text to edit. A
one-shot effect gets an `impact` at the frame where the most starts at
once; `og vfx import --hit` also adds a light and a camera shake there,
for hits and explosions.

```text
type vfx
version 2
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

The elements are `emitter`, `mesh`, `trail`, `beam`, `light`, `decal`,
`glow`, `shake`, `sound` and `flipbook`; every line each takes, with its
default, is in `effect-files §3`.

- A `mesh ring` builds its own shape at load: u runs along the arc and v
  from the outer edge to the inner.
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

<a id="guide-loading-12"></a>
### `loading §12` Preloading the next scene

A scene whose assets are in streaming groups (world §4) changes in
three steps: preload while the current scene plays, wait on a loading
screen for what is still on its way, then change.

```ts
// A command or a system, a level before the door:
og.scene.preload(world, "level2");
// The loading screen's bar (a UI store, or a system that only draws):
const p = og.scene.progress(world, "level2");
if (p.ok) bar.set(p.value.fraction);
// At the door:
og.scene.change(world, "level2");
```

- og.scene.preload requests every group the scene declares and keeps
  them requested until og.scene.release. Request groups a few seconds
  before they are needed: a pack of a few megabytes arrives in that on
  a normal connection.
- It also loads what the scene spawns with, without placing it: its
  models and their textures into the renderer, their pipelines started,
  its sub-scenes and prefabs. og.scene.progress answers `prepared` once
  all of that is done and every group is in: a scene changed to then is
  drawn whole in its first frame, with nothing left to compile. `ready`
  is the groups alone.
- og.scene.change puts the scene in place as the next tick starts,
  whatever has arrived: the current scene's root and everything under
  it go (the boot scene's own entities for the first change), the new
  scene spawns under a new root, and og.SceneChanged arrives in that
  tick. Entities the game spawned outside a scene (the player, a
  vehicle) stay; give them names the new scene does not use.
- What has not arrived draws its stand-ins (world §4). Cover them:
  keep a loading screen up until og.scene.progress answers `ready`,
  or change at a door the player walks to after `ready`.
- og.SceneState (registered at the first change) is world state, so a
  save, a restore and a replay change at the same tick. Progress is
  not: show it, never branch a tick on it in a networked game.
- og.stream.status and og.stream.budget give each group's `received`
  and `progress`; og.load.progress gives a bundle's.
- On the web, preloading also fetches the engine code the scene needs:
  its physics, navigation, dialogue or video, and those of the scenes
  and prefabs it places. A level that is the first to use physics
  therefore does not wait for that code when it starts.

The built-in `loading` screen (key art, place, tip, bar, percent) is the
cover. Mount it with stores, feed its bar each tick, change once the
groups are resident, and unmount it when og.SceneChanged arrives: that
tick starts with the old scene gone and the new one spawned, so nothing
uncovered shows a stand-in or the old scene.

```ts
const Travel = defineResource("Travel", { scene: str(64), changing: bool() });
let unmount: (() => void) | undefined;

// A command, a trigger or a menu: fetch the scene and cover the screen.
function travel(world: World, scene: string, place: string): void {
  const t = world.resource(Travel);
  if (!t.ok || t.value.scene || !og.scene.preload(world, scene).ok) return;
  t.value.scene = scene;
  t.value.changing = false;
  const ui = og.ui.scope(world, "loading");
  const progress = ui.writable("progress", 0);
  const ready = ui.writable("ready", false);
  progress.set(0);
  ready.set(false);
  unmount = mount(world, () => {
    const p = useStore(progress), r = useStore(ready);
    return <LoadingScreen keyArt={0} place={place} tip="" progress={p()} ready={r()} />;
  }, {}, { layer: "system" });
}

const Arrive = defineSystem("Arrive", { reads: [og.SceneChanged], writes: [Travel] }, (_q, _time, world) => {
  const t = world.resource(Travel);
  if (!t.ok || !t.value.scene) return;
  const ui = og.ui.scope(world, "loading");
  if (world.events(og.SceneChanged).length > 0) {
    unmount?.(); // the new scene is in place
    unmount = undefined;
    t.value.scene = "";
    return;
  }
  const p = og.scene.progress(world, t.value.scene);
  if (!p.ok) return;
  ui.writable("progress", 0).set(p.value.fraction);
  ui.writable("ready", false).set(p.value.ready);
  if (p.value.ready && !t.value.changing) {
    og.scene.change(world, t.value.scene);
    t.value.changing = true;
  }
});

// mount and useStore from "@origingame/engine/ui",
// LoadingScreen from "@origingame/engine/ui/components".
// og.game({ resources: [Travel], systems: [Arrive], … })
```

- Waiting on `ready` branches a tick on progress. That is fine alone;
  in a networked game change at a tick the game agrees on (a door, a
  vote, a timer) and keep the cover up until `ready` instead.
- Pass `keyArt` and `tip` the same way. `og ui eject loading` copies the
  screen to rearrange it; a style pack dresses its parts
  (`loading.scrim`, `loading.frame`, `loading.title`).
- The opening load is §13 (og.loading); this is every load after it.

### Pipelines behind the loading screen

A level's first frames compile its shaders' pipelines. On the web a cold
driver takes seconds over each, and a large scene compiles dozens: keep
them behind the loading screen, not in play.

```ts
// Once the level is spawned (after og.SceneChanged, or in start):
og.render.compile(world);
// The loading screen's bar, each tick:
const s = og.loading.status(world);
ui.writable("progress", 0).set(s.pipelinesRequested ? s.pipelinesReady / s.pipelinesRequested : 0);
// Uncover once nothing is left:
if (og.loading.idle(world)) og.loading.done(world);
```

- og.render.compile compiles a few pipelines a frame in the background:
  every model, light and terrain the world draws now, wherever the
  camera points, with every lit feature, so rain or a lamp switched on
  later draws at once. A model or effect spawned after it compiles when
  it first draws: spawn a level's content before calling it.
- pipelinesRequested and pipelinesReady only go up; a bar of their ratio
  can jump back when a frame asks for more, so show the count
  (`41/96`) or never let the bar go back.
- og.loading.idle stays false until compile has made everything and
  nothing compiles. The play gate does the same for the first scene, and
  the engine splash counts it ("Preparing graphics 41/96").
- After the next frame it also does, once and without a tick, the work
  a level's first played frame would do for the first time: the UI's
  documents are built, laid out and painted on a copy of the UI, and
  physics builds the chunk shapes of its height fields. The play gate
  does the same behind its Play button. That frame then reuses compiled
  code, shaped text and the chunk shapes instead of stalling on them;
  the world, the UI, its stores and its events stay exactly as they were.
- It is presentation: never wait on it in a tick's logic in a networked
  game, as with og.scene.progress.

<a id="guide-loading-13"></a>
### `loading §13` Loading screens

project.ts's `defineProject` picks who draws the loading screen:

```ts
publish: {
  loading: "game", // engine (the default) | game | page
  loadingLabels: {
    fetch: "下载中",
    transcode: "准备纹理",
    prime: "准备场景",
    present: "准备画面",
  },
},
```

| `loading` | Before the first frame | After it |
|---|---|---|
| `engine` | the engine's splash, its bar and labels | the play gate, then the game |
| `game` | the splash, while the engine boots | the game's own frames, the clock running, until og.loading.done; then the gate (with `gate tap`) |
| `page` | nothing: the hosting page draws its loader | the canvas hidden, the clock running, until og.loading.done; then the gate (with `gate tap`) |

- Report stages with og.loading.stage (`{ id, fraction, label }`) and
  end with og.loading.done once og.loading.idle is true (after
  og.render.compile, §12, its pipelines are made too). Until then a
  `page` screen keeps the canvas hidden while the frames prepare their
  pipelines and textures.
- With `game`, the first screen the game shows is its loading screen:
  draw it from a document or a few sprites in the boot bundle, and keep
  the rest in streaming groups (§12).
- `loading_labels` names the engine splash's caption for each boot
  stage; without it the splash says "Loading" in the browser's
  language.
- Any web page that frames or hosts the game hears the same loading:
  window events on the game's page and messages to a framing page
  (docs/web-export.md §Loading events).

<a id="guide-loading-14"></a>
### `loading §14` Content catalogs

A catalog is content this game's host publishes beside the game: a
world's index pages, shared levels, anything static and versioned that
outgrows a store value (64 KiB) and that every player reads alike.
og.load.data reads one file of one version of it.

- A version never changes once published. Its manifest lists each file
  with its size and integrity; a file is read against that integrity,
  and bytes that differ fail the load. Game code names a catalog, a
  version and a path, never a URL: the content base is the host's.
- Where the content base comes from:

| Host | Content base |
|---|---|
| `og run`, `og replay`, a playtest | `--content <dir>` (PlaytestDesc.content, relative to the project), a directory |
| a page `og build` wrote | the Player's or the page's provider's, else `og build --content <url>`, else project.ts `publish: { content: "<url>" }` |

- Its layout, the same on disk and over https:

```text
<base>/<catalog>/<version>/manifest.json
<base>/<catalog>/blobs/<sha384 as hex>
```

```json
{ "format": 1, "catalog": "world", "version": "v12",
  "files": { "index/small/3/-2.json": { "size": 184320, "integrity": "sha384-…" } } }
```

  A file's bytes are under its integrity's digest, so a file a new
  version did not change keeps its address, and a browser keeps it.
- Completion is host input. Natively a file is in place at the first
  boundary after the call; on the web, at the first boundary after it
  arrives. og.load.completions journals the tick and the integrity read
  (`kind` `data`), so `og replay` reads the same bytes in at the same
  tick, and a system may act on the load's og.load.status.
- A failed load is not retried: call og.load.data again. Its error
  starts with the reason: `not_found` (no such catalog, version or
  path), `unavailable` (no content base), `failed` (the read failed, or
  failed in the recorded run), `timeout` (nothing for 30 seconds), `integrity`,
  `too_large` (over LoadBudget.bytes, 32 MiB by default) or `invalid`
  (not JSON, or not a manifest).
- og.load.release frees a file once it is read. A version's manifest
  is read once a session.
- Which version is current is the host's answer, not a file:
  og.platform.catalogHead (guide saves §11). Its answer, or a `catalog`
  notice, pins that version's manifest to the integrity it names, and a
  manifest read later is checked against it; one read before any answer
  named the version is read as it is.

### Packs

A pack is a cooked bundle a version lists by name: a region's models,
scenes and cell groups that the host publishes with the catalog instead
of with the game. og.load.pack opens one; once ready it is a runtime
bundle as og.load.bundle opens (§10 Release, guide world §4).

```text
<base>/<catalog>/packs/<name>/cooked/.cook.ogt   (and the rest of cooked/)
```

```json
{ "format": 1, "catalog": "world", "version": "v12", "files": { … },
  "packs": { "region-3-2": { "integrity": "sha384-…" } } }
```

- A pack's directory is the same in every version that lists it: a
  version that changes a pack lists it under a new name. Its `integrity`
  is that of its `cooked/.cook.ogt`, which is read against it; on the web
  the browser then checks every other file of the pack against that
  manifest.
- To make one, `og cook` a project holding the pack's assets and publish
  its `cooked/` directory there.
- Completion is journaled as a `bundle` open with its tick and manifest
  integrity, so a replay opens the same pack at the same tick. Its error
  starts with a data load's reasons; `not_found` when the version lists
  no such pack.
- Asking for a pack that is opening or open answers its handle.
