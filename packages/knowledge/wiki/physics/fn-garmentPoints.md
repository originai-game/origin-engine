<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-garmentPoints"></a>

### `og.physics.garmentPoints`

An og.Garment's points after the last step, in the world: one per position of the garment's mesh (vertices at the same position, split along a seam, share one), in the order the mesh first uses them.

```ts
og.physics.garmentPoints(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The garment's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Garment, or one not simulated yet (it needs og.Model, og.Skeleton and a loaded og.load.animation of the model).

Binding symbol `og_physics_garment_points` · publish builds: kept

```ogt
fn garmentPoints {
  doc "An og.Garment's points after the last step, in the world: one per position of the garment's mesh (vertices at the same position, split along a seam, share one), in the order the mesh first uses them."
  param world World "The world."
  param entity Entity "The garment's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Garment, or one not simulated yet (it needs og.Model, og.Skeleton and a loaded og.load.animation of the model)."
}
```

<sub>Module `physics` line 2116</sub>
