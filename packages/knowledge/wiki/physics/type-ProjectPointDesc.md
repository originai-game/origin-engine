<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ProjectPointDesc"></a>

### `ProjectPointDesc` (desc struct)

A point for og.physics.projectPoint.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The point to project (its z is ignored in 2D). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far a collider may be; 0 is unbounded. |
| `solid` | `boolean` | `true` | — | — | — | A point inside a collider is its own projection (distance 0); false projects it onto the collider's surface. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct ProjectPointDesc {
  doc "A point for og.physics.projectPoint."
  desc true
  field point Vector3 "The point to project (its z is ignored in 2D)." {
    unit m
  }
  field maxDistance f32 "How far a collider may be; 0 is unbounded." {
    min 0
    unit m
  }
  field solid bool "A point inside a collider is its own projection (distance 0); false projects it onto the collider's surface." {
    default true
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>`api.ogt` line 10237</sub>
