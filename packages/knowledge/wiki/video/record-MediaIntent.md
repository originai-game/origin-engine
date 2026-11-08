<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="record-MediaIntent"></a>

### `og.MediaIntent` (event)

A document's `<video>` or `<audio>` asking its player to change at the next tick boundary (docs/ui.md §2), sent through the recorded intent path as UI pause is, so playback stays world state. The og.video system applies it to the entity's og.VideoPlayer and og.AudioSource.

Token `og.MediaIntent`, registered under the schema name `og.MediaIntent`; its value type is `MediaIntent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `player` | `Entity` | `0` | — | — | — | The entity whose player it asks. |
| `playing` | `number` | `-1` | -1 … 1 | — | — | 1 play, 0 pause, -1 as it is. |
| `seek` | `number` | `-1` | — | — | — | Seconds to seek a video to; negative: no seek. |
| `gain` | `number` | `-1` | — | — | — | Its sound's volume (0 while muted); negative: as it is. |
| `repeat` | `number` | `-1` | -1 … 1 | — | — | 1 loop, 0 not, -1 as it is. |

```ogt
event MediaIntent {
  doc "A document's `<video>` or `<audio>` asking its player to change at the next tick boundary (docs/ui.md §2), sent through the recorded intent path as UI pause is, so playback stays world state. The og.video system applies it to the entity's og.VideoPlayer and og.AudioSource."
  field player Entity "The entity whose player it asks."
  field playing i32 "1 play, 0 pause, -1 as it is." {
    default -1
    min -1
    max 1
  }
  field seek f64 "Seconds to seek a video to; negative: no seek." {
    default -1
  }
  field gain f32 "Its sound's volume (0 while muted); negative: as it is." {
    default -1
  }
  field repeat i32 "1 loop, 0 not, -1 as it is." {
    default -1
    min -1
    max 1
  }
}
```

<sub>Module `video` line 50</sub>
