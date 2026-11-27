<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-glyph"></a>

### `og.input.glyph`

An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). Null for an unknown action or one with no binding on that device.

```ts
og.input.glyph(world: World, action: string, device?: InputDevice): InputGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device; omitted is the one used last. |

**Returns** `InputGlyph | null`.

Binding symbol `og_input_glyph` · publish builds: kept

```ogt
fn glyph {
  doc "An action's prompt glyph: its first binding on the device (the device used last when omitted); a composite's in one (`W A S D`, `key.wasd`). Null for an unknown action or one with no binding on that device."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device; omitted is the one used last." {
    optional true
  }
  returns Option.InputGlyph
}
```

<sub>Module `input` line 479</sub>
