<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-InputAreaDesc"></a>

### `InputAreaDesc` (desc struct)

A text area with its label, helper, error and counter.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name, above it; empty for none. |
| `placeholder` | `string` | `""` | — | — | — | Shown while it is empty. |
| `helper` | `string` | `""` | — | — | — | A line under it; empty for none. |
| `error` | `string` | `""` | — | — | — | An error under it; empty for none. |
| `rows` | `number` | `4` | — | — | — | Lines shown at once (more scroll). |
| `maxChars` | `number` | `0` | — | — | — | The most characters (0: no limit). |
| `counter` | `boolean` | `false` | — | — | — | Show the count under it. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |

```ogt
struct InputAreaDesc {
  doc "A text area with its label, helper, error and counter."
  desc true
  field label String "Its name, above it; empty for none."
  field placeholder String "Shown while it is empty."
  field helper String "A line under it; empty for none."
  field error String "An error under it; empty for none."
  field rows u32 "Lines shown at once (more scroll)." {
    default 4
  }
  field maxChars u32 "The most characters (0: no limit)."
  field counter bool "Show the count under it."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
}
```

<sub>Module `ui` line 3991</sub>
