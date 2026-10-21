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

<sub>`api.ogt` line 3350</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Asset`](#type-Asset) | handle | A loaded bundle, scene or glTF asset. |
| [`Font`](#type-Font) | handle | A cooked MSDF font: its glyph table and one or more atlas pages. |
| [`Sound`](#type-Sound) | handle | A cooked sound: QOA-encoded PCM the audio module plays. |
| [`Skin`](#type-Skin) | handle | A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure. |
| [`TextureArray`](#type-TextureArray) | handle | A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray). |
| [`TextureArray()`](#column-TextureArray) | column kind | A TextureArray handle field (0 is none). |
| [`Asset()`](#column-Asset) | column kind | An Asset handle field. |
| [`Sound()`](#column-Sound) | column kind | A Sound handle field. |
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
| [`og.load.font`](#fn-font) | function | Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font. |
| [`og.load.fontInfo`](#fn-fontInfo) | function | A loaded font's metrics. |
| [`og.load.sound`](#fn-sound) | function | Load a cooked sound for the audio module. |
| [`og.load.lut`](#fn-lut) | function | A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle. |
| [`og.load.environment`](#fn-environment) | function | A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path. |
| [`og.load.emitter`](#fn-emitter) | function | A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the og-game skill's Effects section gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off \| life <cycles> \| fps <n> \| random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path. |
| [`og.load.shader`](#fn-shader) | function | A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). Loaded once per path. |
| [`og.load.texture`](#fn-texture) | function | Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load. |
| [`og.load.textureArray`](#fn-textureArray) | function | Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color\|normal\|linear\|pixel` and one `layer "<file>"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array. |
| [`og.load.skin`](#fn-skin) | function | Load a cooked UI skin (an `.ogt` of `type skin`: the atlas and its nine-slice frames, ornaments, palette, radii, type ramp, fonts, icons, motion and sounds; the grammar is og-ui's `skin` module) for UiDesc.skin. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. |
| [`og.load.look`](#fn-look) | function | Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `skin` (a UI skin's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and skin it names are loaded with it. Loaded once per path. |
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

<sub>`api.ogt` line 3454</sub>

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

<sub>`api.ogt` line 3461</sub>

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

<sub>`api.ogt` line 3468</sub>

<a id="fn-gltf"></a>

### `og.load.gltf`

Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.

A MagicaVoxel `.vox` cooks to a glTF and loads here too: greedy-meshed, flat-shaded, its palette as vertex colours, z-up turned y-up, each model about its centre (MagicaVoxel's pivot, with a scene graph or without). Its import settings (`models/hut.vox.ogt`, `type import`) size and place it: `model { voxScale 0.1 voxOrigin bottom }` is 0.1 m a voxel (default 1) and the lowest voxel on y = 0 (default `center`, the editor's origin). A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap the frames' models, played by og.AnimationPlayer as any skinned clip: `voxFps 8` (default 10) and `voxClip walk 0 3` (frames 0 to 3; default one clip `frames` of every frame).

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
  doc "A MagicaVoxel `.vox` cooks to a glTF and loads here too: greedy-meshed, flat-shaded, its palette as vertex colours, z-up turned y-up, each model about its centre (MagicaVoxel's pivot, with a scene graph or without). Its import settings (`models/hut.vox.ogt`, `type import`) size and place it: `model { voxScale 0.1 voxOrigin bottom }` is 0.1 m a voxel (default 1) and the lowest voxel on y = 0 (default `center`, the editor's origin). A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap the frames' models, played by og.AnimationPlayer as any skinned clip: `voxFps 8` (default 10) and `voxClip walk 0 3` (frames 0 to 3; default one clip `frames` of every frame)."
  param world World "The world that owns the asset."
  param path String "The asset id, such as `models/ship.glb`."
  param desc GltfDesc "Scene and shadow options."
  returns Result.Asset "The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle)."
}
```

<sub>`api.ogt` line 3474</sub>

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

<sub>`api.ogt` line 3483</sub>

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

<sub>`api.ogt` line 3490</sub>

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

