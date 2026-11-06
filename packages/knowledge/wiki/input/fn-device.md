<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-device"></a>

### `og.input.device`

The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. Prompts follow it live.

```ts
og.input.device(world: World): InputDevice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `InputDevice`.

Binding symbol `og_input_device` · publish builds: kept

```ogt
fn device {
  doc "The device the player used last: a key or the mouse, a pad (its family as the host told it, else xbox), or a touch. Prompts follow it live."
  param world World "The world."
  returns InputDevice
}
```

<sub>`api.ogt` line 2479</sub>
