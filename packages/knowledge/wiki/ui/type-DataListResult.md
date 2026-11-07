<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DataListResult"></a>

### `DataListResult` (struct)

What a data list or grid answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `number \| null` | `null` | — | — | — | The row or cell pressed (or activated). |
| `retry` | `boolean` | `false` | — | — | — | Retry was pressed. |

```ogt
struct DataListResult {
  doc "What a data list or grid answered this tick."
  field clicked Option.u32 "The row or cell pressed (or activated)."
  field retry bool "Retry was pressed."
}
```

<sub>Module `ui` line 2474</sub>
