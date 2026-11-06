<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="record-TimelineMarker"></a>

### `og.TimelineMarker` (event)

Playback passed a marker key (a seek over one does not send it).

Token `og.TimelineMarker`, registered under the schema name `og.TimelineMarker`; its value type is `TimelineMarker`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |
| `name` | `string` | `""` | — | — | — | The marker's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The marker's time. |

```ogt
event TimelineMarker {
  doc "Playback passed a marker key (a seek over one does not send it)."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
  field name String "The marker's name (at most 63 bytes)."
  field time f32 "The marker's time." {
    unit s
  }
}
```

<sub>`api.ogt` line 26054</sub>
