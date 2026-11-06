<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

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

<sub>`api.ogt` line 8078</sub>
