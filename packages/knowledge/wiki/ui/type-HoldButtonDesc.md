<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HoldButtonDesc"></a>

### `HoldButtonDesc` (desc struct)

A hold-to-confirm button.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `variant` | `ButtonVariant` | `"danger"` | — | — | — | Its role (danger for a destructive confirm). |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its height. |
| `holdMs` | `number` | `1200` | ≥ 100 | `ms` | — | How long the hold takes. |
| `icon` | `string` | `""` | — | — | — | An icon before the label. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the label. |
| `toggle` | `boolean` | `false` | — | — | — | The hold's toggle alternative for the pointer and touch too (XAG 107): a tap starts the fill, which runs on by itself; a second tap stops it. Keys and pads always work so (Enter, Space or A start it, again stops it, moving the focus away stops it). |
| `holdingLabel` | `string` | `""` | — | — | — | What it says while it fills (`Keep holding…`); empty keeps its label. |
| `hint` | `string` | `""` | — | — | — | A line under it, what to do (`Hold to delete`); empty for none. |

```ogt
struct HoldButtonDesc {
  doc "A hold-to-confirm button."
  desc true
  field variant ButtonVariant "Its role (danger for a destructive confirm)." {
    default danger
  }
  field size ControlSize "Its height."
  field holdMs f32 "How long the hold takes." {
    default 1200
    unit ms
    min 100
  }
  field icon String "An icon before the label."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null fits the label."
  field toggle bool "The hold's toggle alternative for the pointer and touch too (XAG 107): a tap starts the fill, which runs on by itself; a second tap stops it. Keys and pads always work so (Enter, Space or A start it, again stops it, moving the focus away stops it)."
  field holdingLabel String "What it says while it fills (`Keep holding…`); empty keeps its label."
  field hint String "A line under it, what to do (`Hold to delete`); empty for none."
}
```

<sub>Module `ui` line 3455</sub>
