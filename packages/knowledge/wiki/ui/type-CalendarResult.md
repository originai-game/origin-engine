<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CalendarResult"></a>

### `CalendarResult` (struct)

What a calendar answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `string \| null` | `null` | — | — | — | The day pressed. |
| `year` | `number` | `0` | — | — | — | The year shown after this tick: keep it. |
| `month` | `number` | `0` | — | — | — | The month shown: keep it. |
| `changed` | `boolean` | `false` | — | — | — | The month was turned this tick. |

```ogt
struct CalendarResult {
  doc "What a calendar answered this tick."
  field selected Option.String "The day pressed."
  field year u32 "The year shown after this tick: keep it."
  field month u32 "The month shown: keep it."
  field changed bool "The month was turned this tick."
}
```

<sub>`api.ogt` line 18675</sub>
