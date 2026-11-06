<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="record-MotionAgent"></a>

### `og.MotionAgent` (component)

A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). Goals queue with og.motion.pose, place, face and walkTo. The generated pose is drawn in place of og.anim's (presentation only); og.anim still poses the skeleton every tick for everything the simulation reads, and is what is drawn until the model is resident.

Token `og.MotionAgent`, registered under the schema name `og.MotionAgent`; its value type is `MotionAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig` | `0` | — | — | — | The humanoid map of the skeleton (og.load.rig); 0 detects the bones from their names, as og.anim.humanoid does. |
| `drive` | `boolean` | `false` | — | — | — | og.motion moves og.Transform to the goals in the tick (walkTo, place, face) with fixed math, and the pose is re-rooted onto it; off, the game moves the Transform and the motion follows it, and only pose goals are taken. |
| `turnSpeed` | `number` | `6` | 0.1 … 50 | `radps` | tunable | How fast a driven character turns to its heading. |
| `lookahead` | `number` | `0.6` | 0.1 … 4 | `s` | tunable | How much motion the model keeps generated ahead of what is drawn: more absorbs a slow GPU, less follows a changed goal sooner. Presentation only. |

```ogt
component MotionAgent {
  doc "A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). Goals queue with og.motion.pose, place, face and walkTo. The generated pose is drawn in place of og.anim's (presentation only); og.anim still poses the skeleton every tick for everything the simulation reads, and is what is drawn until the model is resident."
  field rig Rig "The humanoid map of the skeleton (og.load.rig); 0 detects the bones from their names, as og.anim.humanoid does."
  field drive bool "og.motion moves og.Transform to the goals in the tick (walkTo, place, face) with fixed math, and the pose is re-rooted onto it; off, the game moves the Transform and the motion follows it, and only pose goals are taken."
  field turnSpeed f32 "How fast a driven character turns to its heading." {
    default 6
    min 0.1
    max 50
    unit radps
    tunable true
  }
  field lookahead f32 "How much motion the model keeps generated ahead of what is drawn: more absorbs a slow GPU, less follows a changed goal sooner. Presentation only." {
    default 0.6
    min 0.1
    max 4
    unit s
    tunable true
  }
}
```

<sub>`api.ogt` line 26658</sub>
