<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-startCapture"></a>

### `og.input.startCapture`

Start capturing the player's next input for a slot of an action (a controls screen's "press a key"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture.

```ts
og.input.startCapture(world: World, desc?: CaptureDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `CaptureDesc` | yes | The action, slot, part and the devices it takes. |

**Returns** `Result<void>`: Refused for an unknown or locked action, a slot or part a rebinding would refuse, or a cancel control that names none.

Binding symbol `og_input_start_capture` · publish builds: kept

```ogt
fn startCapture {
  doc "Start capturing the player's next input for a slot of an action (a controls screen's \"press a key\"): read og.input.capture each tick until it is done or cancelled, then og.input.applyCapture."
  param world World "The world."
  param desc CaptureDesc "The action, slot, part and the devices it takes."
  returns Result.Void "Refused for an unknown or locked action, a slot or part a rebinding would refuse, or a cancel control that names none."
}
```

<sub>Module `input` line 450</sub>
