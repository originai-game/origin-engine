<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-numberStepper"></a>

### `og.ui.numberStepper`

A number stepper: minus, the value and plus. A press steps once, a held press repeats after 400 ms and speeds up (large steps after a second); Left and Right step it while the value has the focus. A bound's button goes disabled at the bound. Plays sliderTick per step. Parts: `Stepper.frame[focused|disabled]`, `Stepper.button[dec|inc][hover|pressed|disabled]`, `Stepper.value`. Its label and the buttons share a line while they fit, the buttons go under the label when they do not (larger text, a narrow column). Role spinbutton with its range; its − and + are buttons under a finger (stops keys and pads reach too) and named images on a fine pointer (Left and Right on the focused stepper are theirs). Probe: `<id>/Frame/Dec`, `<id>/Frame/Value`, `<id>/Frame/Inc`.

```ts
og.ui.numberStepper(id: string, value: number, desc?: NumberStepperDesc): NumberResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `NumberStepperDesc` | yes | Its range, step and look. |

**Returns** `NumberResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn numberStepper {
  doc "A number stepper: minus, the value and plus. A press steps once, a held press repeats after 400 ms and speeds up (large steps after a second); Left and Right step it while the value has the focus. A bound's button goes disabled at the bound. Plays sliderTick per step. Parts: `Stepper.frame[focused|disabled]`, `Stepper.button[dec|inc][hover|pressed|disabled]`, `Stepper.value`. Its label and the buttons share a line while they fit, the buttons go under the label when they do not (larger text, a narrow column). Role spinbutton with its range; its − and + are buttons under a finger (stops keys and pads reach too) and named images on a fine pointer (Left and Right on the focused stepper are theirs). Probe: `<id>/Frame/Dec`, `<id>/Frame/Value`, `<id>/Frame/Inc`."
  host false
  param id String "Its id."
  param value f32 "The value from game state."
  param desc NumberStepperDesc "Its range, step and look."
  returns NumberResult
}
```

<sub>`api.ogt` line 19584</sub>
