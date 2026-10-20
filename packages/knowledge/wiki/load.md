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

<sub>`api.ogt` line 2909</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Asset`](#type-Asset) | handle | A loaded bundle, scene or glTF asset. |
| [`Font`](#type-Font) | handle | A cooked MSDF font: its glyph table and one or more atlas pages. |
| [`Sound`](#type-Sound) | handle | A cooked sound: QOA-encoded PCM the audio module plays. |
| [`Skin`](#type-Skin) | handle | A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure. |
| [`Asset()`](#column-Asset) | column kind | An Asset handle field. |
| [`Sound()`](#column-Sound) | column kind | A Sound handle field. |
| [`LoadStatus`](#type-LoadStatus) | enum | Where a load is. |
| [`GltfDesc`](#type-GltfDesc) | desc struct | How to load a glTF, as three.js GLTFLoader options where they apply. |
| [`GltfInfo`](#type-GltfInfo) | struct | What a loaded glTF holds. |
| [`BundleInfo`](#type-BundleInfo) | struct | An open cooked bundle. |
| [`FontInfo`](#type-FontInfo) | struct | A loaded font's metrics, in em units unless stated. |
| [`SoundInfo`](#type-SoundInfo) | struct | A loaded sound. |
| [`og.Model`](#record-Model) | component | Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one. |
| [`og.load.bundle`](#fn-bundle) | function | Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready. |
| [`og.load.bundleInfo`](#fn-bundleInfo) | function | What an open bundle holds. |
| [`og.load.defaultBundle`](#fn-defaultBundle) | function | The game's own bundle, opened before the first tick from `cooked/`. |
| [`og.load.gltf`](#fn-gltf) | function | Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once. |
| [`og.load.scene`](#fn-scene) | function | Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning. |
| [`og.load.font`](#fn-font) | function | Load a cooked MSDF font for og.render.text. |
| [`og.load.fontInfo`](#fn-fontInfo) | function | A loaded font's metrics. |
| [`og.load.sound`](#fn-sound) | function | Load a cooked sound for the audio module. |
| [`og.load.lut`](#fn-lut) | function | A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. |
| [`og.load.environment`](#fn-environment) | function | A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it), `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path. |
| [`og.load.emitter`](#fn-emitter) | function | A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the grammar is the `particles` page of the wiki's render module and og-cook's fx module): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. The textures it names are loaded with it. Loaded once per path. |
| [`og.load.shader`](#fn-shader) | function | A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). Loaded once per path. |
| [`og.load.texture`](#fn-texture) | function | Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load. |
| [`og.load.skin`](#fn-skin) | function | Load a cooked UI skin (an `.ogt` of `type skin`: the atlas and its nine-slice frames, ornaments, palette, radii, type ramp, fonts, icons, motion and sounds; the grammar is og-ui's `skin` module) for UiDesc.skin. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. |
| [`og.load.soundInfo`](#fn-soundInfo) | function | A loaded sound's format and length. |
| [`og.load.status`](#fn-status) | function | Where a load is. |
| [`og.load.error`](#fn-error) | function | Why a load failed. |
| [`og.load.gltfInfo`](#fn-gltfInfo) | function | What a ready glTF holds. |
| [`og.load.animation`](#fn-animation) | function | Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles. |
| [`og.load.rig`](#fn-rig) | function | Load a cooked humanoid map (an `.ogt` of `type rig`: a naming scheme and bones named outright) for og.anim.retarget and og.anim.humanoid. Loading the same path again answers the same handle. |
| [`og.load.flipbook`](#fn-flipbook) | function | Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. Loading the same path again answers the same handles. |
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

<sub>`api.ogt` line 2992</sub>

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

<sub>`api.ogt` line 2999</sub>

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

<sub>`api.ogt` line 3006</sub>

<a id="fn-gltf"></a>

### `og.load.gltf`

Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.

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
  param world World "The world that owns the asset."
  param path String "The asset id, such as `models/ship.glb`."
  param desc GltfDesc "Scene and shadow options."
  returns Result.Asset "The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle)."
}
```

<sub>`api.ogt` line 3012</sub>

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

<sub>`api.ogt` line 3020</sub>

<a id="fn-font"></a>

### `og.load.font`

Load a cooked MSDF font for og.render.text.

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
  doc "Load a cooked MSDF font for og.render.text."
  param world World "The world that owns the font."
  param path String "The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`."
  returns Result.Font
}
```

<sub>`api.ogt` line 3027</sub>

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

<sub>`api.ogt` line 3034</sub>

<a id="fn-sound"></a>

### `og.load.sound`

Load a cooked sound for the audio module.

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
  param world World "The world that owns the sound."
  param path String "The sound's asset id, such as `audio/blip.wav`."
  returns Result.Sound
}
```

<sub>`api.ogt` line 3041</sub>

<a id="fn-lut"></a>

### `og.load.lut`

A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path.

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
  doc "A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `luts/warm.cube`."
  returns Result.Lut "Refused for a path not in the bundle (the reason says to run og cook)."
}
```

<sub>`api.ogt` line 3048</sub>

<a id="fn-environment"></a>

### `og.load.environment`

A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it), `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.

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
  doc "A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it), `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `env/studio.ogt`."
  returns Result.EnvironmentMap "Refused for a path not in the bundle (the reason says to run og cook) or not an environment map."
}
```

<sub>`api.ogt` line 3055</sub>

<a id="fn-emitter"></a>

### `og.load.emitter`

A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the grammar is the `particles` page of the wiki's render module and og-cook's fx module): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. The textures it names are loaded with it. Loaded once per path.

```ts
og.load.emitter(world: World, path: string): Result<ParticleEmitter>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `fx/sparks.ogt`. |

**Returns** `Result<ParticleEmitter>`: Refused for a path not in the bundle or a texture it cannot load.

Binding symbol `og_load_emitter` · publish builds: kept

```ogt
fn emitter {
  doc "A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the grammar is the `particles` page of the wiki's render module and og-cook's fx module): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. The textures it names are loaded with it. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/sparks.ogt`."
  returns Result.ParticleEmitter "Refused for a path not in the bundle or a texture it cannot load."
}
```

<sub>`api.ogt` line 3062</sub>

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

<sub>`api.ogt` line 3069</sub>

<a id="fn-texture"></a>

### `og.load.texture`

Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load.

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
  param world World "The world whose renderer owns the texture."
  param path String "The texture's asset id, such as `textures/stone.png`."
  returns Result.Texture
}
```

<sub>`api.ogt` line 3076</sub>

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

<sub>`api.ogt` line 3083</sub>

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

<sub>`api.ogt` line 3090</sub>

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

<sub>`api.ogt` line 3097</sub>

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

<sub>`api.ogt` line 3104</sub>

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

<sub>`api.ogt` line 3111</sub>

<a id="fn-animation"></a>

### `og.load.animation`

Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles.

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
  param world World "The world whose animation owns the assets."
  param path String "The model's asset id, such as `models/walker.glb` (a `#anim` suffix is accepted)."
  returns Result.Animation "Refused for an uncooked path, a model the cook wrote no `#anim` blob for (no skins, clips or morph targets), a cook stamp mismatch or an invalid rig."
}
```

<sub>`api.ogt` line 3118</sub>

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

<sub>`api.ogt` line 3125</sub>

<a id="fn-flipbook"></a>

### `og.load.flipbook`

Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. Loading the same path again answers the same handles.

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
  doc "Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. Loading the same path again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The sheet's asset id, such as `sprites/hero.ogt`."
  returns Result.FlipbookInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>`api.ogt` line 3132</sub>

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

<sub>`api.ogt` line 3139</sub>

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

<sub>`api.ogt` line 2987</sub>

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

<sub>`api.ogt` line 3146</sub>

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

<sub>`api.ogt` line 3152</sub>

## Types

<a id="type-Asset"></a>

### `Asset` (handle)

A loaded bundle, scene or glTF asset.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Asset "A loaded bundle, scene or glTF asset."
```

<sub>`api.ogt` line 2915</sub>

<a id="type-Font"></a>

### `Font` (handle)

A cooked MSDF font: its glyph table and one or more atlas pages.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Font "A cooked MSDF font: its glyph table and one or more atlas pages."
```

<sub>`api.ogt` line 2916</sub>

<a id="type-Sound"></a>

### `Sound` (handle)

A cooked sound: QOA-encoded PCM the audio module plays.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Sound "A cooked sound: QOA-encoded PCM the audio module plays."
```

<sub>`api.ogt` line 2917</sub>

<a id="type-Skin"></a>

### `Skin` (handle)

A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Skin "A cooked UI skin: its atlas, fonts, icons and sounds loaded with it, for og.ui.configure."
```

<sub>`api.ogt` line 2918</sub>

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

<sub>`api.ogt` line 2934</sub>

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

<sub>`api.ogt` line 2941</sub>

<a id="type-GltfInfo"></a>

### `GltfInfo` (struct)

What a loaded glTF holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scenes` | `number` | `0` | — | — | — | How many scenes it has. |
| `geometries` | `readonly Geometry[]` | `[]` | — | — | — | Its geometries, in glTF mesh-primitive order. |
| `materials` | `readonly Material[]` | `[]` | — | — | — | Its materials, in glTF order, then the default material. |
| `animations` | `readonly string[]` | `[]` | — | — | — | Its animation clip names (the cook counts clips and does not cook them yet, so this is empty). |

```ogt
struct GltfInfo {
  doc "What a loaded glTF holds."
  field scenes u32 "How many scenes it has."
  field geometries List.Geometry "Its geometries, in glTF mesh-primitive order."
  field materials List.Material "Its materials, in glTF order, then the default material."
  field animations List.String "Its animation clip names (the cook counts clips and does not cook them yet, so this is empty)."
}
```

<sub>`api.ogt` line 2949</sub>

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

<sub>`api.ogt` line 2957</sub>

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
}
```

<sub>`api.ogt` line 2964</sub>

<a id="type-SoundInfo"></a>

### `SoundInfo` (struct)

A loaded sound.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rate` | `number` | `0` | — | `hz` | — | Samples per second. |
| `channels` | `number` | `0` | — | — | — | Interleaved channels. |
| `frames` | `number` | `0` | — | — | — | Samples per channel. |
| `seconds` | `number` | `0` | — | `s` | — | Its length. |

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
}
```

<sub>`api.ogt` line 2975</sub>

## Column kinds

Each is a factory, `Asset(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Asset"></a>`Asset` | `Asset` | `Uint32Array` | 2 | An Asset handle field. |
| <a id="column-Sound"></a>`Sound` | `Sound` | `Uint32Array` | 2 | A Sound handle field. |

```ogt
column Asset {
  doc "An Asset handle field."
  array Uint32Array
  stride 2
  value Asset
}
```

<sub>`api.ogt` line 2920</sub>

```ogt
column Sound {
  doc "A Sound handle field."
  array Uint32Array
  stride 2
  value Sound
}
```

<sub>`api.ogt` line 2927</sub>
