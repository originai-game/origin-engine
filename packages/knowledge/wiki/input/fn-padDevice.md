<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-padDevice"></a>

### `og.input.padDevice`

A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). Null for a pad that is not connected or that the host did not name.

```ts
og.input.padDevice(world: World, pad: number): InputDevice | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `pad` | `number` | no | The pad, 0 to 7. |

**Returns** `InputDevice | null`.

Binding symbol `og_input_pad_device` · publish builds: kept

```ogt
fn padDevice {
  doc "A pad's family as the host told it from the pad's id (Xbox, PlayStation, Switch or a generic pad). Null for a pad that is not connected or that the host did not name."
  param world World "The world."
  param pad u32 "The pad, 0 to 7."
  returns Option.InputDevice
}
```

<sub>Module `input` line 510</sub>
