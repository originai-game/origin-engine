<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-GeometryHit"></a>

### `GeometryHit` (struct)

Where a ray meets a drawn triangle, as three.js Raycaster's intersection.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Mesh's or og.Model's entity. |
| `geometry` | `Geometry` | `0` | — | — | — | The geometry hit. |
| `distance` | `number` | `0` | — | `m` | — | From the ray's origin along its normalised direction. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where, in world space. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The triangle's face normal in world space (its front side, counter-clockwise), unit length. |
| `uv` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The texture coordinates there, interpolated. |
| `faceIndex` | `number` | `0` | — | — | — | The triangle's index in the geometry, as three.js faceIndex. |

```ogt
struct GeometryHit {
  doc "Where a ray meets a drawn triangle, as three.js Raycaster's intersection."
  field entity Entity "The og.Mesh's or og.Model's entity."
  field geometry Geometry "The geometry hit."
  field distance f32 "From the ray's origin along its normalised direction." {
    unit m
  }
  field point Vector3 "Where, in world space." {
    unit m
  }
  field normal Vector3 "The triangle's face normal in world space (its front side, counter-clockwise), unit length."
  field uv Vector2 "The texture coordinates there, interpolated."
  field faceIndex u32 "The triangle's index in the geometry, as three.js faceIndex."
}
```

<sub>Module `render` line 3632</sub>
