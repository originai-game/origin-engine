<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-clothPoints"></a>

### `og.physics.clothPoints`

An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge).

```ts
og.physics.clothPoints(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The cloth's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Cloth.

Binding symbol `og_physics_cloth_points` · publish builds: kept

```ogt
fn clothPoints {
  doc "An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge)."
  param world World "The world."
  param entity Entity "The cloth's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Cloth."
}
```

<sub>`api.ogt` line 10480</sub>
