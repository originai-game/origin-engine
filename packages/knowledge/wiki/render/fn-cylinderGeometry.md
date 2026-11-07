<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-cylinderGeometry"></a>

### `og.render.cylinderGeometry`

Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself).

```ts
og.render.cylinderGeometry(world: World, desc?: CylinderGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CylinderGeometryDesc` | yes | Its radii, height and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_cylinder_geometry` · publish builds: kept

```ogt
fn cylinderGeometry {
  doc "Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself)."
  param world World "The world whose renderer owns the geometry."
  param desc CylinderGeometryDesc "Its radii, height and tessellation."
  returns Result.Geometry
}
```

<sub>Module `render` line 3102</sub>
