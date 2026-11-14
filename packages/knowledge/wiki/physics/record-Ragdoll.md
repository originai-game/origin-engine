<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Ragdoll"></a>

### `og.Ragdoll` (component)

A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state, not entities.

Also known as: physical animation, death fall, limp body, Unreal physics asset, Unity ragdoll wizard

Token `og.Ragdoll`, registered under the schema name `og.Ragdoll`; its value type is `Ragdoll`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `weight` | `number` | `0` | 0 … 1 | — | tunable | 0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover. |
| `blendTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | How long the blend between animation and physics takes, both ways. |
| `mass` | `number` | `60` | ≥ 0.1 | `kg` | tunable | The whole body's mass, shared by the bones by length. |
| `thickness` | `number` | `0.22` | 0.01 … 1 | — | tunable | A bone capsule's radius as a fraction of its length. |
| `jointLimit` | `number` | `0.9` | 0 … 3.1 | `rad` | tunable | How far each joint bends from the rest pose. |
| `damping` | `number` | `0.6` | 0 … 20 | `perSecond` | tunable | How much of their motion the bones lose per second. |
| `impulse` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | An impulse given to every bone when weight first goes up (a hit). |
| `blend` | `number` | `0` | — | — | read only | Written: the blend in use (0 animation .. 1 physics). |

```ogt
component Ragdoll {
  doc "A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state, not entities."
  aliases "physical animation, death fall, limp body, Unreal physics asset, Unity ragdoll wizard"
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
  field mass f32 "The whole body's mass, shared by the bones by length." {
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
  field impulse Vector3 "An impulse given to every bone when weight first goes up (a hit)."
  field blend f32 "Written: the blend in use (0 animation .. 1 physics)." {
    readonly true
  }
}
```

<sub>Module `physics` line 1554</sub>
