<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-LookAtDesc"></a>

### `LookAtDesc` (desc struct)

A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. An omitted target releases the look over blend seconds.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | World point to look at; omitted releases the current look. |
| `duration` | `number` | `2` | ≥ 0 | `s` | — | Seconds held after blending in, before blending out. |
| `blend` | `number` | `0.2` | ≥ 0 | `s` | — | Seconds to enter and leave; 0 is immediate. |
| `maxYaw` | `number` | `1.92` | 0 … 3.141593 | `rad` | — | Maximum yaw from the animated forward. |
| `maxPitch` | `number` | `0.6` | 0 … 1.570796 | `rad` | — | Maximum pitch from the animated forward. |
| `weight` | `number` | `1` | 0 … 1 | — | — | Contribution of the look. |

```ogt
struct LookAtDesc {
  doc "A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. An omitted target releases the look over blend seconds."
  desc true
  field target Vector3 "World point to look at; omitted releases the current look." {
    unit m
  }
  field duration f32 "Seconds held after blending in, before blending out." {
    default 2
    min 0
    unit s
  }
  field blend f32 "Seconds to enter and leave; 0 is immediate." {
    default 0.2
    min 0
    unit s
  }
  field maxYaw f32 "Maximum yaw from the animated forward." {
    default 1.92
    min 0
    max 3.141593
    unit rad
  }
  field maxPitch f32 "Maximum pitch from the animated forward." {
    default 0.6
    min 0
    max 1.570796
    unit rad
  }
  field weight f32 "Contribution of the look." {
    default 1
    min 0
    max 1
  }
}
```

<sub>Module `motion` line 144</sub>
