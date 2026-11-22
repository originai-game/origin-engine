<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setSensitivity"></a>

### `og.input.setSensitivity`

Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings.

```ts
og.input.setSensitivity(world: World, desc?: SensitivityDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SensitivityDesc` | yes | The action, the device class, the scale and the inversions. |

**Returns** `Result<void>`: Refused for an unknown action, a button action, or a scale that is not above 0.

Binding symbol `og_input_set_sensitivity` · publish builds: kept

```ogt
fn setSensitivity {
  doc "Set the player's sensitivity and inversion of an axis action on a device class (SensitivityDesc): applied from the next tick's actions, kept by og.input.saveBindings."
  param world World "The world."
  param desc SensitivityDesc "The action, the device class, the scale and the inversions."
  returns Result.Void "Refused for an unknown action, a button action, or a scale that is not above 0."
}
```

<sub>Module `input` line 528</sub>
