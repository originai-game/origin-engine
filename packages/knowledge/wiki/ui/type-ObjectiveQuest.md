<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ObjectiveQuest"></a>

### `ObjectiveQuest` (desc struct)

An objective in the tracker.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key (its probe id). |
| `title` | `string` | `""` | — | — | — | Its title. |
| `steps` | `readonly ObjectiveStep[]` | `[]` | — | — | — | Its steps. |
| `timer` | `number \| null` | `null` | — | `s` | — | Seconds left, counting down in its header. |
| `distance` | `number \| null` | `null` | — | `m` | — | How far its marker is. |
| `icon` | `string` | `""` | — | — | — | Its icon (a main quest's, a side quest's). |
| `focused` | `boolean` | `false` | — | — | — | The tracked one: its bar in highlight. |
| `completedAt` | `number \| null` | `null` | — | — | — | og.ui.tick() when it was completed: it flashes Complete and leaves. |
| `completeLabel` | `string` | `"Complete"` | — | — | — | The completed word. |

```ogt
struct ObjectiveQuest {
  doc "An objective in the tracker."
  desc true
  field key String "Its key (its probe id)."
  field title String "Its title."
  field steps List.ObjectiveStep "Its steps."
  field timer Option.f32 "Seconds left, counting down in its header." {
    unit s
  }
  field distance Option.f32 "How far its marker is." {
    unit m
  }
  field icon String "Its icon (a main quest's, a side quest's)."
  field focused bool "The tracked one: its bar in highlight."
  field completedAt Option.u32 "og.ui.tick() when it was completed: it flashes Complete and leaves."
  field completeLabel String "The completed word." {
    default "Complete"
  }
}
```

<sub>Module `ui` line 6446</sub>
