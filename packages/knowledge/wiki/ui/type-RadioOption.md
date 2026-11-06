<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadioOption"></a>

### `RadioOption` (desc struct)

One choice of a radio group.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its label. |
| `description` | `string` | `""` | — | — | — | A line under it; empty for none. |
| `icon` | `string` | `""` | — | — | — | An icon before it (cards); empty for none. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but cannot be chosen. |

```ogt
struct RadioOption {
  doc "One choice of a radio group."
  desc true
  field label String "Its label."
  field description String "A line under it; empty for none."
  field icon String "An icon before it (cards); empty for none."
  field disabled bool "Shown but cannot be chosen."
}
```

<sub>`api.ogt` line 19437</sub>
