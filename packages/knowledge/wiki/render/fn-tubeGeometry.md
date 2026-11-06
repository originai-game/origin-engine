<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-tubeGeometry"></a>

### `og.render.tubeGeometry`

Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh.

```ts
og.render.tubeGeometry(world: World, desc?: TubeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `TubeGeometryDesc` | yes | The points, radius and tessellation. |

**Returns** `Result<Geometry>`: Refused for fewer than two points or a point that is not finite (without a path), or a path that is not alive.

Binding symbol `og_render_tube_geometry` · publish builds: kept

```ogt
fn tubeGeometry {
  doc "Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc TubeGeometryDesc "The points, radius and tessellation."
  returns Result.Geometry "Refused for fewer than two points or a point that is not finite (without a path), or a path that is not alive."
}
```

<sub>`api.ogt` line 6910</sub>
