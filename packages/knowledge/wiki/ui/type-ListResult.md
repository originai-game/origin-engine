<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ListResult"></a>

### `ListResult` (struct)

What happened in a list or grid this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `number \| null` | `null` | — | — | — | The row or cell pressed (or activated) this tick. |
| `dropFrom` | `number \| null` | `null` | — | — | — | A drag and drop ended: the index it started on. |
| `dropTo` | `number \| null` | `null` | — | — | — | And the index it was released over. |

```ogt
struct ListResult {
  doc "What happened in a list or grid this tick."
  field clicked Option.u32 "The row or cell pressed (or activated) this tick."
  field dropFrom Option.u32 "A drag and drop ended: the index it started on."
  field dropTo Option.u32 "And the index it was released over."
}
```

<sub>`api.ogt` line 16652</sub>
