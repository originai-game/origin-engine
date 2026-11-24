<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-castShapeAll"></a>

### `og.physics.castShapeAll`

Every collider a shape meets when swept along a direction, nearest first (ties in entity order), at most `maxHits`; the same view of the colliders as castRay.

```ts
og.physics.castShapeAll(world: World, desc?: ShapeCastAllDesc): Result<readonly ShapeCastHit[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ShapeCastAllDesc` | yes | The shape, its sweep, the filter and the cap. |

**Returns** `Result<readonly ShapeCastHit[]>`: The hits, nearest first; refused for a zero direction or an unknown mesh.

Binding symbol `og_physics_cast_shape_all` · publish builds: kept

```ogt
fn castShapeAll {
  doc "Every collider a shape meets when swept along a direction, nearest first (ties in entity order), at most `maxHits`; the same view of the colliders as castRay."
  param world World "The world."
  param desc ShapeCastAllDesc "The shape, its sweep, the filter and the cap."
  returns Result.List.ShapeCastHit "The hits, nearest first; refused for a zero direction or an unknown mesh."
}
```

<sub>Module `physics` line 2420</sub>
