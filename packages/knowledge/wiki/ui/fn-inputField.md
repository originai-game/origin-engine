<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-inputField"></a>

### `og.ui.inputField`

A text field (og.ui.textField's editing: IME, selection, clipboard, undo) in a framed row with a label above, an icon, a suffix, a clear button and a helper, error or counter line below. Its frame takes the focus ring as it is edited, turns danger and shakes once when an error appears (validity group). On a coarse pointer its row is 44 CSS px at least. Parts: `Input.frame[focused][invalid][disabled][hover]`, `Input.clear`. Role textbox named by its label; the error is an alert; the clear button is a button under a finger (a stop keys and pads reach too) and a named image on a fine pointer. Disabled, its text is drawn in muted ink on a fainter frame, out of the focus order, a textbox the tree calls disabled. Probe: `<id>/Frame/Field` (its text), `<id>/Frame/Clear`, `<id>/Foot/Error`.

```ts
og.ui.inputField(id: string, value: string, desc?: InputFieldDesc): InputResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `InputFieldDesc` | yes | Its label and adornments. |

**Returns** `InputResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn inputField {
  doc "A text field (og.ui.textField's editing: IME, selection, clipboard, undo) in a framed row with a label above, an icon, a suffix, a clear button and a helper, error or counter line below. Its frame takes the focus ring as it is edited, turns danger and shakes once when an error appears (validity group). On a coarse pointer its row is 44 CSS px at least. Parts: `Input.frame[focused][invalid][disabled][hover]`, `Input.clear`. Role textbox named by its label; the error is an alert; the clear button is a button under a finger (a stop keys and pads reach too) and a named image on a fine pointer. Disabled, its text is drawn in muted ink on a fainter frame, out of the focus order, a textbox the tree calls disabled. Probe: `<id>/Frame/Field` (its text), `<id>/Frame/Clear`, `<id>/Foot/Error`."
  host false
  param id String "Its id."
  param value String "The text from game state."
  param desc InputFieldDesc "Its label and adornments."
  returns InputResult
}
```

<sub>Module `ui` line 3882</sub>
