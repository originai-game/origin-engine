<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-button"></a>

### `og.ui.button`

A button with a label; `clicked` is a press and a release inside it. The probe reads its label.

Also known as: click, press, Unity Button, Godot Button, HTML button

```ts
og.ui.button(id: string, text: string, desc?: ButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label. |
| `desc` | `ButtonDesc` | yes | Variant, size, disabled and width. |

**Returns** `Response`.

Binding symbol `og_ui_button` · publish builds: kept

```ogt
fn button {
  doc "A button with a label; `clicked` is a press and a release inside it. The probe reads its label."
  aliases "click, press, Unity Button, Godot Button, HTML button"
  param id String "Its id."
  param text String "The label."
  param desc ButtonDesc "Variant, size, disabled and width."
  returns Response
}
```

<sub>Module `ui` line 699</sub>
