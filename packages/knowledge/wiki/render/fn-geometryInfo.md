<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-geometryInfo"></a>

### `og.render.geometryInfo`

A geometry's counts and bounds.

```ts
og.render.geometryInfo(world: World, geometry: Geometry): Result<GeometryInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `geometry` | `Geometry` | no | Any geometry: made, a primitive or a loaded glTF's. |

**Returns** `Result<GeometryInfo>`: Refused for a dead geometry.

Binding symbol `og_render_geometry_info` · publish builds: kept

```ogt
fn geometryInfo {
  doc "A geometry's counts and bounds."
  param world World "The world whose renderer owns the geometry."
  param geometry Geometry "Any geometry: made, a primitive or a loaded glTF's."
  returns Result.GeometryInfo "Refused for a dead geometry."
}
```

<sub>Module `render` line 3123</sub>
