<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextEffectsDesc"></a>

### `TextEffectsDesc` (desc struct)

Effects of the text an element holds (its labels and rich text).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shadows` | `readonly TextShadowDesc[]` | `[]` | — | — | — | Shadows, under everything, in order. |
| `outline` | `TextOutlineDesc \| null` | `null` | — | — | — | An outline. |
| `fill` | `Gradient \| null` | `null` | — | — | — | A gradient fill over each line's box instead of its colour. |

```ogt
struct TextEffectsDesc {
  doc "Effects of the text an element holds (its labels and rich text)."
  desc true
  field shadows List.TextShadowDesc "Shadows, under everything, in order."
  field outline Option.TextOutlineDesc "An outline."
  field fill Option.Gradient "A gradient fill over each line's box instead of its colour."
}
```

<sub>Module `ui` line 2824</sub>
