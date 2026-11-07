<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-sensitivity"></a>

### `og.input.sensitivity`

An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted.

```ts
og.input.sensitivity(world: World, action: string, device?: InputDevice): SensitivityDesc
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device class; omitted is the one used last. |

**Returns** `SensitivityDesc`.

Binding symbol `og_input_sensitivity` · publish builds: kept

```ogt
fn sensitivity {
  doc "An action's sensitivity on a device class (the device used last when omitted): the player's, else scale 1 and nothing inverted."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device class; omitted is the one used last." {
    optional true
  }
  returns SensitivityDesc
}
```

<sub>Module `input` line 531</sub>
