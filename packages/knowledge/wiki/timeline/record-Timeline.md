<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="record-Timeline"></a>

### `og.Timeline` (component)

Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns.

Token `og.Timeline`, registered under the schema name `og.Timeline`; its value type is `Timeline`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline (og.load.timeline); 0 plays nothing. |
| `playing` | `boolean` | `false` | — | — | — | Advances while true. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Where it is, in seconds; written every tick it plays, set it to seek. |
| `speed` | `number` | `1` | 0 … 16 | — | tunable | Playback rate: 1 as authored, 0.5 half speed (0 holds the time while playing). |
| `loop` | `boolean` | `false` | — | — | — | Start over at the end; otherwise stop playing there, hand the camera back and send og.TimelineEnded. |

```ogt
component Timeline {
  doc "Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns."
  field asset TimelineAsset "The timeline (og.load.timeline); 0 plays nothing."
  field playing bool "Advances while true."
  field time f32 "Where it is, in seconds; written every tick it plays, set it to seek." {
    min 0
    unit s
  }
  field speed f32 "Playback rate: 1 as authored, 0.5 half speed (0 holds the time while playing)." {
    default 1
    min 0
    max 16
    tunable true
  }
  field loop bool "Start over at the end; otherwise stop playing there, hand the camera back and send og.TimelineEnded."
}
```

<sub>`api.ogt` line 26158</sub>
