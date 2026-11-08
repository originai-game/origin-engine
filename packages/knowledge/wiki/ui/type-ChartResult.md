<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChartResult"></a>

### `ChartResult` (struct)

What a chart answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hovered` | `number \| null` | `null` | — | — | — | The category under the pointer or the focus. |
| `clicked` | `number \| null` | `null` | — | — | — | The category pressed. |
| `retry` | `boolean` | `false` | — | — | — | Retry was pressed. |

```ogt
struct ChartResult {
  doc "What a chart answered this tick."
  field hovered Option.u32 "The category under the pointer or the focus."
  field clicked Option.u32 "The category pressed."
  field retry bool "Retry was pressed."
}
```

<sub>Module `ui` line 2584</sub>
