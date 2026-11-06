<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-ReactionDesc"></a>

### `ReactionDesc` (desc struct)

An authored humanoid reaction over locomotion. Entry, a held interval, and exit come from a cooked type motion-action asset. Timing and strength belong to the tick; the generated pose is presentation only.

**Action asset.** An .ogt starts with `type motion-action`, `version 1`, then `---`. Its body has `source "anims/brace.glb"`, `clip "brace"`, optional `rig "rigs/meshy.ogt"`, `holdStart 0.5` and `holdEnd 1.5`, each on its own line. Paths are relative to assets/. The hold must lie inside the clip (0 <= holdStart < holdEnd < clip duration), with matching poses at both ends for looping. Use a stationary root, +Z facing the reaction source, entry before the hold, and recovery after it. A 30 Hz clip is recommended; the cook also accepts other sample rates.

**Hood contact.** Optional `hand right` (or left) and `handTarget x y z` attach that wrist to an offset from the head during the hold. Supply the offset in metres along the source head bone's local axes, independent of the GLB's unit scale. Retargeting adjusts it to the target's proportions; IK follows the final head pose after lookAt. The clip owns the stance step and lean: the engine does not synthesize an untrained action. Keep provenance and licensing beside the source asset.

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
  doc "An authored humanoid reaction over locomotion. Entry, a held interval, and exit come from a cooked type motion-action asset. Timing and strength belong to the tick; the generated pose is presentation only."
  doc "**Action asset.** An .ogt starts with `type motion-action`, `version 1`, then `---`. Its body has `source \"anims/brace.glb\"`, `clip \"brace\"`, optional `rig \"rigs/meshy.ogt\"`, `holdStart 0.5` and `holdEnd 1.5`, each on its own line. Paths are relative to assets/. The hold must lie inside the clip (0 <= holdStart < holdEnd < clip duration), with matching poses at both ends for looping. Use a stationary root, +Z facing the reaction source, entry before the hold, and recovery after it. A 30 Hz clip is recommended; the cook also accepts other sample rates."
  doc "**Hood contact.** Optional `hand right` (or left) and `handTarget x y z` attach that wrist to an offset from the head during the hold. Supply the offset in metres along the source head bone's local axes, independent of the GLB's unit scale. Retargeting adjusts it to the target's proportions; IK follows the final head pose after lookAt. The clip owns the stance step and lean: the engine does not synthesize an untrained action. Keep provenance and licensing beside the source asset."
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

<sub>`api.ogt` line 26948</sub>
