<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RaycastAllDesc"></a>

### `RaycastAllDesc` (desc struct)

A ray for og.physics.castRayAll: og.physics.castRay's fields and a cap on the hits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: -1, z: 0 }` | — | — | — | Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far it reaches; 0 is unbounded. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |
| `maxHits` | `number` | `0` | 0 … 256 | — | — | The most hits returned, nearest first; 0 is MAX_RAYCAST_HITS (256), and more is held to it. |

```ogt
struct RaycastAllDesc {
  doc "A ray for og.physics.castRayAll: og.physics.castRay's fields and a cap on the hits."
  desc true
  field origin Vector3 "Where the ray starts." {
    unit m
  }
  field direction Vector3 "Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy)." {
    default 0 -1 0
  }
  field maxDistance f32 "How far it reaches; 0 is unbounded." {
    min 0
    unit m
  }
  field filter QueryFilter "Which colliders count."
  field maxHits u32 "The most hits returned, nearest first; 0 is MAX_RAYCAST_HITS (256), and more is held to it." {
    min 0
    max 256
  }
}
```

<sub>Module `physics` line 1778</sub>
