<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-sweepGeometry"></a>

### `og.render.sweepGeometry`

Make a section swept along a path for og.Mesh, tapered and twisted by keys.

```ts
og.render.sweepGeometry(world: World, desc?: SweepGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SweepGeometryDesc` | yes | The section, path, keys and caps. |

**Returns** `Result<Geometry>`: Refused for a path that is not alive, a section of fewer than three points or of no area, or a value that is not finite.

Binding symbol `og_render_sweep_geometry` · publish builds: kept

```ogt
fn sweepGeometry {
  doc "Make a section swept along a path for og.Mesh, tapered and twisted by keys."
  param world World "The world whose renderer owns the geometry."
  param desc SweepGeometryDesc "The section, path, keys and caps."
  returns Result.Geometry "Refused for a path that is not alive, a section of fewer than three points or of no area, or a value that is not finite."
}
```

<sub>Module `render` line 3734</sub>
