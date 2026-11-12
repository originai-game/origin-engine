<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-gltfData"></a>

### `og.load.gltfData`

Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs. Its size, triangles and image headers are checked against the budget at the call; then its images decode off the engine's thread and its primitives convert at tick boundaries. The handle is pending until the model is in place, then ready (world.instantiate spawns it, og.load.gltfInfo describes it) or failed (og.load.error says why).

It reads materials, nodes and scenes, without skins, animations or levels of detail.

Completion is host input, journaled as og.load.imageData's is.

See `loading §10`.

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
  doc "Load a glTF 2.0 model from bytes, uncooked, at run time: a .glb, or a .gltf whose buffers and images are data: URIs. Its size, triangles and image headers are checked against the budget at the call; then its images decode off the engine's thread and its primitives convert at tick boundaries. The handle is pending until the model is in place, then ready (world.instantiate spawns it, og.load.gltfInfo describes it) or failed (og.load.error says why)."
  guide "loading §10"
  doc "It reads materials, nodes and scenes, without skins, animations or levels of detail."
  doc "Completion is host input, journaled as og.load.imageData's is."
  param world World "The world that owns the asset."
  param name String "A name for it in errors and dumps, such as `creations/kite.glb`; its textures are `<name>#tex<n>`."
  param bytes Bytes "The file's bytes."
  param desc GltfDataDesc "Its budget."
  returns Result.Asset "The handle (pending), or why it was refused now: over the budget's bytes, triangles or textureSize, not a glTF 2.0, an image that is not a PNG or JPEG, or something the reader refuses by name."
}
```

<sub>Module `load` line 577</sub>
