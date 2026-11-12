<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-BufferGeometryDesc"></a>

### `BufferGeometryDesc` (desc struct)

Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices).

Missing normals are computed, as three.js computeVertexNormals (area-weighted: shared vertices shade smooth, an unindexed geometry flat); missing tangents are generated when a normal map needs them. An unindexed geometry whose vertex count is not a multiple of three has no triangles: it draws nothing as an og.Mesh and is a vertex set for og.Lines and og.Points.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `Float32Array` | — | — | — | — | The vertex positions, x y z per vertex (three.js `position`); at least one vertex. |
| `normals` | `Float32Array \| null` | `null` | — | — | — | Unit normals, x y z per vertex; null computes them. |
| `uvs` | `Float32Array \| null` | `null` | — | — | — | Texture coordinates, u v per vertex, v down the image (glTF's, as og-render's); null is 0 0. |
| `colors` | `Float32Array \| null` | `null` | — | — | — | Linear vertex colours, r g b or r g b a per vertex, multiplied into the material's colour (three.js `color` with vertexColors); null is white. |
| `tangents` | `Float32Array \| null` | `null` | — | — | — | Tangents, x y z w per vertex (glTF's: w the bitangent's sign); null generates them. |
| `indices` | `Uint32Array \| null` | `null` | — | — | — | Triangles, three vertex indices each, counter-clockwise seen from the front; null takes every three vertices as one. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it (a mesh made each tick: water, a deforming terrain, a trail of geometry). A static geometry is refused there. |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too, so an og.Collider with it as `mesh` (SHAPE_MESH or SHAPE_CONVEX) collides with what is drawn; equivalent to og.physics.setMesh with its positions and indices. An update re-registers it. |

```ogt
struct BufferGeometryDesc {
  doc "Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices)."
  doc "Missing normals are computed, as three.js computeVertexNormals (area-weighted: shared vertices shade smooth, an unindexed geometry flat); missing tangents are generated when a normal map needs them. An unindexed geometry whose vertex count is not a multiple of three has no triangles: it draws nothing as an og.Mesh and is a vertex set for og.Lines and og.Points."
  desc true
  field positions Float32Array "The vertex positions, x y z per vertex (three.js `position`); at least one vertex."
  field normals Option.Float32Array "Unit normals, x y z per vertex; null computes them."
  field uvs Option.Float32Array "Texture coordinates, u v per vertex, v down the image (glTF's, as og-render's); null is 0 0."
  field colors Option.Float32Array "Linear vertex colours, r g b or r g b a per vertex, multiplied into the material's colour (three.js `color` with vertexColors); null is white."
  field tangents Option.Float32Array "Tangents, x y z w per vertex (glTF's: w the bitangent's sign); null generates them."
  field indices Option.Uint32Array "Triangles, three vertex indices each, counter-clockwise seen from the front; null takes every three vertices as one."
  field dynamic bool "Whether og.render.updateGeometry may rewrite it (a mesh made each tick: water, a deforming terrain, a trail of geometry). A static geometry is refused there."
  field physics bool "Register it as a physics mesh too, so an og.Collider with it as `mesh` (SHAPE_MESH or SHAPE_CONVEX) collides with what is drawn; equivalent to og.physics.setMesh with its positions and indices. An update re-registers it." {
    default true
  }
}
```

<sub>Module `render` line 3109</sub>
