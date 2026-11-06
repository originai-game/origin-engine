<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RangeResult"></a>

### `RangeResult` (struct)

A range slider's values after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `low` | `number` | `0` | — | — | — | The low end, to keep and pass back. |
| `high` | `number` | `0` | — | — | — | The high end. |
| `changed` | `boolean` | `false` | — | — | — | Either changed this tick. |
| `committed` | `boolean` | `false` | — | — | — | A drag ended or a step landed. |

```ogt
struct RangeResult {
  doc "A range slider's values after this tick."
  field low f32 "The low end, to keep and pass back."
  field high f32 "The high end."
  field changed bool "Either changed this tick."
  field committed bool "A drag ended or a step landed."
}
```

<sub>`api.ogt` line 19468</sub>
