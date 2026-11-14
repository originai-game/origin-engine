<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-Music"></a>

### `og.Music` (component)

A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat.

Token `og.Music`, registered under the schema name `og.Music`; its value type is `Music`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bpm` | `number` | `120` | 20 … 400 | `bpm` | tunable | Beats per minute. |
| `beatsPerBar` | `number` | `4` | 1 … 16 | — | — | Beats per bar. |
| `start` | `number` | `0` | — | `s` | — | The world time of bar 1's first beat. Can be negative when starting partway through a song. |
| `bar` | `number` | `0` | — | — | read only | The bar the next tick starts in, from 1 (0 before start; written by og.music). |
| `beat` | `number` | `0` | — | — | read only | The beat in that bar, from 1 (written by og.music). |
| `nextBeat` | `number` | `0` | — | `s` | read only | The world time of the next beat from the next tick on (written by og.music). |
| `nextBar` | `number` | `0` | — | `s` | read only | The world time of the next bar line from the next tick on (written by og.music). |

```ogt
component Music {
  doc "A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat."
  field bpm f32 "Beats per minute." {
    default 120
    min 20
    max 400
    unit bpm
    tunable true
  }
  field beatsPerBar u32 "Beats per bar." {
    default 4
    min 1
    max 16
  }
  field start f64 "The world time of bar 1's first beat. Can be negative when starting partway through a song." {
    unit s
  }
  field bar u32 "The bar the next tick starts in, from 1 (0 before start; written by og.music)." {
    readonly true
  }
  field beat u32 "The beat in that bar, from 1 (written by og.music)." {
    readonly true
  }
  field nextBeat f64 "The world time of the next beat from the next tick on (written by og.music)." {
    unit s
    readonly true
  }
  field nextBar f64 "The world time of the next bar line from the next tick on (written by og.music)." {
    unit s
    readonly true
  }
}
```

<sub>Module `audio` line 653</sub>
