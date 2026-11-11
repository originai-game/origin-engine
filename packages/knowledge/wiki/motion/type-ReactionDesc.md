<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-ReactionDesc"></a>

### `ReactionDesc` (desc struct)

An authored humanoid reaction over locomotion: entry, a held interval and exit from a cooked `type motion-action` asset (source, clip, optional rig, holdStart and holdEnd, optional hand and handTarget). Timing and strength belong to the tick; the generated pose is presentation only.

See `animation §11`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The motion-action asset path, such as actions/brace.ogt. Empty releases the current reaction. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | World direction the character reacts toward (toward the wind source for a brace); zero keeps the character's facing. |
| `strength` | `number` | `1` | 0 … 1 | — | — | Contribution of the authored reaction; 0 leaves locomotion unchanged. |
| `duration` | `number` | `1` | ≥ 0 | `s` | — | Seconds to hold between the action's holdStart and holdEnd, after its entry; then play its exit. |
| `blend` | `number` | `0.2` | ≥ 0 | `s` | — | Seconds to blend in and out; 0 is immediate. |

```ogt
struct ReactionDesc {
  doc "An authored humanoid reaction over locomotion: entry, a held interval and exit from a cooked `type motion-action` asset (source, clip, optional rig, holdStart and holdEnd, optional hand and handTarget). Timing and strength belong to the tick; the generated pose is presentation only."
  guide "animation §11"
  desc true
  field action String "The motion-action asset path, such as actions/brace.ogt. Empty releases the current reaction."
  field direction Vector3 "World direction the character reacts toward (toward the wind source for a brace); zero keeps the character's facing."
  field strength f32 "Contribution of the authored reaction; 0 leaves locomotion unchanged." {
    default 1
    min 0
    max 1
  }
  field duration f32 "Seconds to hold between the action's holdStart and holdEnd, after its entry; then play its exit." {
    default 1
    min 0
    unit s
  }
  field blend f32 "Seconds to blend in and out; 0 is immediate." {
    default 0.2
    min 0
    unit s
  }
}
```

<sub>Module `motion` line 186</sub>
