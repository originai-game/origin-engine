<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TreeResult"></a>

### `TreeResult` (struct)

What a tree answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `string \| null` | `null` | — | — | — | The item pressed. |
| `toggled` | `string \| null` | `null` | — | — | — | The item opened or closed. |
| `expanded` | `readonly string[]` | `[]` | — | — | — | The open items after this tick: keep them. |

```ogt
struct TreeResult {
  doc "What a tree answered this tick."
  field chosen Option.String "The item pressed."
  field toggled Option.String "The item opened or closed."
  field expanded List.String "The open items after this tick: keep them."
}
```

<sub>Module `ui` line 2652</sub>
