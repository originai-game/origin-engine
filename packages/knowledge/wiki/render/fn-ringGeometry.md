<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-ringGeometry"></a>

### `og.render.ringGeometry`

Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind).

```ts
og.render.ringGeometry(world: World, desc?: RingGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `RingGeometryDesc` | yes | Its radii and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_ring_geometry` · publish builds: kept

```ogt
fn ringGeometry {
  doc "Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind)."
  param world World "The world whose renderer owns the geometry."
  param desc RingGeometryDesc "Its radii and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 6922</sub>
