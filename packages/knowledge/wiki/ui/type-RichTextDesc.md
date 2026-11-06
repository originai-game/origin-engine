<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RichTextDesc"></a>

### `RichTextDesc` (desc struct)

How a rich text paragraph draws: its defaults, which its markup's spans override.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `align` | `TextAlign` | `"start"` | — | — | — | How the lines align. |
| `nowrap` | `boolean` | `false` | — | — | — | Keep to one line (breaking only at `\n`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font. |

```ogt
struct RichTextDesc {
  doc "How a rich text paragraph draws: its defaults, which its markup's spans override."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is ink."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
}
```

<sub>`api.ogt` line 16327</sub>
