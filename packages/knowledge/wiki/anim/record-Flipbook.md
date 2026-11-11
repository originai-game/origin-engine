<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Flipbook"></a>

### `og.Flipbook` (component)

Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or the frame's layer of an `array true` sheet's texture array. Played by og.flipbook by scaled time (Time.scaledDt), after the game's systems.

Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent; the playback position is world state (the og.flipbook state block). A sprite graph on the entity writes clip, time and speed.

See `animation §9`.

Token `og.Flipbook`, registered under the schema name `og.Flipbook`; its value type is `Flipbook`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sheet` | `FlipbookSheet` | `0` | — | — | — | The sheet (og.load.flipbook's `sheet`); 0 is none. |
| `clip` | `number` | `0` | ≥ 0 | — | — | The clip: an index into FlipbookInfo.clips (og.anim.clip finds one by name). |
| `speed` | `number` | `1` | 0 … 10 | — | tunable | The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame. |
| `playing` | `boolean` | `true` | — | — | — | Whether time advances. |
| `frame` | `number` | `0` | — | — | read only | Written: the clip frame shown (an index into the clip's frames). |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the current cycle; written every tick, set it to seek (no events fire across a seek). |
| `finished` | `boolean` | `false` | — | — | read only | Written: a `once` clip reached and holds its last frame. |
| `loops` | `number` | `0` | — | — | read only | Written: cycles completed by a looping or ping-pong clip. |

```ogt
component Flipbook {
  doc "Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or the frame's layer of an `array true` sheet's texture array. Played by og.flipbook by scaled time (Time.scaledDt), after the game's systems."
  doc "Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent; the playback position is world state (the og.flipbook state block). A sprite graph on the entity writes clip, time and speed."
  guide "animation §9"
  field sheet FlipbookSheet "The sheet (og.load.flipbook's `sheet`); 0 is none."
  field clip u32 "The clip: an index into FlipbookInfo.clips (og.anim.clip finds one by name)." {
    min 0
  }
  field speed f32 "The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame." {
    default 1
    min 0
    max 10
    tunable true
  }
  field playing bool "Whether time advances." {
    default true
  }
  field frame u32 "Written: the clip frame shown (an index into the clip's frames)." {
    readonly true
  }
  field time f32 "Seconds into the current cycle; written every tick, set it to seek (no events fire across a seek)." {
    min 0
    unit s
  }
  field finished bool "Written: a `once` clip reached and holds its last frame." {
    readonly true
  }
  field loops u32 "Written: cycles completed by a looping or ping-pong clip." {
    readonly true
  }
}
```

<sub>Module `anim` line 677</sub>
