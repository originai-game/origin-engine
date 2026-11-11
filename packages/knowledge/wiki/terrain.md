<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `terrain` module

Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a heightmap, a control map, up to sixteen layered materials, a holes map) into tiles; drawn as a clipmap around the camera, collided with as a height field, baked into navmeshes.

Load it with og.terrain.load, then place it with an og.Terrain and an og.Collider of shape SHAPE_HEIGHTFIELD naming TerrainInfo.geometry. Edits are world state. Terrain is 3D: where RenderCapabilities.world3d is false every call answers a refusal naming that field. Its tiles stream; no tick reads which are resident.

See `world §1`.

- **TypeScript**: functions are the namespace `og.terrain`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module terrain {
  doc "Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a heightmap, a control map, up to sixteen layered materials, a holes map) into tiles; drawn as a clipmap around the camera, collided with as a height field, baked into navmeshes."
  doc "Load it with og.terrain.load, then place it with an og.Terrain and an og.Collider of shape SHAPE_HEIGHTFIELD naming TerrainInfo.geometry. Edits are world state. Terrain is 3D: where RenderCapabilities.world3d is false every call answers a refusal naming that field. Its tiles stream; no tick reads which are resident."
  # declarations follow, each on its own page
```

<sub>Module `terrain` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`TerrainData`](terrain/type-TerrainData.md) | handle |  | A loaded terrain: its heights, control map and layers. |
| [`TerrainData()`](#column-TerrainData) | column kind |  | A TerrainData handle field. |
| [`TerrainBrushMode`](terrain/type-TerrainBrushMode.md) | enum |  | What og.terrain.edit does under its brush. |
| [`TerrainStampMode`](terrain/type-TerrainStampMode.md) | enum |  | What og.terrain.stamp does to the height field under a stamp's mesh. |
| [`og.Terrain`](terrain/record-Terrain.md) | component |  | Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows u… |
| [`og.TerrainStamp`](terrain/record-TerrainStamp.md) | component |  | Draws a mesh as part of a loaded terrain: a cliff, arch or overhang the height field cannot make, placed by the entity's world transform. |
| [`og.TerrainScatter`](terrain/record-TerrainScatter.md) | component |  | Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, each cell growing one with the probab… |
| [`og.Grass`](terrain/record-Grass.md) | component |  | A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance`: curved blades on a blue-noise pattern fixed to the terrain, growing where the… |
| [`og.GrassType`](terrain/record-GrassType.md) | component |  | Another kind of blade for og.Grass fields that list this entity in their `types`: its shape and colours replace the field's for the clumps that grow it, the rest (density, wind, lighting, dryness) is… |
| [`og.GrassPusher`](terrain/record-GrassPusher.md) | component |  | Presses og.Grass blades and og.TerrainScatter copies where the entity stands: a vertical capsule from its position up `height`, reaching `radius` on the ground. |
| [`TerrainInfo`](terrain/type-TerrainInfo.md) | struct |  | A loaded terrain. |
| [`TerrainSampleDesc`](terrain/type-TerrainSampleDesc.md) | desc struct |  | Where og.terrain.sample looks. |
| [`TerrainSample`](terrain/type-TerrainSample.md) | struct |  | The terrain under a point. |
| [`TerrainBounds`](terrain/type-TerrainBounds.md) | struct |  | A terrain's box in world space. |
| [`TerrainBrush`](terrain/type-TerrainBrush.md) | desc struct |  | One runtime edit of a terrain (og.terrain.edit). |
| [`TerrainEdit`](terrain/type-TerrainEdit.md) | struct |  | What an edit changed. |
| [`TerrainStampDesc`](terrain/type-TerrainStampDesc.md) | desc struct |  | One stamp of a mesh into a terrain (og.terrain.stamp). |
| [`TerrainLayerDesc`](terrain/type-TerrainLayerDesc.md) | desc struct |  | One material layer of a terrain: its look and the rules that place it. |
| [`og.terrain.load`](terrain/fn-load.md) | function | `(world: World, path: string): Result<TerrainInfo>` | Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInf… |
| [`og.terrain.sample`](terrain/fn-sample.md) | function | `(world: World, desc?: TerrainSampleDesc): Result<TerrainSample>` | The surface under a world point: its height, normal and slope. |
| [`og.terrain.bounds`](terrain/fn-bounds.md) | function | `(world: World, data: TerrainData, origin?: Vector3, quaternion?: Quaternion, scale?: Vector3): Result<TerrainBounds>` | A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners. |
| [`og.terrain.edit`](terrain/fn-edit.md) | function | `(world: World, data: TerrainData, brush?: TerrainBrush): Result<TerrainEdit>` | Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its … |
| [`og.terrain.stamp`](terrain/fn-stamp.md) | function | `(world: World, desc?: TerrainStampDesc): Result<TerrainEdit>` | Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like o… |
| [`og.terrain.unstamp`](terrain/fn-unstamp.md) | function | `(world: World, data: TerrainData, stamp: number): Result<TerrainEdit>` | Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made. |
| [`og.terrain.layer`](terrain/fn-layer.md) | function | `(world: World, data: TerrainData, index: number): Result<TerrainLayerDesc>` | A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it). |
| [`og.terrain.setLayer`](terrain/fn-setLayer.md) | function | `(world: World, data: TerrainData, index: number, layer?: TerrainLayerDesc): Result<void>` | Replace a terrain's layer look and rules (its textures stay). |

## Column kinds

Each is a factory, `TerrainData(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-TerrainData"></a>`TerrainData` | `TerrainData` | `Uint32Array` | 2 | A TerrainData handle field. |

```ogt
column TerrainData {
  doc "A TerrainData handle field."
  array Uint32Array
  stride 2
  value TerrainData
}
```

<sub>Module `terrain` line 8</sub>
