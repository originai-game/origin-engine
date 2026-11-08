<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StarBarDesc"></a>

### `StarBarDesc` (desc struct)

A level's score toward its stars.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | The score. |
| `max` | `number` | `0` | — | — | — | The score that fills the bar; 0 is the last threshold. |
| `thresholds` | `readonly number[]` | `[]` | — | — | — | The scores for each star, low to high; empty is a third, two thirds and all of max. |
| `label` | `string` | `""` | — | — | — | The caption over the score (Score); empty is none. |
| `valueText` | `string` | `""` | — | — | — | The score as written (18,450); empty writes value grouped. |
| `compact` | `boolean` | `false` | — | — | — | One line: the score beside the bar (a phone's top strip). |
| `starsLabel` | `string` | `"{n} of {max} stars"` | — | — | — | Its stars to a screen reader ({n} earned of {max}). |

```ogt
struct StarBarDesc {
  doc "A level's score toward its stars."
  desc true
  field value f32 "The score."
  field max f32 "The score that fills the bar; 0 is the last threshold."
  field thresholds List.f32 "The scores for each star, low to high; empty is a third, two thirds and all of max."
  field label String "The caption over the score (Score); empty is none."
  field valueText String "The score as written (18,450); empty writes value grouped."
  field compact bool "One line: the score beside the bar (a phone's top strip)."
  field starsLabel String "Its stars to a screen reader ({n} earned of {max})." {
    default "{n} of {max} stars"
  }
}
```

<sub>Module `ui` line 4231</sub>
