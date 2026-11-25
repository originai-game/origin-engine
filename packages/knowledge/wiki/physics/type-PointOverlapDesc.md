<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-PointOverlapDesc"></a>

### `PointOverlapDesc` (desc struct)

A point placed in the world for og.physics.overlapPoint.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The point to test (z is ignored in 2D). |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct PointOverlapDesc {
  doc "A point placed in the world for og.physics.overlapPoint."
  desc true
  field point Vector3 "The point to test (z is ignored in 2D)." {
    unit m
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>Module `physics` line 2380</sub>
