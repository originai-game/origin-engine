<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `load` module

Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.

Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.

The game's bundle (`cooked`) opens before GameDesc.scene spawns. Runtime manifests and scene blobs may still be fetching: keep the pending handle and query status before instantiating. Pending loads complete at tick boundaries.

See `loading §1`.

- **TypeScript**: functions are the namespace `og.load`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.load.gltf`, `og.load.font`, `og.load.emitter`, `og.load.texture`, `og.load.animation`, `og.load.rig`, `og.load.flipbook`, `og.load.tilemap`, `og.load.vfx`; 82 more are advanced (`og api show load --all`).
- **Index**: [every module and guide](index.md).

```ogt
module load {
  doc "Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/."
  doc "Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp."
  doc "The game's bundle (`cooked`) opens before GameDesc.scene spawns. Runtime manifests and scene blobs may still be fetching: keep the pending handle and query status before instantiating. Pending loads complete at tick boundaries."
  # declarations follow, each on its own page
```

<sub>Module `load` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`FieldData`](load/type-FieldData.md) | struct |  | Lossless sampled scalar/vector lattice from a TS file.field declaration; x fastest, then y, then z, channels interleaved. |
| [`og.load.field`](load/fn-field.md) | function | `(world: World, path: string): Result<FieldData>` | Read a cooked file.field lattice by its canonical module/content ID without allocating world or GPU handles. |
| [`Asset`](load/type-Asset.md) | handle |  | A loaded bundle, scene or glTF asset. |
| [`Font`](load/type-Font.md) | handle |  | A cooked font: its file, which text is laid out and rastered from at the size it is drawn, or an icon set. |
| [`Sound`](load/type-Sound.md) | handle |  | A cooked sound: QOA-encoded PCM the audio module plays. |
| [`Style`](load/type-Style.md) | handle |  | A cooked UI style: its values, atlas, fonts, icons and sounds loaded with it, for og.ui.configure. |
| [`TextureArray`](load/type-TextureArray.md) | handle |  | A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray). |
| [`TextureArray()`](#column-TextureArray) | column kind |  | A TextureArray handle field (0 is none). |
| [`Asset()`](#column-Asset) | column kind |  | An Asset handle field. |
| [`Sound()`](#column-Sound) | column kind |  | A Sound handle field. |
| [`Font()`](#column-Font) | column kind |  | A Font handle field (0 is none): a font og.load.font answered, kept for og.render.text or og.ui.configure. |
| [`Style()`](#column-Style) | column kind |  | A Style handle field (0 is none): a UI style og.load.style answered, kept for og.ui.configure. |
| [`LoadStatus`](load/type-LoadStatus.md) | enum |  | Where a load is. |
| [`GltfDesc`](load/type-GltfDesc.md) | desc struct |  | How to load a glTF, as three.js GLTFLoader options where they apply. |
| [`GltfPart`](load/type-GltfPart.md) | struct |  | One placed primitive in the model's default scene. |
| [`ModelShape`](load/type-ModelShape.md) | struct |  | A cooked model's shape in its own space (og.Model's frame before its og.Transform): the axes a principal-axis fit of its triangles finds, its length and its top surface along that length. |
| [`GltfInfo`](load/type-GltfInfo.md) | struct |  | What a loaded glTF holds. |
| [`BundleInfo`](load/type-BundleInfo.md) | struct |  | An open cooked bundle. |
| [`FontInfo`](load/type-FontInfo.md) | struct |  | A loaded font's metrics, in em units unless stated. |
| [`SoundInfo`](load/type-SoundInfo.md) | struct |  | A loaded sound. |
| [`TextureArrayInfo`](load/type-TextureArrayInfo.md) | struct |  | What og.load.textureArray uploaded. |
| [`og.Model`](load/record-Model.md) | component |  | Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. |
| [`og.load.bundle`](load/fn-bundle.md) | function | `(world: World, path: string): Result<Asset>` | Open another cooked bundle and register its streaming groups, including cell groups. |
| [`og.load.bundleInfo`](load/fn-bundleInfo.md) | function | `(world: World, asset: Asset): Result<BundleInfo>` | What an open bundle holds. |
| [`LoadProgress`](load/type-LoadProgress.md) | struct |  | How far a load has arrived (og.load.progress). |
| [`og.load.progress`](load/fn-progress.md) | function | `(world: World, asset: Asset): Result<LoadProgress>` | How far a load has arrived: a bundle og.load.bundle opened, or any handle (a cooked load answers ready at once). |
| [`og.load.defaultBundle`](load/fn-defaultBundle.md) | function | `(world: World): Result<Asset>` | The game's own bundle, opened before the first tick from `cooked/`. |
| [`og.load.gltf`](load/fn-gltf.md) | function | `(world: World, path: string, desc?: GltfDesc): Result<Asset>` | Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once. |
| [`og.load.scene`](load/fn-scene.md) | function | `(world: World, path: string): Result<Asset>` | Load a cooked scene (.ogt). |
| [`og.load.prefab`](load/fn-prefab.md) | function | `(world: World, path: string): Result<Asset>` | Load a cooked prefab (an .ogt of `type prefab`: one entity tree authored once and placed many times, docs/design.md §2.5). |
| [`og.PrefabLink`](load/record-PrefabLink.md) | component |  | Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. |
| [`og.load.font`](load/fn-font.md) | function | `(world: World, path: string): Result<Font>` | Load a cooked font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font. |
| [`og.load.fontInfo`](load/fn-fontInfo.md) | function | `(world: World, font: Font): Result<FontInfo>` | A loaded font's metrics. |
| [`og.load.sound`](load/fn-sound.md) | function | `(world: World, path: string): Result<Sound>` | Load a cooked sound for the audio module. |
| [`og.load.lut`](load/fn-lut.md) | function | `(world: World, path: string): Result<Lut>` | A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). |
| [`og.load.environment`](load/fn-environment.md) | function | `(world: World, path: string): Result<EnvironmentMap>` | A prefiltered environment map for Environment.map, from a TS file.environment declaration (an equirectangular HDR/EXR or sRGB image, a KTX2 cube map, or six faces). |
| [`EnvironmentSun`](load/type-EnvironmentSun.md) | struct |  | The sun a cooked environment took out of its lighting image, for the DirectionalLight that puts it back. |
| [`og.load.environmentSun`](load/fn-environmentSun.md) | function | `(world: World, map: EnvironmentMap, rotation?: number, intensity?: number): Result<EnvironmentSun>` | The sun a cooked environment map holds apart from its image (sun: 'extract' in file.environment): the direction, colour and intensity to give a DirectionalLight, so the sun is counted once, casts sha… |
| [`EnvironmentSunTurn`](load/type-EnvironmentSunTurn.md) | struct |  | The Environment.rotation og.load.environmentSunRotation answered, and where the map's sun stands after it. |
| [`og.load.environmentSunRotation`](load/fn-environmentSunRotation.md) | function | `(world: World, map: EnvironmentMap, view: Vector3, azimuth: number): Result<EnvironmentSunTurn>` | The Environment.rotation that puts a cooked map's sun at `azimuth` around a view direction, so a sky is turned by where its sun should be ("light from behind the camera's left") instead of by a hand-… |
| [`og.load.emitter`](load/fn-emitter.md) | function | `(world: World, path: string): Result<ParticleEmitter>` | A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles`: its shape, emission, motion, look, rendering (billboards, meshes or 2D sprites), collision, sub-emitters… |
| [`og.load.effect`](load/fn-effect.md) | function | `(world: World, path: string): Result<EffectMaterial>` | An effect material for og.Trail.effect and og.Beam.effect, from a cooked `.ogt` of `type effect` (the file emitters and composed effects name with `effect`): its textures, UV motion, erosion, fresnel… |
| [`og.load.shader`](load/fn-shader.md) | function | `(world: World, path: string): Result<Shader>` | A material shader for og.render.shaderMaterial, from a cooked TS file.shader declaration (its WGSL compiled by the cook for every back end): mode {kind:'hooks'} (the default) defines hooks of the lit… |
| [`og.load.post`](load/fn-post.md) | function | `(world: World, path: string): Result<Post>` | A custom post pass for PostEffects.custom, from a cooked TS file.post declaration: code (inline WGSL or input.text of a module-relative source) defining fn post(f: PostInput, p: Params) -> vec4<f32>,… |
| [`og.load.vector`](load/fn-vector.md) | function | `(world: World, path: string): Result<VectorArt>` | A cooked vector from a TS file.vector declaration: source is a module-relative SVG path, or svg supplies inline XML, with optional positive tolerance in view-box units. |
| [`og.load.uiMaterial`](load/fn-uiMaterial.md) | function | `(world: World, path: string): Result<UiMaterial>` | A custom UI material resource from a cooked TS file['ui-material'] declaration: code (inline WGSL or input.text of a module-relative source) defining fn material(m: MaterialInput, p: Params) -> vec4<… |
| [`og.load.texture`](load/fn-texture.md) | function | `(world: World, path: string): Result<Texture>` | Load a cooked texture for og.render.sprite and a document's `<img>`, as three.js TextureLoader.load. |
| [`og.load.textureArray`](load/fn-textureArray.md) | function | `(world: World, path: string): Result<TextureArrayInfo>` | Load a cooked TS file['texture-array'] declaration as one 2D-array texture: 1..256 equally sized images, in declaration order, encoded for their kind. |
| [`og.load.style`](load/fn-style.md) | function | `(world: World, path: string): Result<Style>` | Load a cooked UI style for UiDesc.style: the directory whose `pack.json` it reads. |
| [`og.load.look`](load/fn-look.md) | function | `(world: World, path: string): Result<Look>` | Load a look: an `.ogt` of `type look` binding a frame's art direction as data, with `name`, `extends` (a preset slug or another look), `ui` and `antialias` lines and one block per resource it sets (L… |
| [`og.load.lookInfo`](load/fn-lookInfo.md) | function | `(world: World, look: Look): Result<LookInfo>` | What a loaded look holds and sets. |
| [`og.load.palette`](load/fn-palette.md) | function | `(world: World, path: string): Result<Palette>` | A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. |
| [`og.load.soundInfo`](load/fn-soundInfo.md) | function | `(world: World, sound: Sound): Result<SoundInfo>` | A loaded sound's format and length. |
| [`og.load.status`](load/fn-status.md) | function | `(world: World, asset: Asset): LoadStatus` | Where a load is. |
| [`og.load.error`](load/fn-error.md) | function | `(world: World, asset: Asset): string \| null` | Why a load failed. |
| [`og.load.gltfInfo`](load/fn-gltfInfo.md) | function | `(world: World, asset: Asset): Result<GltfInfo>` | What a ready glTF holds. |
| [`og.load.animation`](load/fn-animation.md) | function | `(world: World, path: string): Result<Animation>` | Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. |
| [`og.load.animgraph`](load/fn-animgraph.md) | function | `(world: World, path: string): Result<GraphAsset>` | Load a cooked type animgraph asset. |
| [`og.load.rig`](load/fn-rig.md) | function | `(world: World, path: string): Result<Rig>` | Load a checked file.rig humanoid map for og.anim.retarget and og.anim.humanoid. |
| [`og.load.flipbook`](load/fn-flipbook.md) | function | `(world: World, path: string): Result<FlipbookInfo>` | Load a cooked flipbook sheet (an `.ogt` of `type flipbook`: an Aseprite or TexturePacker atlas, or animated GIF, WebP or APNG sources the cook packs into `<sheet>#sheet`) and its texture, and registe… |
| [`og.load.tilemap`](load/fn-tilemap.md) | function | `(world: World, path: string): Result<TilemapInfo>` | Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. |
| [`og.load.video`](load/fn-video.md) | function | `(world: World, path: string): Result<VideoInfo>` | Load cooked footage authored with file.video in content/**/*.ts, which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. |
| [`og.load.videoInfo`](load/fn-videoInfo.md) | function | `(world: World, video: Video): Result<VideoInfo>` | A loaded video's size, frames, rate, length, texture and sound. |
| [`og.load.timeline`](load/fn-timeline.md) | function | `(world: World, path: string): Result<TimelineAsset>` | Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. |
| [`og.load.vfx`](load/fn-vfx.md) | function | `(world: World, path: string): Result<VfxAsset>` | Load a composed effect: an `.ogt` of `type vfx` (the og.vfx module's file) or an Effekseer effect (`.efkproj`, `.efkefc`) the cook converts, with every emitter, effect material, model, flipbook sheet… |
| [`og.load.rig2d`](load/fn-rig2d.md) | function | `(world: World, path: string): Result<Rig2DInfo>` | Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. |
| [`LoadBudget`](load/type-LoadBudget.md) | desc struct |  | The limits an uncooked load (og.load.imageData, og.load.gltfData, og.load.environmentData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's head… |
| [`ImageKind`](load/type-ImageKind.md) | enum |  | How an uncooked image's texels are read, as a cooked texture's `kind`. |
| [`GltfDataDesc`](load/type-GltfDataDesc.md) | desc struct |  | How og.load.gltfData reads a model. |
| [`ImageDataDesc`](load/type-ImageDataDesc.md) | desc struct |  | How og.load.imageData reads an image. |
| [`CatalogEntry`](load/type-CatalogEntry.md) | desc struct |  | A file of a content catalog's version: what og.load.data reads. |
| [`DataFormat`](load/type-DataFormat.md) | enum |  | How og.load.data reads a file. |
| [`DataDesc`](load/type-DataDesc.md) | desc struct |  | Which catalog file og.load.data reads, and how. |
| [`EnvironmentDataDesc`](load/type-EnvironmentDataDesc.md) | desc struct |  | How og.load.environmentData reads an HDR environment. |
| [`EnvironmentInfo`](load/type-EnvironmentInfo.md) | struct |  | What a ready uncooked environment holds. |
| [`ImageInfo`](load/type-ImageInfo.md) | struct |  | What a ready uncooked image holds. |
| [`LoadCompletion`](load/type-LoadCompletion.md) | struct |  | When a host-timed load finished: an uncooked load put in place or a runtime bundle opened. |
| [`og.load.imageData`](load/fn-imageData.md) | function | `(world: World, name: string, bytes: Uint8Array, desc?: ImageDataDesc): Result<Asset>` | Load an image from bytes, uncooked, at run time: a PNG or JPEG a player drew, uploaded or a generator made. |
| [`og.load.gltfData`](load/fn-gltfData.md) | function | `(world: World, name: string, bytes: Uint8Array, desc?: GltfDataDesc): Result<Asset>` | Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs. |
| [`og.load.environmentData`](load/fn-environmentData.md) | function | `(world: World, name: string, bytes: Uint8Array, desc?: EnvironmentDataDesc): Result<Asset>` | Load an HDR environment from bytes, uncooked, at run time, for Environment.map: an equirectangular Radiance .hdr (or .exr where RenderCapabilities.exrDecode is true), twice as wide as tall. |
| [`og.load.data`](load/fn-data.md) | function | `(world: World, desc?: DataDesc): Result<Asset>` | Read a file of a content catalog at run time: static, versioned content the game's host publishes (a world's index pages, a shared level), well past a store value's 64 KiB. |
| [`og.load.dataJson`](load/fn-dataJson.md) | function | `(world: World, asset: Asset): Result<unknown>` | The value of a ready og.load.data file read as `json`. |
| [`og.load.dataBytes`](load/fn-dataBytes.md) | function | `(world: World, asset: Asset): Result<Uint8Array>` | The bytes of a ready og.load.data file read as `bytes`. |
| [`og.load.pack`](load/fn-pack.md) | function | `(world: World, entry?: CatalogEntry): Result<Asset>` | Open a cooked bundle a content catalog's version lists by name (its manifest's `packs`), read from the content base against the integrity the version lists. |
| [`og.load.environmentInfo`](load/fn-environmentInfo.md) | function | `(world: World, asset: Asset): Result<EnvironmentInfo>` | A ready uncooked environment's map and sizes. |
| [`og.load.release`](load/fn-release.md) | function | `(world: World, asset: Asset): Result<void>` | Free an uncooked load's memory, cancelling it if pending; its handle then answers failed, naming the release. |
| [`og.load.imageInfo`](load/fn-imageInfo.md) | function | `(world: World, asset: Asset): Result<ImageInfo>` | A ready uncooked image's texture and size. |
| [`og.load.completions`](load/fn-completions.md) | function | `(world: World): readonly LoadCompletion[]` | The world's journal of host-timed loads, oldest first: uncooked loads put in place and runtime bundles opened, each with its tick. |
| [`og.load.replayCompletions`](load/fn-replayCompletions.md) | function | `(world: World, completions: readonly LoadCompletion[]): Result<void>` | Finish this world's host-timed loads at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits … |
| [`og.AssetReady`](load/record-AssetReady.md) | event |  | A load completed. |
| [`og.AssetFailed`](load/record-AssetFailed.md) | event |  | A load failed. |

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

<sub>Module `load` line 30</sub>

```ogt
column Asset {
  doc "An Asset handle field."
  array Uint32Array
  stride 2
  value Asset
}
```

<sub>Module `load` line 37</sub>

```ogt
column Sound {
  doc "A Sound handle field."
  array Uint32Array
  stride 2
  value Sound
}
```

<sub>Module `load` line 44</sub>

```ogt
column Font {
  doc "A Font handle field (0 is none): a font og.load.font answered, kept for og.render.text or og.ui.configure."
  array Uint32Array
  stride 2
  value Font
}
```

<sub>Module `load` line 51</sub>

```ogt
column Style {
  doc "A Style handle field (0 is none): a UI style og.load.style answered, kept for og.ui.configure."
  array Uint32Array
  stride 2
  value Style
}
```

<sub>Module `load` line 58</sub>
