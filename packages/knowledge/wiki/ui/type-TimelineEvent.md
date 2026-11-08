<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimelineEvent"></a>

### `TimelineEvent` (struct)

An event at a time, on the play's element or a descendant.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `action` | `TimelineAction` | — | — | — | — | What it does. |

```ogt
struct TimelineEvent {
  doc "An event at a time, on the play's element or a descendant."
  field atMs f32 "Its time." {
    unit ms
  }
  field target String "A path below the play's element (empty: the element itself)."
  field action TimelineAction "What it does."
}
```

<sub>Module `ui` line 4917</sub>
