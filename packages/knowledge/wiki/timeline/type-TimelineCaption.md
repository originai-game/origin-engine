<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="type-TimelineCaption"></a>

### `TimelineCaption` (struct)

A caption a playing timeline is saying.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Stable while it shows: the timeline's entity, its track and its key. |
| `text` | `string` | `""` | — | — | — | Its words, in the player's locale. |

```ogt
struct TimelineCaption {
  doc "A caption a playing timeline is saying."
  field id String "Stable while it shows: the timeline's entity, its track and its key."
  field text String "Its words, in the player's locale."
}
```

<sub>Module `timeline` line 75</sub>
