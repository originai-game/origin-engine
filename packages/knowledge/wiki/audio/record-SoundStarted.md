<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-SoundStarted"></a>

### `og.SoundStarted` (event)

A source started, sent in the tick it happens (systems read it the tick after).

Token `og.SoundStarted`, registered under the schema name `og.SoundStarted`; its value type is `SoundStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource started. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |

```ogt
event SoundStarted {
  doc "A source started, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource started."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
}
```

<sub>`api.ogt` line 13504</sub>
