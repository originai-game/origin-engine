<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-OverlapDesc"></a>

### `OverlapDesc` (desc struct)

A shape placed in the world for og.physics.overlap.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ShapeDesc` | `{}` | — | — | — | The shape. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation; a zero quaternion is no rotation. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct OverlapDesc {
  doc "A shape placed in the world for og.physics.overlap."
  desc true
  field shape ShapeDesc "The shape."
  field position Vector3 "Where it is." {
    unit m
  }
  field rotation Quaternion "Its rotation; a zero quaternion is no rotation."
  field filter QueryFilter "Which colliders count."
}
```

<sub>`api.ogt` line 10309</sub>
