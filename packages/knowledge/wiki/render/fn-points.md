<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-points"></a>

### `og.render.points`

Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick.

```ts
og.render.points(world: World, desc?: PointsDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `PointsDesc` | yes | The positions and the look. |

**Returns** `Result<void>`: Refused for a colours list whose length is not the points'.

Binding symbol `og_render_points` · publish builds: kept

```ogt
fn points {
  doc "Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick."
  param world World "The world whose renderer draws it."
  param desc PointsDesc "The positions and the look."
  returns Result.Void "Refused for a colours list whose length is not the points'."
}
```

<sub>Module `render` line 3823</sub>
