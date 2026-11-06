<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-FaceGoalDesc"></a>

### `FaceGoalDesc` (desc struct)

Turn to a heading where the character stands (og.motion.face).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `heading` | `number` | `0` | — | `rad` | — | The facing: 0 looks down +Z, π/2 down +X. |
| `duration` | `number` | `0.8` | ≥ 0 | `s` | — | How long the goal is current at least; a driven character's turn at turnSpeed may take longer. |

```ogt
struct FaceGoalDesc {
  doc "Turn to a heading where the character stands (og.motion.face)."
  desc true
  field heading f32 "The facing: 0 looks down +Z, π/2 down +X." {
    unit rad
  }
  field duration f32 "How long the goal is current at least; a driven character's turn at turnSpeed may take longer." {
    default 0.8
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 26839</sub>
