<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PolyhedronGeometryDesc"></a>

### `PolyhedronGeometryDesc` (desc struct)

A polyhedron from vertices and faces, every face subdivided `detail` times and every vertex pushed onto the sphere of `radius`, as three.js PolyhedronGeometry(vertices, indices, radius, detail): unindexed, flat normals at detail 0 and the sphere's above, three.js's spherical uvs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vertices` | `readonly number[]` | `[]` | — | — | — | The corners, x y z each (three.js's flat array). |
| `indices` | `readonly number[]` | `[]` | — | — | — | The faces, three corner indices each, counter-clockwise seen from outside. |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The sphere the vertices are pushed onto. |
| `detail` | `number` | `0` | ≤ 64 | — | — | Subdivisions of each face: (detail + 1)² triangles a face. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct PolyhedronGeometryDesc {
  doc "A polyhedron from vertices and faces, every face subdivided `detail` times and every vertex pushed onto the sphere of `radius`, as three.js PolyhedronGeometry(vertices, indices, radius, detail): unindexed, flat normals at detail 0 and the sphere's above, three.js's spherical uvs."
  desc true
  field vertices List.f32 "The corners, x y z each (three.js's flat array)."
  field indices List.u32 "The faces, three corner indices each, counter-clockwise seen from outside."
  field radius f32 "The sphere the vertices are pushed onto." {
    default 1
    min 0
    unit m
  }
  field detail u32 "Subdivisions of each face: (detail + 1)² triangles a face." {
    max 64
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>`api.ogt` line 7039</sub>
