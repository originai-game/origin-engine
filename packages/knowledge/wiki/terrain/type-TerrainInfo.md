<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

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

<sub>`api.ogt` line 25194</sub>
