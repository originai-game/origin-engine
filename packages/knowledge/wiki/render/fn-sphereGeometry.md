<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-sphereGeometry"></a>

### `og.render.sphereGeometry`

Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh).

```ts
og.render.sphereGeometry(world: World, desc?: SphereGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SphereGeometryDesc` | yes | Its radius, tessellation and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_sphere_geometry` · publish builds: kept

```ogt
fn sphereGeometry {
  doc "Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh)."
  param world World "The world whose renderer owns the geometry."
  param desc SphereGeometryDesc "Its radius, tessellation and texture repeat."
  returns Result.Geometry
}
```

<sub>Module `render` line 2561</sub>
