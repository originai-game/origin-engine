<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-JumpGoalDesc"></a>

### `JumpGoalDesc` (desc struct)

A jump (og.motion.jump); needs og.MotionAgent.drive. The simulation owns its whole arc: after `prepare` the og.motion system takes off in the tick, with an og.CharacterController through the controller's jump (it sets the controller's jumpSpeed for `height` under the controller's gravity and holds `jump` on the takeoff tick, so a ceiling cuts the rise and the landing is wherever the controller reaches ground), without one by the fixed ballistic arc of `height` back to the takeoff height. The goal ends `recover` seconds after the landing tick. The generated jump is drawn in place on that flight, never adding the model's own vertical travel (docs/design.md §2.7).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0.5` | 0 … 5 | `m` | — | The apex over the takeoff height, with gravity og.Physics.gravity (times the controller's gravityScale). |
| `distance` | `number` | `0` | 0 … 20 | `m` | — | How far it travels across the ground while in the air, along `heading`; 0 jumps in place. |
| `heading` | `number` | `0` | — | `rad` | — | The way it faces and travels: 0 looks down +Z, π/2 down +X; omitted keeps its facing when the jump starts. It turns there at turnSpeed while preparing. |
| `prepare` | `number` | `0.25` | ≥ 0 | `s` | — | Seconds of preparation (the crouch) before takeoff. |
| `recover` | `number` | `0.35` | ≥ 0 | `s` | — | Seconds after the landing tick before the goal ends. |
| `pose` | `string` | `""` | — | — | — | The pose held after the jump; empty is `idle`. |

```ogt
struct JumpGoalDesc {
  doc "A jump (og.motion.jump); needs og.MotionAgent.drive. The simulation owns its whole arc: after `prepare` the og.motion system takes off in the tick, with an og.CharacterController through the controller's jump (it sets the controller's jumpSpeed for `height` under the controller's gravity and holds `jump` on the takeoff tick, so a ceiling cuts the rise and the landing is wherever the controller reaches ground), without one by the fixed ballistic arc of `height` back to the takeoff height. The goal ends `recover` seconds after the landing tick. The generated jump is drawn in place on that flight, never adding the model's own vertical travel (docs/design.md §2.7)."
  desc true
  field height f32 "The apex over the takeoff height, with gravity og.Physics.gravity (times the controller's gravityScale)." {
    default 0.5
    min 0
    max 5
    unit m
  }
  field distance f32 "How far it travels across the ground while in the air, along `heading`; 0 jumps in place." {
    default 0
    min 0
    max 20
    unit m
  }
  field heading f32 "The way it faces and travels: 0 looks down +Z, π/2 down +X; omitted keeps its facing when the jump starts. It turns there at turnSpeed while preparing." {
    unit rad
  }
  field prepare f32 "Seconds of preparation (the crouch) before takeoff." {
    default 0.25
    min 0
    unit s
  }
  field recover f32 "Seconds after the landing tick before the goal ends." {
    default 0.35
    min 0
    unit s
  }
  field pose String "The pose held after the jump; empty is `idle`."
}
```

<sub>`api.ogt` line 26741</sub>
