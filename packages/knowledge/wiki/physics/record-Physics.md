<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Physics"></a>

### `og.Physics` (resource)

The physics settings of the world. The profile is the project's; the same components serve both profiles.

Token `og.Physics`, registered under the schema name `og.Physics`; its value type is `Physics`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `profile` | `number` | `3` | 2 … 3 | — | label “Profile” | PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. Set the other one at run time to change profile: the next step builds that profile's simulation from the components (contacts, joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again), so a 3D game can play a 2D scene and back. |
| `gravity` | `Vector3` | `{ x: 0, y: -9.81, z: 0 }` | — | `mps2` | tunable | The acceleration of every dynamic body, times its gravityScale; 2D uses x and y. |
| `substeps` | `number` | `1` | 1 … 16 | — | tunable | Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost. |
| `solverIterations` | `number` | `4` | 1 … 32 | — | tunable | Solver iterations per step. |
| `restSpeed` | `number` | `0.02` | 0 … 1 | `mps` | tunable | A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it. |
| `sleepTicks` | `number` | `0` | 0 … 3600 | — | tunable | Ticks in a row a dynamic body must stay slower than sleepSpeed before it sleeps, together with every dynamic body it touches (contacts and joints, transitively): a settled pile sleeps whole and costs nothing until something wakes it (a body landing on it, a push, a joint, a moving kinematic body, a write to its Transform, Velocity, RigidBody or Collider, its support going away). BodySlept and BodyWoke report it. Deterministic: counted in ticks, saved with the world. 0, the default, never sleeps; new projects use about 30. |
| `sleepSpeed` | `number` | `0.1` | 0 … 10 | `mps` | tunable | The speed under which a body counts towards sleepTicks, before and after each step (the same number in m/s and rad/s). |

```ogt
resource Physics {
  doc "The physics settings of the world. The profile is the project's; the same components serve both profiles."
  field profile u32 "PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. Set the other one at run time to change profile: the next step builds that profile's simulation from the components (contacts, joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again), so a 3D game can play a 2D scene and back." {
    default 3
    min 2
    max 3
    label "Profile"
  }
  field gravity Vector3 "The acceleration of every dynamic body, times its gravityScale; 2D uses x and y." {
    default 0 -9.81 0
    unit mps2
    tunable true
  }
  field substeps u32 "Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost." {
    default 1
    min 1
    max 16
    tunable true
  }
  field solverIterations u32 "Solver iterations per step." {
    default 4
    min 1
    max 32
    tunable true
  }
  field restSpeed f32 "A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it." {
    default 0.02
    min 0
    max 1
    unit mps
    tunable true
  }
  field sleepTicks u32 "Ticks in a row a dynamic body must stay slower than sleepSpeed before it sleeps, together with every dynamic body it touches (contacts and joints, transitively): a settled pile sleeps whole and costs nothing until something wakes it (a body landing on it, a push, a joint, a moving kinematic body, a write to its Transform, Velocity, RigidBody or Collider, its support going away). BodySlept and BodyWoke report it. Deterministic: counted in ticks, saved with the world. 0, the default, never sleeps; new projects use about 30." {
    min 0
    max 3600
    tunable true
  }
  field sleepSpeed f32 "The speed under which a body counts towards sleepTicks, before and after each step (the same number in m/s and rad/s)." {
    default 0.1
    min 0
    max 10
    unit mps
    tunable true
  }
}
```

<sub>`api.ogt` line 8656</sub>
