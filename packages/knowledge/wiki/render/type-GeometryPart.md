<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GeometryPart"></a>

### `GeometryPart` (desc struct)

A geometry placed as an Object3D is (scale, then rotate, then move), then mirrored: one part of og.render.mergeGeometries or an operand of og.render.csgGeometry.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | Any geometry: made, a primitive or a loaded glTF's. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it goes. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How it turns. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | Its scale (a negative component mirrors it as `mirror` does). |
| `mirror` | `MirrorAxis` | `"none"` | — | — | — | Mirror it along a world axis after placing it: a symmetric prop is one half twice, the second mirrored. |

```ogt
struct GeometryPart {
  doc "A geometry placed as an Object3D is (scale, then rotate, then move), then mirrored: one part of og.render.mergeGeometries or an operand of og.render.csgGeometry."
  desc true
  field geometry Geometry "Any geometry: made, a primitive or a loaded glTF's."
  field position Vector3 "Where it goes." {
    unit m
  }
  field quaternion Quaternion "How it turns."
  field scale Vector3 "Its scale (a negative component mirrors it as `mirror` does)." {
    default 1 1 1
  }
  field mirror MirrorAxis "Mirror it along a world axis after placing it: a symmetric prop is one half twice, the second mirrored."
}
```

<sub>Module `render` line 3591</sub>