<sub>`api.ogt` line 3497</sub>

<a id="fn-sound"></a>

### `og.load.sound`

Load a cooked sound for the audio module.

A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its `.wav.ogt` import settings (chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group.

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
  doc "Load a cooked sound for the audio module."
  doc "A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its `.wav.ogt` import settings (chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group."
  param world World "The world that owns the sound."
  param path String "The sound's asset id, such as `audio/blip.wav`."
  returns Result.Sound
}
```

<sub>`api.ogt` line 3504</sub>

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

<sub>`api.ogt` line 3512</sub>

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

<sub>`api.ogt` line 3519</sub>

<a id="fn-emitter"></a>

### `og.load.emitter`

A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the og-game skill's Effects section gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path.

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
  doc "A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the og-game skill's Effects section gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array \"<id>\" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/sparks.ogt`."
  returns Result.ParticleEmitter "Refused for a path not in the bundle, or a texture or texture array it cannot load."
}
```

<sub>`api.ogt` line 3526</sub>

<a id="fn-shader"></a>

### `og.load.shader`

A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). Loaded once per path.

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
  doc "A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `shaders/dissolve.ogt`."
  returns Result.Shader "Refused for a path not in the bundle or not a shader."
}
```

<sub>`api.ogt` line 3533</sub>

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

<sub>`api.ogt` line 3540</sub>

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

<sub>`api.ogt` line 3548</sub>

<a id="fn-skin"></a>

### `og.load.skin`

