<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AccordionResult"></a>

### `AccordionResult` (struct)

What an accordion answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toggled` | `number \| null` | `null` | — | — | — | The section whose header was pressed. |
| `expanded` | `readonly number[]` | `[]` | — | — | — | The open sections after this tick: keep them and pass them back. |

```ogt
struct AccordionResult {
  doc "What an accordion answered this tick."
  field toggled Option.u32 "The section whose header was pressed."
  field expanded List.u32 "The open sections after this tick: keep them and pass them back."
}
```

<sub>`api.ogt` line 17788</sub>
