<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SplitResult"></a>

### `SplitResult` (struct)

What a split view answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ratio` | `number` | `0` | — | — | — | The start pane's share after this tick: keep it and pass it back. |
| `changed` | `boolean` | `false` | — | — | — | The divider was moved or stepped this tick. |
| `stacked` | `boolean` | `false` | — | — | — | It is stacked (a phone in portrait). |

```ogt
struct SplitResult {
  doc "What a split view answered this tick."
  field ratio f32 "The start pane's share after this tick: keep it and pass it back."
  field changed bool "The divider was moved or stepped this tick."
  field stacked bool "It is stacked (a phone in portrait)."
}
```

<sub>Module `ui` line 2033</sub>
