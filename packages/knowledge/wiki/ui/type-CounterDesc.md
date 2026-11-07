<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CounterDesc"></a>

### `CounterDesc` (desc struct)

A number that counts to its value.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font: mono (the numeric role) for counts in a row, display for a big score. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `align` | `TextAlign` | `"start"` | — | — | — | How it aligns in its width. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a change takes to count; 0 is 900 ms. |
| `from` | `number \| null` | `null` | — | — | — | Where the first count starts; null starts at the value (no count). |
| `decimals` | `number` | `0` | — | — | — | Digits after the point. |
| `prefix` | `string` | `""` | — | — | — | Text before the number. |
| `suffix` | `string` | `""` | — | — | — | Text after the number. |

```ogt
struct CounterDesc {
  doc "A number that counts to its value."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field font FontRole "Which font: mono (the numeric role) for counts in a row, display for a big score."
  field color Option.Paint "The ink; null is ink."
  field align TextAlign "How it aligns in its width."
  field width Option.Size "Its width; null fits the text."
  field durationMs f32 "How long a change takes to count; 0 is 900 ms." {
    unit ms
    min 0
  }
  field from Option.f64 "Where the first count starts; null starts at the value (no count)."
  field decimals u32 "Digits after the point."
  field prefix String "Text before the number."
  field suffix String "Text after the number."
}
```

<sub>Module `ui` line 1802</sub>
