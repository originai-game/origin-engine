<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-ReverbZone"></a>

### `og.ReverbZone` (component)

The shared reverb sounds like this room inside the og.Volume on its entity (none: everywhere). While the og.AudioListener is inside, og.audio eases the reverb's room size, damping and one bus's send towards the zone's, fading with the volume's `fade`.

Each tick the listener's weight in every zone (`weight`) is a function of its position, the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream. It never writes Audio.reverbRoom, Audio.reverbDamping or AudioBusFx.reverb, the open air's; it sits at its world transform.

See `audio §4`.

Token `og.ReverbZone`, registered under the schema name `og.ReverbZone`; its value type is `ReverbZone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bus` | `AudioBus` | `"sfx"` | — | — | — | The bus whose reverb send the zone sets. |
| `send` | `number` | `0.5` | 0 … 1 | — | tunable | That bus's send to the reverb inside (its AudioBusFx.reverb outside). |
| `room` | `number` | `0.85` | 0 … 1 | — | tunable | The reverb's room size inside (Audio.reverbRoom outside). |
| `damping` | `number` | `0.3` | 0 … 1 | — | tunable | The reverb's damping of high frequencies inside (Audio.reverbDamping outside). |
| `weight` | `number` | `0` | 0 … 1 | — | read only | How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick). |

```ogt
component ReverbZone {
  doc "The shared reverb sounds like this room inside the og.Volume on its entity (none: everywhere). While the og.AudioListener is inside, og.audio eases the reverb's room size, damping and one bus's send towards the zone's, fading with the volume's `fade`."
  doc "Each tick the listener's weight in every zone (`weight`) is a function of its position, the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream. It never writes Audio.reverbRoom, Audio.reverbDamping or AudioBusFx.reverb, the open air's; it sits at its world transform."
  guide "audio §4"
  field bus AudioBus "The bus whose reverb send the zone sets." {
    default sfx
  }
  field send f32 "That bus's send to the reverb inside (its AudioBusFx.reverb outside)." {
    default 0.5
    min 0
    max 1
    tunable true
  }
  field room f32 "The reverb's room size inside (Audio.reverbRoom outside)." {
    default 0.85
    min 0
    max 1
    tunable true
  }
  field damping f32 "The reverb's damping of high frequencies inside (Audio.reverbDamping outside)." {
    default 0.3
    min 0
    max 1
    tunable true
  }
  field weight f32 "How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick)." {
    min 0
    max 1
    readonly true
  }
}
```

<sub>Module `audio` line 621</sub>
