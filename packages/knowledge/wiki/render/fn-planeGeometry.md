<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-planeGeometry"></a>

### `og.render.planeGeometry`

Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too.

```ts
og.render.planeGeometry(world: World, desc?: PlaneGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `PlaneGeometryDesc` | yes | Its extents and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_plane_geometry` · publish builds: kept

```ogt
fn planeGeometry {
  doc "Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too."
  param world World "The world whose renderer owns the geometry."
  param desc PlaneGeometryDesc "Its extents and texture repeat."
  returns Result.Geometry
}
```

<sub>Module `render` line 2763</sub>
