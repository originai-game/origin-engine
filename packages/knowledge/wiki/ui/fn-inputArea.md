<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-inputArea"></a>

### `og.ui.inputArea`

A text area (og.ui.textArea's editing) framed as inputField is, with its label, helper, error and counter. Parts: `Input.frame[area][focused][invalid]` and the field's. Probe: `<id>/Frame/Area`, `<id>/Foot/Counter`.

```ts
og.ui.inputArea(id: string, value: string, desc?: InputAreaDesc): InputResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `InputAreaDesc` | yes | Its label and adornments. |

**Returns** `InputResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn inputArea {
  doc "A text area (og.ui.textArea's editing) framed as inputField is, with its label, helper, error and counter. Parts: `Input.frame[area][focused][invalid]` and the field's. Probe: `<id>/Frame/Area`, `<id>/Foot/Counter`."
  host false
  param id String "Its id."
  param value String "The text from game state."
  param desc InputAreaDesc "Its label and adornments."
  returns InputResult
}
```

<sub>`api.ogt` line 19750</sub>
