<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Skeleton2DEvent"></a>

### `og.Skeleton2DEvent` (event)

A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass.

Token `og.Skeleton2DEvent`, registered under the schema name `og.Skeleton2DEvent`; its value type is `Skeleton2DEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton2D. |
| `clip` | `number` | `0` | — | — | — | The clip index. |
| `name` | `string` | `""` | — | — | — | The event's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The event's time in the clip. |
| `armature` | `string` | `""` | — | — | — | Empty for the skeleton's own clip; for an event of a child armature shown in one of its slots, that armature's name (`clip` and `time` are then the child's clip and its time): a flame's crackle apart from the keeper's footsteps (at most 63 bytes). |

```ogt
event Skeleton2DEvent {
  doc "A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 \"step\"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass."
  field entity Entity "The entity with the og.Skeleton2D."
  field clip u32 "The clip index."
  field name String "The event's name (at most 63 bytes)."
  field time f32 "The event's time in the clip." {
    unit s
  }
  field armature String "Empty for the skeleton's own clip; for an event of a child armature shown in one of its slots, that armature's name (`clip` and `time` are then the child's clip and its time): a flame's crackle apart from the keeper's footsteps (at most 63 bytes)."
}
```

<sub>`api.ogt` line 12258</sub>
