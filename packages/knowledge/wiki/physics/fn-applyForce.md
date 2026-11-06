<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-applyForce"></a>

### `og.physics.applyForce`

Push a dynamic body with a force (N, world axes) over the next physics step, at a world point or its centre of mass: call it every tick for a steady force (thrust, wind, a car's downforce or an anti-roll assist). Applied by the og.physics step after the systems, in the order given, then gone; a hit stop drops it. Kinematic and static bodies ignore it. Pushes not yet applied are world state (in snapshots and the hash).

Also known as: force, thrust, push over time, Unity AddForce, Godot apply_force

```ts
og.physics.applyForce(world: World, entity: Entity, force: Vector3, point?: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity (an og.RigidBody). |
| `force` | `Vector3` | no | The force in newtons (2D uses x and y). |
| `point` | `Vector3` | yes | Where it acts, in world space; omitted is the centre of mass (no spin). |

**Returns** `Result<void>`: Refused for an entity without an og.RigidBody, or a force or point that is not finite.

Binding symbol `og_physics_apply_force` · publish builds: kept

```ogt
fn applyForce {
  doc "Push a dynamic body with a force (N, world axes) over the next physics step, at a world point or its centre of mass: call it every tick for a steady force (thrust, wind, a car's downforce or an anti-roll assist). Applied by the og.physics step after the systems, in the order given, then gone; a hit stop drops it. Kinematic and static bodies ignore it. Pushes not yet applied are world state (in snapshots and the hash)."
  aliases "force, thrust, push over time, Unity AddForce, Godot apply_force"
  param world World "The world."
  param entity Entity "The body's entity (an og.RigidBody)."
  param force Vector3 "The force in newtons (2D uses x and y)."
  param point Vector3 "Where it acts, in world space; omitted is the centre of mass (no spin)." {
    optional true
  }
  returns Result.Void "Refused for an entity without an og.RigidBody, or a force or point that is not finite."
}
```

<sub>`api.ogt` line 10364</sub>
