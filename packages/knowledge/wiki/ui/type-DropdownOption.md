<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DropdownOption"></a>

### `DropdownOption` (desc struct)

One option of a dropdown.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its label. |
| `description` | `string` | `""` | — | — | — | A line under it in the list; empty for none. |
| `icon` | `string` | `""` | — | — | — | An icon before it; empty for none. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but cannot be chosen. |
| `group` | `string` | `""` | — | — | — | The heading it sits under; options of one group follow each other. |

```ogt
struct DropdownOption {
  doc "One option of a dropdown."
  desc true
  field label String "Its label."
  field description String "A line under it in the list; empty for none."
  field icon String "An icon before it; empty for none."
  field disabled bool "Shown but cannot be chosen."
  field group String "The heading it sits under; options of one group follow each other."
}
```

<sub>`api.ogt` line 19665</sub>
