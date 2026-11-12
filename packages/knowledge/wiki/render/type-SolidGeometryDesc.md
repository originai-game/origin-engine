<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SolidGeometryDesc"></a>

### `SolidGeometryDesc` (desc struct)

A regular solid, as three.js IcosahedronGeometry, DodecahedronGeometry, OctahedronGeometry and TetrahedronGeometry(radius, detail): PolyhedronGeometryDesc with the solid's own vertices and faces. An icosahedron at detail 2 or more is a sphere with even triangles.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The radius. |
| `detail` | `number` | `0` | ≤ 64 | — | — | Subdivisions of each face. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct SolidGeometryDesc {
  doc "A regular solid, as three.js IcosahedronGeometry, DodecahedronGeometry, OctahedronGeometry and TetrahedronGeometry(radius, detail): PolyhedronGeometryDesc with the solid's own vertices and faces. An icosahedron at detail 2 or more is a sphere with even triangles."
  desc true
  field radius f32 "The radius." {
    default 1
    min 0
    unit m
  }
  field detail u32 "Subdivisions of each face." {
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

<sub>Module `render` line 3528</sub>
