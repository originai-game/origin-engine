<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-counter"></a>

### `og.ui.counter`

A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards.

```ts
og.ui.counter(id: string, value: number, desc?: CounterDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value to count to. |
| `desc` | `CounterDesc` | yes | Look, duration, start and format. |

**Returns** `number`: The number shown this tick.

Binding symbol `og_ui_counter` · publish builds: kept

```ogt
fn counter {
  doc "A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards."
  param id String "Its id."
  param value f64 "The value to count to."
  param desc CounterDesc "Look, duration, start and format."
  returns f64 "The number shown this tick."
}
```

<sub>Module `ui` line 1831</sub>
