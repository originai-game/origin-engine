<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-coneGeometry"></a>

### `og.render.coneGeometry`

Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh.

```ts
og.render.coneGeometry(world: World, desc?: ConeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `ConeGeometryDesc` | yes | Its radius, height and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_cone_geometry` · publish builds: kept

```ogt
fn coneGeometry {
  doc "Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc ConeGeometryDesc "Its radius, height and tessellation."
  returns Result.Geometry
}
```

<sub>Module `render` line 3122</sub>
