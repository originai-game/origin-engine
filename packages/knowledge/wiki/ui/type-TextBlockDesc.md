<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextBlockDesc"></a>

### `TextBlockDesc` (desc struct)

How a text block reads: its role, tone and how it fits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `role` | `TextRole` | `"body"` | — | — | — | Its role: the type step, font and whether it is a heading. |
| `tone` | `TextTone` | `"normal"` | — | — | — | Its ink by meaning. |
| `align` | `TextAlign` | `"start"` | — | — | — | How its lines align (start and end follow the writing direction). |
| `maxLines` | `number` | `0` | — | — | — | The most lines it shows, cut with an ellipsis after (0: all). |
| `nowrap` | `boolean` | `false` | — | — | — | One line; with ellipsis it shrinks in a row and cuts. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text (a centred or end-aligned text in a column takes the column). |
| `step` | `TypeStepName \| null` | `null` | — | — | — | A type step over the role's (a role's size, not its kind, changed). |

```ogt
struct TextBlockDesc {
  doc "How a text block reads: its role, tone and how it fits."
  desc true
  field role TextRole "Its role: the type step, font and whether it is a heading."
  field tone TextTone "Its ink by meaning."
  field align TextAlign "How its lines align (start and end follow the writing direction)."
  field maxLines u32 "The most lines it shows, cut with an ellipsis after (0: all)."
  field nowrap bool "One line; with ellipsis it shrinks in a row and cuts."
  field width Option.Size "Its width; null fits the text (a centred or end-aligned text in a column takes the column)."
  field step Option.TypeStepName "A type step over the role's (a role's size, not its kind, changed)."
}
```

<sub>Module `ui` line 3182</sub>
