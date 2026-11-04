<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `terrain` module

Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a 16-bit grey PNG or an EXR heightmap, an optional control map, up to sixteen layered materials with slope and altitude rules (each pixel blends its four strongest), their maps separate textures (the first four layers) or layers of two texture arrays: `layers` and `layerNormals` with each layer's `albedoLayer` and `normalLayer`, two sampled textures instead of up to eight and the only way past four layers, the control map one image per four layers, and an optional `holes` map whose black or transparent cells are cut out of the surface: a cave mouth, a pit, a tunnel through a hill) into tiles; drawn as a geometry clipmap around the camera (nested levels by distance, no cracks) that casts and receives shadows under the fog; collided with as a height field; baked into navmeshes by a navmesh file's `terrain` entry.

Load it with og.terrain.load, then place it: an og.Terrain on an entity draws it placed by the entity's world transform (centred on its position, turned and scaled with it), and an og.Collider of shape SHAPE_HEIGHTFIELD naming the TerrainInfo's geometry makes it solid. og.TerrainScatter grows grass, flowers or stones over a layer near the camera. og.terrain.sample answers the surface's height and normal anywhere (pass it the entity's position, quaternion and scale); og.terrain.edit raises, lowers, flattens, smooths or paints it at runtime, and og.terrain.setLayer retunes a layer. Terrain is 3D: the 2D profile's publish and lanes modules leave it out (RenderCapabilities.world3d is false), where every call here answers a refusal naming that field. Edits are world state: a snapshot keeps them, the world hash covers them, and a restore brings the ground back as it was (rebuilt from the cook with the restored edits replayed, for the renderer, physics and sample alike).

