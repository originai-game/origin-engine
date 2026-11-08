<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-label"></a>

### `og.ui.label`

MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text.

```ts
og.ui.label(id: string, text: string, desc?: LabelDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The text. |
| `desc` | `LabelDesc` | yes | Type step, ink, alignment, wrapping, width and font. |

**Returns** `void`.

Binding symbol `og_ui_label` · publish builds: kept

```ogt
fn label {
  doc "MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text."
  param id String "Its id."
  param text String "The text."
  param desc LabelDesc "Type step, ink, alignment, wrapping, width and font."
  returns Void
}
```

<sub>Module `ui` line 649</sub>
