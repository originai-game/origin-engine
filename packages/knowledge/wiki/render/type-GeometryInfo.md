<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GeometryInfo"></a>

### `GeometryInfo` (struct)

What a geometry holds (og.render.geometryInfo).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vertices` | `number` | `0` | — | — | — | Vertices. |
| `triangles` | `number` | `0` | — | — | — | Triangles. |
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bounds' least corner in the geometry's own space, as three.js boundingBox.min. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bounds' greatest corner. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it. |

```ogt
struct GeometryInfo {
  doc "What a geometry holds (og.render.geometryInfo)."
  field vertices u32 "Vertices."
  field triangles u32 "Triangles."
  field min Vector3 "The bounds' least corner in the geometry's own space, as three.js boundingBox.min." {
    unit m
  }
  field max Vector3 "The bounds' greatest corner." {
    unit m
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it."
}
```

<sub>Module `render` line 3094</sub>
