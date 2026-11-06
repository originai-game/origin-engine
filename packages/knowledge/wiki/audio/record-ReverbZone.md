<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-ReverbZone"></a>

### `og.ReverbZone` (component)

A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener.

Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it.

Token `og.ReverbZone`, registered under the schema name `og.ReverbZone`; its value type is `ReverbZone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 10, y: 4, z: 10 }` | — | `m` | — | The box's full size along the entity's axes, centred on it (times og.Transform.scale). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box. |
| `fade` | `number` | `2` | ≥ 0 | `m` | tunable | How far past the volume's surface the zone fades out, linearly; 0 is a hard edge. |
| `bus` | `AudioBus` | `"sfx"` | — | — | — | The bus whose reverb send the zone sets. |
| `send` | `number` | `0.5` | 0 … 1 | — | tunable | That bus's send to the reverb inside (its AudioBusFx.reverb outside). |
| `room` | `number` | `0.85` | 0 … 1 | — | tunable | The reverb's room size inside (Audio.reverbRoom outside). |
| `damping` | `number` | `0.3` | 0 … 1 | — | tunable | The reverb's damping of high frequencies inside (Audio.reverbDamping outside). |
| `weight` | `number` | `0` | 0 … 1 | — | read only | How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick). |

```ogt
component ReverbZone {
  doc "A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener."
  doc "Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it."
  field size Vector3 "The box's full size along the entity's axes, centred on it (times og.Transform.scale)." {
    default 10 4 10
    unit m
  }
  field radius f32 "A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box." {
    min 0
    unit m
    tunable true
  }
  field fade f32 "How far past the volume's surface the zone fades out, linearly; 0 is a hard edge." {
    default 2
    min 0
    unit m
    tunable true
  }
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

<sub>`api.ogt` line 13424</sub>
