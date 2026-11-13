<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-applyTorqueImpulse"></a>

### `og.physics.applyTorqueImpulse`

Give a dynamic body an angular impulse (N·m·s about each world axis; 2D uses z): its spin changes at once, as applyImpulse.

```ts
og.physics.applyTorqueImpulse(world: World, entity: Entity, torque: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity (an og.RigidBody). |
| `torque` | `Vector3` | no | The angular impulse in newton-metre-seconds. |

**Returns** `Result<void>`: Refused for an entity without an og.RigidBody, or a value that is not finite.

Binding symbol `og_physics_apply_torque_impulse` · publish builds: kept

```ogt
fn applyTorqueImpulse {
  doc "Give a dynamic body an angular impulse (N·m·s about each world axis; 2D uses z): its spin changes at once, as applyImpulse."
  param world World "The world."
  param entity Entity "The body's entity (an og.RigidBody)."
  param torque Vector3 "The angular impulse in newton-metre-seconds."
  returns Result.Void "Refused for an entity without an og.RigidBody, or a value that is not finite."
}
```

<sub>Module `physics` line 1973</sub>
