<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-WalkGoalDesc"></a>

### `WalkGoalDesc` (desc struct)

Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where to go (y is ignored). |
| `speed` | `number` | `1.4` | 0.1 … 10 | `mps` | — | How fast: the motion walks below 2.5 m/s and runs above. |
| `pose` | `string` | `""` | — | — | — | The pose taken on arrival; empty is `idle`. |
| `arrivalHeading` | `number` | `0` | — | `rad` | — | Facing after arrival; omitted keeps the travel heading. The goal ends only after this turn completes. |
| `deceleration` | `number` | `3` | ≥ 0 | — | — | Braking near the target; 0 uses 3 m/s². A CharacterController owns ground height and collision when present; otherwise motion keeps the Transform's height. |

```ogt
struct WalkGoalDesc {
  doc "Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive."
  desc true
  field position Vector3 "Where to go (y is ignored)." {
    unit m
  }
  field speed f32 "How fast: the motion walks below 2.5 m/s and runs above." {
    default 1.4
    min 0.1
    max 10
    unit mps
  }
  field pose String "The pose taken on arrival; empty is `idle`."
  field arrivalHeading f32 "Facing after arrival; omitted keeps the travel heading. The goal ends only after this turn completes." {
    unit rad
  }
  field deceleration f32 "Braking near the target; 0 uses 3 m/s². A CharacterController owns ground height and collision when present; otherwise motion keeps the Transform's height." {
    default 3
    min 0
  }
}
```

<sub>Module `motion` line 91</sub>
