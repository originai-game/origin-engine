<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-overlapPoint"></a>

### `og.physics.overlapPoint`

Every collider containing a point, sorted by entity id; the same view of the colliders as castRay.

```ts
og.physics.overlapPoint(world: World, desc?: PointOverlapDesc): Result<readonly Entity[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `PointOverlapDesc` | yes | The point and the filter. |

**Returns** `Result<readonly Entity[]>`.

Binding symbol `og_physics_overlap_point` · publish builds: kept

```ogt
fn overlapPoint {
  doc "Every collider containing a point, sorted by entity id; the same view of the colliders as castRay."
  param world World "The world."
  param desc PointOverlapDesc "The point and the filter."
  returns Result.List.Entity
}
```

<sub>Module `physics` line 2408</sub>
