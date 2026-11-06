<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-icosahedronGeometry"></a>

### `og.render.icosahedronGeometry`

Make an icosahedron (twenty faces) for og.Mesh; with detail, a geodesic sphere.

```ts
og.render.icosahedronGeometry(world: World, desc?: SolidGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SolidGeometryDesc` | yes | Its radius and detail. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_icosahedron_geometry` · publish builds: kept

```ogt
fn icosahedronGeometry {
  doc "Make an icosahedron (twenty faces) for og.Mesh; with detail, a geodesic sphere."
  param world World "The world whose renderer owns the geometry."
  param desc SolidGeometryDesc "Its radius and detail."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 7201</sub>
