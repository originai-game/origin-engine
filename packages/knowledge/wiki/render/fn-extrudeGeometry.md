<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-extrudeGeometry"></a>

### `og.render.extrudeGeometry`

Make an extruded solid for og.Mesh: a three.js extrusion is a paste (`og.render.extrudeGeometry(world, { shapes: shape, depth: 0.2, bevelEnabled: true })`). Gears, signs, keys, logos, cookie cutters; along a path, a rail or a moulding.

```ts
og.render.extrudeGeometry(world: World, desc?: ExtrudeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `ExtrudeGeometryDesc` | yes | The shapes, depth, bevel and path. |

**Returns** `Result<Geometry>`: Refused for a shape whose points are not finite or that triangulates to nothing, or an extrudePath that is not alive.

Binding symbol `og_render_extrude_geometry` · publish builds: kept

```ogt
fn extrudeGeometry {
  doc "Make an extruded solid for og.Mesh: a three.js extrusion is a paste (`og.render.extrudeGeometry(world, { shapes: shape, depth: 0.2, bevelEnabled: true })`). Gears, signs, keys, logos, cookie cutters; along a path, a rail or a moulding."
  param world World "The world whose renderer owns the geometry."
  param desc ExtrudeGeometryDesc "The shapes, depth, bevel and path."
  returns Result.Geometry "Refused for a shape whose points are not finite or that triangulates to nothing, or an extrudePath that is not alive."
}
```

<sub>Module `render` line 3402</sub>
