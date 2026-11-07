<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-csgGeometry"></a>

### `og.render.csgGeometry`

Make the union, difference or intersection of two placed geometries for og.Mesh. Chain calls for more operands (a cube minus six spheres is six calls); each result is a geometry like any other.

```ts
og.render.csgGeometry(world: World, desc?: CsgGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CsgGeometryDesc` | yes | The operation and its operands. |

**Returns** `Result<Geometry>`: Refused for a dead geometry, a placement with a zero scale, or a position that is not finite.

Binding symbol `og_render_csg_geometry` · publish builds: kept

```ogt
fn csgGeometry {
  doc "Make the union, difference or intersection of two placed geometries for og.Mesh. Chain calls for more operands (a cube minus six spheres is six calls); each result is a geometry like any other."
  param world World "The world whose renderer owns the geometry."
  param desc CsgGeometryDesc "The operation and its operands."
  returns Result.Geometry "Refused for a dead geometry, a placement with a zero scale, or a position that is not finite."
}
```

<sub>Module `render` line 3458</sub>
