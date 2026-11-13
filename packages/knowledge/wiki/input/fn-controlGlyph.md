<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-controlGlyph"></a>

### `og.input.controlGlyph`

A control's glyph (what a capture took, before it is applied). Null for a name that names no control.

```ts
og.input.controlGlyph(control: string, device?: InputDevice): InputGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `control` | `string` | no | The control. |
| `device` | `InputDevice` | yes | The device it is drawn for (a pad's family); omitted is xbox for a pad control. |

**Returns** `InputGlyph | null`.

Binding symbol `og_input_control_glyph` · publish builds: kept

```ogt
fn controlGlyph {
  doc "A control's glyph (what a capture took, before it is applied). Null for a name that names no control."
  param control String "The control."
  param device InputDevice "The device it is drawn for (a pad's family); omitted is xbox for a pad control." {
    optional true
  }
  returns Option.InputGlyph
}
```

<sub>Module `input` line 509</sub>
