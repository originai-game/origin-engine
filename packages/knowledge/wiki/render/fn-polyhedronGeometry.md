<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-polyhedronGeometry"></a>

### `og.render.polyhedronGeometry`

Make a polyhedron from vertices and faces for og.Mesh, subdivided onto a sphere.

```ts
og.render.polyhedronGeometry(world: World, desc?: PolyhedronGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `PolyhedronGeometryDesc` | yes | The vertices, faces, radius and detail. |

**Returns** `Result<Geometry>`: Refused for no faces, an index past the vertices, or a count that is not a multiple of three.

Binding symbol `og_render_polyhedron_geometry` · publish builds: kept

```ogt
fn polyhedronGeometry {
  doc "Make a polyhedron from vertices and faces for og.Mesh, subdivided onto a sphere."
  param world World "The world whose renderer owns the geometry."
  param desc PolyhedronGeometryDesc "The vertices, faces, radius and detail."
  returns Result.Geometry "Refused for no faces, an index past the vertices, or a count that is not a multiple of three."
}
```

<sub>`api.ogt` line 7194</sub>
