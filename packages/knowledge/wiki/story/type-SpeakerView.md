<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-SpeakerView"></a>

### `SpeakerView` (struct)

Who speaks a line.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The speaker's id (`narrator` for the narrator). |
| `name` | `string` | `""` | — | — | — | Its name, resolved in the active locale (empty for the narrator). |
| `entity` | `Entity` | `0` | — | — | — | The entity playing it (StartDesc.cast; 0: none). |
| `portrait` | `string` | `""` | — | — | — | A portrait image's asset path (empty: none). |
| `expression` | `string` | `""` | — | — | — | The portrait's variant: the line's `expression`, else the speaker's. |
| `side` | `string` | `""` | — | — | — | left, right or center (VN staging). |

```ogt
struct SpeakerView {
  doc "Who speaks a line."
  field id String "The speaker's id (`narrator` for the narrator)."
  field name String "Its name, resolved in the active locale (empty for the narrator)."
  field entity Entity "The entity playing it (StartDesc.cast; 0: none)."
  field portrait String "A portrait image's asset path (empty: none)."
  field expression String "The portrait's variant: the line's `expression`, else the speaker's."
  field side String "left, right or center (VN staging)."
}
```

<sub>`api.ogt` line 14535</sub>
