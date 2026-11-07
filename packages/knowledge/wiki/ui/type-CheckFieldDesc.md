<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CheckFieldDesc"></a>

### `CheckFieldDesc` (desc struct)

A checkbox: its label, description and states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its label (the whole row takes the press). |
| `description` | `string` | `""` | — | — | — | A line under the label; empty for none. |
| `mixed` | `boolean` | `false` | — | — | — | Neither: some of what it stands for is on (a group's parent). A press turns it on. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `invalid` | `boolean` | `false` | — | — | — | It must be checked (terms to accept): danger outline and the validity group. |
| `side` | `LabelSide` | `"end"` | — | — | — | Where the label sits. |
| `width` | `Size \| null` | `null` | — | — | — | The row's width; null fits. |

```ogt
struct CheckFieldDesc {
  doc "A checkbox: its label, description and states."
  desc true
  field label String "Its label (the whole row takes the press)."
  field description String "A line under the label; empty for none."
  field mixed bool "Neither: some of what it stands for is on (a group's parent). A press turns it on."
  field disabled bool "Shown but inert."
  field invalid bool "It must be checked (terms to accept): danger outline and the validity group."
  field side LabelSide "Where the label sits." {
    default end
  }
  field width Option.Size "The row's width; null fits."
}
```

<sub>Module `ui` line 3629</sub>
