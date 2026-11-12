<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

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

<sub>Module `terrain` line 658</sub>
