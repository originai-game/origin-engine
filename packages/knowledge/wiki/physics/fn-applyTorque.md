<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-applyTorque"></a>

### `og.physics.applyTorque`

Turn a dynamic body with a torque (N·m about each world axis; 2D uses z) over the next physics step, as applyForce: call it every tick for a steady twist.

```ts
og.physics.applyTorque(world: World, entity: Entity, torque: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity (an og.RigidBody). |
| `torque` | `Vector3` | no | The torque in newton-metres. |

**Returns** `Result<void>`: Refused for an entity without an og.RigidBody, or a torque that is not finite.

Binding symbol `og_physics_apply_torque` · publish builds: kept

```ogt
fn applyTorque {
  doc "Turn a dynamic body with a torque (N·m about each world axis; 2D uses z) over the next physics step, as applyForce: call it every tick for a steady twist."
  param world World "The world."
  param entity Entity "The body's entity (an og.RigidBody)."
  param torque Vector3 "The torque in newton-metres."
  returns Result.Void "Refused for an entity without an og.RigidBody, or a torque that is not finite."
}
```

<sub>Module `physics` line 1931</sub>
