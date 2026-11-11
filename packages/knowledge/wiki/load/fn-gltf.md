<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-gltf"></a>

### `og.load.gltf`

Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.

A MagicaVoxel `.vox` cooks to a glTF and loads here too, its frames as clips og.AnimationPlayer plays; its import settings size and place it.

See `loading §2`.

Also known as: load a 3D model, model loader, GLTFLoader, glb, gltf model, import a mesh

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
  aliases "load a 3D model, model loader, GLTFLoader, glb, gltf model, import a mesh"
  doc "Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once."
  doc "A MagicaVoxel `.vox` cooks to a glTF and loads here too, its frames as clips og.AnimationPlayer plays; its import settings size and place it."
  guide "loading §2"
  param world World "The world that owns the asset."
  param path String "The asset id, such as `models/ship.glb`."
  param desc GltfDesc "Scene and shadow options."
  returns Result.Asset "The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle)."
}
```

<sub>Module `load` line 205</sub>