Its tiles stream (phase 3): the cook stores the field as a pyramid of tiles, each compressed on its own, and every frame the renderer keeps on the GPU only the tiles the camera's clipmap wants, within each terrain's tile budget (16 MiB by default), decoded off the frame (a worker thread natively, the browser's DecompressionStream on the Web) a few at a time, the most needed first; until a tile arrives its area draws from the coarser level that holds it, then fades in. Nothing a tick reads depends on which tiles are resident (collision and og.terrain.sample read the whole field), so a run's world hash is the same streamed or not. The probe's `terrain_stream:resident` decodes every wanted tile before each frame instead (`terrain_stream:stream[:<MiB>]` streams again, within that budget), and `dump_stream`'s `terrain` gives the tiles resident, waiting, decoding and cached.

- **TypeScript**: functions are the namespace `og.terrain`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-render`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module terrain {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 28357</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`TerrainData`](#type-TerrainData) | handle | A loaded terrain: its heights, control map and layers. |
| [`TerrainData()`](#column-TerrainData) | column kind | A TerrainData handle field. |
| [`TerrainBrushMode`](#type-TerrainBrushMode) | enum | What og.terrain.edit does under its brush. |
| [`TerrainStampMode`](#type-TerrainStampMode) | enum | What og.terrain.stamp does to the height field under a stamp's mesh. |
| [`og.Terrain`](#record-Terrain) | component | Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to. |
| [`og.TerrainStamp`](#record-TerrainStamp) | component | Draws a mesh as part of a loaded terrain: a cliff, arch or overhang the height field cannot make, placed by the entity's world transform. It is shaded with the terrain's own layers and rules (by its own normal: grass on its top, rock on its walls, biplanar where steep) and, within `blend` metres above the ground under it, takes the ground's look and normal, so it meets the terrain without a seam. Drawn with the og.Terrain drawing the same data. Only drawn: carve the ground under it with og.terrain.stamp, and make it solid with an og.Collider naming the same geometry. |
| [`og.TerrainScatter`](#record-TerrainScatter) | component | Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities. |
| [`og.Grass`](#record-Grass) | component | A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance` (a meadow with every other field at its default). Blades are curved strips placed on a blue-noise pattern fixed to the terrain, so each keeps its place as the camera moves; they grow where the layer's weight is (its control-map channel times its slope and altitude rules), never over holes, in clumps that share their tint and height. With distance they thin out, the ones left widening to keep the ground covered, and take fewer vertices (15, 7, then 3); under and past them the terrain takes the field's average colour (`farTint`), so the far field reads as grass to the horizon with no seam. og.Wind bends them in gusts that roll across the field, each blade swaying on its own; lit like the terrain under them, with light through blades lit from behind (`translucency`), a sheen on the upper blade and the roots shaded by their neighbours. Visual only: drawn from the tick, nothing in the simulation reads it, the same tick draws the same blades on every run. It grows over the og.Terrain drawing the same data (a visible one). |
| [`TerrainInfo`](#type-TerrainInfo) | struct | A loaded terrain. |
| [`TerrainSampleDesc`](#type-TerrainSampleDesc) | desc struct | Where og.terrain.sample looks. |
| [`TerrainSample`](#type-TerrainSample) | struct | The terrain under a point. |
| [`TerrainBounds`](#type-TerrainBounds) | struct | A terrain's box in world space. |
| [`TerrainBrush`](#type-TerrainBrush) | desc struct | One runtime edit of a terrain (og.terrain.edit). |
| [`TerrainEdit`](#type-TerrainEdit) | struct | What an edit changed. |
| [`TerrainStampDesc`](#type-TerrainStampDesc) | desc struct | One stamp of a mesh into a terrain (og.terrain.stamp). |
| [`TerrainLayerDesc`](#type-TerrainLayerDesc) | desc struct | One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised. |
| [`og.terrain.load`](#fn-load) | function | Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain. |
| [`og.terrain.sample`](#fn-sample) | function | The surface under a world point: its height, normal and slope. Each height-field cell splits along the diagonal whose corner heights differ less; ties keep the (-x, -z) to (+x, +z) diagonal. Sampling, rendering at the finest level and the native height-field collider use this same rule. Callable inside systems (it reads the terrain, not the world). |
| [`og.terrain.bounds`](#fn-bounds) | function | A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners. |
| [`og.terrain.edit`](#fn-edit) | function | Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none). |
| [`og.terrain.stamp`](#fn-stamp) | function | Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like og.terrain.edit's (a snapshot keeps it, the world hash covers it, a restore replays it, and the replay needs only the log, not the mesh). The renderer, physics and og.terrain.sample see the change from the next frame and tick. Answers the stamp's id in TerrainEdit.stamp. |
| [`og.terrain.unstamp`](#fn-unstamp) | function | Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made. |
| [`og.terrain.layer`](#fn-layer) | function | A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it). |
| [`og.terrain.setLayer`](#fn-setLayer) | function | Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame. |

## Functions

<a id="fn-load"></a>

### `og.terrain.load`

Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain.

```ts
og.terrain.load(world: World, path: string): Result<TerrainInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The terrain file, such as `terrain/island.ogt`. |

**Returns** `Result<TerrainInfo>`: Refused for a path that is not a cooked terrain, or a layer texture that does not load.

Binding symbol `og_terrain_load` · publish builds: kept

```ogt
fn load {
  doc "Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain."
  param world World "The world."
  param path String "The terrain file, such as `terrain/island.ogt`."
  returns Result.TerrainInfo "Refused for a path that is not a cooked terrain, or a layer texture that does not load."
}
```

<sub>`api.ogt` line 28857</sub>

<a id="fn-sample"></a>

### `og.terrain.sample`

The surface under a world point: its height, normal and slope. Each height-field cell splits along the diagonal whose corner heights differ less; ties keep the (-x, -z) to (+x, +z) diagonal. Sampling, rendering at the finest level and the native height-field collider use this same rule. Callable inside systems (it reads the terrain, not the world).

```ts
og.terrain.sample(world: World, desc?: TerrainSampleDesc): Result<TerrainSample>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TerrainSampleDesc` | yes | The terrain, where it stands and the point. |

**Returns** `Result<TerrainSample>`: Refused for an unknown terrain.

Binding symbol `og_terrain_sample` · publish builds: kept

```ogt
fn sample {
  doc "The surface under a world point: its height, normal and slope. Each height-field cell splits along the diagonal whose corner heights differ less; ties keep the (-x, -z) to (+x, +z) diagonal. Sampling, rendering at the finest level and the native height-field collider use this same rule. Callable inside systems (it reads the terrain, not the world)."
  param world World "The world."
  param desc TerrainSampleDesc "The terrain, where it stands and the point."
  returns Result.TerrainSample "Refused for an unknown terrain."
}
```

<sub>`api.ogt` line 28864</sub>

<a id="fn-bounds"></a>

### `og.terrain.bounds`

A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners.

```ts
og.terrain.bounds(world: World, data: TerrainData, origin?: Vector3, quaternion?: Quaternion, scale?: Vector3): Result<TerrainBounds>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `origin` | `Vector3` | yes | Where it stands. |
| `quaternion` | `Quaternion` | yes | How it is turned. |
| `scale` | `Vector3` | yes | How it is scaled. |

**Returns** `Result<TerrainBounds>`: Refused for an unknown terrain.

Binding symbol `og_terrain_bounds` · publish builds: kept

```ogt
fn bounds {
  doc "A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners."
  param world World "The world."
  param data TerrainData "The terrain."
  param origin Vector3 "Where it stands." {
    optional true
  }
  param quaternion Quaternion "How it is turned." {
    optional true
  }
  param scale Vector3 "How it is scaled." {
    optional true
  }
  returns Result.TerrainBounds "Refused for an unknown terrain."
}
```

<sub>`api.ogt` line 28871</sub>

<a id="fn-edit"></a>

### `og.terrain.edit`

Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none).

```ts
og.terrain.edit(world: World, data: TerrainData, brush?: TerrainBrush): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `brush` | `TerrainBrush` | yes | What to do and where. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain, a layer beyond its layers, or a brush that misses it.

Binding symbol `og_terrain_edit` · publish builds: kept

```ogt
fn edit {
  doc "Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none)."
  param world World "The world."
  param data TerrainData "The terrain."
  param brush TerrainBrush "What to do and where."
  returns Result.TerrainEdit "Refused for an unknown terrain, a layer beyond its layers, or a brush that misses it."
}
```

<sub>`api.ogt` line 28887</sub>

<a id="fn-stamp"></a>

### `og.terrain.stamp`

Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like og.terrain.edit's (a snapshot keeps it, the world hash covers it, a restore replays it, and the replay needs only the log, not the mesh). The renderer, physics and og.terrain.sample see the change from the next frame and tick. Answers the stamp's id in TerrainEdit.stamp.

```ts
og.terrain.stamp(world: World, desc?: TerrainStampDesc): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TerrainStampDesc` | yes | The terrain, the mesh, where both stand, and what to do. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain, a geometry that is not loaded, or a mesh that covers none of the terrain.

Binding symbol `og_terrain_stamp` · publish builds: kept

```ogt
fn stamp {
  doc "Carve a terrain under a mesh (its og.TerrainStamp's), or cut holes under it: the triangles are rasterised from above over the height samples, deterministically, and what changed is world state like og.terrain.edit's (a snapshot keeps it, the world hash covers it, a restore replays it, and the replay needs only the log, not the mesh). The renderer, physics and og.terrain.sample see the change from the next frame and tick. Answers the stamp's id in TerrainEdit.stamp."
  param world World "The world."
  param desc TerrainStampDesc "The terrain, the mesh, where both stand, and what to do."
  returns Result.TerrainEdit "Refused for an unknown terrain, a geometry that is not loaded, or a mesh that covers none of the terrain."
}
```

<sub>`api.ogt` line 28895</sub>

<a id="fn-unstamp"></a>

### `og.terrain.unstamp`

Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made.

```ts
og.terrain.unstamp(world: World, data: TerrainData, stamp: number): Result<TerrainEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `stamp` | `number` | no | The id og.terrain.stamp answered. |

**Returns** `Result<TerrainEdit>`: Refused for an unknown terrain or a stamp it does not hold.

Binding symbol `og_terrain_unstamp` · publish builds: kept

```ogt
fn unstamp {
  doc "Take a stamp back: the terrain is rebuilt from its cook with every other edit and stamp replayed in order, as if it had never been made."
  param world World "The world."
  param data TerrainData "The terrain."
  param stamp u32 "The id og.terrain.stamp answered."
  returns Result.TerrainEdit "Refused for an unknown terrain or a stamp it does not hold."
}
```

<sub>`api.ogt` line 28902</sub>

<a id="fn-layer"></a>

### `og.terrain.layer`

A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it).

```ts
og.terrain.layer(world: World, data: TerrainData, index: number): Result<TerrainLayerDesc>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `index` | `number` | no | The layer, from 0. |

**Returns** `Result<TerrainLayerDesc>`: Refused for an unknown terrain or layer.

Binding symbol `og_terrain_layer` · publish builds: kept

```ogt
fn layer {
  doc "A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it)."
  param world World "The world."
  param data TerrainData "The terrain."
  param index u32 "The layer, from 0."
  returns Result.TerrainLayerDesc "Refused for an unknown terrain or layer."
}
```

<sub>`api.ogt` line 28910</sub>

<a id="fn-setLayer"></a>

### `og.terrain.setLayer`

Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame.

```ts
og.terrain.setLayer(world: World, data: TerrainData, index: number, layer?: TerrainLayerDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `TerrainData` | no | The terrain. |
| `index` | `number` | no | The layer, from 0. |
| `layer` | `TerrainLayerDesc` | yes | The layer. |

**Returns** `Result<void>`: Refused for an unknown terrain or layer.

Binding symbol `og_terrain_set_layer` · publish builds: kept

```ogt
fn setLayer {
  doc "Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame."
  param world World "The world."
  param data TerrainData "The terrain."
  param index u32 "The layer, from 0."
  param layer TerrainLayerDesc "The layer."
  returns Result.Void "Refused for an unknown terrain or layer."
}
```

<sub>`api.ogt` line 28918</sub>

## Components, resources and events

<a id="record-Terrain"></a>

### `og.Terrain` (component)

Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to.

Token `og.Terrain`, registered under the schema name `og.Terrain`; its value type is `Terrain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `visible` | `boolean` | `true` | — | — | — | Draw it; false keeps it only as the ground its og.TerrainScatter copies stand on (nothing else sees it). |
| `castShadows` | `boolean` | `true` | — | — | — | Cast the directional and local lights' shadows (it always receives them). |
| `horizonShadows` | `boolean` | `true` | — | — | — | Shade it by its own horizon, baked by the cook: past the sun's shadow cascades a valley behind a ridge falls into its shade (to the horizon), and everywhere the sky a valley's walls hide lights it less. |
| `lodSpacing` | `number` | `0` | 0 … 64 | `m` | tunable | The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing). |
| `lodRing` | `number` | `0` | 0 … 256 | — | — | Quads per side of every clipmap level (0: the terrain file's, else 64; 16 to 256, rounded to a multiple of 4). |

```ogt
component Terrain {
  doc "Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field visible bool "Draw it; false keeps it only as the ground its og.TerrainScatter copies stand on (nothing else sees it)." {
    default true
  }
  field castShadows bool "Cast the directional and local lights' shadows (it always receives them)." {
    default true
  }
  field horizonShadows bool "Shade it by its own horizon, baked by the cook: past the sun's shadow cascades a valley behind a ridge falls into its shade (to the horizon), and everywhere the sky a valley's walls hide lights it less." {
    default true
  }
  field lodSpacing f32 "The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing)." {
    min 0
    max 64
    unit m
    tunable true
  }
  field lodRing u32 "Quads per side of every clipmap level (0: the terrain file's, else 64; 16 to 256, rounded to a multiple of 4)." {
    min 0
    max 256
  }
}
```

<sub>`api.ogt` line 28389</sub>

<a id="record-TerrainStamp"></a>

### `og.TerrainStamp` (component)

Draws a mesh as part of a loaded terrain: a cliff, arch or overhang the height field cannot make, placed by the entity's world transform. It is shaded with the terrain's own layers and rules (by its own normal: grass on its top, rock on its walls, biplanar where steep) and, within `blend` metres above the ground under it, takes the ground's look and normal, so it meets the terrain without a seam. Drawn with the og.Terrain drawing the same data. Only drawn: carve the ground under it with og.terrain.stamp, and make it solid with an og.Collider naming the same geometry.

Token `og.TerrainStamp`, registered under the schema name `og.TerrainStamp`; its value type is `TerrainStamp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh. |
| `blend` | `number` | `1` | 0 … 64 | `m` | tunable | Metres above the ground under it over which the mesh fades into the ground's look: wholly the ground over its lowest third, then fading to the mesh's own (0: a hard edge). |
| `visible` | `boolean` | `true` | — | — | — | Draw it. |
| `castShadows` | `boolean` | `true` | — | — | — | Cast shadows (with its terrain's own castShadows). |

```ogt
component TerrainStamp {
  doc "Draws a mesh as part of a loaded terrain: a cliff, arch or overhang the height field cannot make, placed by the entity's world transform. It is shaded with the terrain's own layers and rules (by its own normal: grass on its top, rock on its walls, biplanar where steep) and, within `blend` metres above the ground under it, takes the ground's look and normal, so it meets the terrain without a seam. Drawn with the og.Terrain drawing the same data. Only drawn: carve the ground under it with og.terrain.stamp, and make it solid with an og.Collider naming the same geometry."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field geometry Geometry "The mesh."
  field blend f32 "Metres above the ground under it over which the mesh fades into the ground's look: wholly the ground over its lowest third, then fading to the mesh's own (0: a hard edge)." {
    default 1
    min 0
    max 64
    unit m
    tunable true
  }
  field visible bool "Draw it." {
    default true
  }
  field castShadows bool "Cast shadows (with its terrain's own castShadows)." {
    default true
  }
}
```

<sub>`api.ogt` line 28413</sub>

<a id="record-TerrainScatter"></a>

### `og.TerrainScatter` (component)

Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities.

Token `og.TerrainScatter`, registered under the schema name `og.TerrainScatter`; its value type is `TerrainScatter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh, its origin at its root. When it belongs to the named impostor model, all of that primitive's node placements are applied in model space, at every distance (including impostorDistance 0). Other geometry must already be in that space. |
| `material` | `Material` | `0` | — | — | — | Its material. |
| `layer` | `number` | `0` | ≤ 15 | — | — | The terrain layer that places it, 0 to 15. |
| `density` | `number` | `2` | 0 … 400 | — | tunable | Copies per square metre where the layer's weight is 1. |
| `distance` | `number` | `40` | 0 … 500 | `m` | tunable | How far from the camera, along x and z, copies are placed. |
| `fade` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The outer part of `distance` over which copies thin out to nothing. |
| `minScale` | `number` | `0.8` | ≥ 0 | — | tunable | The smallest copy's uniform scale. |
| `maxScale` | `number` | `1.2` | ≥ 0 | — | tunable | The largest copy's uniform scale. |
| `align` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far copies lean with the ground: 0 upright, 1 along its normal. |
| `seed` | `number` | `1` | — | — | — | Another seed places the copies elsewhere. |
| `castShadows` | `boolean` | `true` | — | — | — | Whether the copies cast shadows (grass usually does not: three.js meshes cast only with castShadow set). |
| `sway` | `number` | `1` | 0 … 2 | — | tunable | How much og.Wind sways the copies: 1 as it blows, 0 still (stones), between for stiffer stems. Each vertex above the mesh's origin moves by its height, so model the copy with its root at the origin. |
| `impostor` | `Asset` | `0` | — | — | — | A glTF (og.load.gltf) of the same complete mesh, cooked with an octahedral impostor (its import settings' `model { impostor true }`): copies farther than impostorDistance draw as that impostor, cross-dithered over the last tenth before it. Mesh and impostor bounds must agree in model space; a mismatch skips the scatter and reports render.scatterIssue in its entity dump and a runtime warning. None: the supplied geometry at every distance, without model placement. |
| `impostorDistance` | `number` | `0` | 0 … 500 | `m` | tunable | How far from the camera, along x and z, copies draw as their mesh before their impostor takes over; 0 never switches. Useful below `distance`. |

```ogt
component TerrainScatter {
  doc "Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field geometry Geometry "The mesh, its origin at its root. When it belongs to the named impostor model, all of that primitive's node placements are applied in model space, at every distance (including impostorDistance 0). Other geometry must already be in that space."
  field material Material "Its material."
  field layer u32 "The terrain layer that places it, 0 to 15." {
    max 15
  }
  field density f32 "Copies per square metre where the layer's weight is 1." {
    default 2
    min 0
    max 400
    tunable true
  }
  field distance f32 "How far from the camera, along x and z, copies are placed." {
    default 40
    min 0
    max 500
    unit m
    tunable true
  }
  field fade f32 "The outer part of `distance` over which copies thin out to nothing." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field minScale f32 "The smallest copy's uniform scale." {
    default 0.8
    min 0
    tunable true
  }
  field maxScale f32 "The largest copy's uniform scale." {
    default 1.2
    min 0
    tunable true
  }
  field align f32 "How far copies lean with the ground: 0 upright, 1 along its normal." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field seed u32 "Another seed places the copies elsewhere." {
    default 1
  }
  field castShadows bool "Whether the copies cast shadows (grass usually does not: three.js meshes cast only with castShadow set)." {
    default true
  }
  field sway f32 "How much og.Wind sways the copies: 1 as it blows, 0 still (stones), between for stiffer stems. Each vertex above the mesh's origin moves by its height, so model the copy with its root at the origin." {
    default 1
    min 0
    max 2
    tunable true
  }
  field impostor Asset "A glTF (og.load.gltf) of the same complete mesh, cooked with an octahedral impostor (its import settings' `model { impostor true }`): copies farther than impostorDistance draw as that impostor, cross-dithered over the last tenth before it. Mesh and impostor bounds must agree in model space; a mismatch skips the scatter and reports render.scatterIssue in its entity dump and a runtime warning. None: the supplied geometry at every distance, without model placement."
  field impostorDistance f32 "How far from the camera, along x and z, copies draw as their mesh before their impostor takes over; 0 never switches. Useful below `distance`." {
    min 0
    max 500
    unit m
    tunable true
  }
}
```

<sub>`api.ogt` line 28432</sub>

<a id="record-Grass"></a>

### `og.Grass` (component)

A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance` (a meadow with every other field at its default). Blades are curved strips placed on a blue-noise pattern fixed to the terrain, so each keeps its place as the camera moves; they grow where the layer's weight is (its control-map channel times its slope and altitude rules), never over holes, in clumps that share their tint and height. With distance they thin out, the ones left widening to keep the ground covered, and take fewer vertices (15, 7, then 3); under and past them the terrain takes the field's average colour (`farTint`), so the far field reads as grass to the horizon with no seam. og.Wind bends them in gusts that roll across the field, each blade swaying on its own; lit like the terrain under them, with light through blades lit from behind (`translucency`), a sheen on the upper blade and the roots shaded by their neighbours. Visual only: drawn from the tick, nothing in the simulation reads it, the same tick draws the same blades on every run. It grows over the og.Terrain drawing the same data (a visible one).

Token `og.Grass`, registered under the schema name `og.Grass`; its value type is `Grass`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `layer` | `number` | `0` | ≤ 15 | — | — | The terrain layer that places it, 0 to 15. |
| `density` | `number` | `300` | 0 … 4000 | — | tunable | Blades per square metre near the camera where the layer's weight is 1. |
| `distance` | `number` | `150` | 0 … 1000 | `m` | tunable | How far from the camera blades are drawn; they thin out to nothing there. |
| `farTint` | `boolean` | `true` | — | — | — | The terrain takes the field's average colour under its blades, where they thin out and past them, so the gaps and the far field read as the same grass. |
| `height` | `number` | `0.45` | 0 … 4 | `m` | tunable | The tallest blade. |
| `heightVariation` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | How much shorter blades are than the tallest: 0 all alike, 1 down to nothing. |
| `width` | `number` | `0.012` | 0 … 0.5 | `m` | tunable | A blade's width at its root. |
| `curve` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How far blades droop along their length: 0 straight, 1 arching over. |
| `lean` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How far blades lean at rest, as a share of their height. |
| `clumpSize` | `number` | `1.2` | 0.05 … 20 | `m` | tunable | The size of the clumps blades grow in, sharing their tint and height. |
| `clumpLean` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | How far blades lean out from their clump's centre, as a share of their height. |
| `tufts` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far blades are pulled in towards their clump's centre: 0 an even lawn, 1 tight tufts. |
| `baseColor` | `Color` | `{ r: 0.04, g: 0.085, b: 0.024 }` | — | — | — | The roots' colour (linear). |
| `tipColor` | `Color` | `{ r: 0.235, g: 0.39, b: 0.11 }` | — | — | — | The tips' colour (linear). |
| `dryColor` | `Color` | `{ r: 0.45, g: 0.38, b: 0.17 }` | — | — | — | Dry grass's colour (linear): dry clumps and drying tips. |
| `colorVariation` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much blades vary in brightness by clump, blade and patch. |
| `dryness` | `number` | `0.15` | 0 … 1 | `ratio` | tunable | The share of clumps that are dry, and how far tips dry. |
| `translucency` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How much sunlight passes through a blade lit from behind, tinted by it. |
| `sheen` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | The upper blade's gloss: gusts show as a moving sheen. |
| `roughness` | `number` | `0.6` | 0.04 … 1 | — | tunable | The blade's roughness. |
| `sway` | `number` | `1` | 0 … 2 | — | tunable | How much og.Wind bends the blades: 1 as it blows, 0 still. |
| `stiffness` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How stiff the blades are: 0 bends furthest, 1 least, and sways faster. |
| `flutter` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much the tips of near blades shiver in the wind. |
| `castShadows` | `boolean` | `false` | — | — | — | Whether the blades cast the sun's shadows (near the camera, into the first cascade). |
| `seed` | `number` | `1` | — | — | — | Another seed places the blades elsewhere. |

```ogt
component Grass {
  doc "A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance` (a meadow with every other field at its default). Blades are curved strips placed on a blue-noise pattern fixed to the terrain, so each keeps its place as the camera moves; they grow where the layer's weight is (its control-map channel times its slope and altitude rules), never over holes, in clumps that share their tint and height. With distance they thin out, the ones left widening to keep the ground covered, and take fewer vertices (15, 7, then 3); under and past them the terrain takes the field's average colour (`farTint`), so the far field reads as grass to the horizon with no seam. og.Wind bends them in gusts that roll across the field, each blade swaying on its own; lit like the terrain under them, with light through blades lit from behind (`translucency`), a sheen on the upper blade and the roots shaded by their neighbours. Visual only: drawn from the tick, nothing in the simulation reads it, the same tick draws the same blades on every run. It grows over the og.Terrain drawing the same data (a visible one)."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field layer u32 "The terrain layer that places it, 0 to 15." {
    max 15
  }
  field density f32 "Blades per square metre near the camera where the layer's weight is 1." {
    default 300
    min 0
    max 4000
    tunable true
  }
  field distance f32 "How far from the camera blades are drawn; they thin out to nothing there." {
    default 150
    min 0
    max 1000
    unit m
    tunable true
  }
  field farTint bool "The terrain takes the field's average colour under its blades, where they thin out and past them, so the gaps and the far field read as the same grass." {
    default true
  }
  field height f32 "The tallest blade." {
    default 0.45
    min 0
    max 4
    unit m
    tunable true
  }
  field heightVariation f32 "How much shorter blades are than the tallest: 0 all alike, 1 down to nothing." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field width f32 "A blade's width at its root." {
    default 0.012
    min 0
    max 0.5
    unit m
    tunable true
  }
  field curve f32 "How far blades droop along their length: 0 straight, 1 arching over." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field lean f32 "How far blades lean at rest, as a share of their height." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field clumpSize f32 "The size of the clumps blades grow in, sharing their tint and height." {
    default 1.2
    min 0.05
    max 20
    unit m
    tunable true
  }
  field clumpLean f32 "How far blades lean out from their clump's centre, as a share of their height." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field tufts f32 "How far blades are pulled in towards their clump's centre: 0 an even lawn, 1 tight tufts." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field baseColor Color "The roots' colour (linear)." {
    default 0.04 0.085 0.024
  }
  field tipColor Color "The tips' colour (linear)." {
    default 0.235 0.39 0.11
  }
  field dryColor Color "Dry grass's colour (linear): dry clumps and drying tips." {
    default 0.45 0.38 0.17
  }
  field colorVariation f32 "How much blades vary in brightness by clump, blade and patch." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field dryness f32 "The share of clumps that are dry, and how far tips dry." {
    default 0.15
    min 0
    max 1
    unit ratio
    tunable true
  }
  field translucency f32 "How much sunlight passes through a blade lit from behind, tinted by it." {
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
  field sheen f32 "The upper blade's gloss: gusts show as a moving sheen." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field roughness f32 "The blade's roughness." {
    default 0.6
    min 0.04
    max 1
    tunable true
  }
  field sway f32 "How much og.Wind bends the blades: 1 as it blows, 0 still." {
    default 1
    min 0
    max 2
    tunable true
  }
  field stiffness f32 "How stiff the blades are: 0 bends furthest, 1 least, and sways faster." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field flutter f32 "How much the tips of near blades shiver in the wind." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field castShadows bool "Whether the blades cast the sun's shadows (near the camera, into the first cascade)."
  field seed u32 "Another seed places the blades elsewhere." {
    default 1
  }
}
```

<sub>`api.ogt` line 28497</sub>

## Types

<a id="type-TerrainData"></a>

### `TerrainData` (handle)

A loaded terrain: its heights, control map and layers.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TerrainData "A loaded terrain: its heights, control map and layers."
```

<sub>`api.ogt` line 28363</sub>

<a id="type-TerrainBrushMode"></a>

### `TerrainBrushMode` (enum)

What og.terrain.edit does under its brush.

A string: one of the variants below. The zero value is `"raise"`.

| Variant | Doc |
|---|---|
| `"raise"` | Raise the heights by `strength` metres at the centre. |
| `"lower"` | Lower them by `strength` metres at the centre. |
| `"flatten"` | Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre. |
| `"smooth"` | Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre. |
| `"paint"` | Paint `layer` into the control map, by `strength` (0 to 1) at the centre. |
| `"hole"` | Cut holes: every cell whose centre the brush weighs at least half (its falloff times `strength`) is cut out of the surface, neither drawn nor collided with. |
| `"fill"` | Fill holes in again where `hole` would cut them. |

```ogt
enum TerrainBrushMode {
  doc "What og.terrain.edit does under its brush."
  variant raise "Raise the heights by `strength` metres at the centre."
  variant lower "Lower them by `strength` metres at the centre."
  variant flatten "Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre."
  variant smooth "Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre."
  variant paint "Paint `layer` into the control map, by `strength` (0 to 1) at the centre."
  variant hole "Cut holes: every cell whose centre the brush weighs at least half (its falloff times `strength`) is cut out of the surface, neither drawn nor collided with."
  variant fill "Fill holes in again where `hole` would cut them."
}
```

<sub>`api.ogt` line 28372</sub>

<a id="type-TerrainStampMode"></a>

### `TerrainStampMode` (enum)

What og.terrain.stamp does to the height field under a stamp's mesh.

A string: one of the variants below. The zero value is `"carve"`.

| Variant | Doc |
|---|---|
| `"carve"` | Lower the ground under the mesh to `depth` below its underside (the lowest of its triangles over each height sample), so the mesh is the surface there: a cliff's face and lip, an arch, an overhang. On the edge of what the mesh covers the ground goes down to the underside itself, so the ground beside the mesh meets its foot with no trench. Ground already lower is left as it is. |
| `"hole"` | Cut out every cell whose centre the mesh covers (seen from above), neither drawn nor collided with: the mesh stands in the ground's place (a cave mouth, a tunnel's portal). |

```ogt
enum TerrainStampMode {
  doc "What og.terrain.stamp does to the height field under a stamp's mesh."
  variant carve "Lower the ground under the mesh to `depth` below its underside (the lowest of its triangles over each height sample), so the mesh is the surface there: a cliff's face and lip, an arch, an overhang. On the edge of what the mesh covers the ground goes down to the underside itself, so the ground beside the mesh meets its foot with no trench. Ground already lower is left as it is."
  variant hole "Cut out every cell whose centre the mesh covers (seen from above), neither drawn nor collided with: the mesh stands in the ground's place (a cave mouth, a tunnel's portal)."
}
```

<sub>`api.ogt` line 28383</sub>

<a id="type-TerrainInfo"></a>

### `TerrainInfo` (struct)

A loaded terrain.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | Its handle, for og.Terrain.data and this module's functions. |
| `geometry` | `Geometry` | `0` | — | — | — | The height field registered with physics under this handle: the mesh of an og.Collider of shape SHAPE_HEIGHTFIELD. |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its extent along x and z, centred on its entity. |
| `columns` | `number` | `0` | — | — | — | Height samples along x. |
| `rows` | `number` | `0` | — | — | — | Height samples along z. |
| `minHeight` | `number` | `0` | — | `m` | — | The lowest point above its entity. |
| `maxHeight` | `number` | `0` | — | `m` | — | The highest. |
| `layers` | `number` | `0` | — | — | — | Its material layers (up to 16). |
| `collision` | `boolean` | `false` | — | — | — | Whether its terrain file asks for a collider (`collision`, default true). |
| `castShadows` | `boolean` | `false` | — | — | — | Its terrain file's `castShadows`. |
| `holes` | `boolean` | `false` | — | — | — | Whether it has holes (its terrain file's `holes` map, or og.terrain.edit's `hole` brush): cells cut out of the surface, neither drawn nor collided with. |
| `seaLevel` | `number \| null` | `null` | — | `m` | — | A generated terrain's sea level (`generate { sea <m> }`) above its entity: place an og.Water there, the terrain's size; null without a sea. |

```ogt
struct TerrainInfo {
  doc "A loaded terrain."
  field data TerrainData "Its handle, for og.Terrain.data and this module's functions."
  field geometry Geometry "The height field registered with physics under this handle: the mesh of an og.Collider of shape SHAPE_HEIGHTFIELD."
  field size Vector2 "Its extent along x and z, centred on its entity." {
    unit m
  }
  field columns u32 "Height samples along x."
  field rows u32 "Height samples along z."
  field minHeight f32 "The lowest point above its entity." {
    unit m
  }
  field maxHeight f32 "The highest." {
    unit m
  }
  field layers u32 "Its material layers (up to 16)."
  field collision bool "Whether its terrain file asks for a collider (`collision`, default true)."
  field castShadows bool "Its terrain file's `castShadows`."
  field holes bool "Whether it has holes (its terrain file's `holes` map, or og.terrain.edit's `hole` brush): cells cut out of the surface, neither drawn nor collided with."
  field seaLevel Option.f32 "A generated terrain's sea level (`generate { sea <m> }`) above its entity: place an og.Water there, the terrain's size; null without a sea." {
    unit m
  }
}
```

<sub>`api.ogt` line 28643</sub>

<a id="type-TerrainSampleDesc"></a>

### `TerrainSampleDesc` (desc struct)

Where og.terrain.sample looks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain. |
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it stands: its og.Terrain entity's world position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How it is turned: its og.Terrain entity's world rotation (the zero value: along world x and z). |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How it is scaled: its og.Terrain entity's world scale. |
| `x` | `number` | `0` | — | `m` | — | The world x. |
| `z` | `number` | `0` | — | `m` | — | The world z. |

```ogt
struct TerrainSampleDesc {
  doc "Where og.terrain.sample looks."
  desc true
  field data TerrainData "The terrain."
  field origin Vector3 "Where it stands: its og.Terrain entity's world position." {
    unit m
  }
  field quaternion Quaternion "How it is turned: its og.Terrain entity's world rotation (the zero value: along world x and z)."
  field scale Vector3 "How it is scaled: its og.Terrain entity's world scale." {
    default 1 1 1
  }
  field x f32 "The world x." {
    unit m
  }
  field z f32 "The world z." {
    unit m
  }
}
```

<sub>`api.ogt` line 28667</sub>

<a id="type-TerrainSample"></a>

### `TerrainSample` (struct)

The terrain under a point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0` | — | `m` | — | The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface normal, smoothed between samples as the renderer shades it. |
| `slope` | `number` | `0` | — | `deg` | — | The normal's angle from straight up. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point is over the terrain (outside, the nearest edge answers). |
| `hole` | `boolean` | `false` | — | — | — | Whether the point is over one of its holes: nothing collides there, and `height` is the surface the hole was cut from. |

```ogt
struct TerrainSample {
  doc "The terrain under a point."
  field height f32 "The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y." {
    unit m
  }
  field normal Vector3 "The surface normal, smoothed between samples as the renderer shades it."
  field slope f32 "The normal's angle from straight up." {
    unit deg
  }
  field inside bool "Whether the point is over the terrain (outside, the nearest edge answers)."
  field hole bool "Whether the point is over one of its holes: nothing collides there, and `height` is the surface the hole was cut from."
}
```

<sub>`api.ogt` line 28686</sub>

<a id="type-TerrainBounds"></a>

### `TerrainBounds` (struct)

A terrain's box in world space.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The lowest corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The highest corner. |

```ogt
struct TerrainBounds {
  doc "A terrain's box in world space."
  field min Vector3 "The lowest corner." {
    unit m
  }
  field max Vector3 "The highest corner." {
    unit m
  }
}
```

<sub>`api.ogt` line 28699</sub>

<a id="type-TerrainBrush"></a>

### `TerrainBrush` (desc struct)

One runtime edit of a terrain (og.terrain.edit).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mode` | `TerrainBrushMode` | `"raise"` | — | — | — | What it does. |
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the terrain stands: its og.Terrain entity's world position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How it is turned: its og.Terrain entity's world rotation. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How it is scaled: its og.Terrain entity's world scale. |
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The brush's centre in world space; its y is flatten's target height. |
| `radius` | `number` | `8` | ≥ 0.01 | `m` | — | The brush's radius in world metres (across the terrain's surface). |
| `strength` | `number` | `1` | ≥ 0 | — | — | World metres at the centre (raise, lower), or how far to go, 0 to 1 (flatten, smooth, paint; hole and fill cut where it times the falloff is at least half). |
| `falloff` | `number` | `0.5` | 0 … 1 | `ratio` | — | The outer part of the radius over which it fades to nothing (0: a hard edge, 1: fading from the centre). |
| `layer` | `number` | `0` | ≤ 15 | — | — | paint: the layer painted, 0 to 15. |

```ogt
struct TerrainBrush {
  doc "One runtime edit of a terrain (og.terrain.edit)."
  desc true
  field mode TerrainBrushMode "What it does."
  field origin Vector3 "Where the terrain stands: its og.Terrain entity's world position." {
    unit m
  }
  field quaternion Quaternion "How it is turned: its og.Terrain entity's world rotation."
  field scale Vector3 "How it is scaled: its og.Terrain entity's world scale." {
    default 1 1 1
  }
  field center Vector3 "The brush's centre in world space; its y is flatten's target height." {
    unit m
  }
  field radius f32 "The brush's radius in world metres (across the terrain's surface)." {
    default 8
    min 0.01
    unit m
  }
  field strength f32 "World metres at the centre (raise, lower), or how far to go, 0 to 1 (flatten, smooth, paint; hole and fill cut where it times the falloff is at least half)." {
    default 1
    min 0
  }
  field falloff f32 "The outer part of the radius over which it fades to nothing (0: a hard edge, 1: fading from the centre)." {
    default 0.5
    min 0
    max 1
    unit ratio
  }
  field layer u32 "paint: the layer painted, 0 to 15." {
    max 15
  }
}
```

<sub>`api.ogt` line 28709</sub>

<a id="type-TerrainEdit"></a>

### `TerrainEdit` (struct)

What an edit changed.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `number` | `0` | — | — | — | Height or control-map samples it touched. |
| `minHeight` | `number` | `0` | — | `m` | — | The terrain's lowest point now. |
| `maxHeight` | `number` | `0` | — | `m` | — | Its highest now. |
| `stamp` | `number` | `0` | — | — | — | og.terrain.stamp: the stamp's id, for og.terrain.unstamp (from 1). 0 for every other edit. |

```ogt
struct TerrainEdit {
  doc "What an edit changed."
  field samples u32 "Height or control-map samples it touched."
  field minHeight f32 "The terrain's lowest point now." {
    unit m
  }
  field maxHeight f32 "Its highest now." {
    unit m
  }
  field stamp u32 "og.terrain.stamp: the stamp's id, for og.terrain.unstamp (from 1). 0 for every other edit."
}
```

<sub>`api.ogt` line 28743</sub>

<a id="type-TerrainStampDesc"></a>

### `TerrainStampDesc` (desc struct)

One stamp of a mesh into a terrain (og.terrain.stamp).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain. |
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the terrain stands: its og.Terrain entity's world position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How it is turned: its og.Terrain entity's world rotation. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How it is scaled: its og.Terrain entity's world scale. |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh (its og.TerrainStamp's geometry). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the mesh stands: its og.TerrainStamp entity's world position. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How the mesh is turned: its entity's world rotation. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How the mesh is scaled: its entity's world scale. |
| `mode` | `TerrainStampMode` | `"carve"` | — | — | — | What it does to the ground. |
| `depth` | `number` | `0.25` | ≥ 0 | `m` | — | carve: how far below the mesh's underside the ground goes, so it stays hidden under the mesh. |

```ogt
struct TerrainStampDesc {
  doc "One stamp of a mesh into a terrain (og.terrain.stamp)."
  desc true
  field data TerrainData "The terrain."
  field origin Vector3 "Where the terrain stands: its og.Terrain entity's world position." {
    unit m
  }
  field quaternion Quaternion "How it is turned: its og.Terrain entity's world rotation."
  field scale Vector3 "How it is scaled: its og.Terrain entity's world scale." {
    default 1 1 1
  }
  field geometry Geometry "The mesh (its og.TerrainStamp's geometry)."
  field position Vector3 "Where the mesh stands: its og.TerrainStamp entity's world position." {
    unit m
  }
  field rotation Quaternion "How the mesh is turned: its entity's world rotation."
  field size Vector3 "How the mesh is scaled: its entity's world scale." {
    default 1 1 1
  }
  field mode TerrainStampMode "What it does to the ground."
  field depth f32 "carve: how far below the mesh's underside the ground goes, so it stays hidden under the mesh." {
    default 0.25
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 28755</sub>

<a id="type-TerrainLayerDesc"></a>

### `TerrainLayerDesc` (desc struct)

One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the layer's texture (linear). |
| `roughness` | `number` | `0.9` | 0.04 … 1 | — | — | Its roughness. |
| `tile` | `number` | `4` | 0.01 … 1000 | `m` | — | Metres per repeat of its textures. |
| `triplanar` | `boolean` | `false` | — | — | — | Sample by three projections blended by the normal (for cliffs). `projectSlope` above 0 decides instead. |
| `projectSlope` | `number` | `0` | 0 … 90 | `deg` | — | The slope past which the layer is sampled by the two projections facing the surface most (biplanar: no projection seen edge-on, so a steep face does not streak) instead of from above, faded over 10° around it; below it, from above. 0: `triplanar` decides. |
| `antiTiling` | `number` | `0` | 0 … 1 | `ratio` | — | How far the texture's repeat is broken: each point blends three hexagonal tiles of it, each turned (by up to half a turn at 1) and moved by its own hash, so no period shows across a field. 0: the plain repeat. |
| `heightContrast` | `number` | `0` | 0 … 1 | `ratio` | — | How sharp the height blend (the terrain file's `heightBlend`) is at this layer's edges: 0 the default window, 1 ten times narrower (a crisp rim). |
| `macroVariation` | `number` | `0` | 0 … 1 | `ratio` | — | A slow drift of brightness and hue over the ground and, with distance, the texture at a quarter of its frequency blended in, so a field of one layer does not read as one texture from afar. 0: none. |
| `normalStrength` | `number` | `1` | 0 … 4 | — | — | How strongly its normal map bends the surface. |
| `slopeMin` | `number` | `0` | 0 … 90 | `deg` | — | The flattest slope it shows on. |
| `slopeMax` | `number` | `90` | 0 … 90 | `deg` | — | The steepest. |
| `slopeFade` | `number` | `4` | 0 … 90 | `deg` | — | How far past its slopes it fades out. |
| `altitudeMin` | `number` | `-1000000` | — | `m` | — | The lowest height above the terrain's origin it shows at. |
| `altitudeMax` | `number` | `1000000` | — | `m` | — | The highest. |
| `altitudeFade` | `number` | `4` | ≥ 0 | `m` | — | How far past its heights it fades out. |

```ogt
struct TerrainLayerDesc {
  doc "One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised."
  desc true
  field color Color "Multiplies the layer's texture (linear)." {
    default 1 1 1
  }
  field roughness f32 "Its roughness." {
    default 0.9
    min 0.04
    max 1
  }
  field tile f32 "Metres per repeat of its textures." {
    default 4
    min 0.01
    max 1000
    unit m
  }
  field triplanar bool "Sample by three projections blended by the normal (for cliffs). `projectSlope` above 0 decides instead."
  field projectSlope f32 "The slope past which the layer is sampled by the two projections facing the surface most (biplanar: no projection seen edge-on, so a steep face does not streak) instead of from above, faded over 10° around it; below it, from above. 0: `triplanar` decides." {
    min 0
    max 90
    unit deg
  }
  field antiTiling f32 "How far the texture's repeat is broken: each point blends three hexagonal tiles of it, each turned (by up to half a turn at 1) and moved by its own hash, so no period shows across a field. 0: the plain repeat." {
    min 0
    max 1
    unit ratio
  }
  field heightContrast f32 "How sharp the height blend (the terrain file's `heightBlend`) is at this layer's edges: 0 the default window, 1 ten times narrower (a crisp rim)." {
    min 0
    max 1
    unit ratio
  }
  field macroVariation f32 "A slow drift of brightness and hue over the ground and, with distance, the texture at a quarter of its frequency blended in, so a field of one layer does not read as one texture from afar. 0: none." {
    min 0
    max 1
    unit ratio
  }
  field normalStrength f32 "How strongly its normal map bends the surface." {
    default 1
    min 0
    max 4
  }
  field slopeMin f32 "The flattest slope it shows on." {
    min 0
    max 90
    unit deg
  }
  field slopeMax f32 "The steepest." {
    default 90
    min 0
    max 90
    unit deg
  }
  field slopeFade f32 "How far past its slopes it fades out." {
    default 4
    min 0
    max 90
    unit deg
  }
  field altitudeMin f32 "The lowest height above the terrain's origin it shows at." {
    default -1000000
    unit m
  }
  field altitudeMax f32 "The highest." {
    default 1000000
    unit m
  }
  field altitudeFade f32 "How far past its heights it fades out." {
    default 4
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 28782</sub>

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

<sub>`api.ogt` line 28365</sub>
