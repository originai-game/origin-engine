<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-Transform"></a>

### `og.Transform` (component)

Where an entity is, relative to its parent (or the world).

Token `og.Transform`, registered under the schema name `og.Transform`; its value type is `Transform`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `root` | The position, as three.js Object3D.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The rotation, as three.js Object3D.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale, as three.js Object3D.scale. |

```ogt
component Transform {
  doc "Where an entity is, relative to its parent (or the world)."
  field position Vector3 "The position, as three.js Object3D.position." {
    worldspace root
    unit m
  }
  field quaternion Quaternion "The rotation, as three.js Object3D.quaternion."
  field scale Vector3 "The scale, as three.js Object3D.scale." {
    default 1 1 1
  }
}
```

<sub>`api.ogt` line 1482</sub>
