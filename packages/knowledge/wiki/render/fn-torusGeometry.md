<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-torusGeometry"></a>

### `og.render.torusGeometry`

Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh.

```ts
og.render.torusGeometry(world: World, desc?: TorusGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `TorusGeometryDesc` | yes | Its radii and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_torus_geometry` · publish builds: kept

```ogt
fn torusGeometry {
  doc "Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc TorusGeometryDesc "Its radii and tessellation."
  returns Result.Geometry
}
```

<sub>Module `render` line 3412</sub>
