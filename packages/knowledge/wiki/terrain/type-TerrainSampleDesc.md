<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

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

<sub>`api.ogt` line 25175</sub>
