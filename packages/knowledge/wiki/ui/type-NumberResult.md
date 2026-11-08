<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NumberResult"></a>

### `NumberResult` (struct)

A stepper's value after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | The value, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |

```ogt
struct NumberResult {
  doc "A stepper's value after this tick."
  field value f32 "The value, to keep and pass back."
  field changed bool "It changed this tick."
}
```

<sub>Module `ui` line 3736</sub>
