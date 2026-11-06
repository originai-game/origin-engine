<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-textArea"></a>

### `og.ui.textArea`

Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

```ts
og.ui.textArea(id: string, value: string, desc?: TextAreaDesc): FieldResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `TextAreaDesc` | yes | Placeholder, width, rows and length limit. |

**Returns** `FieldResult`.

Binding symbol `og_ui_text_area` · publish builds: kept

```ogt
fn textArea {
  doc "Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc TextAreaDesc "Placeholder, width, rows and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 16394</sub>
