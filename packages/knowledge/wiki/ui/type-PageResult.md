<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PageResult"></a>

### `PageResult` (struct)

What pagination answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `page` | `number` | `0` | — | — | — | The current page after this tick: keep it. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |

```ogt
struct PageResult {
  doc "What pagination answered this tick."
  field page u32 "The current page after this tick: keep it."
  field changed bool "It changed this tick."
}
```

<sub>Module `ui` line 2146</sub>
