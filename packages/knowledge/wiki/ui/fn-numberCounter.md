<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-numberCounter"></a>

### `og.ui.numberCounter`

A number display for scores, currency and resources: written in its notation (idle letters past trillions, numbers past 1e308), counting to a new value on an ease-out, its digits rolling like an odometer, a +n delta flying off on a rise, a pop when it lands (emphasis spring) and a flash of danger on a fall; on the HUD chip with its outline when anchored. Returns the number shown this tick.

Parts: NumberCounter.frame[hud], NumberCounter.value, NumberCounter.digit, NumberCounter.icon, NumberCounter.label, NumberCounter.delta[up|down]. States: group change (still, rising, falling). Sounds: counterTick while counting, if ticks. Accessibility: text named by the label, value the final number written out (not each frame of the count). Probe ids: <id>, <id>/Value (its text is the whole number shown), <id>/Label, <id>/Delta.

```ts
og.ui.numberCounter(id: string, value: number, desc?: NumberCounterDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value to count to (times 10^format.exponent). |
| `desc` | `NumberCounterDesc` | yes | How it looks and counts. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn numberCounter {
  doc "A number display for scores, currency and resources: written in its notation (idle letters past trillions, numbers past 1e308), counting to a new value on an ease-out, its digits rolling like an odometer, a +n delta flying off on a rise, a pop when it lands (emphasis spring) and a flash of danger on a fall; on the HUD chip with its outline when anchored. Returns the number shown this tick."
  doc "Parts: NumberCounter.frame[hud], NumberCounter.value, NumberCounter.digit, NumberCounter.icon, NumberCounter.label, NumberCounter.delta[up|down]. States: group change (still, rising, falling). Sounds: counterTick while counting, if ticks. Accessibility: text named by the label, value the final number written out (not each frame of the count). Probe ids: <id>, <id>/Value (its text is the whole number shown), <id>/Label, <id>/Delta."
  host false
  param id String "Its id."
  param value f64 "The value to count to (times 10^format.exponent)."
  param desc NumberCounterDesc "How it looks and counts."
  returns f64
}
```

<sub>`api.ogt` line 21423</sub>
