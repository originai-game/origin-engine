<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HeroButtonDesc"></a>

### `HeroButtonDesc` (desc struct)

A call to action: tone, height, a keycap and whether it can be pressed now.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tone` | `HeroTone` | `"primary"` | — | — | — | Its material. |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its height: the control heights of the density, at least 44 CSS px on touch. |
| `disabled` | `boolean` | `false` | — | — | — | Not now (the enemy's turn): it lies flat and inert, its label in the faint ink. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the label. |
| `key` | `string` | `""` | — | — | — | The key or pad glyph of its action, in a keycap after the label (E, Space); empty shows none. |

```ogt
struct HeroButtonDesc {
  doc "A call to action: tone, height, a keycap and whether it can be pressed now."
  desc true
  field tone HeroTone "Its material."
  field size ControlSize "Its height: the control heights of the density, at least 44 CSS px on touch."
  field disabled bool "Not now (the enemy's turn): it lies flat and inert, its label in the faint ink."
  field width Option.Size "Its width; null fits the label."
  field key String "The key or pad glyph of its action, in a keycap after the label (E, Space); empty shows none."
}
```

<sub>Module `ui` line 4204</sub>
