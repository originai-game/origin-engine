<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-MergeGeometriesDesc"></a>

### `MergeGeometriesDesc` (desc struct)

Geometries placed and merged into one, as three.js BufferGeometryUtils.mergeGeometries after applying each part's matrix: a prop of parts drawn in one draw, a mirrored half, an array (the same part at several placements). Normals follow each placement; a mirrored part turns its triangles so they still face out. Vertex colours are kept when any part has them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `parts` | `readonly GeometryPart[]` | `[]` | — | — | — | The parts, in order (at least one). |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct MergeGeometriesDesc {
  doc "Geometries placed and merged into one, as three.js BufferGeometryUtils.mergeGeometries after applying each part's matrix: a prop of parts drawn in one draw, a mirrored half, an array (the same part at several placements). Normals follow each placement; a mirrored part turns its triangles so they still face out. Vertex colours are kept when any part has them."
  desc true
  field parts List.GeometryPart "The parts, in order (at least one)."
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>Module `render` line 3657</sub>
