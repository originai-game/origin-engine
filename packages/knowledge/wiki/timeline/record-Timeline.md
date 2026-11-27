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
| `loop` | `boolean` | `false` | — | — | — | Start over at loopStart past loopEnd (or 0 past the end); otherwise stop playing there, hand the camera back and send og.TimelineEnded. |
| `loopStart` | `number` | `0` | ≥ 0 | `s` | — | With `loop`: start of the loop region, in seconds (0: the timeline's start). |
| `loopEnd` | `number` | `0` | ≥ 0 | `s` | — | With `loop`: end of the loop region, in seconds (0: the timeline's duration). |
| `preroll` | `boolean` | `false` | — | — | — | Prepare without showing: while stopped and holding nothing, the timeline is not evaluated; the entity's og.VideoPlayer holds the frame its own video track starts with, paused, its scene tracks' scenes are loaded, and the engine modules it needs are fetched. Playing or seeking evaluates it as always. |

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
  field loop bool "Start over at loopStart past loopEnd (or 0 past the end); otherwise stop playing there, hand the camera back and send og.TimelineEnded."
  field loopStart f32 "With `loop`: start of the loop region, in seconds (0: the timeline's start)." {
    min 0
    unit s
  }
  field loopEnd f32 "With `loop`: end of the loop region, in seconds (0: the timeline's duration)." {
    min 0
    unit s
  }
  field preroll bool "Prepare without showing: while stopped and holding nothing, the timeline is not evaluated; the entity's og.VideoPlayer holds the frame its own video track starts with, paused, its scene tracks' scenes are loaded, and the engine modules it needs are fetched. Playing or seeking evaluates it as always."
}
```

<sub>Module `timeline` line 15</sub>
