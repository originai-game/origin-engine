<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-PoseGoalDesc"></a>

### `PoseGoalDesc` (desc struct)

A pose of the model's library where the character stands (og.motion.pose).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pose` | `string` | `""` | — | — | — | The library pose (og.motion.poses); empty is `idle`. |
| `duration` | `number` | `1.5` | ≥ 0 | `s` | — | How long the goal is current. |

```ogt
struct PoseGoalDesc {
  doc "A pose of the model's library where the character stands (og.motion.pose)."
  desc true
  field pose String "The library pose (og.motion.poses); empty is `idle`."
  field duration f32 "How long the goal is current." {
    default 1.5
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 26812</sub>
