<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-circleGeometry"></a>

### `og.render.circleGeometry`

Make a circle geometry for og.Mesh (one-sided, as ringGeometry).

```ts
og.render.circleGeometry(world: World, desc?: CircleGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CircleGeometryDesc` | yes | Its radius and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_circle_geometry` · publish builds: kept

```ogt
fn circleGeometry {
  doc "Make a circle geometry for og.Mesh (one-sided, as ringGeometry)."
  param world World "The world whose renderer owns the geometry."
  param desc CircleGeometryDesc "Its radius and tessellation."
  returns Result.Geometry
}
```

<sub>Module `render` line 3147</sub>
