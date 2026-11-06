<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-bufferGeometry"></a>

### `og.render.bufferGeometry`

Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once.

```ts
og.render.bufferGeometry(world: World, desc?: BufferGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `BufferGeometryDesc` | yes | The attributes. |

**Returns** `Result<Geometry>`: Refused for an attribute whose length does not fit the vertex count, an index past the vertices, or a number that is not finite (the reason names the attribute).

Binding symbol `og_render_buffer_geometry` · publish builds: kept

```ogt
fn bufferGeometry {
  doc "Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once."
  param world World "The world whose renderer owns the geometry."
  param desc BufferGeometryDesc "The attributes."
  returns Result.Geometry "Refused for an attribute whose length does not fit the vertex count, an index past the vertices, or a number that is not finite (the reason names the attribute)."
}
```

<sub>Module `render` line 2891</sub>
