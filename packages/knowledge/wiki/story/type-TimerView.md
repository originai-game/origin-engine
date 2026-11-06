<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-TimerView"></a>

### `TimerView` (struct)

A node's choice timer.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ticksLeft` | `number` | `0` | — | — | — | Ticks until it runs out. |
| `ticksTotal` | `number` | `0` | — | — | — | Its length in ticks. |
| `defaultChoice` | `number` | `0` | — | — | — | The ChoiceView.index taken when it runs out. |

```ogt
struct TimerView {
  doc "A node's choice timer."
  field ticksLeft u32 "Ticks until it runs out."
  field ticksTotal u32 "Its length in ticks."
  field defaultChoice u32 "The ChoiceView.index taken when it runs out."
}
```

<sub>`api.ogt` line 14570</sub>
