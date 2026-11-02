<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `load` module

Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.

Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.

The game's bundle (`cooked`) is opened before the first tick and GameDesc.scene is spawned then. Loads from an open bundle complete at once on both hosts (the web host fetches the bundle's files before the engine starts); only og.load.bundle of another bundle can be pending, and it completes at a tick boundary.

- **TypeScript**: functions are the namespace `og.load`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module load {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 7102</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Asset`](#type-Asset) | handle | A loaded bundle, scene or glTF asset. |
| [`Font`](#type-Font) | handle | A cooked MSDF font: its glyph table and one or more atlas pages. |
| [`Sound`](#type-Sound) | handle | A cooked sound: QOA-encoded PCM the audio module plays. |
| [`Style`](#type-Style) | handle | A cooked UI style: its values, atlas, fonts, icons and sounds loaded with it, for og.ui.configure. |
| [`TextureArray`](#type-TextureArray) | handle | A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray). |
| [`TextureArray()`](#column-TextureArray) | column kind | A TextureArray handle field (0 is none). |
| [`Asset()`](#column-Asset) | column kind | An Asset handle field. |
| [`Sound()`](#column-Sound) | column kind | A Sound handle field. |
| [`Font()`](#column-Font) | column kind | A Font handle field (0 is none): a font og.load.font answered, kept for og.render.text or og.ui.configure. |
| [`Style()`](#column-Style) | column kind | A Style handle field (0 is none): a UI style og.load.style answered, kept for og.ui.configure. |
| [`LoadStatus`](#type-LoadStatus) | enum | Where a load is. |
| [`GltfDesc`](#type-GltfDesc) | desc struct | How to load a glTF, as three.js GLTFLoader options where they apply. |
| [`GltfInfo`](#type-GltfInfo) | struct | What a loaded glTF holds. |
| [`BundleInfo`](#type-BundleInfo) | struct | An open cooked bundle. |
| [`FontInfo`](#type-FontInfo) | struct | A loaded font's metrics, in em units unless stated. |
| [`SoundInfo`](#type-SoundInfo) | struct | A loaded sound. |
| [`TextureArrayInfo`](#type-TextureArrayInfo) | struct | What og.load.textureArray uploaded. |
| [`og.Model`](#record-Model) | component | Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one. |
| [`og.load.bundle`](#fn-bundle) | function | Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready. |
| [`og.load.bundleInfo`](#fn-bundleInfo) | function | What an open bundle holds. |
| [`og.load.defaultBundle`](#fn-defaultBundle) | function | The game's own bundle, opened before the first tick from `cooked/`. |
| [`og.load.gltf`](#fn-gltf) | function | Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once. |
| [`og.load.scene`](#fn-scene) | function | Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning. |
| [`og.load.prefab`](#fn-prefab) | function | Load a cooked prefab (an .ogt of `type prefab`: one entity tree authored once and placed many times, docs/design.md §2.5). world.instantiate spawns it as it spawns a scene, its root under the desc's root entity, each entity unnamed (place a prefab many times without name clashes) and carrying an og.PrefabLink. A scene places one with a `prefab` line in an entity, and the cook flattens it there. |
| [`og.PrefabLink`](#record-PrefabLink) | component | Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. The engine writes it as the entity spawns and never changes it; it is world state like any component (snapshots, restores, the hash). A world that never placed a prefab does not register it, so its saves and hash are as before. |
| [`og.load.font`](#fn-font) | function | Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font. |
| [`og.load.fontInfo`](#fn-fontInfo) | function | A loaded font's metrics. |
| [`og.load.sound`](#fn-sound) | function | Load a cooked sound for the audio module. Sounds are `.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` files under assets/ (and procedural `.ogt` presets): og cook decodes each to 16-bit PCM and encodes it to QOA, so every format plays, streams and loops the same. The asset id keeps the source's extension (`audio/theme.mp3`). |
| [`og.load.lut`](#fn-lut) | function | A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle. |
| [`og.load.environment`](#fn-environment) | function | A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path. |
| [`og.load.emitter`](#fn-emitter) | function | A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (og-render §2 gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off \| life <cycles> \| fps <n> \| random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path. |
| [`og.load.shader`](#fn-shader) | function | A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots; og-render §10), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path. |
| [`og.load.post`](#fn-post) | function | A custom post pass for PostEffects.custom, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float\|vec2\|vec3\|vec4\|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour (og-render §10 has the contract). The cook compiles it for every back end. Loaded once per path. |
| [`og.load.vector`](#fn-vector) | function | A cooked vector for og.ui's VectorFill, from an `.ogt` of `type vector`: `source` an SVG (or `svg` inline) of paths, basic shapes, groups and transforms, filled and stroked with colours or linear and radial gradients, and `tolerance` (curves flattened to within it, view-box units). The cook cuts it into triangles whose edges the UI pass antialiases at any size; anything outside the subset is refused by name. Loaded once per path. |
| [`og.load.uiMaterial`](#fn-uiMaterial) | function | A custom UI material for og.ui's LayerDesc, from a cooked `.ogt` of `type ui-material`: `code` or `source` (a `.wgsl`) defining `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, which returns the composite's straight-alpha linear colour at m.uv (0 to 1 over the layer; m.pixel, m.size in CSS px, m.time the UI clock), reading the layer with layer(uv) and layer_texel(), and `param <name> float\|vec2\|vec3\|vec4\|color <default...>` lines (at most four). The cook compiles it for every back end. Loaded once per path. |
| [`og.load.texture`](#fn-texture) | function | Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load. |
| [`og.load.textureArray`](#fn-textureArray) | function | Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color\|normal\|linear\|pixel` and one `layer "<file>"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array. |
| [`og.load.style`](#fn-style) | function | Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 2 (docs/ui.md §12, what `og style upgrade` writes from version 1 and every built-in style is) adds the tokens (rarity, team, damage and chart colours each with its cue, the type scale per breakpoint, the space, size, radius and stroke scales, elevation, the shape language, motion springs and eases, sounds and haptics per UI event) and dresses parts by selector (`dress "Button.base[pressed]" { … }`); the tokens are carried, not yet drawn from. Version 1 still loads. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`. |
| [`og.load.document`](#fn-document) | function | Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props; the grammar is og-ui's README §Documents) into the world's UI, where og.ui.screen, og.ui.push and og.ui.component name it by its `name` line; its timelines are defined as `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept; `og dev` does this for an edited document. |
| [`og.load.look`](#fn-look) | function | Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path. |
| [`og.load.lookInfo`](#fn-lookInfo) | function | What a loaded look holds and sets. |
| [`og.load.palette`](#fn-palette) | function | A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle. |
| [`og.load.soundInfo`](#fn-soundInfo) | function | A loaded sound's format and length. |
| [`og.load.status`](#fn-status) | function | Where a load is. |
| [`og.load.error`](#fn-error) | function | Why a load failed. |
| [`og.load.gltfInfo`](#fn-gltfInfo) | function | What a ready glTF holds. |
| [`og.load.animation`](#fn-animation) | function | Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles. |
| [`og.load.rig`](#fn-rig) | function | Load a cooked humanoid map (an `.ogt` of `type rig`: a naming scheme and bones named outright) for og.anim.retarget and og.anim.humanoid. Loading the same path again answers the same handle. |
| [`og.load.flipbook`](#fn-flipbook) | function | Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles. |
| [`og.load.tilemap`](#fn-tilemap) | function | Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles. |
| [`og.load.video`](#fn-video) | function | Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color\|pixel`, `audio`, and for a stream `stream true`, `keyframe`, `crf`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. A clip's frames are uploaded as one 2D-array texture; a stream's units are fetched (on the web, from now: it is not part of the startup bundle) and decoded ahead of its players. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles. |
| [`og.load.videoInfo`](#fn-videoInfo) | function | A loaded video's size, frames, rate, length, texture and sound. |
| [`og.load.timeline`](#fn-timeline) | function | Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel. |
| [`og.load.rig2d`](#fn-rig2d) | function | Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles. |
| [`LoadBudget`](#type-LoadBudget) | desc struct | The limits an uncooked load (og.load.imageData, og.load.gltfData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's headers before anything is decoded, and a load over one is refused naming the limit, the value and the fix. |
| [`ImageKind`](#type-ImageKind) | enum | How an uncooked image's texels are read, as a cooked texture's `kind`. |
| [`GltfDataDesc`](#type-GltfDataDesc) | desc struct | How og.load.gltfData reads a model. |
| [`ImageDataDesc`](#type-ImageDataDesc) | desc struct | How og.load.imageData reads an image. |
| [`ImageInfo`](#type-ImageInfo) | struct | What a ready uncooked image holds. |
| [`LoadCompletion`](#type-LoadCompletion) | struct | When an uncooked load was put in place: the journal a replay applies again (og.load.completions). |
| [`og.load.imageData`](#fn-imageData) | function | Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are read from its header and checked against the budget before it is decoded. On the web the browser decodes it off the engine's thread (createImageBitmap in the engine's Worker), and no decoder is in the wasm; natively a worker thread decodes it. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why). |
| [`og.load.gltfData`](#fn-gltfData) | function | Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs, that a player made or a generator wrote. At the call its JSON is read and its size, its triangles (every triangle primitive of every mesh) and each image's header are checked against the budget before anything is decoded. Then its images decode off the engine's thread, as og.load.imageData's do, while its primitives are converted at the tick boundaries, one at a time within LoadBudget.frameMs on the web. The handle is pending until the model is in place, then ready: world.instantiate spawns it (from a system, which cannot instantiate, insert og.Model { asset } on an entity it spawns) and og.load.gltfInfo describes it, as for a cooked glTF. Or it is failed, and og.load.error says why. |
| [`og.load.release`](#fn-release) | function | Free an uncooked asset (og.load.imageData, og.load.gltfData): a load still pending is dropped (it never completes, so it is never journaled), and a ready one's textures and meshes are emptied under their handles, so what still names them draws nothing and nothing breaks, while their memory on the GPU and their copies on the CPU go (the probe's memory report and `dump_memory` show both). An og.Model of a released model draws nothing; a sprite or material of a released texture draws one clear texel. The handle then answers failed, naming the release. Physics shapes built from its geometries are world state and stay. Releasing again does nothing. |
| [`og.load.imageInfo`](#fn-imageInfo) | function | A ready uncooked image's texture and size. |
| [`og.load.completions`](#fn-completions) | function | The world's journal of uncooked loads put in place, oldest first: what a replay hands og.load.replayCompletions to put each in place at the same tick. og replay --record keeps it in the trace. |
| [`og.load.replayCompletions`](#fn-replayCompletions) | function | Put this world's uncooked loads in place at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits for it. Call it before the first tick; og replay --compare does. A load the journal names as failed fails at its tick whatever this host made of it. |
| [`og.AssetReady`](#record-AssetReady) | event | A load completed. |
| [`og.AssetFailed`](#record-AssetFailed) | event | A load failed. |

## Functions

<a id="fn-bundle"></a>

### `og.load.bundle`

Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready.

```ts
og.load.bundle(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The cooked directory, relative to the project (or the page), such as `dlc/cooked`. |

**Returns** `Result<Asset>`: The bundle's handle: pending until its manifest is read, then ready or failed.

Binding symbol `og_load_bundle` · publish builds: kept

```ogt
fn bundle {
  doc "Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready."
  param world World "The world that owns the asset."
  param path String "The cooked directory, relative to the project (or the page), such as `dlc/cooked`."
  returns Result.Asset "The bundle's handle: pending until its manifest is read, then ready or failed."
}
```

<sub>`api.ogt` line 7262</sub>

<a id="fn-bundleInfo"></a>

### `og.load.bundleInfo`

What an open bundle holds.

```ts
og.load.bundleInfo(world: World, asset: Asset): Result<BundleInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`. |

**Returns** `Result<BundleInfo>`: Refused for a pending, failed or non-bundle asset.

Binding symbol `og_load_bundle_info` · publish builds: kept

```ogt
fn bundleInfo {
  doc "What an open bundle holds."
  param world World "The world that owns the asset."
  param asset Asset "A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`."
  returns Result.BundleInfo "Refused for a pending, failed or non-bundle asset."
}
```

<sub>`api.ogt` line 7269</sub>

<a id="fn-defaultBundle"></a>

### `og.load.defaultBundle`

The game's own bundle, opened before the first tick from `cooked/`.

```ts
og.load.defaultBundle(world: World): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |

**Returns** `Result<Asset>`: Refused when cooked/ was missing or unreadable (run og cook); the reason names the path.

Binding symbol `og_load_default_bundle` · publish builds: kept

```ogt
fn defaultBundle {
  doc "The game's own bundle, opened before the first tick from `cooked/`."
  param world World "The world that owns the asset."
  returns Result.Asset "Refused when cooked/ was missing or unreadable (run og cook); the reason names the path."
}
```

<sub>`api.ogt` line 7276</sub>

<a id="fn-gltf"></a>

### `og.load.gltf`

Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.

A MagicaVoxel `.vox` cooks to a glTF and loads here too: greedy-meshed, flat-shaded, its palette as vertex colours, z-up turned y-up, each model about its centre (MagicaVoxel's pivot, with a scene graph or without). Its import settings (`models/hut.vox.ogt`, `type import`) size and place it: `model { voxScale 0.1 voxOrigin bottom }` is 0.1 m a voxel (default 1) and the lowest voxel on y = 0 (default `center`, the editor's origin). A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap the frames' models, played by og.AnimationPlayer as any skinned clip: `voxFps 8` (default 10) and `voxClip walk 0 3` (frames 0 to 3; default one clip `frames` of the file's playback range, else of every frame); a node's `_loop` repeats its keys as the editor plays them.

```ts
og.load.gltf(world: World, path: string, desc?: GltfDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The asset id, such as `models/ship.glb`. |
| `desc` | `GltfDesc` | yes | Scene and shadow options. |

**Returns** `Result<Asset>`: The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle).

Binding symbol `og_load_gltf` · publish builds: kept

```ogt
fn gltf {
  doc "Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once."
  doc "A MagicaVoxel `.vox` cooks to a glTF and loads here too: greedy-meshed, flat-shaded, its palette as vertex colours, z-up turned y-up, each model about its centre (MagicaVoxel's pivot, with a scene graph or without). Its import settings (`models/hut.vox.ogt`, `type import`) size and place it: `model { voxScale 0.1 voxOrigin bottom }` is 0.1 m a voxel (default 1) and the lowest voxel on y = 0 (default `center`, the editor's origin). A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap the frames' models, played by og.AnimationPlayer as any skinned clip: `voxFps 8` (default 10) and `voxClip walk 0 3` (frames 0 to 3; default one clip `frames` of the file's playback range, else of every frame); a node's `_loop` repeats its keys as the editor plays them."
  param world World "The world that owns the asset."
  param path String "The asset id, such as `models/ship.glb`."
  param desc GltfDesc "Scene and shadow options."
  returns Result.Asset "The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle)."
}
```

<sub>`api.ogt` line 7282</sub>

<a id="fn-scene"></a>

### `og.load.scene`

Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning.

```ts
og.load.scene(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The scene's asset id, such as `scenes/main.ogt`. |

**Returns** `Result<Asset>`.

Binding symbol `og_load_scene` · publish builds: kept

```ogt
fn scene {
  doc "Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning."
  param world World "The world that owns the asset."
  param path String "The scene's asset id, such as `scenes/main.ogt`."
  returns Result.Asset
}
```

<sub>`api.ogt` line 7291</sub>

<a id="fn-prefab"></a>

### `og.load.prefab`

Load a cooked prefab (an .ogt of `type prefab`: one entity tree authored once and placed many times, docs/design.md §2.5). world.instantiate spawns it as it spawns a scene, its root under the desc's root entity, each entity unnamed (place a prefab many times without name clashes) and carrying an og.PrefabLink. A scene places one with a `prefab` line in an entity, and the cook flattens it there.

```ts
og.load.prefab(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The prefab's asset id, such as `prefabs/crate.ogt`. |

**Returns** `Result<Asset>`: The prefab, or why not (not cooked, or not a prefab).

Binding symbol `og_load_prefab` · publish builds: kept

```ogt
fn prefab {
  doc "Load a cooked prefab (an .ogt of `type prefab`: one entity tree authored once and placed many times, docs/design.md §2.5). world.instantiate spawns it as it spawns a scene, its root under the desc's root entity, each entity unnamed (place a prefab many times without name clashes) and carrying an og.PrefabLink. A scene places one with a `prefab` line in an entity, and the cook flattens it there."
  param world World "The world that owns the asset."
  param path String "The prefab's asset id, such as `prefabs/crate.ogt`."
  returns Result.Asset "The prefab, or why not (not cooked, or not a prefab)."
}
```

<sub>`api.ogt` line 7298</sub>

<a id="fn-font"></a>

### `og.load.font`

Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font.

```ts
og.load.font(world: World, path: string): Result<Font>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the font. |
| `path` | `string` | no | The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`. |

**Returns** `Result<Font>`.

Binding symbol `og_load_font` · publish builds: kept

```ogt
fn font {
  doc "Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font."
  param world World "The world that owns the font."
  param path String "The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`."
  returns Result.Font
}
```

<sub>`api.ogt` line 7318</sub>

<a id="fn-fontInfo"></a>

### `og.load.fontInfo`

A loaded font's metrics.

```ts
og.load.fontInfo(world: World, font: Font): Result<FontInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the font. |
| `font` | `Font` | no | The font. |

**Returns** `Result<FontInfo>`: Refused for a dead font.

Binding symbol `og_load_font_info` · publish builds: kept

```ogt
fn fontInfo {
  doc "A loaded font's metrics."
  param world World "The world that owns the font."
  param font Font "The font."
  returns Result.FontInfo "Refused for a dead font."
}
```

<sub>`api.ogt` line 7325</sub>

<a id="fn-sound"></a>

### `og.load.sound`

Load a cooked sound for the audio module. Sounds are `.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` files under assets/ (and procedural `.ogt` presets): og cook decodes each to 16-bit PCM and encodes it to QOA, so every format plays, streams and loops the same. The asset id keeps the source's extension (`audio/theme.mp3`).

A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its import settings, `<file>.ogt` (`theme.mp3.ogt`; chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group.

```ts
og.load.sound(world: World, path: string): Result<Sound>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `path` | `string` | no | The sound's asset id, such as `audio/blip.wav`. |

**Returns** `Result<Sound>`.

Binding symbol `og_load_sound` · publish builds: kept

```ogt
fn sound {
  doc "Load a cooked sound for the audio module. Sounds are `.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` files under assets/ (and procedural `.ogt` presets): og cook decodes each to 16-bit PCM and encodes it to QOA, so every format plays, streams and loops the same. The asset id keeps the source's extension (`audio/theme.mp3`)."
  doc "A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its import settings, `<file>.ogt` (`theme.mp3.ogt`; chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group."
  param world World "The world that owns the sound."
  param path String "The sound's asset id, such as `audio/blip.wav`."
  returns Result.Sound
}
```

<sub>`api.ogt` line 7332</sub>

<a id="fn-lut"></a>

### `og.load.lut`

A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle.

```ts
og.load.lut(world: World, path: string): Result<Lut>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `luts/warm.cube`. |

**Returns** `Result<Lut>`: Refused for a path not in the bundle (the reason says to run og cook).

Binding symbol `og_load_lut` · publish builds: kept

```ogt
fn lut {
  doc "A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `luts/warm.cube`."
  returns Result.Lut "Refused for a path not in the bundle (the reason says to run og cook)."
}
```

<sub>`api.ogt` line 7340</sub>

<a id="fn-environment"></a>

### `og.load.environment`

A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.

```ts
og.load.environment(world: World, path: string): Result<EnvironmentMap>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `env/studio.ogt`. |

**Returns** `Result<EnvironmentMap>`: Refused for a path not in the bundle (the reason says to run og cook) or not an environment map.

Binding symbol `og_load_environment` · publish builds: kept

```ogt
fn environment {
  doc "A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `env/studio.ogt`."
  returns Result.EnvironmentMap "Refused for a path not in the bundle (the reason says to run og cook) or not an environment map."
}
```

<sub>`api.ogt` line 7347</sub>

<a id="fn-emitter"></a>

### `og.load.emitter`

A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (og-render §2 gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path.

```ts
og.load.emitter(world: World, path: string): Result<ParticleEmitter>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `fx/sparks.ogt`. |

**Returns** `Result<ParticleEmitter>`: Refused for a path not in the bundle, or a texture or texture array it cannot load.

Binding symbol `og_load_emitter` · publish builds: kept

```ogt
fn emitter {
  doc "A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (og-render §2 gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array \"<id>\" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/sparks.ogt`."
  returns Result.ParticleEmitter "Refused for a path not in the bundle, or a texture or texture array it cannot load."
}
```

<sub>`api.ogt` line 7354</sub>

<a id="fn-shader"></a>

### `og.load.shader`

A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots; og-render §10), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path.

```ts
og.load.shader(world: World, path: string): Result<Shader>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `shaders/dissolve.ogt`. |

**Returns** `Result<Shader>`: Refused for a path not in the bundle or not a shader.

Binding symbol `og_load_shader` · publish builds: kept

```ogt
fn shader {
  doc "A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots; og-render §10), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `shaders/dissolve.ogt`."
  returns Result.Shader "Refused for a path not in the bundle or not a shader."
}
```

<sub>`api.ogt` line 7361</sub>

<a id="fn-post"></a>

### `og.load.post`

A custom post pass for PostEffects.custom, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour (og-render §10 has the contract). The cook compiles it for every back end. Loaded once per path.

```ts
og.load.post(world: World, path: string): Result<Post>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `post/crt.ogt`. |

**Returns** `Result<Post>`: Refused for a path not in the bundle or not a post pass.

Binding symbol `og_load_post` · publish builds: kept

```ogt
fn post {
  doc "A custom post pass for PostEffects.custom, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour (og-render §10 has the contract). The cook compiles it for every back end. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `post/crt.ogt`."
  returns Result.Post "Refused for a path not in the bundle or not a post pass."
}
```

<sub>`api.ogt` line 7368</sub>

<a id="fn-vector"></a>

### `og.load.vector`

A cooked vector for og.ui's VectorFill, from an `.ogt` of `type vector`: `source` an SVG (or `svg` inline) of paths, basic shapes, groups and transforms, filled and stroked with colours or linear and radial gradients, and `tolerance` (curves flattened to within it, view-box units). The cook cuts it into triangles whose edges the UI pass antialiases at any size; anything outside the subset is refused by name. Loaded once per path.

```ts
og.load.vector(world: World, path: string): Result<VectorArt>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `ui/emblem.ogt`. |

**Returns** `Result<VectorArt>`: Refused for a path not in the bundle or not a vector.

Binding symbol `og_load_vector` · publish builds: kept

```ogt
fn vector {
  doc "A cooked vector for og.ui's VectorFill, from an `.ogt` of `type vector`: `source` an SVG (or `svg` inline) of paths, basic shapes, groups and transforms, filled and stroked with colours or linear and radial gradients, and `tolerance` (curves flattened to within it, view-box units). The cook cuts it into triangles whose edges the UI pass antialiases at any size; anything outside the subset is refused by name. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `ui/emblem.ogt`."
  returns Result.VectorArt "Refused for a path not in the bundle or not a vector."
}
```

<sub>`api.ogt` line 7375</sub>

<a id="fn-uiMaterial"></a>

### `og.load.uiMaterial`

A custom UI material for og.ui's LayerDesc, from a cooked `.ogt` of `type ui-material`: `code` or `source` (a `.wgsl`) defining `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, which returns the composite's straight-alpha linear colour at m.uv (0 to 1 over the layer; m.pixel, m.size in CSS px, m.time the UI clock), reading the layer with layer(uv) and layer_texel(), and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most four). The cook compiles it for every back end. Loaded once per path.

```ts
og.load.uiMaterial(world: World, path: string): Result<UiMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `ui/duotone.ogt`. |

**Returns** `Result<UiMaterial>`: Refused for a path not in the bundle or not a UI material.

Binding symbol `og_load_ui_material` · publish builds: kept

```ogt
fn uiMaterial {
  doc "A custom UI material for og.ui's LayerDesc, from a cooked `.ogt` of `type ui-material`: `code` or `source` (a `.wgsl`) defining `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, which returns the composite's straight-alpha linear colour at m.uv (0 to 1 over the layer; m.pixel, m.size in CSS px, m.time the UI clock), reading the layer with layer(uv) and layer_texel(), and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most four). The cook compiles it for every back end. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `ui/duotone.ogt`."
  returns Result.UiMaterial "Refused for a path not in the bundle or not a UI material."
}
```

<sub>`api.ogt` line 7382</sub>

<a id="fn-texture"></a>

### `og.load.texture`

Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load.

Sources are PNG, JPEG or KTX2 files under assets/ (`texture { kind color|normal|linear|pixel }` in `<file>.ogt`). A KTX2's `KTXorientation` and `KTXswizzle` are honoured: one that is not `rd` and `rgba` is decoded, flipped to rows top to bottom and swizzled, and cooked again; one that is (Basis ETC1S or UASTC, or RGBA8, of a colour or linear kind) is taken as it is.

```ts
og.load.texture(world: World, path: string): Result<Texture>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the texture. |
| `path` | `string` | no | The texture's asset id, such as `textures/stone.png`. |

**Returns** `Result<Texture>`.

Binding symbol `og_load_texture` · publish builds: kept

```ogt
fn texture {
  doc "Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load."
  doc "Sources are PNG, JPEG or KTX2 files under assets/ (`texture { kind color|normal|linear|pixel }` in `<file>.ogt`). A KTX2's `KTXorientation` and `KTXswizzle` are honoured: one that is not `rd` and `rgba` is decoded, flipped to rows top to bottom and swizzled, and cooked again; one that is (Basis ETC1S or UASTC, or RGBA8, of a colour or linear kind) is taken as it is."
  param world World "The world whose renderer owns the texture."
  param path String "The texture's asset id, such as `textures/stone.png`."
  returns Result.Texture
}
```

<sub>`api.ogt` line 7389</sub>

<a id="fn-textureArray"></a>

### `og.load.textureArray`

Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color|normal|linear|pixel` and one `layer "<file>"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array.

SpriteDesc.array (world and screen sprites), og.Sprite.array and 2D particle emitters (an emitter's `array` line) draw from its layers, one draw per array whatever layer each shows, with glow and the additive blend; an `array true` flipbook sheet cooks its frames into one (materials and terrain layers do not sample arrays yet). TextureArrayInfo and the frame's texture bytes report it. One in a streaming group's pack loads once its group is resident (refused before, naming the group); when the group unloads the renderer lets it go, og.render.sprite of it is refused and og.Sprite and particles of it draw nothing until the group is resident again, when it comes back under the same handle.

```ts
og.load.textureArray(world: World, path: string): Result<TextureArrayInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the array. |
| `path` | `string` | no | The array's asset id, such as `tiles/ground.ogt`. |

**Returns** `Result<TextureArrayInfo>`: Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree.

Binding symbol `og_load_texture_array` · publish builds: kept

```ogt
fn textureArray {
  doc "Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color|normal|linear|pixel` and one `layer \"<file>\"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array."
  doc "SpriteDesc.array (world and screen sprites), og.Sprite.array and 2D particle emitters (an emitter's `array` line) draw from its layers, one draw per array whatever layer each shows, with glow and the additive blend; an `array true` flipbook sheet cooks its frames into one (materials and terrain layers do not sample arrays yet). TextureArrayInfo and the frame's texture bytes report it. One in a streaming group's pack loads once its group is resident (refused before, naming the group); when the group unloads the renderer lets it go, og.render.sprite of it is refused and og.Sprite and particles of it draw nothing until the group is resident again, when it comes back under the same handle."
  param world World "The world whose renderer owns the array."
  param path String "The array's asset id, such as `tiles/ground.ogt`."
  returns Result.TextureArrayInfo "Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree."
}
```

<sub>`api.ogt` line 7397</sub>

<a id="fn-style"></a>

### `og.load.style`

Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 2 (docs/ui.md §12, what `og style upgrade` writes from version 1 and every built-in style is) adds the tokens (rarity, team, damage and chart colours each with its cue, the type scale per breakpoint, the space, size, radius and stroke scales, elevation, the shape language, motion springs and eases, sounds and haptics per UI event) and dresses parts by selector (`dress "Button.base[pressed]" { … }`); the tokens are carried, not yet drawn from. Version 1 still loads. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`.

```ts
og.load.style(world: World, path: string): Result<Style>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`. |

**Returns** `Result<Style>`: Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load.

Binding symbol `og_load_style` · publish builds: kept

```ogt
fn style {
  doc "Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 2 (docs/ui.md §12, what `og style upgrade` writes from version 1 and every built-in style is) adds the tokens (rarity, team, damage and chart colours each with its cue, the type scale per breakpoint, the space, size, radius and stroke scales, elevation, the shape language, motion springs and eases, sounds and haptics per UI event) and dresses parts by selector (`dress \"Button.base[pressed]\" { … }`); the tokens are carried, not yet drawn from. Version 1 still loads. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`."
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`."
  returns Result.Style "Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 7405</sub>

<a id="fn-document"></a>

### `og.load.document`

Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props; the grammar is og-ui's README §Documents) into the world's UI, where og.ui.screen, og.ui.push and og.ui.component name it by its `name` line; its timelines are defined as `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept; `og dev` does this for an edited document.

```ts
og.load.document(world: World, path: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The asset id, such as `ui/shop.ogt`. |

**Returns** `Result<void>`: Refused for a path not in the bundle or a document that does not parse (the reason names the line).

Binding symbol `og_load_document` · publish builds: kept

```ogt
fn document {
  doc "Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props; the grammar is og-ui's README §Documents) into the world's UI, where og.ui.screen, og.ui.push and og.ui.component name it by its `name` line; its timelines are defined as `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept; `og dev` does this for an edited document."
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `ui/shop.ogt`."
  returns Result.Void "Refused for a path not in the bundle or a document that does not parse (the reason names the line)."
}
```

<sub>`api.ogt` line 7412</sub>

<a id="fn-look"></a>

### `og.load.look`

Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path.

```ts
og.load.look(world: World, path: string): Result<Look>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `looks/main.ogt`. |

**Returns** `Result<Look>`: Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load.

Binding symbol `og_load_look` · publish builds: kept

```ogt
fn look {
  doc "Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion, ArtCamera) with that resource's fields by name, an asset id where a field is a handle (`lut \"luts/warm.cube\"`, `palette \"palettes/dusk.hex\"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `looks/main.ogt`."
  returns Result.Look "Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 7419</sub>

<a id="fn-lookInfo"></a>

### `og.load.lookInfo`

What a loaded look holds and sets.

```ts
og.load.lookInfo(world: World, look: Look): Result<LookInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the look. |
| `look` | `Look` | no | The look. |

**Returns** `Result<LookInfo>`: Refused for a look not loaded in this world.

Binding symbol `og_load_look_info` · publish builds: kept

```ogt
fn lookInfo {
  doc "What a loaded look holds and sets."
  param world World "The world that owns the look."
  param look Look "The look."
  returns Result.LookInfo "Refused for a look not loaded in this world."
}
```

<sub>`api.ogt` line 7426</sub>

<a id="fn-palette"></a>

### `og.load.palette`

A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle.

```ts
og.load.palette(world: World, path: string): Result<Palette>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `palettes/dusk.hex`. |

**Returns** `Result<Palette>`: Refused for a path not in the bundle (the reason says to run og cook).

Binding symbol `og_load_palette` · publish builds: kept

```ogt
fn palette {
  doc "A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `palettes/dusk.hex`."
  returns Result.Palette "Refused for a path not in the bundle (the reason says to run og cook)."
}
```

<sub>`api.ogt` line 7433</sub>

<a id="fn-soundInfo"></a>

### `og.load.soundInfo`

A loaded sound's format and length.

```ts
og.load.soundInfo(world: World, sound: Sound): Result<SoundInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `sound` | `Sound` | no | The sound. |

**Returns** `Result<SoundInfo>`: Refused for a dead sound.

Binding symbol `og_load_sound_info` · publish builds: kept

```ogt
fn soundInfo {
  doc "A loaded sound's format and length."
  param world World "The world that owns the sound."
  param sound Sound "The sound."
  returns Result.SoundInfo "Refused for a dead sound."
}
```

<sub>`api.ogt` line 7440</sub>

<a id="fn-status"></a>

### `og.load.status`

Where a load is.

```ts
og.load.status(world: World, asset: Asset): LoadStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | The handle. |

**Returns** `LoadStatus`.

Binding symbol `og_load_status` · publish builds: kept

```ogt
fn status {
  doc "Where a load is."
  param world World "The world that owns the asset."
  param asset Asset "The handle."
  returns LoadStatus
}
```

<sub>`api.ogt` line 7447</sub>

<a id="fn-error"></a>

### `og.load.error`

Why a load failed.

```ts
og.load.error(world: World, asset: Asset): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | The handle. |

**Returns** `string | null`: The reason, naming the fix; null unless the load failed.

Binding symbol `og_load_error` · publish builds: kept

```ogt
fn error {
  doc "Why a load failed."
  param world World "The world that owns the asset."
  param asset Asset "The handle."
  returns Option.String "The reason, naming the fix; null unless the load failed."
}
```

<sub>`api.ogt` line 7454</sub>

<a id="fn-gltfInfo"></a>

### `og.load.gltfInfo`

What a ready glTF holds.

```ts
og.load.gltfInfo(world: World, asset: Asset): Result<GltfInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A ready glTF handle. |

**Returns** `Result<GltfInfo>`: Refused for a pending, failed or non-glTF asset.

Binding symbol `og_load_gltf_info` · publish builds: kept

```ogt
fn gltfInfo {
  doc "What a ready glTF holds."
  param world World "The world that owns the asset."
  param asset Asset "A ready glTF handle."
  returns Result.GltfInfo "Refused for a pending, failed or non-glTF asset."
}
```

<sub>`api.ogt` line 7461</sub>

<a id="fn-animation"></a>

### `og.load.animation`

Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles.

Clips are cooked as f32 samples, or with `animation { compress true tolerance 0.0001 }` in the model's import settings as quantised keys (rotations as 48-bit smallest-three, translations, scales and weights as 16 bits over each channel's range) with every frame dropped that interpolation rebuilds within the tolerance; either way every frame is rebuilt here and the clip plays as f32. Morph targets are cooked sparse (the vertices each moves).

```ts
og.load.animation(world: World, path: string): Result<Animation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the assets. |
| `path` | `string` | no | The model's asset id, such as `models/walker.glb` (a `#anim` suffix is accepted). |

**Returns** `Result<Animation>`: Refused for an uncooked path, a model the cook wrote no `#anim` blob for (no skins, clips or morph targets), a cook stamp mismatch or an invalid rig.

Binding symbol `og_load_animation` · publish builds: kept

```ogt
fn animation {
  doc "Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles."
  doc "Clips are cooked as f32 samples, or with `animation { compress true tolerance 0.0001 }` in the model's import settings as quantised keys (rotations as 48-bit smallest-three, translations, scales and weights as 16 bits over each channel's range) with every frame dropped that interpolation rebuilds within the tolerance; either way every frame is rebuilt here and the clip plays as f32. Morph targets are cooked sparse (the vertices each moves)."
  param world World "The world whose animation owns the assets."
  param path String "The model's asset id, such as `models/walker.glb` (a `#anim` suffix is accepted)."
  returns Result.Animation "Refused for an uncooked path, a model the cook wrote no `#anim` blob for (no skins, clips or morph targets), a cook stamp mismatch or an invalid rig."
}
```

<sub>`api.ogt` line 7468</sub>

<a id="fn-rig"></a>

### `og.load.rig`

Load a cooked humanoid map (an `.ogt` of `type rig`: a naming scheme and bones named outright) for og.anim.retarget and og.anim.humanoid. Loading the same path again answers the same handle.

```ts
og.load.rig(world: World, path: string): Result<Rig>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation uses it. |
| `path` | `string` | no | The rig's asset id, such as `rigs/strider.ogt`. |

**Returns** `Result<Rig>`: Refused for a path not in the bundle (run og cook) or a cook stamp mismatch.

Binding symbol `og_load_rig` · publish builds: kept

```ogt
fn rig {
  doc "Load a cooked humanoid map (an `.ogt` of `type rig`: a naming scheme and bones named outright) for og.anim.retarget and og.anim.humanoid. Loading the same path again answers the same handle."
  param world World "The world whose animation uses it."
  param path String "The rig's asset id, such as `rigs/strider.ogt`."
  returns Result.Rig "Refused for a path not in the bundle (run og cook) or a cook stamp mismatch."
}
```

<sub>`api.ogt` line 7476</sub>

<a id="fn-flipbook"></a>

### `og.load.flipbook`

Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles.

```ts
og.load.flipbook(world: World, path: string): Result<FlipbookInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The sheet's asset id, such as `sprites/hero.ogt`. |

**Returns** `Result<FlipbookInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_flipbook` · publish builds: kept

```ogt
fn flipbook {
  doc "Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The sheet's asset id, such as `sprites/hero.ogt`."
  returns Result.FlipbookInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>`api.ogt` line 7483</sub>

<a id="fn-tilemap"></a>

### `og.load.tilemap`

Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles.

```ts
og.load.tilemap(world: World, path: string): Result<TilemapInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer and tilemaps own it. |
| `path` | `string` | no | The map's asset id, such as `maps/room.tmj`. |

**Returns** `Result<TilemapInfo>`: Refused for a path not in the bundle, a missing tileset or texture, or a terrain its tileset lacks.

Binding symbol `og_load_tilemap` · publish builds: kept

```ogt
fn tilemap {
  doc "Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles."
  param world World "The world whose renderer and tilemaps own it."
  param path String "The map's asset id, such as `maps/room.tmj`."
  returns Result.TilemapInfo "Refused for a path not in the bundle, a missing tileset or texture, or a terrain its tileset lacks."
}
```

<sub>`api.ogt` line 7490</sub>

<a id="fn-video"></a>

### `og.load.video`

Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color|pixel`, `audio`, and for a stream `stream true`, `keyframe`, `crf`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. A clip's frames are uploaded as one 2D-array texture; a stream's units are fetched (on the web, from now: it is not part of the startup bundle) and decoded ahead of its players. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles.

```ts
og.load.video(world: World, path: string): Result<VideoInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer and audio own it. |
| `path` | `string` | no | The video's asset id: the `.ogt`, such as `clips/intro.ogt`. |

**Returns** `Result<VideoInfo>`: Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video.

Binding symbol `og_load_video` · publish builds: kept

```ogt
fn video {
  doc "Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color|pixel`, `audio`, and for a stream `stream true`, `keyframe`, `crf`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. A clip's frames are uploaded as one 2D-array texture; a stream's units are fetched (on the web, from now: it is not part of the startup bundle) and decoded ahead of its players. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles."
  param world World "The world whose renderer and audio own it."
  param path String "The video's asset id: the `.ogt`, such as `clips/intro.ogt`."
  returns Result.VideoInfo "Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video."
}
```

<sub>`api.ogt` line 7497</sub>

<a id="fn-videoInfo"></a>

### `og.load.videoInfo`

A loaded video's size, frames, rate, length, texture and sound.

```ts
og.load.videoInfo(world: World, video: Video): Result<VideoInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that loaded it. |
| `video` | `Video` | no | The video. |

**Returns** `Result<VideoInfo>`: Refused for a handle og.load.video did not answer.

Binding symbol `og_load_video_info` · publish builds: kept

```ogt
fn videoInfo {
  doc "A loaded video's size, frames, rate, length, texture and sound."
  param world World "The world that loaded it."
  param video Video "The video."
  returns Result.VideoInfo "Refused for a handle og.load.video did not answer."
}
```

<sub>`api.ogt` line 7504</sub>

<a id="fn-timeline"></a>

### `og.load.timeline`

Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel.

```ts
og.load.timeline(world: World, path: string): Result<TimelineAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The timeline's asset id, such as `timelines/intro.ogt`. |

**Returns** `Result<TimelineAsset>`: Refused for a path not in the bundle (run og cook) or a file that is not a timeline.

Binding symbol `og_load_timeline` · publish builds: kept

```ogt
fn timeline {
  doc "Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel."
  param world World "The world."
  param path String "The timeline's asset id, such as `timelines/intro.ogt`."
  returns Result.TimelineAsset "Refused for a path not in the bundle (run og cook) or a file that is not a timeline."
}
```

<sub>`api.ogt` line 7511</sub>

<a id="fn-rig2d"></a>

### `og.load.rig2d`

Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles.

```ts
og.load.rig2d(world: World, path: string): Result<Rig2DInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The rig's asset id, such as `rigs/hero.ogt`. |

**Returns** `Result<Rig2DInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_rig2d` · publish builds: kept

```ogt
fn rig2d {
  doc "Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The rig's asset id, such as `rigs/hero.ogt`."
  returns Result.Rig2DInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>`api.ogt` line 7518</sub>

<a id="fn-imageData"></a>

### `og.load.imageData`

Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are read from its header and checked against the budget before it is decoded. On the web the browser decodes it off the engine's thread (createImageBitmap in the engine's Worker), and no decoder is in the wasm; natively a worker thread decodes it. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why).

Completion is host input, applied at a tick boundary and journaled like a key press: natively a load is in place before the tick after the call; on the web it lands at the first boundary after the browser has decoded it, and og.load.completions records that tick, so a replay (og replay --compare, og.load.replayCompletions) puts it in place at the same tick. In a lockstep or rollback room content made by one player is not on the others' machines, and the completion is local to each session (an asset's status is not world state and is in no snapshot or hash): send the bytes with og.platform.roomSend (at most 64 KiB a message), let each peer load them, and let the shared simulation use the content only from a tick the peers agree on through their inputs (og.net.submit), never from og.load.status.

```ts
og.load.imageData(world: World, name: string, bytes: Uint8Array, desc?: ImageDataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `name` | `string` | no | A name for it in errors and dumps, such as `drawings/cat.png`. |
| `bytes` | `Uint8Array` | no | The file's bytes. |
| `desc` | `ImageDataDesc` | yes | How to read it and its budget. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: over the budget's bytes or textureSize, or not a PNG or JPEG.

Binding symbol `og_load_image_data` · publish builds: kept

```ogt
fn imageData {
  doc "Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. Its size, width and height are read from its header and checked against the budget before it is decoded. On the web the browser decodes it off the engine's thread (createImageBitmap in the engine's Worker), and no decoder is in the wasm; natively a worker thread decodes it. The handle is pending until the image is in place, then ready (og.load.imageInfo gives its texture) or failed (og.load.error says why)."
  doc "Completion is host input, applied at a tick boundary and journaled like a key press: natively a load is in place before the tick after the call; on the web it lands at the first boundary after the browser has decoded it, and og.load.completions records that tick, so a replay (og replay --compare, og.load.replayCompletions) puts it in place at the same tick. In a lockstep or rollback room content made by one player is not on the others' machines, and the completion is local to each session (an asset's status is not world state and is in no snapshot or hash): send the bytes with og.platform.roomSend (at most 64 KiB a message), let each peer load them, and let the shared simulation use the content only from a tick the peers agree on through their inputs (og.net.submit), never from og.load.status."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `drawings/cat.png`."
  param bytes Bytes "The file's bytes."
  param desc ImageDataDesc "How to read it and its budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes or textureSize, or not a PNG or JPEG."
}
```

<sub>`api.ogt` line 7582</sub>

<a id="fn-gltfData"></a>

### `og.load.gltfData`

Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs, that a player made or a generator wrote. At the call its JSON is read and its size, its triangles (every triangle primitive of every mesh) and each image's header are checked against the budget before anything is decoded. Then its images decode off the engine's thread, as og.load.imageData's do, while its primitives are converted at the tick boundaries, one at a time within LoadBudget.frameMs on the web. The handle is pending until the model is in place, then ready: world.instantiate spawns it (from a system, which cannot instantiate, insert og.Model { asset } on an entity it spawns) and og.load.gltfInfo describes it, as for a cooked glTF. Or it is failed, and og.load.error says why.

Read as the cook would, without what only the cook makes: the metallic-roughness materials (base colour, metallic-roughness, normal, occlusion and emissive maps, alpha mask, double-sided, KHR_materials_unlit, KHR_materials_emissive_strength), the node tree and scenes, vertex colours and tangents. A colour texture magnified nearest is drawn as pixel art. Skins and animations are skipped and counted, points and lines skipped; no levels of detail or raycast BVH are cooked (a raycast builds one when first needed), and the other KHR_materials extensions, KHR_texture_transform and KHR_lights_punctual are not read. Refused by name: Draco and meshopt compression, Basis, WebP and AVIF textures, sparse accessors, a buffer or image in another file (pack the model as one .glb) and any other required extension.

Completion is host input, applied at a tick boundary and journaled as og.load.imageData's is: natively the model is in place before the tick after the call; on the web the first boundary after its images are decoded and its primitives converted, recorded by og.load.completions, so a replay puts it in place at the same tick. In a lockstep or rollback room it is local to each session, as og.load.imageData's is: share the bytes with og.platform.roomSend and agree on the tick through the peers' inputs.

```ts
og.load.gltfData(world: World, name: string, bytes: Uint8Array, desc?: GltfDataDesc): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `name` | `string` | no | A name for it in errors and dumps, such as `creations/kite.glb`; its textures are `<name>#tex<n>`. |
| `bytes` | `Uint8Array` | no | The file's bytes. |
| `desc` | `GltfDataDesc` | yes | Its budget. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: over the budget's bytes, triangles or textureSize, not a glTF 2.0, an image that is not a PNG or JPEG, or something the reader refuses by name.

Binding symbol `og_load_gltf_data` · publish builds: kept

```ogt
fn gltfData {
  doc "Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs, that a player made or a generator wrote. At the call its JSON is read and its size, its triangles (every triangle primitive of every mesh) and each image's header are checked against the budget before anything is decoded. Then its images decode off the engine's thread, as og.load.imageData's do, while its primitives are converted at the tick boundaries, one at a time within LoadBudget.frameMs on the web. The handle is pending until the model is in place, then ready: world.instantiate spawns it (from a system, which cannot instantiate, insert og.Model { asset } on an entity it spawns) and og.load.gltfInfo describes it, as for a cooked glTF. Or it is failed, and og.load.error says why."
  doc "Read as the cook would, without what only the cook makes: the metallic-roughness materials (base colour, metallic-roughness, normal, occlusion and emissive maps, alpha mask, double-sided, KHR_materials_unlit, KHR_materials_emissive_strength), the node tree and scenes, vertex colours and tangents. A colour texture magnified nearest is drawn as pixel art. Skins and animations are skipped and counted, points and lines skipped; no levels of detail or raycast BVH are cooked (a raycast builds one when first needed), and the other KHR_materials extensions, KHR_texture_transform and KHR_lights_punctual are not read. Refused by name: Draco and meshopt compression, Basis, WebP and AVIF textures, sparse accessors, a buffer or image in another file (pack the model as one .glb) and any other required extension."
  doc "Completion is host input, applied at a tick boundary and journaled as og.load.imageData's is: natively the model is in place before the tick after the call; on the web the first boundary after its images are decoded and its primitives converted, recorded by og.load.completions, so a replay puts it in place at the same tick. In a lockstep or rollback room it is local to each session, as og.load.imageData's is: share the bytes with og.platform.roomSend and agree on the tick through the peers' inputs."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `creations/kite.glb`; its textures are `<name>#tex<n>`."
  param bytes Bytes "The file's bytes."
  param desc GltfDataDesc "Its budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes, triangles or textureSize, not a glTF 2.0, an image that is not a PNG or JPEG, or something the reader refuses by name."
}
```

<sub>`api.ogt` line 7592</sub>

<a id="fn-release"></a>

### `og.load.release`

Free an uncooked asset (og.load.imageData, og.load.gltfData): a load still pending is dropped (it never completes, so it is never journaled), and a ready one's textures and meshes are emptied under their handles, so what still names them draws nothing and nothing breaks, while their memory on the GPU and their copies on the CPU go (the probe's memory report and `dump_memory` show both). An og.Model of a released model draws nothing; a sprite or material of a released texture draws one clear texel. The handle then answers failed, naming the release. Physics shapes built from its geometries are world state and stay. Releasing again does nothing.

```ts
og.load.release(world: World, asset: Asset): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.imageData or og.load.gltfData answered. |

**Returns** `Result<void>`: Refused for a cooked asset, which stays with its bundle (a streamed one with its group).

Binding symbol `og_load_release` · publish builds: kept

```ogt
fn release {
  doc "Free an uncooked asset (og.load.imageData, og.load.gltfData): a load still pending is dropped (it never completes, so it is never journaled), and a ready one's textures and meshes are emptied under their handles, so what still names them draws nothing and nothing breaks, while their memory on the GPU and their copies on the CPU go (the probe's memory report and `dump_memory` show both). An og.Model of a released model draws nothing; a sprite or material of a released texture draws one clear texel. The handle then answers failed, naming the release. Physics shapes built from its geometries are world state and stay. Releasing again does nothing."
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.imageData or og.load.gltfData answered."
  returns Result.Void "Refused for a cooked asset, which stays with its bundle (a streamed one with its group)."
}
```

<sub>`api.ogt` line 7603</sub>

<a id="fn-imageInfo"></a>

### `og.load.imageInfo`

A ready uncooked image's texture and size.

```ts
og.load.imageInfo(world: World, asset: Asset): Result<ImageInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.imageData answered. |

**Returns** `Result<ImageInfo>`: Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset.

Binding symbol `og_load_image_info` · publish builds: kept

```ogt
fn imageInfo {
  doc "A ready uncooked image's texture and size."
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.imageData answered."
  returns Result.ImageInfo "Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset."
}
```

<sub>`api.ogt` line 7610</sub>

<a id="fn-completions"></a>

### `og.load.completions`

The world's journal of uncooked loads put in place, oldest first: what a replay hands og.load.replayCompletions to put each in place at the same tick. og replay --record keeps it in the trace.

```ts
og.load.completions(world: World): readonly LoadCompletion[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly LoadCompletion[]`.

Binding symbol `og_load_completions` · publish builds: kept

```ogt
fn completions {
  doc "The world's journal of uncooked loads put in place, oldest first: what a replay hands og.load.replayCompletions to put each in place at the same tick. og replay --record keeps it in the trace."
  param world World "The world."
  returns List.LoadCompletion
}
```

<sub>`api.ogt` line 7617</sub>

<a id="fn-replayCompletions"></a>

### `og.load.replayCompletions`

Put this world's uncooked loads in place at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits for it. Call it before the first tick; og replay --compare does. A load the journal names as failed fails at its tick whatever this host made of it.

```ts
og.load.replayCompletions(world: World, completions: readonly LoadCompletion[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `completions` | `readonly LoadCompletion[]` | no | A recorded journal. |

**Returns** `Result<void>`: Refused once a tick has run.

Binding symbol `og_load_replay_completions` · publish builds: kept

```ogt
fn replayCompletions {
  doc "Put this world's uncooked loads in place at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits for it. Call it before the first tick; og replay --compare does. A load the journal names as failed fails at its tick whatever this host made of it."
  param world World "The world."
  param completions List.LoadCompletion "A recorded journal."
  returns Result.Void "Refused once a tick has run."
}
```

<sub>`api.ogt` line 7623</sub>

## Components, resources and events

<a id="record-Model"></a>

### `og.Model` (component)

Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.

The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck.

Token `og.Model`, registered under the schema name `og.Model`; its value type is `Model`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The glTF asset. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour, 0 to 1 a channel, multiplied with every material's base colour on this instance only (white: as the glTF). |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash on this instance only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `castShadow` | `boolean` | `true` | — | — | — | Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's (a loaded model casts, as it always has); false takes this instance out of every shadow pass. world.instantiate of a glTF and a scene's `model` line set it from the load's GltfDesc.castShadow, unless the scene gives the entity a `Model` table (whose castShadow is on unless it says false). |
| `receiveShadow` | `boolean` | `true` | — | — | — | Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away. |
| `lodBias` | `number` | `1` | 0 … 16 | — | tunable | This instance's level of detail bias, multiplied with RenderSettings.lodBias: the cook simplifies every mesh into levels (a model's import settings, `model { lods 1 0.5 0.25 }` by default, the triangle fraction of each level; a glTF with MSFT_lod keeps its own levels) and measures each level's error against the full mesh, and the renderer draws the coarsest level whose error covers less than a pixel at the model's distance, with hysteresis so it does not flicker at the boundary. Above 1 keeps detail farther (a hero), below 1 drops it sooner (a crowd); 0 always draws the full mesh. dump_entity shows the level drawn. |
| `layers` | `number` | `1` | — | — | — | Three.js layers of every mesh of the model, as Mesh.layers. |
| `static` | `boolean` | `false` | — | — | — | Part of its scene's baked lighting: og bake lightmaps gives a static model lightmap UVs (xatlas, over its world-space triangles) and traces the light arriving at its surfaces from the scene into the scene's lightmap (`<scene>.lightmap.ogt` and `.ktx2` beside the scene). Once baked, the model draws with it (its ambient light from the lightmap instead of the sky; its lights' direct light, shadows and highlights still drawn every frame) for as long as the bake matches the scene. A scene entity says `static true` beside its `model` line; an unbaked static model draws as any model, and og doctor --project names the scene. |
| `variant` | `number` | `-1` | ≥ -1 | — | tunable | The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only (a car's paint, a chair's fabric picked at run time, one draw per material as before). -1 draws the glTF's own materials; an index past the model's variants does too. |

```ogt
component Model {
  doc "Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one."
  doc "The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck."
  field asset Asset "The glTF asset."
  field tint Color "A linear colour, 0 to 1 a channel, multiplied with every material's base colour on this instance only (white: as the glTF)." {
    default 1 1 1
  }
  field flash f32 "A hit flash on this instance only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out)." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field castShadow bool "Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's (a loaded model casts, as it always has); false takes this instance out of every shadow pass. world.instantiate of a glTF and a scene's `model` line set it from the load's GltfDesc.castShadow, unless the scene gives the entity a `Model` table (whose castShadow is on unless it says false)." {
    default true
  }
  field receiveShadow bool "Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table." {
    default true
  }
  field visible bool "Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away." {
    default true
  }
  field lodBias f32 "This instance's level of detail bias, multiplied with RenderSettings.lodBias: the cook simplifies every mesh into levels (a model's import settings, `model { lods 1 0.5 0.25 }` by default, the triangle fraction of each level; a glTF with MSFT_lod keeps its own levels) and measures each level's error against the full mesh, and the renderer draws the coarsest level whose error covers less than a pixel at the model's distance, with hysteresis so it does not flicker at the boundary. Above 1 keeps detail farther (a hero), below 1 drops it sooner (a crowd); 0 always draws the full mesh. dump_entity shows the level drawn." {
    default 1
    min 0
    max 16
    tunable true
  }
  field layers u32 "Three.js layers of every mesh of the model, as Mesh.layers." {
    default 1
  }
  field static bool "Part of its scene's baked lighting: og bake lightmaps gives a static model lightmap UVs (xatlas, over its world-space triangles) and traces the light arriving at its surfaces from the scene into the scene's lightmap (`<scene>.lightmap.ogt` and `.ktx2` beside the scene). Once baked, the model draws with it (its ambient light from the lightmap instead of the sky; its lights' direct light, shadows and highlights still drawn every frame) for as long as the bake matches the scene. A scene entity says `static true` beside its `model` line; an unbaked static model draws as any model, and og doctor --project names the scene."
  field variant i32 "The glTF material variant this instance draws (KHR_materials_variants), an index into GltfInfo.variants: every mesh the variant maps takes its material, the others keep theirs, on this instance only (a car's paint, a chair's fabric picked at run time, one draw per material as before). -1 draws the glTF's own materials; an index past the model's variants does too." {
    default -1
    min -1
    tunable true
  }
}
```

<sub>`api.ogt` line 7221</sub>

<a id="record-PrefabLink"></a>

### `og.PrefabLink` (component)

Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. The engine writes it as the entity spawns and never changes it; it is world state like any component (snapshots, restores, the hash). A world that never placed a prefab does not register it, so its saves and hash are as before.

Token `og.PrefabLink`, registered under the schema name `og.PrefabLink`; its value type is `PrefabLink`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prefab` | `Asset` | `0` | — | — | read only | The prefab placed (a prefab nested in another links to the outermost one placed). Loaded as og.load.prefab loads it. |
| `node` | `number` | `0` | — | — | read only | The entity's node: its index in the prefab's own entity table, depth first with parents first (0 is the prefab's root). |
| `root` | `Entity` | `0` | — | — | read only | The instance's root entity (the root's own, for the root). |

```ogt
component PrefabLink {
  doc "Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. The engine writes it as the entity spawns and never changes it; it is world state like any component (snapshots, restores, the hash). A world that never placed a prefab does not register it, so its saves and hash are as before."
  field prefab Asset "The prefab placed (a prefab nested in another links to the outermost one placed). Loaded as og.load.prefab loads it." {
    readonly true
  }
  field node u32 "The entity's node: its index in the prefab's own entity table, depth first with parents first (0 is the prefab's root)." {
    readonly true
  }
  field root Entity "The instance's root entity (the root's own, for the root)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 7305</sub>

<a id="record-AssetReady"></a>

### `og.AssetReady` (event)

A load completed.

Token `og.AssetReady`, registered under the schema name `og.AssetReady`; its value type is `AssetReady`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The handle. |
| `path` | `string` | `""` | — | — | — | The path it was loaded from. |

```ogt
event AssetReady {
  doc "A load completed."
  field asset Asset "The handle."
  field path String "The path it was loaded from."
}
```

<sub>`api.ogt` line 7630</sub>

<a id="record-AssetFailed"></a>

### `og.AssetFailed` (event)

A load failed.

Token `og.AssetFailed`, registered under the schema name `og.AssetFailed`; its value type is `AssetFailed`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The handle. |
| `path` | `string` | `""` | — | — | — | The path it was loaded from. |
| `reason` | `string` | `""` | — | — | — | Why, naming the fix. |

```ogt
event AssetFailed {
  doc "A load failed."
  field asset Asset "The handle."
  field path String "The path it was loaded from."
  field reason String "Why, naming the fix."
}
```

<sub>`api.ogt` line 7636</sub>

## Types

<a id="type-Asset"></a>

### `Asset` (handle)

A loaded bundle, scene or glTF asset.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Asset "A loaded bundle, scene or glTF asset."
```

<sub>`api.ogt` line 7108</sub>

<a id="type-Font"></a>

### `Font` (handle)

A cooked MSDF font: its glyph table and one or more atlas pages.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Font "A cooked MSDF font: its glyph table and one or more atlas pages."
```

<sub>`api.ogt` line 7109</sub>

<a id="type-Sound"></a>

### `Sound` (handle)

A cooked sound: QOA-encoded PCM the audio module plays.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Sound "A cooked sound: QOA-encoded PCM the audio module plays."
```

<sub>`api.ogt` line 7110</sub>

<a id="type-Style"></a>

### `Style` (handle)

A cooked UI style: its values, atlas, fonts, icons and sounds loaded with it, for og.ui.configure.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Style "A cooked UI style: its values, atlas, fonts, icons and sounds loaded with it, for og.ui.configure."
```

<sub>`api.ogt` line 7111</sub>

<a id="type-TextureArray"></a>

### `TextureArray` (handle)

A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TextureArray "A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray)."
```

<sub>`api.ogt` line 7112</sub>

<a id="type-LoadStatus"></a>

### `LoadStatus` (enum)

Where a load is.

A string: one of the variants below. The zero value is `"pending"`.

| Variant | Doc |
|---|---|
| `"pending"` | Still loading. |
| `"ready"` | Loaded; it can be instantiated. |
| `"failed"` | Failed; og.load.error gives the reason. |

```ogt
enum LoadStatus {
  doc "Where a load is."
  variant pending "Still loading."
  variant ready "Loaded; it can be instantiated."
  variant failed "Failed; og.load.error gives the reason."
}
```

<sub>`api.ogt` line 7149</sub>

<a id="type-GltfDesc"></a>

### `GltfDesc` (desc struct)

How to load a glTF, as three.js GLTFLoader options where they apply.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `number` | `0` | — | — | — | Which glTF scene to instantiate. |
| `castShadow` | `boolean` | `true` | — | — | — | Whether the models instantiated from it cast shadows: world.instantiate of the handle and a scene's `model` line give their og.Model this castShadow. On when omitted, so a model loaded without a desc casts as it always has; false turns it off for the model (og.Model.castShadow turns it off for one instance). The flags belong to the asset: another og.load.gltf of the same path answers the same handle and sets them again, for the models instantiated after it. |
| `receiveShadow` | `boolean` | `true` | — | — | — | Whether the models instantiated from it receive shadows (og.Model.receiveShadow, as castShadow): on when omitted; false lights them as if unshadowed by every light. |

```ogt
struct GltfDesc {
  doc "How to load a glTF, as three.js GLTFLoader options where they apply."
  desc true
  field scene u32 "Which glTF scene to instantiate."
  field castShadow bool "Whether the models instantiated from it cast shadows: world.instantiate of the handle and a scene's `model` line give their og.Model this castShadow. On when omitted, so a model loaded without a desc casts as it always has; false turns it off for the model (og.Model.castShadow turns it off for one instance). The flags belong to the asset: another og.load.gltf of the same path answers the same handle and sets them again, for the models instantiated after it." {
    default true
  }
  field receiveShadow bool "Whether the models instantiated from it receive shadows (og.Model.receiveShadow, as castShadow): on when omitted; false lights them as if unshadowed by every light." {
    default true
  }
}
```

<sub>`api.ogt` line 7156</sub>

<a id="type-GltfInfo"></a>

### `GltfInfo` (struct)

What a loaded glTF holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scenes` | `number` | `0` | — | — | — | How many scenes it has. |
| `geometries` | `readonly Geometry[]` | `[]` | — | — | — | Its geometries, in glTF mesh-primitive order. |
| `materials` | `readonly Material[]` | `[]` | — | — | — | Its materials, in glTF order, then the default material. |
| `variants` | `readonly string[]` | `[]` | — | — | — | Its material variants' names (KHR_materials_variants), in glTF order: og.Model.variant draws an instance with one, by its index here. Empty for a model without variants. |
| `lights` | `number` | `0` | — | — | — | How many punctual lights it has (KHR_lights_punctual) in its default scene: world.instantiate and a scene's `model` line spawn each as a child entity of the model's, with an og.PointLight, og.SpotLight or og.DirectionalLight at its node's place (glTF's candela as the radiance at one metre, its range as distance, a light of infinite range ending where its radiance falls to 0.01, a spot's outer cone as angle and 1 - inner / outer as penumbra). |
| `animations` | `readonly string[]` | `[]` | — | — | — | Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name. |

```ogt
struct GltfInfo {
  doc "What a loaded glTF holds."
  field scenes u32 "How many scenes it has."
  field geometries List.Geometry "Its geometries, in glTF mesh-primitive order."
  field materials List.Material "Its materials, in glTF order, then the default material."
  field variants List.String "Its material variants' names (KHR_materials_variants), in glTF order: og.Model.variant draws an instance with one, by its index here. Empty for a model without variants."
  field lights u32 "How many punctual lights it has (KHR_lights_punctual) in its default scene: world.instantiate and a scene's `model` line spawn each as a child entity of the model's, with an og.PointLight, og.SpotLight or og.DirectionalLight at its node's place (glTF's candela as the radiance at one metre, its range as distance, a light of infinite range ending where its radiance falls to 0.01, a spot's outer cone as angle and 1 - inner / outer as penumbra)."
  field animations List.String "Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name."
}
```

<sub>`api.ogt` line 7168</sub>

<a id="type-BundleInfo"></a>

### `BundleInfo` (struct)

An open cooked bundle.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | The cooked directory it was opened from. |
| `assets` | `number` | `0` | — | — | — | Source assets it holds. |
| `outputs` | `number` | `0` | — | — | — | Cooked outputs (blobs) it holds. |

```ogt
struct BundleInfo {
  doc "An open cooked bundle."
  field path String "The cooked directory it was opened from."
  field assets u32 "Source assets it holds."
  field outputs u32 "Cooked outputs (blobs) it holds."
}
```

<sub>`api.ogt` line 7178</sub>

<a id="type-FontInfo"></a>

### `FontInfo` (struct)

A loaded font's metrics, in em units unless stated.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `glyphs` | `number` | `0` | — | — | — | Glyphs cooked into it. |
| `pages` | `number` | `0` | — | — | — | Atlas pages; the renderer draws one page per pass with one shared layout. |
| `emPx` | `number` | `0` | — | — | — | Atlas pixels per em. |
| `pxRange` | `number` | `0` | — | — | — | The distance range in atlas pixels. |
| `ascender` | `number` | `0` | — | — | — | Ascender above the baseline. |
| `descender` | `number` | `0` | — | — | — | Descender below the baseline (negative). |
| `lineHeight` | `number` | `0` | — | — | — | Ascender minus descender plus the line gap. |
| `icons` | `number` | `0` | — | — | — | The icons of an icon set (an `.ogt` of `type icons`, SVGs cooked like a font for UiDesc.icons.font); 0 for a text font. |

```ogt
struct FontInfo {
  doc "A loaded font's metrics, in em units unless stated."
  field glyphs u32 "Glyphs cooked into it."
  field pages u32 "Atlas pages; the renderer draws one page per pass with one shared layout."
  field emPx u32 "Atlas pixels per em."
  field pxRange f32 "The distance range in atlas pixels."
  field ascender f32 "Ascender above the baseline."
  field descender f32 "Descender below the baseline (negative)."
  field lineHeight f32 "Ascender minus descender plus the line gap."
  field icons u32 "The icons of an icon set (an `.ogt` of `type icons`, SVGs cooked like a font for UiDesc.icons.font); 0 for a text font."
}
```

<sub>`api.ogt` line 7185</sub>

<a id="type-SoundInfo"></a>

### `SoundInfo` (struct)

A loaded sound.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rate` | `number` | `0` | — | `hz` | — | Samples per second. |
| `channels` | `number` | `0` | — | — | — | Interleaved channels. |
| `frames` | `number` | `0` | — | — | — | Samples per channel. |
| `seconds` | `number` | `0` | — | `s` | — | Its length. |
| `chunks` | `number` | `0` | — | — | — | The chunks it was cooked in (`sound { stream true }` in its import settings, or a QOA file above 1 MiB), the first included; 0 when it was cooked whole. A chunked sound is never resident whole: the engine reads its chunks from about half a second behind each playhead to four seconds ahead (on the web, fetched like a streaming group's pack) and drops the rest. |

```ogt
struct SoundInfo {
  doc "A loaded sound."
  field rate u32 "Samples per second." {
    unit hz
  }
  field channels u32 "Interleaved channels."
  field frames u32 "Samples per channel."
  field seconds f32 "Its length." {
    unit s
  }
  field chunks u32 "The chunks it was cooked in (`sound { stream true }` in its import settings, or a QOA file above 1 MiB), the first included; 0 when it was cooked whole. A chunked sound is never resident whole: the engine reads its chunks from about half a second behind each playhead to four seconds ahead (on the web, fetched like a streaming group's pack) and drops the rest."
}
```

<sub>`api.ogt` line 7197</sub>

<a id="type-TextureArrayInfo"></a>

### `TextureArrayInfo` (struct)

What og.load.textureArray uploaded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `array` | `TextureArray` | `0` | — | — | — | The array. |
| `width` | `number` | `0` | — | — | — | Each layer's width in texels. |
| `height` | `number` | `0` | — | — | — | Each layer's height in texels. |
| `layers` | `number` | `0` | — | — | — | Its layers, in the order the file lists them. |
| `levels` | `number` | `0` | — | — | — | Mip levels of each layer. |
| `format` | `string` | `""` | — | — | — | The encoding on the GPU: `rgba8`, or the block format the device takes for its kind (`bc7`, `bc5`, `astc4x4`, `etc2`, `eacrg11`). |
| `bytes` | `number` | `0` | — | — | — | Bytes it takes on the GPU, every level of every layer (counted in the frame's texture bytes). |

```ogt
struct TextureArrayInfo {
  doc "What og.load.textureArray uploaded."
  field array TextureArray "The array."
  field width u32 "Each layer's width in texels."
  field height u32 "Each layer's height in texels."
  field layers u32 "Its layers, in the order the file lists them."
  field levels u32 "Mip levels of each layer."
  field format String "The encoding on the GPU: `rgba8`, or the block format the device takes for its kind (`bc7`, `bc5`, `astc4x4`, `etc2`, `eacrg11`)."
  field bytes f64 "Bytes it takes on the GPU, every level of every layer (counted in the frame's texture bytes)."
}
```

<sub>`api.ogt` line 7210</sub>

<a id="type-LoadBudget"></a>

### `LoadBudget` (desc struct)

The limits an uncooked load (og.load.imageData, og.load.gltfData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's headers before anything is decoded, and a load over one is refused naming the limit, the value and the fix.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bytes` | `number` | `33554432` | ≥ 0 | `bytes` | — | The most bytes the source may have. |
| `triangles` | `number` | `500000` | — | — | — | The most triangles a glTF's meshes may have, every primitive of every mesh. |
| `textureSize` | `number` | `4096` | — | — | — | The widest or tallest an image may be, in texels (the device's own limit lowers it further). |
| `frameMs` | `number` | `2` | ≥ 0 | `ms` | — | How long a frame may spend on the engine's thread putting finished loads in place, in milliseconds; at least one load is put in place a frame, and decoding never runs on the engine's thread on the web. |

```ogt
struct LoadBudget {
  doc "The limits an uncooked load (og.load.imageData, og.load.gltfData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's headers before anything is decoded, and a load over one is refused naming the limit, the value and the fix."
  desc true
  field bytes f64 "The most bytes the source may have." {
    default 33554432
    min 0
    unit bytes
  }
  field triangles u32 "The most triangles a glTF's meshes may have, every primitive of every mesh." {
    default 500000
  }
  field textureSize u32 "The widest or tallest an image may be, in texels (the device's own limit lowers it further)." {
    default 4096
  }
  field frameMs f32 "How long a frame may spend on the engine's thread putting finished loads in place, in milliseconds; at least one load is put in place a frame, and decoding never runs on the engine's thread on the web." {
    default 2
    min 0
    unit ms
  }
}
```

<sub>`api.ogt` line 7525</sub>

<a id="type-ImageKind"></a>

### `ImageKind` (enum)

How an uncooked image's texels are read, as a cooked texture's `kind`.

A string: one of the variants below. The zero value is `"color"`.

| Variant | Doc |
|---|---|
| `"color"` | sRGB colour (albedo, UI art), sampled linear. |
| `"linear"` | Data stored linearly (masks, roughness). |
| `"pixel"` | sRGB colour sampled nearest (pixel art). |

```ogt
enum ImageKind {
  doc "How an uncooked image's texels are read, as a cooked texture's `kind`."
  variant color "sRGB colour (albedo, UI art), sampled linear."
  variant linear "Data stored linearly (masks, roughness)."
  variant pixel "sRGB colour sampled nearest (pixel art)."
}
```

<sub>`api.ogt` line 7546</sub>

<a id="type-GltfDataDesc"></a>

### `GltfDataDesc` (desc struct)

How og.load.gltfData reads a model.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to. |

```ogt
struct GltfDataDesc {
  doc "How og.load.gltfData reads a model."
  desc true
  field budget LoadBudget "The limits it is held to."
}
```

<sub>`api.ogt` line 7553</sub>

<a id="type-ImageDataDesc"></a>

### `ImageDataDesc` (desc struct)

How og.load.imageData reads an image.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ImageKind` | `"color"` | — | — | — | How its texels are read. |
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to. |

```ogt
struct ImageDataDesc {
  doc "How og.load.imageData reads an image."
  desc true
  field kind ImageKind "How its texels are read."
  field budget LoadBudget "The limits it is held to."
}
```

<sub>`api.ogt` line 7559</sub>

<a id="type-ImageInfo"></a>

### `ImageInfo` (struct)

What a ready uncooked image holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture: og.render.sprite, og.ui.image and material maps draw it like a cooked one. |
| `width` | `number` | `0` | — | — | — | Its width in texels. |
| `height` | `number` | `0` | — | — | — | Its height in texels. |

```ogt
struct ImageInfo {
  doc "What a ready uncooked image holds."
  field texture Texture "The texture: og.render.sprite, og.ui.image and material maps draw it like a cooked one."
  field width u32 "Its width in texels."
  field height u32 "Its height in texels."
}
```

<sub>`api.ogt` line 7566</sub>

<a id="type-LoadCompletion"></a>

### `LoadCompletion` (struct)

When an uncooked load was put in place: the journal a replay applies again (og.load.completions).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `request` | `number` | `0` | — | — | — | The load's number: 0 for the world's first uncooked load, then one more for each. |
| `tick` | `number` | `0` | — | `ticks` | — | The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed. |
| `ok` | `boolean` | `false` | — | — | — | Whether it was ready (else failed). |

```ogt
struct LoadCompletion {
  doc "When an uncooked load was put in place: the journal a replay applies again (og.load.completions)."
  field request u32 "The load's number: 0 for the world's first uncooked load, then one more for each."
  field tick f64 "The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed." {
    unit ticks
  }
  field ok bool "Whether it was ready (else failed)."
}
```

<sub>`api.ogt` line 7573</sub>

## Column kinds

Each is a factory, `TextureArray(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TextureArray"></a>`TextureArray` | `TextureArray` | `Uint32Array` | 2 | A TextureArray handle field (0 is none). |
| <a id="column-Asset"></a>`Asset` | `Asset` | `Uint32Array` | 2 | An Asset handle field. |
| <a id="column-Sound"></a>`Sound` | `Sound` | `Uint32Array` | 2 | A Sound handle field. |
| <a id="column-Font"></a>`Font` | `Font` | `Uint32Array` | 2 | A Font handle field (0 is none): a font og.load.font answered, kept for og.render.text or og.ui.configure. |
| <a id="column-Style"></a>`Style` | `Style` | `Uint32Array` | 2 | A Style handle field (0 is none): a UI style og.load.style answered, kept for og.ui.configure. |

```ogt
column TextureArray {
  doc "A TextureArray handle field (0 is none)."
  array Uint32Array
  stride 2
  value TextureArray
}
```

<sub>`api.ogt` line 7114</sub>

```ogt
column Asset {
  doc "An Asset handle field."
  array Uint32Array
  stride 2
  value Asset
}
```

<sub>`api.ogt` line 7121</sub>

```ogt
column Sound {
  doc "A Sound handle field."
  array Uint32Array
  stride 2
  value Sound
}
```

<sub>`api.ogt` line 7128</sub>

```ogt
column Font {
  doc "A Font handle field (0 is none): a font og.load.font answered, kept for og.render.text or og.ui.configure."
  array Uint32Array
  stride 2
  value Font
}
```

<sub>`api.ogt` line 7135</sub>

```ogt
column Style {
  doc "A Style handle field (0 is none): a UI style og.load.style answered, kept for og.ui.configure."
  array Uint32Array
  stride 2
  value Style
}
```

<sub>`api.ogt` line 7142</sub>
