<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-WindowView"></a>

### `WindowView` (struct)

A segment's choice window.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ticksLeft` | `number` | `0` | — | — | — | Ticks until it locks (film.lock). |
| `ticksTotal` | `number` | `0` | — | — | — | Ticks from film.open to film.lock. |
| `defaultChoice` | `number` | `0` | — | — | — | The ChoiceView.index taken when nothing was chosen (`#default`, else the first enabled). |
| `pending` | `number \| null` | `null` | — | — | — | The choice taken so far. |
| `locked` | `boolean` | `false` | — | — | — | Past film.lock: choosing is over; the cut takes pending, else the default. |

```ogt
struct WindowView {
  doc "A segment's choice window."
  field ticksLeft u32 "Ticks until it locks (film.lock)."
  field ticksTotal u32 "Ticks from film.open to film.lock."
  field defaultChoice u32 "The ChoiceView.index taken when nothing was chosen (`#default`, else the first enabled)."
  field pending Option.u32 "The choice taken so far."
  field locked bool "Past film.lock: choosing is over; the cut takes pending, else the default."
}
```

<sub>Module `film` line 80</sub>
