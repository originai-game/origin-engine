<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextRole"></a>

### `TextRole` (enum)

A text's role in the hierarchy (docs/ui.md §12.2): its type step, font and weight come from the style's type tokens for the role.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | Running text (the default). |
| `"display"` | The largest: a title screen's name, a victory banner. A heading. |
| `"title"` | A screen's or a dialog's title. A heading. |
| `"heading"` | A section's heading. A heading. |
| `"label"` | A control's or a field's name: one line, medium size. |
| `"caption"` | Secondary detail under something: a timestamp, a hint. |
| `"numeric"` | A number that changes (a score, a price, a timer): monospaced figures, so it never jiggles. |

```ogt
enum TextRole {
  doc "A text's role in the hierarchy (docs/ui.md §12.2): its type step, font and weight come from the style's type tokens for the role."
  variant body "Running text (the default)."
  variant display "The largest: a title screen's name, a victory banner. A heading."
  variant title "A screen's or a dialog's title. A heading."
  variant heading "A section's heading. A heading."
  variant label "A control's or a field's name: one line, medium size."
  variant caption "Secondary detail under something: a timestamp, a hint."
  variant numeric "A number that changes (a score, a price, a timer): monospaced figures, so it never jiggles."
}
```

<sub>`api.ogt` line 18911</sub>
