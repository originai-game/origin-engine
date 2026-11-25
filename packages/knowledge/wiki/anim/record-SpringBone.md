<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-SpringBone"></a>

### `og.SpringBone` (component)

A spring chain: joint and its parents swing towards animated tails under independent stiffness, drag, gravity and optional wind. tailOffset defines a leaf tail. Full Parent transforms, segment collision and JointLimit constrain the result. Applied after IK and look-at, top first. Tails and velocities are saved; a changed chain or detected teleport resets them.

See `animation §8`.

Token `og.SpringBone`, registered under the schema name `og.SpringBone`; its value type is `SpringBone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose joints this swings. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The chain's last joint (an index into the skin's joints): the tip of a tail, an ear. |
| `bones` | `number` | `1` | 1 … 16 | — | — | How many joints swing: `joint` and its parents (1 is the joint alone). |
| `stiffness` | `number` | `0` | ≥ 0 | — | tunable | Restoring acceleration per metre of displacement; zero disables restoring force. |
| `drag` | `number` | `0` | ≥ 0 | — | tunable | Linear velocity decay per second, independent of stiffness; zero disables drag. |
| `tailOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Explicit last joint tail in its local frame; zero uses its child. |
| `teleportDistance` | `number` | `0` | ≥ 0 | `m` | — | Root displacement that resets the chain; zero disables teleport detection. |
| `maxSpeed` | `number` | `0` | ≥ 0 | `mps` | tunable | Maximum tail velocity; zero disables the cap. |
| `wind` | `number` | `0` | ≥ 0 | — | tunable | Coupling to world wind velocity; zero disables wind. |
| `gravity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | — | An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop. |
| `space` | `SpringSpace` | `"world"` | — | — | — | World (including all Parent transforms, so the chain lags behind entity and parent motion) or model (only animation moves it). |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the swing is applied (0 = the animated pose, 1 = the spring's). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | Bone segment radius in joint-local metres, scaled with the joint and skeleton for spring and world collisions (0: a line). |
| `colliders` | `number` | `4294967295` | — | — | — | The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these. |
| `worldColliders` | `number` | `0` | — | — | — | World collider layers tested against each bone segment; zero disables world collision. |
| `lag` | `number` | `0` | — | `m` | read only | Written: the farthest a tail is from its animated place this tick. |

```ogt
component SpringBone {
  doc "A spring chain: joint and its parents swing towards animated tails under independent stiffness, drag, gravity and optional wind. tailOffset defines a leaf tail. Full Parent transforms, segment collision and JointLimit constrain the result. Applied after IK and look-at, top first. Tails and velocities are saved; a changed chain or detected teleport resets them."
  guide "animation §8"
  field skeleton Entity "The entity with the og.Skeleton whose joints this swings."
  field joint u32 "The chain's last joint (an index into the skin's joints): the tip of a tail, an ear." {
    min 0
  }
  field bones u32 "How many joints swing: `joint` and its parents (1 is the joint alone)." {
    default 1
    min 1
    max 16
  }
  field stiffness f32 "Restoring acceleration per metre of displacement; zero disables restoring force." {
    min 0
    tunable true
  }
  field drag f32 "Linear velocity decay per second, independent of stiffness; zero disables drag." {
    min 0
    tunable true
  }
  field tailOffset Vector3 "Explicit last joint tail in its local frame; zero uses its child." {
    unit m
  }
  field teleportDistance f32 "Root displacement that resets the chain; zero disables teleport detection." {
    min 0
    unit m
  }
  field maxSpeed f32 "Maximum tail velocity; zero disables the cap." {
    min 0
    unit mps
    tunable true
  }
  field wind f32 "Coupling to world wind velocity; zero disables wind." {
    min 0
    tunable true
  }
  field gravity Vector3 "An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop." {
    unit mps2
  }
  field space SpringSpace "World (including all Parent transforms, so the chain lags behind entity and parent motion) or model (only animation moves it)."
  field weight f32 "How much of the swing is applied (0 = the animated pose, 1 = the spring's)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field radius f32 "Bone segment radius in joint-local metres, scaled with the joint and skeleton for spring and world collisions (0: a line)." {
    min 0
    unit m
    tunable true
  }
  field colliders u32 "The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these." {
    default 4294967295
  }
  field worldColliders u32 "World collider layers tested against each bone segment; zero disables world collision."
  field lag f32 "Written: the farthest a tail is from its animated place this tick." {
    readonly true
    unit m
  }
}
```

<sub>Module `anim` line 536</sub>
