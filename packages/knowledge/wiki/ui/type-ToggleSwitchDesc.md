<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ToggleSwitchDesc"></a>

### `ToggleSwitchDesc` (desc struct)

A switch: its label, description and states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its label (the whole row takes the press). |
| `description` | `string` | `""` | — | — | — | A line under the label; empty for none. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `side` | `LabelSide` | `"start"` | — | — | — | Where the label sits. |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its size. |
| `width` | `Size \| null` | `null` | — | — | — | The row's width; null fits (a settings row grows: pass grow). |
| `onLabel` | `string` | `""` | — | — | — | What a screen reader and the probe say when on (`On`); empty for none. |
| `offLabel` | `string` | `""` | — | — | — | And when off. |

```ogt
struct ToggleSwitchDesc {
  doc "A switch: its label, description and states."
  desc true
  field label String "Its label (the whole row takes the press)."
  field description String "A line under the label; empty for none."
  field disabled bool "Shown but inert."
  field side LabelSide "Where the label sits."
  field size ControlSize "Its size."
  field width Option.Size "The row's width; null fits (a settings row grows: pass grow)."
  field onLabel String "What a screen reader and the probe say when on (`On`); empty for none."
  field offLabel String "And when off."
}
```

<sub>Module `ui` line 3600</sub>
