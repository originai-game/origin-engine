<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionStatus"></a>

### `MotionStatus` (struct)

An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `goal` | `MotionGoalKind` | `"none"` | — | — | — | The current goal (none: the queue is empty and the character holds). |
| `pose` | `string` | `""` | — | — | — | The current goal's pose (walkTo's arrival pose), empty for none and face. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the current goal goes (pose and face: where the goal started; jump: its start plus its distance along its heading, on flat ground). |
| `heading` | `number` | `0` | — | `rad` | — | The heading the current goal faces (walkTo: towards its target). |
| `elapsed` | `number` | `0` | — | `s` | — | How long the current goal has been current. |
| `airborne` | `boolean` | `false` | — | — | — | The current goal is a jump between its takeoff and its landing tick. |
| `queued` | `number` | `0` | — | — | — | Goals waiting after the current one. |
| `completed` | `number` | `0` | — | — | — | Goals finished since the agent's first. |
| `looking` | `boolean` | `false` | — | — | — | A look is active, including its entry or release. |
| `reacting` | `boolean` | `false` | — | — | — | A reaction is active, including its entry or release. |

```ogt
struct MotionStatus {
  doc "An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here."
  field goal MotionGoalKind "The current goal (none: the queue is empty and the character holds)."
  field pose String "The current goal's pose (walkTo's arrival pose), empty for none and face."
  field target Vector3 "Where the current goal goes (pose and face: where the goal started; jump: its start plus its distance along its heading, on flat ground)." {
    unit m
  }
  field heading f32 "The heading the current goal faces (walkTo: towards its target)." {
    unit rad
  }
  field elapsed f32 "How long the current goal has been current." {
    unit s
  }
  field airborne bool "The current goal is a jump between its takeoff and its landing tick."
  field queued u32 "Goals waiting after the current one."
  field completed u32 "Goals finished since the agent's first."
  field looking bool "A look is active, including its entry or release."
  field reacting bool "A reaction is active, including its entry or release."
}
```

<sub>Module `motion` line 203</sub>
