<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-velocityAtPoint"></a>

### `og.physics.velocityAtPoint`

The live simulation's world-space linear velocity of a RigidBody entity at a world-space point (linear + angular × (point - com)), or null if not a body. Uses the same initialization and tick boundary as castRay.

```ts
og.physics.velocityAtPoint(world: World, entity: Entity, point: Vector3): Result<Vector3 | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity. |
| `point` | `Vector3` | no | The world-space point. |

**Returns** `Result<Vector3 | null>`.

Binding symbol `og_physics_velocity_at_point` · publish builds: kept

```ogt
fn velocityAtPoint {
  doc "The live simulation's world-space linear velocity of a RigidBody entity at a world-space point (linear + angular × (point - com)), or null if not a body. Uses the same initialization and tick boundary as castRay."
  param world World "The world."
  param entity Entity "The body's entity."
  param point Vector3 "The world-space point."
  returns Result.Option.Vector3
}
```

<sub>Module `physics` line 2464</sub>
