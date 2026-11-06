<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

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

<sub>`api.ogt` line 24859</sub>
