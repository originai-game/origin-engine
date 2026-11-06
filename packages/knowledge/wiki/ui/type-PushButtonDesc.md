<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PushButtonDesc"></a>

### `PushButtonDesc` (desc struct)

A button: its role, size, icons, badge and states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `variant` | `ButtonVariant` | `"secondary"` | — | — | — | Its role: primary (one per screen), secondary, ghost, quiet or danger. |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its height (the style's control sizes; at least 44 CSS px of reach on a coarse pointer). |
| `icon` | `string` | `""` | — | — | — | An icon before the label (the style's set); empty for none. |
| `iconEnd` | `string` | `""` | — | — | — | An icon after it (a chevron, an external mark); empty for none. |
| `badge` | `string` | `""` | — | — | — | A count or word on its top end corner (`3`, `NEW`); empty for none. |
| `loading` | `boolean` | `false` | — | — | — | Working: a spinner takes the icon's place, it takes no press and says busy. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert, dimmed. |
| `disabledReason` | `string` | `""` | — | — | — | Why it is disabled, shown as its tooltip and read after its name (`Needs 300 gold`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the label. |
| `selected` | `boolean` | `false` | — | — | — | On (a toggle button: a filter, a pinned tool): its selection group, filled as pressed. |
| `autofocus` | `boolean` | `false` | — | — | — | Take the focus when it appears and no dialog is open. |
| `tooltip` | `string` | `""` | — | — | — | A tooltip once the pointer rests on it; empty for none. |

```ogt
struct PushButtonDesc {
  doc "A button: its role, size, icons, badge and states."
  desc true
  field variant ButtonVariant "Its role: primary (one per screen), secondary, ghost, quiet or danger."
  field size ControlSize "Its height (the style's control sizes; at least 44 CSS px of reach on a coarse pointer)."
  field icon String "An icon before the label (the style's set); empty for none."
  field iconEnd String "An icon after it (a chevron, an external mark); empty for none."
  field badge String "A count or word on its top end corner (`3`, `NEW`); empty for none."
  field loading bool "Working: a spinner takes the icon's place, it takes no press and says busy."
  field disabled bool "Shown but inert, dimmed."
  field disabledReason String "Why it is disabled, shown as its tooltip and read after its name (`Needs 300 gold`)."
  field width Option.Size "Its width; null fits the label."
  field selected bool "On (a toggle button: a filter, a pinned tool): its selection group, filled as pressed."
  field autofocus bool "Take the focus when it appears and no dialog is open."
  field tooltip String "A tooltip once the pointer rests on it; empty for none."
}
```

<sub>`api.ogt` line 19298</sub>
