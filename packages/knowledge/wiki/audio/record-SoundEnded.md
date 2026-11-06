<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-SoundEnded"></a>

### `og.SoundEnded` (event)

A sound ended, sent in the tick it happens (systems read it the tick after).

Token `og.SoundEnded`, registered under the schema name `og.SoundEnded`; its value type is `SoundEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource ended. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |
| `reason` | `number` | `0` | — | — | — | Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3), SOUND_CULLED (4) or SOUND_RETRIGGERED (5). |

```ogt
event SoundEnded {
  doc "A sound ended, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource ended."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
  field reason u32 "Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3), SOUND_CULLED (4) or SOUND_RETRIGGERED (5)."
}
```

<sub>`api.ogt` line 13510</sub>
