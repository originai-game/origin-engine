<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowTranscriptLine"></a>

### `FlowTranscriptLine` (struct)

One entry in a flow's transcript backlog (og.flow.history): a caption shown or a choice taken.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seq` | `number` | `0` | — | — | — | Monotonic sequence number within the flow. |
| `node` | `string` | `""` | — | — | — | The flow node where it occurred. |
| `speaker` | `string` | `""` | — | — | — | The speaker name, if any. |
| `text` | `string` | `""` | — | — | — | The caption text or chosen option text. |
| `spans` | `readonly TextSpan[]` | `[]` | — | — | — | Inline style spans over `text`. |
| `chosen` | `boolean` | `false` | — | — | — | True for a choice the player took; false for a spoken caption. |

```ogt
struct FlowTranscriptLine {
  doc "One entry in a flow's transcript backlog (og.flow.history): a caption shown or a choice taken."
  field seq u32 "Monotonic sequence number within the flow."
  field node String "The flow node where it occurred."
  field speaker String "The speaker name, if any."
  field text String "The caption text or chosen option text."
  field spans List.TextSpan "Inline style spans over `text`."
  field chosen bool "True for a choice the player took; false for a spoken caption."
}
```

<sub>Module `flow` line 180</sub>
