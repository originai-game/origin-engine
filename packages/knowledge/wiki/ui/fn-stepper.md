<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-stepper"></a>

### `og.ui.stepper`

A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too.

```ts
og.ui.stepper(id: string, value: number, desc?: StepperDesc): StepperResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `StepperDesc` | yes | Range, step and digits. |

**Returns** `StepperResult`.

Binding symbol `og_ui_stepper` · publish builds: kept

```ogt
fn stepper {
  doc "A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too."
  param id String "Its id."
  param value f32 "The value from game state."
  param desc StepperDesc "Range, step and digits."
  returns StepperResult
}
```

<sub>`api.ogt` line 17077</sub>
