<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-applyImpulse"></a>

### `og.physics.applyImpulse`

Give a dynamic body an impulse (N·s, world axes) at a world point or its centre of mass: a sudden change of momentum (a kick, a hit, a jump: its velocity changes by impulse / mass), applied by the next physics step (after a hit stop ends). As applyForce otherwise.

Also known as: knockback, push, hit force, Unity AddForce ForceMode.Impulse, Godot apply_impulse, Unreal AddImpulse

```ts
og.physics.applyImpulse(world: World, entity: Entity, impulse: Vector3, point?: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity (an og.RigidBody). |
| `impulse` | `Vector3` | no | The impulse in newton-seconds (2D uses x and y). |
| `point` | `Vector3` | yes | Where it acts, in world space; omitted is the centre of mass. |

**Returns** `Result<void>`: Refused for an entity without an og.RigidBody, or an impulse or point that is not finite.

Binding symbol `og_physics_apply_impulse` · publish builds: kept

```ogt
fn applyImpulse {
  doc "Give a dynamic body an impulse (N·s, world axes) at a world point or its centre of mass: a sudden change of momentum (a kick, a hit, a jump: its velocity changes by impulse / mass), applied by the next physics step (after a hit stop ends). As applyForce otherwise."
  aliases "knockback, push, hit force, Unity AddForce ForceMode.Impulse, Godot apply_impulse, Unreal AddImpulse"
  param world World "The world."
  param entity Entity "The body's entity (an og.RigidBody)."
  param impulse Vector3 "The impulse in newton-seconds (2D uses x and y)."
  param point Vector3 "Where it acts, in world space; omitted is the centre of mass." {
    optional true
  }
  returns Result.Void "Refused for an entity without an og.RigidBody, or an impulse or point that is not finite."
}
```

<sub>Module `physics` line 1946</sub>
