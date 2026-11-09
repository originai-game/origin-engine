<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UserSelect"></a>

### `UserSelect` (enum)

Whether the player can select an element's text and copy it (CSS user-select, docs/ui.md §2).

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Its parent's; with none set above it, or under a control (a button's label is pressed, not selected), none. |
| `"none"` | Not selectable. |
| `"text"` | A press and drag selects characters, a double press a word, a triple press a line, a long press on a touch screen a word; Ctrl or Cmd+C copies. |
| `"all"` | A press selects its whole text at once. |

```ogt
enum UserSelect {
  doc "Whether the player can select an element's text and copy it (CSS user-select, docs/ui.md §2)."
  variant auto "Its parent's; with none set above it, or under a control (a button's label is pressed, not selected), none."
  variant none "Not selectable."
  variant text "A press and drag selects characters, a double press a word, a triple press a line, a long press on a touch screen a word; Ctrl or Cmd+C copies."
  variant all "A press selects its whole text at once."
}
```

<sub>Module `ui` line 1746</sub>
