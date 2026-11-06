<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-updateGeometry"></a>

### `og.render.updateGeometry`

Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes.

```ts
og.render.updateGeometry(world: World, geometry: Geometry, desc?: BufferGeometryDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `geometry` | `Geometry` | no | A geometry og.render.bufferGeometry made with dynamic true. |
| `desc` | `BufferGeometryDesc` | yes | The new attributes. |

**Returns** `Result<void>`: Refused for a dead or static geometry, or attributes og.render.bufferGeometry refuses.

Binding symbol `og_render_update_geometry` · publish builds: kept

```ogt
fn updateGeometry {
  doc "Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes."
  param world World "The world whose renderer owns the geometry."
  param geometry Geometry "A geometry og.render.bufferGeometry made with dynamic true."
  param desc BufferGeometryDesc "The new attributes."
  returns Result.Void "Refused for a dead or static geometry, or attributes og.render.bufferGeometry refuses."
}
```

<sub>Module `render` line 2898</sub>
