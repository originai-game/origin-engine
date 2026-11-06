<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-PlaceGoalDesc"></a>

### `PlaceGoalDesc` (desc struct)

A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pose` | `string` | `""` | — | — | — | The library pose; empty is `idle`. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the character stands (y is ignored: the ground is the Transform's). |
| `heading` | `number` | `0` | — | `rad` | — | Its facing: 0 looks down +Z, π/2 down +X. |
| `duration` | `number` | `1.5` | ≥ 0 | `s` | — | How long the goal is current; a driven character moves there in a straight line over it. |

```ogt
struct PlaceGoalDesc {
  doc "A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`."
  desc true
  field pose String "The library pose; empty is `idle`."
  field position Vector3 "Where the character stands (y is ignored: the ground is the Transform's)." {
    unit m
  }
  field heading f32 "Its facing: 0 looks down +Z, π/2 down +X." {
    unit rad
  }
  field duration f32 "How long the goal is current; a driven character moves there in a straight line over it." {
    default 1.5
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 26822</sub>