Load a cooked UI skin (an `.ogt` of `type skin`: the atlas and its nine-slice frames, ornaments, palette, radii, type ramp, fonts, icons, motion and sounds; the grammar is og-ui's `skin` module) for UiDesc.skin. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path.

```ts
og.load.skin(world: World, path: string): Result<Skin>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The asset id, such as `skins/pixel/skin.ogt`. |

**Returns** `Result<Skin>`: Refused for a path not in the bundle, a skin that does not parse (the reason names the line) or an asset it names that cannot load.

Binding symbol `og_load_skin` · publish builds: kept

```ogt
fn skin {
  doc "Load a cooked UI skin (an `.ogt` of `type skin`: the atlas and its nine-slice frames, ornaments, palette, radii, type ramp, fonts, icons, motion and sounds; the grammar is og-ui's `skin` module) for UiDesc.skin. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path."
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `skins/pixel/skin.ogt`."
  returns Result.Skin "Refused for a path not in the bundle, a skin that does not parse (the reason names the line) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 3556</sub>

<a id="fn-look"></a>

### `og.load.look`

Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `skin` (a UI skin's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and skin it names are loaded with it. Loaded once per path.

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
  doc "Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `skin` (a UI skin's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment) with that resource's fields by name, an asset id where a field is a handle (`lut \"luts/warm.cube\"`, `palette \"palettes/dusk.hex\"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and skin it names are loaded with it. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `looks/main.ogt`."
  returns Result.Look "Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 3563</sub>

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

<sub>`api.ogt` line 3570</sub>

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

<sub>`api.ogt` line 3577</sub>

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

<sub>`api.ogt` line 3584</sub>

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

<sub>`api.ogt` line 3591</sub>

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

<sub>`api.ogt` line 3598</sub>

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

<sub>`api.ogt` line 3605</sub>

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

<sub>`api.ogt` line 3612</sub>

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

<sub>`api.ogt` line 3620</sub>

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

<sub>`api.ogt` line 3627</sub>

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

<sub>`api.ogt` line 3634</sub>

## Components, resources and events

<a id="record-Model"></a>

### `og.Model` (component)

Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.

Token `og.Model`, registered under the schema name `og.Model`; its value type is `Model`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The glTF asset. |

```ogt
component Model {
  doc "Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one."
  field asset Asset "The glTF asset."
}
```

<sub>`api.ogt` line 3449</sub>

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

<sub>`api.ogt` line 3641</sub>

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

<sub>`api.ogt` line 3647</sub>

## Types

<a id="type-Asset"></a>

### `Asset` (handle)

A loaded bundle, scene or glTF asset.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Asset "A loaded bundle, scene or glTF asset."
```

<sub>`api.ogt` line 3356</sub>

<a id="type-Font"></a>

### `Font` (handle)

A cooked MSDF font: its glyph table and one or more atlas pages.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Font "A cooked MSDF font: its glyph table and one or more atlas pages."
```

<sub>`api.ogt` line 3357</sub>

<a id="type-Sound"></a>

### `Sound` (handle)

A cooked sound: QOA-encoded PCM the audio module plays.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Sound "A cooked sound: QOA-encoded PCM the audio module plays."
```

<sub>`api.ogt` line 3358</sub>

<a id="type-Skin"></a>

### `Skin` (handle)

A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Skin "A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure."
```

<sub>`api.ogt` line 3359</sub>

<a id="type-TextureArray"></a>

### `TextureArray` (handle)

A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TextureArray "A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray)."
```

<sub>`api.ogt` line 3360</sub>

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

<sub>`api.ogt` line 3383</sub>

<a id="type-GltfDesc"></a>

### `GltfDesc` (desc struct)

How to load a glTF, as three.js GLTFLoader options where they apply.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `number` | `0` | — | — | — | Which glTF scene to instantiate. |
| `castShadow` | `boolean` | `false` | — | — | — | Set Mesh.castShadow on every mesh instantiated from it. |
| `receiveShadow` | `boolean` | `false` | — | — | — | Set Mesh.receiveShadow on every mesh instantiated from it. |

```ogt
struct GltfDesc {
  doc "How to load a glTF, as three.js GLTFLoader options where they apply."
  desc true
  field scene u32 "Which glTF scene to instantiate."
  field castShadow bool "Set Mesh.castShadow on every mesh instantiated from it."
  field receiveShadow bool "Set Mesh.receiveShadow on every mesh instantiated from it."
}
```

<sub>`api.ogt` line 3390</sub>

<a id="type-GltfInfo"></a>

### `GltfInfo` (struct)

What a loaded glTF holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scenes` | `number` | `0` | — | — | — | How many scenes it has. |
| `geometries` | `readonly Geometry[]` | `[]` | — | — | — | Its geometries, in glTF mesh-primitive order. |
| `materials` | `readonly Material[]` | `[]` | — | — | — | Its materials, in glTF order, then the default material. |
| `animations` | `readonly string[]` | `[]` | — | — | — | Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name. |

```ogt
struct GltfInfo {
  doc "What a loaded glTF holds."
  field scenes u32 "How many scenes it has."
  field geometries List.Geometry "Its geometries, in glTF mesh-primitive order."
  field materials List.Material "Its materials, in glTF order, then the default material."
  field animations List.String "Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name."
}
```

<sub>`api.ogt` line 3398</sub>

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

<sub>`api.ogt` line 3406</sub>

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

<sub>`api.ogt` line 3413</sub>

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

<sub>`api.ogt` line 3425</sub>

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

<sub>`api.ogt` line 3438</sub>

## Column kinds

Each is a factory, `TextureArray(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TextureArray"></a>`TextureArray` | `TextureArray` | `Uint32Array` | 2 | A TextureArray handle field (0 is none). |
| <a id="column-Asset"></a>`Asset` | `Asset` | `Uint32Array` | 2 | An Asset handle field. |
| <a id="column-Sound"></a>`Sound` | `Sound` | `Uint32Array` | 2 | A Sound handle field. |

```ogt
column TextureArray {
  doc "A TextureArray handle field (0 is none)."
  array Uint32Array
  stride 2
  value TextureArray
}
```

<sub>`api.ogt` line 3362</sub>

```ogt
column Asset {
  doc "An Asset handle field."
  array Uint32Array
  stride 2
  value Asset
}
```

<sub>`api.ogt` line 3369</sub>

```ogt
column Sound {
  doc "A Sound handle field."
  array Uint32Array
  stride 2
  value Sound
}
```

<sub>`api.ogt` line 3376</sub>
