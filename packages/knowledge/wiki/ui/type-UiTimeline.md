<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiTimeline"></a>

### `UiTimeline` (desc struct)

Keyframed tracks and events (docs/ui.md §3.1).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tracks` | `readonly TimelineTrack[]` | `[]` | — | — | — | Its tracks. |
| `events` | `readonly TimelineEvent[]` | `[]` | — | — | — | Its events. |
| `labels` | `readonly TimelineLabel[]` | `[]` | — | — | — | Its named times. |
| `durationMs` | `number` | `0` | — | `ms` | — | Its length (0: its last key or event). |

```ogt
struct UiTimeline {
  doc "Keyframed tracks and events (docs/ui.md §3.1)."
  desc true
  field tracks List.TimelineTrack "Its tracks."
  field events List.TimelineEvent "Its events."
  field labels List.TimelineLabel "Its named times."
  field durationMs f32 "Its length (0: its last key or event)." {
    unit ms
  }
}
```

<sub>Module `ui` line 2079</sub>
