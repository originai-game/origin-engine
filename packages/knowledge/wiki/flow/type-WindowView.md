<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-WindowView"></a>

### `WindowView` (struct)

A segment's choice window.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ticksLeft` | `number` | `0` | — | — | — | Ticks until it locks (flow.lock). |
| `ticksTotal` | `number` | `0` | — | — | — | Ticks from flow.open to flow.lock. |
| `defaultChoice` | `number` | `0` | — | — | — | The ChoiceView.index taken when nothing was chosen (`#default`, else the first enabled). |
| `pending` | `number \| null` | `null` | — | — | — | The choice taken so far. |
| `locked` | `boolean` | `false` | — | — | — | Past flow.lock: choosing is over; the cut takes pending, else the default. |
| `held` | `boolean` | `false` | — | — | — | The segment holds (flow.hold) or loops (flow.loopStart..flow.loopEnd): the window never locks, and a choice cuts at once (or breaks the loop). |

```ogt
struct WindowView {
  doc "A segment's choice window."
  field ticksLeft u32 "Ticks until it locks (flow.lock)."
  field ticksTotal u32 "Ticks from flow.open to flow.lock."
  field defaultChoice u32 "The ChoiceView.index taken when nothing was chosen (`#default`, else the first enabled)."
  field pending Option.u32 "The choice taken so far."
  field locked bool "Past flow.lock: choosing is over; the cut takes pending, else the default."
  field held bool "The segment holds (flow.hold) or loops (flow.loopStart..flow.loopEnd): the window never locks, and a choice cuts at once (or breaks the loop)."
}
```

<sub>Module `flow` line 95</sub>
