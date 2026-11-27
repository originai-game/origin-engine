<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-MeshDesc"></a>

### `MeshDesc` (desc struct)

Geometry for convex, triangle-mesh and polyline colliders, in the mesh's own space.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `readonly Vector3[]` | `[]` | — | — | — | Vertex positions (2D polygons use x and y). |
| `indices` | `readonly number[]` | `[]` | — | — | — | Triangles as triples of indices into positions; a convex hull ignores them; a polyline uses empty for an open chain or [a, b, a] triples per segment. |

```ogt
struct MeshDesc {
  doc "Geometry for convex, triangle-mesh and polyline colliders, in the mesh's own space."
  desc true
  field positions List.Vector3 "Vertex positions (2D polygons use x and y)."
  field indices List.u32 "Triangles as triples of indices into positions; a convex hull ignores them; a polyline uses empty for an open chain or [a, b, a] triples per segment."
}
```

<sub>Module `physics` line 2399</sub>
