<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DropdownResult"></a>

### `DropdownResult` (struct)

A dropdown's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `number` | `0` | — | — | — | The chosen index, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |
| `open` | `boolean` | `false` | — | — | — | Its list is open. |

```ogt
struct DropdownResult {
  doc "A dropdown's state after this tick."
  field selected i32 "The chosen index, to keep and pass back."
  field changed bool "It changed this tick."
  field open bool "Its list is open."
}
```

<sub>Module `ui` line 1980</sub>
