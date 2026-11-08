<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChoiceIndexResult"></a>

### `ChoiceIndexResult` (struct)

A choice control's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `number` | `0` | — | — | — | The chosen index, to keep and pass back (-1: none). |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |
| `focused` | `number` | `0` | — | — | — | The index with the focus (-1: none). |

```ogt
struct ChoiceIndexResult {
  doc "A choice control's state after this tick."
  field selected i32 "The chosen index, to keep and pass back (-1: none)."
  field changed bool "It changed this tick."
  field focused i32 "The index with the focus (-1: none)."
}
```

<sub>Module `ui` line 3610</sub>
