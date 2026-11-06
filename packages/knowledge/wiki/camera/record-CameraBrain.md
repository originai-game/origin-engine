<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-CameraBrain"></a>

### `og.CameraBrain` (resource)

Picks the live og.VirtualCamera every tick and blends og.Camera to it, as Cinemachine's brain: the solo shot if one is set, else the enabled one with the highest priority. A change of live camera blends from the previous camera (still moving) over the incoming one's blendTime by its blendEase; a change during a blend starts from the blended pose, so the view never jumps. The first live camera of a world cuts. With no live camera og.Camera is left where it is.

Registered with og.VirtualCamera. Its state fields are world state, saved and restored with the world, so a replay or a restore mid-blend continues the same blend.

Read and write it as any resource: `const brain = world.resource(og.CameraBrain)`; when `brain.ok`, `brain.value.live` is the live shot and `brain.value.solo = shot` makes one solo (0 releases it). A system that reads it lists og.CameraBrain among its reads, one that sets solo among its writes.

Token `og.CameraBrain`, registered under the schema name `og.CameraBrain`; its value type is `CameraBrain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `true` | — | — | — | Drive og.Camera; false leaves it to the game (the vcams' state still updates). |
| `blendTime` | `number` | `1` | ≥ 0 | `s` | tunable | Seconds of a blend to a camera whose blendTime is below 0. |
| `blendEase` | `Ease` | `"cubicInOut"` | — | — | — | The curve of those blends. |
| `solo` | `Entity` | `0` | — | — | — | A shot that outranks every priority while it is set (a cutscene or a dialogue close-up): an entity with an og.VirtualCamera, live even when it is not enabled; 0 releases it and the brain blends back to the highest priority. Anything else is ignored. og.story sets it for a dialogue line's `shot` and a timeline's shot track for its keys. |
| `soloBlend` | `number` | `-1` | — | `s` | — | Seconds of the blend to the solo shot when it goes live (0 cuts); below 0 the solo camera's own blendTime and blendEase (then the brain's). A timeline's shot key sets it with solo; og.story sets it to -1. |
| `soloEase` | `Ease` | `"cubicInOut"` | — | — | — | The curve of that blend when soloBlend is 0 or more. |
| `live` | `Entity` | `0` | — | — | read only | The live camera (0: none). |
| `from` | `Entity` | `0` | — | — | read only | The camera being blended from; 0 when the blend starts from a held pose (an interrupted blend). |
| `blend` | `number` | `1` | — | — | read only | How far the blend has come, 0 to 1 (1: settled on the live camera); the weight is the curve at this time. |
| `blendDuration` | `number` | `0` | — | `s` | read only | The current blend's length. |
| `blendCurve` | `Ease` | `"quadOut"` | — | — | read only | The current blend's curve. |
| `fromPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | A held pose to blend from: its position, in world space (a floating-origin rebase shifts it). |
| `fromQuaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | read only | Its rotation. |
| `fromFov` | `number` | `0` | — | `deg` | read only | Its field of view. |
| `fromFocus` | `number` | `0` | — | `m` | read only | Its focus distance (0: none). |
| `fromFocusRange` | `number` | `0` | — | `m` | read only | Its focus range. |
| `activations` | `number` | `0` | — | — | read only | How many times a virtual camera has been enabled (og.VirtualCamera.activated). |
| `focusHeld` | `boolean` | `false` | — | — | read only | The brain is driving og.PostEffects' depth of field and holds what it found: heldDepthOfField, heldFocusDistance and heldFocusRange, handed back when no live camera has focus. |
| `heldDepthOfField` | `boolean` | `false` | — | — | read only | og.PostEffects.depthOfField as the brain found it. |
| `heldFocusDistance` | `number` | `0` | — | `m` | read only | og.PostEffects.dofFocusDistance as the brain found it. |
| `heldFocusRange` | `number` | `0` | — | `m` | read only | og.PostEffects.dofFocusRange as the brain found it. |

```ogt
resource CameraBrain {
  doc "Picks the live og.VirtualCamera every tick and blends og.Camera to it, as Cinemachine's brain: the solo shot if one is set, else the enabled one with the highest priority. A change of live camera blends from the previous camera (still moving) over the incoming one's blendTime by its blendEase; a change during a blend starts from the blended pose, so the view never jumps. The first live camera of a world cuts. With no live camera og.Camera is left where it is."
  doc "Registered with og.VirtualCamera. Its state fields are world state, saved and restored with the world, so a replay or a restore mid-blend continues the same blend."
  doc "Read and write it as any resource: `const brain = world.resource(og.CameraBrain)`; when `brain.ok`, `brain.value.live` is the live shot and `brain.value.solo = shot` makes one solo (0 releases it). A system that reads it lists og.CameraBrain among its reads, one that sets solo among its writes."
  field enabled bool "Drive og.Camera; false leaves it to the game (the vcams' state still updates)." {
    default true
  }
  field blendTime f32 "Seconds of a blend to a camera whose blendTime is below 0." {
    default 1
    min 0
    unit s
    tunable true
  }
  field blendEase Ease "The curve of those blends." {
    default cubicInOut
  }
  field solo Entity "A shot that outranks every priority while it is set (a cutscene or a dialogue close-up): an entity with an og.VirtualCamera, live even when it is not enabled; 0 releases it and the brain blends back to the highest priority. Anything else is ignored. og.story sets it for a dialogue line's `shot` and a timeline's shot track for its keys."
  field soloBlend f32 "Seconds of the blend to the solo shot when it goes live (0 cuts); below 0 the solo camera's own blendTime and blendEase (then the brain's). A timeline's shot key sets it with solo; og.story sets it to -1." {
    default -1
    unit s
  }
  field soloEase Ease "The curve of that blend when soloBlend is 0 or more." {
    default cubicInOut
  }
  field live Entity "The live camera (0: none)." {
    readonly true
  }
  field from Entity "The camera being blended from; 0 when the blend starts from a held pose (an interrupted blend)." {
    readonly true
  }
  field blend f32 "How far the blend has come, 0 to 1 (1: settled on the live camera); the weight is the curve at this time." {
    default 1
    readonly true
  }
  field blendDuration f32 "The current blend's length." {
    readonly true
    unit s
  }
  field blendCurve Ease "The current blend's curve." {
    readonly true
  }
  field fromPosition Vector3 "A held pose to blend from: its position, in world space (a floating-origin rebase shifts it)." {
    readonly true
    worldspace world
    unit m
  }
  field fromQuaternion Quaternion "Its rotation." {
    readonly true
  }
  field fromFov f32 "Its field of view." {
    readonly true
    unit deg
  }
  field fromFocus f32 "Its focus distance (0: none)." {
    readonly true
    unit m
  }
  field fromFocusRange f32 "Its focus range." {
    readonly true
    unit m
  }
  field activations u32 "How many times a virtual camera has been enabled (og.VirtualCamera.activated)." {
    readonly true
  }
  field focusHeld bool "The brain is driving og.PostEffects' depth of field and holds what it found: heldDepthOfField, heldFocusDistance and heldFocusRange, handed back when no live camera has focus." {
    readonly true
  }
  field heldDepthOfField bool "og.PostEffects.depthOfField as the brain found it." {
    readonly true
  }
  field heldFocusDistance f32 "og.PostEffects.dofFocusDistance as the brain found it." {
    readonly true
    unit m
  }
  field heldFocusRange f32 "og.PostEffects.dofFocusRange as the brain found it." {
    readonly true
    unit m
  }
}
```

<sub>`api.ogt` line 3679</sub>
