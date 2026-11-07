<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NumberFormatDesc"></a>

### `NumberFormatDesc` (desc struct)

How og.ui.formatNumber and the counters write a number.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `notation` | `Notation` | `"grouped"` | — | — | — | The notation. |
| `decimals` | `number \| null` | `null` | — | — | — | Digits after the point; null is none below 10,000 and three significant digits in the shortened forms. |
| `exponent` | `number` | `0` | — | — | — | The value is value × 10^exponent: numbers past 1e308 (an idle game's 1.5e420 is value 1.5, exponent 420). |
| `sign` | `boolean` | `false` | — | — | — | A + before positive values. |
| `prefix` | `string` | `""` | — | — | — | Text before it ($). |
| `suffix` | `string` | `""` | — | — | — | Text after it ( XP, %). |
| `group` | `string` | `","` | — | — | — | The thousands separator. |
| `point` | `string` | `"."` | — | — | — | The decimal point. |

```ogt
struct NumberFormatDesc {
  doc "How og.ui.formatNumber and the counters write a number."
  desc true
  field notation Notation "The notation."
  field decimals Option.u32 "Digits after the point; null is none below 10,000 and three significant digits in the shortened forms."
  field exponent i32 "The value is value × 10^exponent: numbers past 1e308 (an idle game's 1.5e420 is value 1.5, exponent 420)."
  field sign bool "A + before positive values."
  field prefix String "Text before it ($)."
  field suffix String "Text after it ( XP, %)."
  field group String "The thousands separator." {
    default ","
  }
  field point String "The decimal point." {
    default "."
  }
}
```

<sub>Module `ui` line 5607</sub>
