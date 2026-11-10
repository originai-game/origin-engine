<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-latheGeometry"></a>

### `og.render.latheGeometry`

Make a solid of revolution for og.Mesh: a profile turned round +Y.

```ts
og.render.latheGeometry(world: World, desc?: LatheGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `LatheGeometryDesc` | yes | The profile and the turn. |

**Returns** `Result<Geometry>`: Refused for fewer than two points or one that is not finite.

Binding symbol `og_render_lathe_geometry` · publish builds: kept

```ogt
fn latheGeometry {
  doc "Make a solid of revolution for og.Mesh: a profile turned round +Y."
  param world World "The world whose renderer owns the geometry."
  param desc LatheGeometryDesc "The profile and the turn."
  returns Result.Geometry "Refused for fewer than two points or one that is not finite."
}
```

<sub>Module `render` line 3405</sub>
