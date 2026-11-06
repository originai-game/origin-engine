<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-textField"></a>

### `og.ui.textField`

One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

```ts
og.ui.textField(id: string, value: string, desc?: FieldDesc): FieldResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `FieldDesc` | yes | Placeholder, width and length limit. |

**Returns** `FieldResult`.

Binding symbol `og_ui_text_field` · publish builds: kept

```ogt
fn textField {
  doc "One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc FieldDesc "Placeholder, width and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 16401</sub>
