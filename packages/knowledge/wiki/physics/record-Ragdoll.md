<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Ragdoll"></a>

### `og.Ragdoll` (component)

An explicit rest-pose rig of shaped bones and limited joints. Selected boneMask indices become dynamic with weight, inheriting animation COM velocities; other bones follow animation. Local rotations blend with FK over blendTime. Optional PD motors track animation and driveTransform follows the root. Owner and adjacent contacts are excluded pairwise; other ragdolls can collide. Bodies and motors are saved state, not entities.

Also known as: physical animation, death fall, limp body, Unreal physics asset, Unity ragdoll wizard

Token `og.Ragdoll`, registered under the schema name `og.Ragdoll`; its value type is `Ragdoll`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `boneMask` | `readonly number[]` | `[]` | — | — | a list of at most 64 `u32` (`List.u32.64`): reads a frozen copy, writes a whole array | Rig indices of dynamic bones; an empty list enables none. |
| `motorStiffness` | `number` | `0` | ≥ 0 | — | tunable | PD joint restoring stiffness; zero disables position drive. |
| `motorDamping` | `number` | `0` | ≥ 0 | — | tunable | PD joint velocity damping; zero disables it. |
| `driveTransform` | `boolean` | `false` | — | — | — | Move the entity with the simulated root bone. |
| `layers` | `number` | `0` | — | — | — | Collision membership bits; zero disables contacts. |
| `mask` | `number` | `0` | — | — | — | Collision filter bits; zero disables contacts. |
| `weight` | `number` | `0` | 0 … 1 | — | tunable | 0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover. |
| `blendTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | How long the blend between animation and physics takes, both ways. |
| `mass` | `number` | `60` | ≥ 0.1 | `kg` | tunable | Mass shared by length among bones without a per-bone mass override. |
| `thickness` | `number` | `0.22` | 0.01 … 1 | — | tunable | A bone capsule's radius as a fraction of its length. |
| `jointLimit` | `number` | `0.9` | 0 … 3.1 | `rad` | tunable | How far each joint bends from the rest pose. |
| `damping` | `number` | `0.6` | 0 … 20 | `perSecond` | tunable | How much of their motion the bones lose per second. |
| `impulse` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Total activation impulse distributed among newly dynamic bones in proportion to mass. |
| `blend` | `number` | `0` | — | — | read only | Written: the blend in use (0 animation .. 1 physics). |

```ogt
component Ragdoll {
  doc "An explicit rest-pose rig of shaped bones and limited joints. Selected boneMask indices become dynamic with weight, inheriting animation COM velocities; other bones follow animation. Local rotations blend with FK over blendTime. Optional PD motors track animation and driveTransform follows the root. Owner and adjacent contacts are excluded pairwise; other ragdolls can collide. Bodies and motors are saved state, not entities."
  aliases "physical animation, death fall, limp body, Unreal physics asset, Unity ragdoll wizard"
  field boneMask List.u32.64 "Rig indices of dynamic bones; an empty list enables none."
  field motorStiffness f32 "PD joint restoring stiffness; zero disables position drive." {
    min 0
    tunable true
  }
  field motorDamping f32 "PD joint velocity damping; zero disables it." {
    min 0
    tunable true
  }
  field driveTransform bool "Move the entity with the simulated root bone."
  field layers u32 "Collision membership bits; zero disables contacts."
  field mask u32 "Collision filter bits; zero disables contacts."
  field weight f32 "0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover." {
    min 0
    max 1
    tunable true
  }
  field blendTime f32 "How long the blend between animation and physics takes, both ways." {
    default 0.25
    min 0
    unit s
    tunable true
  }
  field mass f32 "Mass shared by length among bones without a per-bone mass override." {
    default 60
    min 0.1
    unit kg
    tunable true
  }
  field thickness f32 "A bone capsule's radius as a fraction of its length." {
    default 0.22
    min 0.01
    max 1
    tunable true
  }
  field jointLimit f32 "How far each joint bends from the rest pose." {
    default 0.9
    min 0
    max 3.1
    unit rad
    tunable true
  }
  field damping f32 "How much of their motion the bones lose per second." {
    default 0.6
    min 0
    max 20
    unit perSecond
    tunable true
  }
  field impulse Vector3 "Total activation impulse distributed among newly dynamic bones in proportion to mass."
  field blend f32 "Written: the blend in use (0 animation .. 1 physics)." {
    readonly true
  }
}
```

<sub>Module `physics` line 1958</sub>
