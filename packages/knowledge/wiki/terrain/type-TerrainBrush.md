<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

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

<sub>Module `terrain` line 469</sub>
