<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-boxGeometry"></a>

### `og.render.boxGeometry`

Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too.

Also known as: cube mesh, box mesh, three.js BoxGeometry

```ts
og.render.boxGeometry(world: World, desc?: BoxGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `BoxGeometryDesc` | yes | Its extents and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_box_geometry` · publish builds: kept

```ogt
fn boxGeometry {
  doc "Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too."
  aliases "cube mesh, box mesh, three.js BoxGeometry"
  param world World "The world whose renderer owns the geometry."
  param desc BoxGeometryDesc "Its extents and texture repeat."
  returns Result.Geometry
}
```

<sub>Module `render` line 2555</sub>
