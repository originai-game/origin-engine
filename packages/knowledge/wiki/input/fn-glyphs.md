<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-glyphs"></a>

### `og.input.glyphs`

An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). Empty for an unknown action or one with no binding there.

```ts
og.input.glyphs(world: World, action: string, device?: InputDevice): readonly InputGlyph[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `device` | `InputDevice` | yes | The device; omitted is the one used last. |

**Returns** `readonly InputGlyph[]`.

Binding symbol `og_input_glyphs` · publish builds: kept

```ogt
fn glyphs {
  doc "An action's first binding on the device, one glyph per control (a composite's parts in prompt order: up, left, down, right; negative, positive). Empty for an unknown action or one with no binding there."
  param world World "The world."
  param action String "The action."
  param device InputDevice "The device; omitted is the one used last." {
    optional true
  }
  returns List.InputGlyph
}
```

<sub>Module `input` line 488</sub>
