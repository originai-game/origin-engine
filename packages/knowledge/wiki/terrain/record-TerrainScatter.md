<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-TerrainScatter"></a>

### `og.TerrainScatter` (component)

Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, each cell growing one with the probability of the layer's weight there. Over the outer `fade` of `distance` copies shrink and thin out. The renderer places them a chunk at a time, once. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities.

See `world §3`.

Token `og.TerrainScatter`, registered under the schema name `og.TerrainScatter`; its value type is `TerrainScatter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `geometry` | `Geometry` | `0` | — | — | — | The mesh, its origin at its root. When it belongs to the named impostor model, all of that primitive's node placements are applied in model space, at every distance (including impostorDistance 0). Other geometry must already be in that space. |
| `material` | `Material` | `0` | — | — | — | Its material. |
| `layer` | `number` | `0` | ≤ 15 | — | — | The terrain layer that places it, 0 to 15. |
| `density` | `number` | `2` | 0 … 400 | — | tunable | Copies per square metre where the layer's weight is 1. |
| `distance` | `number` | `40` | 0 … 500 | `m` | tunable | How far from the camera, along x and z, copies are placed. |
| `fade` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The outer part of `distance` over which copies shrink and thin out to nothing. |
| `minScale` | `number` | `0.8` | ≥ 0 | — | tunable | The smallest copy's uniform scale. |
| `maxScale` | `number` | `1.2` | ≥ 0 | — | tunable | The largest copy's uniform scale. |
| `align` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far copies lean with the ground: 0 upright, 1 along its normal. |
| `seed` | `number` | `1` | — | — | — | Another seed places the copies elsewhere. |
| `castShadows` | `boolean` | `true` | — | — | — | Whether the copies cast shadows (grass usually does not: three.js meshes cast only with castShadow set). |
| `sway` | `number` | `1` | 0 … 2 | — | tunable | How much og.Wind sways the copies: 1 as it blows, 0 still (stones), between for stiffer stems. Each vertex above the mesh's origin moves by its height, so model the copy with its root at the origin. |
| `impostor` | `Asset` | `0` | — | — | — | A glTF (og.load.gltf) of the same mesh cooked with an octahedral impostor (`model { impostor true }`): copies past impostorDistance draw as it. Mesh and impostor bounds must agree in model space, else the scatter is skipped (render.scatterIssue). None: the mesh at every distance. See `world §3`. |
| `impostorDistance` | `number` | `0` | 0 … 500 | `m` | tunable | How far from the camera, along x and z, copies draw as their mesh before their impostor takes over; 0 never switches. Useful below `distance`. |
| `noiseScale` | `number` | `8` | 0.1 … 1000 | `m` | tunable | The size of the patches noiseThreshold gathers copies into. |
| `noiseThreshold` | `number` | `0` | 0 … 1 | `ratio` | tunable | Gathers the copies into patches: a cell grows its copy only where a smooth noise of noiseScale patches (0 to 1, about half of it past 0.5) rises past this, the patches' edges thinning over a short band. 0: everywhere. |
| `footprint` | `number` | `0` | 0 … 100 | `m` | tunable | How far, along x and z, this scatter's copies keep from the copies of every og.TerrainScatter placed before it over the same og.Terrain: the larger of the two footprints (trees placed first keep flowers from their trunks). 0: none. |
| `priority` | `number` | `0` | — | — | — | Scatters of a higher priority over the same og.Terrain are placed first, their footprints cleared in the ones after; equal priorities are placed in entity order. |
| `avoidWater` | `number` | `0` | 0 … 100 | `m` | tunable | Grows no copy whose ground stands less than this above an og.Water surface over it (an endless sea everywhere), nor under it: reeds stop short of the shore. 0: anywhere. |
| `densityMap` | `Texture` | `0` | — | — | — | Thins the copies where its red channel is below 1, over the terrain's extent (top row along -z, loaded linear), each copy kept while its own value is below the map's. Drawn only, where the renderer culls on the GPU (RenderCapabilities.compute). 0: none. See `world §3`. |
| `gpuPlacement` | `boolean` | `true` | — | — | — | Place the copies on the GPU, a chunk per compute pass by the same rules, nothing per copy on the CPU; matching the CPU's but for the last bits. Without compute (RenderCapabilities.compute), or for a footprint chain too long or wide, the CPU places them. False places every chunk on the CPU. See `world §3`. |

```ogt
component TerrainScatter {
  doc "Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, each cell growing one with the probability of the layer's weight there. Over the outer `fade` of `distance` copies shrink and thin out. The renderer places them a chunk at a time, once. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities."
  guide "world §3"
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
  field fade f32 "The outer part of `distance` over which copies shrink and thin out to nothing." {
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
  field impostor Asset "A glTF (og.load.gltf) of the same mesh cooked with an octahedral impostor (`model { impostor true }`): copies past impostorDistance draw as it. Mesh and impostor bounds must agree in model space, else the scatter is skipped (render.scatterIssue). None: the mesh at every distance." {
    guide "world §3"
  }
  field impostorDistance f32 "How far from the camera, along x and z, copies draw as their mesh before their impostor takes over; 0 never switches. Useful below `distance`." {
    min 0
    max 500
    unit m
    tunable true
  }
  field noiseScale f32 "The size of the patches noiseThreshold gathers copies into." {
    default 8
    min 0.1
    max 1000
    unit m
    tunable true
  }
  field noiseThreshold f32 "Gathers the copies into patches: a cell grows its copy only where a smooth noise of noiseScale patches (0 to 1, about half of it past 0.5) rises past this, the patches' edges thinning over a short band. 0: everywhere." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field footprint f32 "How far, along x and z, this scatter's copies keep from the copies of every og.TerrainScatter placed before it over the same og.Terrain: the larger of the two footprints (trees placed first keep flowers from their trunks). 0: none." {
    min 0
    max 100
    unit m
    tunable true
  }
  field priority u32 "Scatters of a higher priority over the same og.Terrain are placed first, their footprints cleared in the ones after; equal priorities are placed in entity order."
  field avoidWater f32 "Grows no copy whose ground stands less than this above an og.Water surface over it (an endless sea everywhere), nor under it: reeds stop short of the shore. 0: anywhere." {
    min 0
    max 100
    unit m
    tunable true
  }
  field densityMap Texture "Thins the copies where its red channel is below 1, over the terrain's extent (top row along -z, loaded linear), each copy kept while its own value is below the map's. Drawn only, where the renderer culls on the GPU (RenderCapabilities.compute). 0: none." {
    guide "world §3"
  }
  field gpuPlacement bool "Place the copies on the GPU, a chunk per compute pass by the same rules, nothing per copy on the CPU; matching the CPU's but for the last bits. Without compute (RenderCapabilities.compute), or for a footprint chain too long or wide, the CPU places them. False places every chunk on the CPU." {
    guide "world §3"
    default true
  }
}
```

<sub>Module `terrain` line 90</sub>
