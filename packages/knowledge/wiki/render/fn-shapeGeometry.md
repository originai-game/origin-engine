<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-shapeGeometry"></a>

### `og.render.shapeGeometry`

Make a flat shape for og.Mesh (one-sided, as circleGeometry): an og.Shape with holes, or a points array. Physics can use it as a triangle-mesh Collider.mesh.

```ts
og.render.shapeGeometry(world: World, desc?: ShapeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `ShapeGeometryDesc` | yes | The shapes and their tessellation. |

**Returns** `Result<Geometry>`: Refused for a shape whose points are not finite or that triangulates to nothing.

Binding symbol `og_render_shape_geometry` · publish builds: kept

```ogt
fn shapeGeometry {
  doc "Make a flat shape for og.Mesh (one-sided, as circleGeometry): an og.Shape with holes, or a points array. Physics can use it as a triangle-mesh Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc ShapeGeometryDesc "The shapes and their tessellation."
  returns Result.Geometry "Refused for a shape whose points are not finite or that triangulates to nothing."
}
```

<sub>Module `render` line 3395</sub>
