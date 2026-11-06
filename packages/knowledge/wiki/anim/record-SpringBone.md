<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-SpringBone"></a>

### `og.SpringBone` (component)

A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. Each joint's tail (the next joint of the chain; for the last joint its first child, or its parent bone continued at the same length) is a point pulled towards where the animated pose puts it by a damped spring (`frequency`, `damping`), under `gravity`, held at the bone's length, and the joint turns to point at it. Applied after og.IkChain and og.LookAt, top joint first; put it on its own entity (a tail, an ear, a strand of hair, a single jiggling node), one per chain. Deterministic under the fixed tick; the tails' positions and velocities are world state (the `og.anim` state block), so a restore continues a swing exactly. A changed chain (another skeleton, length or space) restarts at the animated pose.

Token `og.SpringBone`, registered under the schema name `og.SpringBone`; its value type is `SpringBone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose joints this swings. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The chain's last joint (an index into the skin's joints): the tip of a tail, an ear. |
| `bones` | `number` | `1` | 1 … 16 | — | — | How many joints swing: `joint` and its parents (1 is the joint alone). |
| `frequency` | `number` | `3` | 0 … 30 | `hz` | tunable | How fast a tail springs back to its animated place: the spring's natural frequency (0: no spring, it only hangs and swings). |
| `damping` | `number` | `0.3` | 0 … 2 | — | tunable | The damping ratio: 0 swings on, 1 settles without overshooting (critical damping). |
| `tipFrequency` | `number` | `1` | 0 … 4 | — | tunable | The last joint's frequency as a multiple of `frequency`, the joints between in proportion along the chain (1: every joint alike; 0.25: the tip springs back four times slower than the top, a soft strand on a stiff root). |
| `tipDamping` | `number` | `1` | 0 … 4 | — | tunable | The last joint's damping ratio as a multiple of `damping`, the joints between in proportion along the chain (1: every joint alike). |
| `gravity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | — | An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop. |
| `space` | `SpringSpace` | `"world"` | — | — | — | Where the tails move: world (the frame the skeleton entity's og.Transform is in, so the chain lags behind the entity's motion) or model (only the animation moves it). |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the swing is applied (0 = the animated pose, 1 = the spring's). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | The tails' radius against og.SpringCollider (three-vrm's hitRadius): a tail stays this far outside every collider it meets (0: a point). |
| `colliders` | `number` | `4294967295` | — | — | — | The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these. |
| `lag` | `number` | `0` | — | `m` | read only | Written: the farthest a tail is from its animated place this tick. |

```ogt
component SpringBone {
  doc "A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. Each joint's tail (the next joint of the chain; for the last joint its first child, or its parent bone continued at the same length) is a point pulled towards where the animated pose puts it by a damped spring (`frequency`, `damping`), under `gravity`, held at the bone's length, and the joint turns to point at it. Applied after og.IkChain and og.LookAt, top joint first; put it on its own entity (a tail, an ear, a strand of hair, a single jiggling node), one per chain. Deterministic under the fixed tick; the tails' positions and velocities are world state (the `og.anim` state block), so a restore continues a swing exactly. A changed chain (another skeleton, length or space) restarts at the animated pose."
  field skeleton Entity "The entity with the og.Skeleton whose joints this swings."
  field joint u32 "The chain's last joint (an index into the skin's joints): the tip of a tail, an ear." {
    min 0
  }
  field bones u32 "How many joints swing: `joint` and its parents (1 is the joint alone)." {
    default 1
    min 1
    max 16
  }
  field frequency f32 "How fast a tail springs back to its animated place: the spring's natural frequency (0: no spring, it only hangs and swings)." {
    default 3
    min 0
    max 30
    unit hz
    tunable true
  }
  field damping f32 "The damping ratio: 0 swings on, 1 settles without overshooting (critical damping)." {
    default 0.3
    min 0
    max 2
    tunable true
  }
  field tipFrequency f32 "The last joint's frequency as a multiple of `frequency`, the joints between in proportion along the chain (1: every joint alike; 0.25: the tip springs back four times slower than the top, a soft strand on a stiff root)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field tipDamping f32 "The last joint's damping ratio as a multiple of `damping`, the joints between in proportion along the chain (1: every joint alike)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field gravity Vector3 "An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop." {
    unit mps2
  }
  field space SpringSpace "Where the tails move: world (the frame the skeleton entity's og.Transform is in, so the chain lags behind the entity's motion) or model (only the animation moves it)."
  field weight f32 "How much of the swing is applied (0 = the animated pose, 1 = the spring's)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field radius f32 "The tails' radius against og.SpringCollider (three-vrm's hitRadius): a tail stays this far outside every collider it meets (0: a point)." {
    min 0
    unit m
    tunable true
  }
  field colliders u32 "The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these." {
    default 4294967295
  }
  field lag f32 "Written: the farthest a tail is from its animated place this tick." {
    readonly true
    unit m
  }
}
```

<sub>`api.ogt` line 11908</sub>
