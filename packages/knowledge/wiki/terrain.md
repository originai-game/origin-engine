<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `terrain` module

Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a 16-bit grey PNG or an EXR heightmap, an optional control map, up to four layered materials with slope and altitude rules) into tiles; drawn as a geometry clipmap around the camera (nested levels by distance, no cracks) that casts and receives shadows under the fog; collided with as a height field; baked into navmeshes by a navmesh file's `terrain` entry.

Load it with og.terrain.load, then place it: an og.Terrain on an entity draws it placed by the entity's world transform (centred on its position, turned and scaled with it), and an og.Collider of shape SHAPE_HEIGHTFIELD naming the TerrainInfo's geometry makes it solid. og.TerrainScatter grows grass, flowers or stones over a layer near the camera. og.terrain.sample answers the surface's height and normal anywhere (pass it the entity's position, quaternion and scale); og.terrain.edit raises, lowers, flattens, smooths or paints it at runtime, and og.terrain.setLayer retunes a layer. Edits are world state: a snapshot keeps them, the world hash covers them, and a restore brings the ground back as it was (rebuilt from the cook with the restored edits replayed, for the renderer, physics and sample alike).

Its tiles stream (phase 3): the cook stores the field as a pyramid of tiles, each compressed on its own, and every frame the renderer keeps on the GPU only the tiles the camera's clipmap wants, within each terrain's tile budget (16 MiB by default), decoded off the frame (a worker thread natively, the browser's DecompressionStream on the Web) a few at a time, the most needed first; until a tile arrives its area draws from the coarser level that holds it, then fades in. Nothing a tick reads depends on which tiles are resident (collision and og.terrain.sample read the whole field), so a run's world hash is the same streamed or not. The probe's `terrain_stream:resident` decodes every wanted tile before each frame instead (`terrain_stream:stream[:<MiB>]` streams again, within that budget), and `dump_stream`'s `terrain` gives the tiles resident, waiting, decoding and cached.

- **TypeScript**: functions are the namespace `og.terrain`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module terrain {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 10449</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`TerrainData`](#type-TerrainData) | handle | A loaded terrain: its heights, control map and layers. |
| [`TerrainData()`](#column-TerrainData) | column kind | A TerrainData handle field. |
| [`TerrainBrushMode`](#type-TerrainBrushMode) | enum | What og.terrain.edit does under its brush. |
| [`og.Terrain`](#record-Terrain) | component | Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to. |
| [`og.TerrainScatter`](#record-TerrainScatter) | component | Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities. |
| [`TerrainInfo`](#type-TerrainInfo) | struct | A loaded terrain. |
| [`TerrainSampleDesc`](#type-TerrainSampleDesc) | desc struct | Where og.terrain.sample looks. |
| [`TerrainSample`](#type-TerrainSample) | struct | The terrain under a point. |
| [`TerrainBounds`](#type-TerrainBounds) | struct | A terrain's box in world space. |
| [`TerrainBrush`](#type-TerrainBrush) | desc struct | One runtime edit of a terrain (og.terrain.edit). |
| [`TerrainEdit`](#type-TerrainEdit) | struct | What an edit changed. |
| [`TerrainLayerDesc`](#type-TerrainLayerDesc) | desc struct | One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised. |
| [`og.terrain.load`](#fn-load) | function | Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures, and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain. |
| [`og.terrain.sample`](#fn-sample) | function | The surface under a world point: its height, normal and slope. Callable inside systems (it reads the terrain, not the world). |
| [`og.terrain.bounds`](#fn-bounds) | function | A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners. |
| [`og.terrain.edit`](#fn-edit) | function | Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none). |
| [`og.terrain.layer`](#fn-layer) | function | A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it). |
| [`og.terrain.setLayer`](#fn-setLayer) | function | Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame. |

## Functions

<a id="fn-load"></a>

### `og.terrain.load`

Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures, and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain.

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
  doc "Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures, and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain."
  param world World "The world."
  param path String "The terrain file, such as `terrain/island.ogt`."
  returns Result.TerrainInfo "Refused for a path that is not a cooked terrain, or a layer texture that does not load."
}
```

<sub>`api.ogt` line 10701</sub>

<a id="fn-sample"></a>

### `og.terrain.sample`

The surface under a world point: its height, normal and slope. Callable inside systems (it reads the terrain, not the world).

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
  doc "The surface under a world point: its height, normal and slope. Callable inside systems (it reads the terrain, not the world)."
  param world World "The world."
  param desc TerrainSampleDesc "The terrain, where it stands and the point."
  returns Result.TerrainSample "Refused for an unknown terrain."
}
```

<sub>`api.ogt` line 10708</sub>

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

<sub>`api.ogt` line 10715</sub>

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

<sub>`api.ogt` line 10731</sub>

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

<sub>`api.ogt` line 10739</sub>

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

<sub>`api.ogt` line 10747</sub>

## Components, resources and events

<a id="record-Terrain"></a>

### `og.Terrain` (component)

Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to.

Token `og.Terrain`, registered under the schema name `og.Terrain`; its value type is `Terrain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `castShadows` | `boolean` | `true` | — | — | — | Cast the directional and local lights' shadows (it always receives them). |
| `lodSpacing` | `number` | `0` | 0 … 64 | `m` | tunable | The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing). |
| `lodRing` | `number` | `0` | 0 … 256 | — | — | Quads per side of every clipmap level (0: the terrain file's, else 64; 16 to 256, rounded to a multiple of 4). |

```ogt
component Terrain {
  doc "Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field castShadows bool "Cast the directional and local lights' shadows (it always receives them)." {
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

<sub>`api.ogt` line 10473</sub>

<a id="record-TerrainScatter"></a>

### `og.TerrainScatter` (component)

Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities.

Token `og.TerrainScatter`, registered under the schema name `og.TerrainScatter`; its value type is `TerrainScatter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh, its origin at its root. |
| `material` | `Material` | `0` | — | — | — | Its material. |
| `layer` | `number` | `0` | ≤ 3 | — | — | The terrain layer that places it, 0 to 3. |
| `density` | `number` | `2` | 0 … 400 | — | tunable | Copies per square metre where the layer's weight is 1. |
| `distance` | `number` | `40` | 0 … 500 | `m` | tunable | How far from the camera, along x and z, copies are placed. |
| `fade` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The outer part of `distance` over which copies thin out to nothing. |
| `minScale` | `number` | `0.8` | ≥ 0 | — | tunable | The smallest copy's uniform scale. |
| `maxScale` | `number` | `1.2` | ≥ 0 | — | tunable | The largest copy's uniform scale. |
| `align` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far copies lean with the ground: 0 upright, 1 along its normal. |
| `seed` | `number` | `1` | — | — | — | Another seed places the copies elsewhere. |

```ogt
component TerrainScatter {
  doc "Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field geometry Geometry "The mesh, its origin at its root."
  field material Material "Its material."
  field layer u32 "The terrain layer that places it, 0 to 3." {
    max 3
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
}
```

<sub>`api.ogt` line 10491</sub>

## Types

<a id="type-TerrainData"></a>

### `TerrainData` (handle)

A loaded terrain: its heights, control map and layers.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TerrainData "A loaded terrain: its heights, control map and layers."
```

<sub>`api.ogt` line 10455</sub>

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

```ogt
enum TerrainBrushMode {
  doc "What og.terrain.edit does under its brush."
  variant raise "Raise the heights by `strength` metres at the centre."
  variant lower "Lower them by `strength` metres at the centre."
  variant flatten "Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre."
  variant smooth "Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre."
  variant paint "Paint `layer` into the control map, by `strength` (0 to 1) at the centre."
}
```

<sub>`api.ogt` line 10464</sub>

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
| `layers` | `number` | `0` | — | — | — | Its material layers (up to 4). |
| `collision` | `boolean` | `false` | — | — | — | Whether its terrain file asks for a collider (`collision`, default true). |
| `castShadows` | `boolean` | `false` | — | — | — | Its terrain file's `castShadows`. |

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
  field layers u32 "Its material layers (up to 4)."
  field collision bool "Whether its terrain file asks for a collider (`collision`, default true)."
  field castShadows bool "Its terrain file's `castShadows`."
}
```

<sub>`api.ogt` line 10540</sub>

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

<sub>`api.ogt` line 10560</sub>

<a id="type-TerrainSample"></a>

### `TerrainSample` (struct)

The terrain under a point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0` | — | `m` | — | The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface normal, smoothed between samples as the renderer shades it. |
| `slope` | `number` | `0` | — | `deg` | — | The normal's angle from straight up. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point is over the terrain (outside, the nearest edge answers). |

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
}
```

<sub>`api.ogt` line 10579</sub>

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

<sub>`api.ogt` line 10591</sub>

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
| `strength` | `number` | `1` | ≥ 0 | — | — | World metres at the centre (raise, lower), or how far to go, 0 to 1 (flatten, smooth, paint). |
| `falloff` | `number` | `0.5` | 0 … 1 | `ratio` | — | The outer part of the radius over which it fades to nothing (0: a hard edge, 1: fading from the centre). |
| `layer` | `number` | `0` | ≤ 3 | — | — | paint: the layer painted, 0 to 3. |

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
  field strength f32 "World metres at the centre (raise, lower), or how far to go, 0 to 1 (flatten, smooth, paint)." {
    default 1
    min 0
  }
  field falloff f32 "The outer part of the radius over which it fades to nothing (0: a hard edge, 1: fading from the centre)." {
    default 0.5
    min 0
    max 1
    unit ratio
  }
  field layer u32 "paint: the layer painted, 0 to 3." {
    max 3
  }
}
```

<sub>`api.ogt` line 10601</sub>

<a id="type-TerrainEdit"></a>

### `TerrainEdit` (struct)

What an edit changed.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `number` | `0` | — | — | — | Height or control-map samples it touched. |
| `minHeight` | `number` | `0` | — | `m` | — | The terrain's lowest point now. |
| `maxHeight` | `number` | `0` | — | `m` | — | Its highest now. |

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
}
```

<sub>`api.ogt` line 10635</sub>

<a id="type-TerrainLayerDesc"></a>

### `TerrainLayerDesc` (desc struct)

One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the layer's texture (linear). |
| `roughness` | `number` | `0.9` | 0.04 … 1 | — | — | Its roughness. |
| `tile` | `number` | `4` | 0.01 … 1000 | `m` | — | Metres per repeat of its textures. |
| `triplanar` | `boolean` | `false` | — | — | — | Sample by three projections blended by the normal (for cliffs). |
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
  field triplanar bool "Sample by three projections blended by the normal (for cliffs)."
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

<sub>`api.ogt` line 10646</sub>

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

<sub>`api.ogt` line 10457</sub>
