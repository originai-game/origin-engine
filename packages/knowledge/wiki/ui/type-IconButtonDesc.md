<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-IconButtonDesc"></a>

### `IconButtonDesc` (desc struct)

An icon button: an icon, its name and a badge.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name: read by a screen reader and shown as its tooltip (an icon alone is not a name). |
| `variant` | `ButtonVariant` | `"quiet"` | — | — | — | Its role; quiet (the default for an icon) draws only the icon until hovered. |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its size: a square of the control height. |
| `badge` | `string` | `""` | — | — | — | A count over its corner; empty for none. |
| `dot` | `boolean` | `false` | — | — | — | A dot over its corner (something new), when there is no badge. |
| `selected` | `boolean` | `false` | — | — | — | On (a toggled tool). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `round` | `boolean` | `false` | — | — | — | A circle rather than the style's control corner. |

```ogt
struct IconButtonDesc {
  doc "An icon button: an icon, its name and a badge."
  desc true
  field label String "Its name: read by a screen reader and shown as its tooltip (an icon alone is not a name)."
  field variant ButtonVariant "Its role; quiet (the default for an icon) draws only the icon until hovered." {
    default quiet
  }
  field size ControlSize "Its size: a square of the control height."
  field badge String "A count over its corner; empty for none."
  field dot bool "A dot over its corner (something new), when there is no badge."
  field selected bool "On (a toggled tool)."
  field disabled bool "Shown but inert."
  field round bool "A circle rather than the style's control corner."
}
```

<sub>`api.ogt` line 19302</sub>
