<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-CollisionStarted"></a>

### `og.CollisionStarted` (event)

Two solid colliders started touching during the tick. Read in the tick after, when og.Velocity already holds the solver's answer (the bodies parting): how hard they met is `impulse` and `relativeVelocity`, taken in the step itself.

Token `og.CollisionStarted`, registered under the schema name `og.CollisionStarted`; its value type is `CollisionStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | The deepest contact point in world space (z is 0 in 2D). |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The contact normal, from a towards b. |
| `impulse` | `number` | `0` | ≥ 0 | `newtonSecond` | — | The normal impulse the solver gave the pair in the step that made the contact (every substep, every contact point), in newton-seconds: b took `impulse` along `normal`, a the same against it. Its momentum answer to the hit, at least 0; heavier and faster hits give more. |
| `relativeVelocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | a's velocity relative to b at `point` as that step began, before the solver answered: each body's linear velocity plus its angular velocity crossed with the point's offset from its centre of mass (a static collider is still; z is 0 in 2D). Along `normal` it is the closing speed, positive while they approach: max(0, relativeVelocity · normal) is how hard they hit, in m/s. |

```ogt
event CollisionStarted {
  doc "Two solid colliders started touching during the tick. Read in the tick after, when og.Velocity already holds the solver's answer (the bodies parting): how hard they met is `impulse` and `relativeVelocity`, taken in the step itself."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
  field point Vector3 "The deepest contact point in world space (z is 0 in 2D)." {
    worldspace world
    unit m
  }
  field normal Vector3 "The contact normal, from a towards b."
  field impulse f32 "The normal impulse the solver gave the pair in the step that made the contact (every substep, every contact point), in newton-seconds: b took `impulse` along `normal`, a the same against it. Its momentum answer to the hit, at least 0; heavier and faster hits give more." {
    min 0
    unit newtonSecond
  }
  field relativeVelocity Vector3 "a's velocity relative to b at `point` as that step began, before the solver answered: each body's linear velocity plus its angular velocity crossed with the point's offset from its centre of mass (a static collider is still; z is 0 in 2D). Along `normal` it is the closing speed, positive while they approach: max(0, relativeVelocity · normal) is how hard they hit, in m/s." {
    unit mps
  }
}
```

<sub>Module `physics` line 197</sub>
