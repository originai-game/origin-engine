<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextAreaDesc"></a>

### `TextAreaDesc` (desc struct)

A multi-line text area.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown, dimmed, while the area is empty. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `rows` | `number` | `0` | — | — | — | Lines shown at once; 0 is 4. More scroll. |
| `maxChars` | `number` | `0` | — | — | — | The most characters it takes, line breaks included; 0 is no limit. |
| `invalid` | `boolean` | `false` | — | — | — | Its text is wrong: its frame is dressed `Field.frame[invalid]`. |
| `bare` | `boolean` | `false` | — | — | — | No frame and no focus ring of its own: a component frames it. |
| `tight` | `boolean` | `false` | — | — | — | No inset and rows lines of its text's height (FieldDesc.tight). |
| `component` | `string \| null` | `null` | — | — | — | The component its style dresses it as (FieldDesc.component). Null: Field. |

```ogt
struct TextAreaDesc {
  doc "A multi-line text area."
  desc true
  field placeholder String "Shown, dimmed, while the area is empty."
  field width Option.Size "Its width; null grows."
  field rows u32 "Lines shown at once; 0 is 4. More scroll."
  field maxChars u32 "The most characters it takes, line breaks included; 0 is no limit."
  field invalid bool "Its text is wrong: its frame is dressed `Field.frame[invalid]`."
  field bare bool "No frame and no focus ring of its own: a component frames it."
  field tight bool "No inset and rows lines of its text's height (FieldDesc.tight)."
  field component Option.String "The component its style dresses it as (FieldDesc.component). Null: Field."
}
```

<sub>`api.ogt` line 16030</sub>
