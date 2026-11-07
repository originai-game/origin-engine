<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

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

<sub>Module `load` line 533</sub>
