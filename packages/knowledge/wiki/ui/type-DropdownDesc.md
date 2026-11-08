<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DropdownDesc"></a>

### `DropdownDesc` (desc struct)

A dropdown: its options, label and states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `options` | `readonly DropdownOption[]` | `[]` | — | — | — | The options. |
| `selected` | `number` | `-1` | — | — | — | The chosen one from game state; -1 shows the placeholder. |
| `label` | `string` | `""` | — | — | — | Its name, above it; empty for none. |
| `placeholder` | `string` | `""` | — | — | — | Shown while nothing is chosen; empty is `Select…`. |
| `helper` | `string` | `""` | — | — | — | A line under it; empty for none. |
| `error` | `string` | `""` | — | — | — | An error under it (and the danger frame); empty for none. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `searchable` | `boolean \| null` | `null` | — | — | — | A filter field at the list's top; null shows it past 10 options. |
| `searchPlaceholder` | `string` | `""` | — | — | — | The filter's placeholder; empty is `Filter`. |
| `maxHeight` | `number` | `320` | — | `cssPx` | — | The list's most height before it scrolls. |

```ogt
struct DropdownDesc {
  doc "A dropdown: its options, label and states."
  desc true
  field options List.DropdownOption "The options."
  field selected i32 "The chosen one from game state; -1 shows the placeholder." {
    default -1
  }
  field label String "Its name, above it; empty for none."
  field placeholder String "Shown while nothing is chosen; empty is `Select…`."
  field helper String "A line under it; empty for none."
  field error String "An error under it (and the danger frame); empty for none."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
  field searchable Option.bool "A filter field at the list's top; null shows it past 10 options."
  field searchPlaceholder String "The filter's placeholder; empty is `Filter`."
  field maxHeight f32 "The list's most height before it scrolls." {
    default 320
    unit cssPx
  }
}
```

<sub>Module `ui` line 2060</sub>
