<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-tetrahedronGeometry"></a>

### `og.render.tetrahedronGeometry`

Make a tetrahedron (four faces) for og.Mesh.

```ts
og.render.tetrahedronGeometry(world: World, desc?: SolidGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SolidGeometryDesc` | yes | Its radius and detail. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_tetrahedron_geometry` · publish builds: kept

```ogt
fn tetrahedronGeometry {
  doc "Make a tetrahedron (four faces) for og.Mesh."
  param world World "The world whose renderer owns the geometry."
  param desc SolidGeometryDesc "Its radius and detail."
  returns Result.Geometry
}
```

<sub>Module `render` line 3444</sub>
