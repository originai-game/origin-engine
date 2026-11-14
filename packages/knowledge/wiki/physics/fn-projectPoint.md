<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-projectPoint"></a>

### `og.physics.projectPoint`

The nearest point on any collider to a point, within maxDistance: snapping to a surface, the closest cover, whether a point is inside a solid. The same view of the colliders as og.physics.castRay.

```ts
og.physics.projectPoint(world: World, desc?: ProjectPointDesc): Result<PointProjection | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ProjectPointDesc` | yes | The point, its reach, solid or surface, and the filter. |

**Returns** `Result<PointProjection | null>`: The projection, or null when no collider is within reach; refused for a point that is not finite.

Binding symbol `og_physics_project_point` · publish builds: kept

```ogt
fn projectPoint {
  doc "The nearest point on any collider to a point, within maxDistance: snapping to a surface, the closest cover, whether a point is inside a solid. The same view of the colliders as og.physics.castRay."
  param world World "The world."
  param desc ProjectPointDesc "The point, its reach, solid or surface, and the filter."
  returns Result.Option.PointProjection "The projection, or null when no collider is within reach; refused for a point that is not finite."
}
```

<sub>Module `physics` line 2020</sub>
