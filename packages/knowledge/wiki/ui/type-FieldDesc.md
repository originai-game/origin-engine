<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FieldDesc"></a>

### `FieldDesc` (desc struct)

A one-line text field.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown, dimmed, while the field is empty. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `maxChars` | `number` | `0` | — | — | — | The most characters it takes; 0 is no limit. |
| `invalid` | `boolean` | `false` | — | — | — | Its text is wrong (a form's error): its frame is dressed `Field.frame[invalid]` and, undressed, takes the danger outline. |
| `bare` | `boolean` | `false` | — | — | — | No frame and no focus ring of its own, a small inset: a component frames it (an input with an icon and a clear button) and shows its focus. |
| `tight` | `boolean` | `false` | — | — | — | No inset and the height of its text's line (on a touch screen at least the 44 CSS px touch target, the line centred): the text sits exactly where the field is laid, in a box the caller draws or on a line of a page. With or without bare. |
| `component` | `string \| null` | `null` | — | — | — | The component its style dresses it as (`Poem`, `Search[large]`): its frame `<component>.frame`, its text `<component>.text` and its placeholder `<component>.placeholder`, in its states (focused, invalid, disabled); `ink` and `step` on the text parts set the text's colour and type step. Null: Field. |

```ogt
struct FieldDesc {
  doc "A one-line text field."
  desc true
  field placeholder String "Shown, dimmed, while the field is empty."
  field width Option.Size "Its width; null grows."
  field maxChars u32 "The most characters it takes; 0 is no limit."
  field invalid bool "Its text is wrong (a form's error): its frame is dressed `Field.frame[invalid]` and, undressed, takes the danger outline."
  field bare bool "No frame and no focus ring of its own, a small inset: a component frames it (an input with an icon and a clear button) and shows its focus."
  field tight bool "No inset and the height of its text's line (on a touch screen at least the 44 CSS px touch target, the line centred): the text sits exactly where the field is laid, in a box the caller draws or on a line of a page. With or without bare."
  field component Option.String "The component its style dresses it as (`Poem`, `Search[large]`): its frame `<component>.frame`, its text `<component>.text` and its placeholder `<component>.placeholder`, in its states (focused, invalid, disabled); `ink` and `step` on the text parts set the text's colour and type step. Null: Field."
}
```

<sub>Module `ui` line 360</sub>
