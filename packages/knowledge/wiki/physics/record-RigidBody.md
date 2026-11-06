<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-RigidBody"></a>

### `og.RigidBody` (component)

A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.

Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).

Also known as: physics body, Rigidbody, RigidBody3D, dynamic body, kinematic body

Token `og.RigidBody`, registered under the schema name `og.RigidBody`; its value type is `RigidBody`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 2 | — | label “Kind” | BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by its Velocity each tick, or by game code through its Transform). |
| `mass` | `number` | `0` | ≥ 0 | `kg` | tunable | Mass; 0 derives it from the collider's density and volume. |
| `linearDamping` | `number` | `0` | 0 … 100 | `perSecond` | tunable | How fast linear velocity decays (0: none). |
| `angularDamping` | `number` | `0` | 0 … 100 | `perSecond` | tunable | How fast angular velocity decays (0: none). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies Physics.gravity for this body: 0 floats, 1 is normal. |
| `lock` | `number` | `0` | 0 … 63 | — | — | Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32). |
| `ccd` | `number` | `0` | 0 … 1 | — | — | 1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off. |
| `centerOfMass` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | tunable | Moves the centre of mass from where the collider puts it (its shape's centre), in the entity's own frame and scaled by og.Transform.scale like Collider.offset; the inertia stays the shape's. A car's lowered below its wheel mounts keeps it on its wheels at full lock; a weight at a toy's base rights it. Needs an og.Collider. |
| `canSleep` | `boolean` | `true` | — | — | — | Whether the body may sleep when Physics.sleepTicks is set. A body sleeps only with every dynamic body it touches, so false keeps it and what touches it awake (a player's ball, a body game code moves every tick). |
| `sleeping` | `boolean` | `false` | — | — | — | Written: true while the body sleeps (Physics.sleepTicks). Write false to wake it; physics then writes it back as the body is. |

```ogt
component RigidBody {
  doc "A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one."
  doc "Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport)."
  aliases "physics body, Rigidbody, RigidBody3D, dynamic body, kinematic body"
  field kind u32 "BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by its Velocity each tick, or by game code through its Transform)." {
    min 0
    max 2
    label "Kind"
  }
  field mass f32 "Mass; 0 derives it from the collider's density and volume." {
    min 0
    unit kg
    tunable true
  }
  field linearDamping f32 "How fast linear velocity decays (0: none)." {
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field angularDamping f32 "How fast angular velocity decays (0: none)." {
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field gravityScale f32 "Multiplies Physics.gravity for this body: 0 floats, 1 is normal." {
    default 1
    min -10
    max 10
    tunable true
  }
  field lock u32 "Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32)." {
    min 0
    max 63
  }
  field ccd u32 "1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off." {
    min 0
    max 1
  }
  field centerOfMass Vector3 "Moves the centre of mass from where the collider puts it (its shape's centre), in the entity's own frame and scaled by og.Transform.scale like Collider.offset; the inertia stays the shape's. A car's lowered below its wheel mounts keeps it on its wheels at full lock; a weight at a toy's base rights it. Needs an og.Collider." {
    unit m
    tunable true
  }
  field canSleep bool "Whether the body may sleep when Physics.sleepTicks is set. A body sleeps only with every dynamic body it touches, so false keeps it and what touches it awake (a player's ball, a body game code moves every tick)." {
    default true
  }
  field sleeping bool "Written: true while the body sleeps (Physics.sleepTicks). Write false to wake it; physics then writes it back as the body is."
}
```

<sub>`api.ogt` line 8541</sub>
