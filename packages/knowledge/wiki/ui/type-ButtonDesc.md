<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ButtonDesc"></a>

### `ButtonDesc` (desc struct)

A button's look.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `variant` | `ButtonVariant` | `"secondary"` | — | — | — | The look. |
| `size` | `ControlSize` | `"medium"` | — | — | — | The control height. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the label. |
| `autofocus` | `boolean` | `false` | — | — | — | Take the focus when it appears (it was not on screen the tick before) and no dialog is open: what opens takes the focus, so a menu's first item answers Enter and the pad's A from its first tick. |
| `square` | `boolean` | `false` | — | — | — | A square of the control height with no side padding: an icon button or a one-glyph label (`+`). Overrides width. |

```ogt
struct ButtonDesc {
  doc "A button's look."
  desc true
  field variant ButtonVariant "The look."
  field size ControlSize "The control height."
  field disabled bool "Neutral and inert."
  field width Option.Size "Its width; null fits the label."
  field autofocus bool "Take the focus when it appears (it was not on screen the tick before) and no dialog is open: what opens takes the focus, so a menu's first item answers Enter and the pad's A from its first tick."
  field square bool "A square of the control height with no side padding: an icon button or a one-glyph label (`+`). Overrides width."
}
```

<sub>`api.ogt` line 15904</sub>
