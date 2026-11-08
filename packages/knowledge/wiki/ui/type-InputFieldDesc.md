<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-InputFieldDesc"></a>

### `InputFieldDesc` (desc struct)

A text field with its label, helper, error and adornments.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name, above it (a screen reader names the field by it); empty for none. |
| `placeholder` | `string` | `""` | — | — | — | Shown while it is empty. |
| `helper` | `string` | `""` | — | — | — | A line under it; empty for none. |
| `error` | `string` | `""` | — | — | — | An error under it, the danger frame, the validity group's invalid (a shake); empty for none. |
| `required` | `boolean` | `false` | — | — | — | A mark after the label. |
| `icon` | `string` | `""` | — | — | — | An icon at its start; empty for none. |
| `clearable` | `boolean` | `true` | — | — | — | A clear button at its end while it has text. |
| `maxChars` | `number` | `0` | — | — | — | The most characters it takes (0: no limit); with counter, `12/20` under it. |
| `counter` | `boolean` | `false` | — | — | — | Show the count under it (with maxChars, against the limit). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `suffix` | `string` | `""` | — | — | — | Text at its end (`gold`, `.com`); empty for none. |

```ogt
struct InputFieldDesc {
  doc "A text field with its label, helper, error and adornments."
  desc true
  field label String "Its name, above it (a screen reader names the field by it); empty for none."
  field placeholder String "Shown while it is empty."
  field helper String "A line under it; empty for none."
  field error String "An error under it, the danger frame, the validity group's invalid (a shake); empty for none."
  field required bool "A mark after the label."
  field icon String "An icon at its start; empty for none."
  field clearable bool "A clear button at its end while it has text." {
    default true
  }
  field maxChars u32 "The most characters it takes (0: no limit); with counter, `12/20` under it."
  field counter bool "Show the count under it (with maxChars, against the limit)."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
  field suffix String "Text at its end (`gold`, `.com`); empty for none."
}
```

<sub>Module `ui` line 3252</sub>
