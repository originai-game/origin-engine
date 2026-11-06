<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-mergeGeometries"></a>

### `og.render.mergeGeometries`

Make one geometry of several placed ones for og.Mesh: one draw instead of many, mirrored halves, arrays.

```ts
og.render.mergeGeometries(world: World, desc?: MergeGeometriesDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `MergeGeometriesDesc` | yes | The parts and their placements. |

**Returns** `Result<Geometry>`: Refused for no parts, a dead geometry, or a placement with a zero scale.

Binding symbol `og_render_merge_geometries` · publish builds: kept

```ogt
fn mergeGeometries {
  doc "Make one geometry of several placed ones for og.Mesh: one draw instead of many, mirrored halves, arrays."
  param world World "The world whose renderer owns the geometry."
  param desc MergeGeometriesDesc "The parts and their placements."
  returns Result.Geometry "Refused for no parts, a dead geometry, or a placement with a zero scale."
}
```

<sub>`api.ogt` line 7242</sub>
