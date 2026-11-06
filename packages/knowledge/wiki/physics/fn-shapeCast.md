<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-shapeCast"></a>

### `og.physics.shapeCast`

The first collider a shape meets when swept along a direction; the same view of the colliders as raycast.

```ts
og.physics.shapeCast(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ShapeCastDesc` | yes | The shape, its sweep and the filter. |

**Returns** `Result<ShapeCastHit | null>`: The hit, or null; refused for a zero direction or an unknown mesh.

Binding symbol `og_physics_shape_cast` · publish builds: kept

```ogt
fn shapeCast {
  doc "The first collider a shape meets when swept along a direction; the same view of the colliders as raycast."
  param world World "The world."
  param desc ShapeCastDesc "The shape, its sweep and the filter."
  returns Result.Option.ShapeCastHit "The hit, or null; refused for a zero direction or an unknown mesh."
}
```

<sub>`api.ogt` line 10357</sub>
