<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-matrixWorld"></a>

### `og.transform.matrixWorld`

An entity's world matrix, as three.js Object3D.matrixWorld after updateMatrixWorld: its og.Transform composed with its parents' down the og.Parent chain (a bone attachment's posed bone included), from the values as they stand now, this tick's writes included. Declare read access to og.Transform.

Also known as: world position of a child, global transform, Unity Transform.position of a child, Godot global_transform, three.js getWorldPosition

```ts
og.transform.matrixWorld(world: World, entity: Entity): Result<Matrix4>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world the entity lives in. |
| `entity` | `Entity` | no | The entity, with og.Transform. |

**Returns** `Result<Matrix4>`: Refused for a dead entity, one without og.Transform, or a system that does not read og.Transform.

Binding symbol `og_transform_matrix_world` · publish builds: kept

```ogt
fn matrixWorld {
  doc "An entity's world matrix, as three.js Object3D.matrixWorld after updateMatrixWorld: its og.Transform composed with its parents' down the og.Parent chain (a bone attachment's posed bone included), from the values as they stand now, this tick's writes included. Declare read access to og.Transform."
  aliases "world position of a child, global transform, Unity Transform.position of a child, Godot global_transform, three.js getWorldPosition"
  param world World "The world the entity lives in."
  param entity Entity "The entity, with og.Transform."
  returns Result.Matrix4 "Refused for a dead entity, one without og.Transform, or a system that does not read og.Transform."
}
```

<sub>Module `transform` line 121</sub>
