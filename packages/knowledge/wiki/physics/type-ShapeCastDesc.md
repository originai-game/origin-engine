<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-ShapeCastDesc"></a>

### `ShapeCastDesc` (desc struct)

A shape swept along a direction for og.physics.castShape.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ShapeDesc` | `{}` | — | — | — | The swept shape. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the sweep starts. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation; a zero quaternion is no rotation. |
| `direction` | `Vector3` | `{ x: 0, y: -1, z: 0 }` | — | — | — | Where it sweeps; it need not be normalised and must not be zero. |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far it sweeps; 0 is unbounded. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct ShapeCastDesc {
  doc "A shape swept along a direction for og.physics.castShape."
  desc true
  field shape ShapeDesc "The swept shape."
  field position Vector3 "Where the sweep starts." {
    unit m
  }
  field rotation Quaternion "The shape's rotation; a zero quaternion is no rotation."
  field direction Vector3 "Where it sweeps; it need not be normalised and must not be zero." {
    default 0 -1 0
  }
  field maxDistance f32 "How far it sweeps; 0 is unbounded." {
    min 0
    unit m
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>Module `physics` line 1897</sub>
