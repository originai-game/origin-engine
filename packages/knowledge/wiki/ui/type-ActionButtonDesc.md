<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ActionButtonDesc"></a>

### `ActionButtonDesc` (desc struct)

A touch action button (jump, fire, a skill): a disc with a rim (part ActionButton.face, flagged primary, pressed and disabled, which a style dresses), its icon on it; a label wider than it with no icon widens it to a capsule instead. A further touch holds it only when it went down on it, so a thumb steering a stick never fires it by sliding over.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 64 (never under 44). |
| `primary` | `boolean` | `false` | — | — | — | The primary button (one per screen). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas by name, instead of the label; empty is the label. |
| `customize` | `Customize \| null` | `null` | — | — | — | The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit). |
| `round` | `boolean` | `false` | — | — | — | Kept a disc when its label is wider than it (it grows as a disc, a pedal) instead of widening to a capsule; a button with an icon, a short label or a charge is a disc anyway. |
| `charge` | `number \| null` | `null` | — | — | — | A ring inside its edge, lit clockwise from the top to this (0 to 1: a boost's charge, a brake's pressure), with a faint track for the rest. It follows the pad's outline, so a pad with a charge is drawn round. null draws no ring. |
| `chargeTone` | `Tone` | `"neutral"` | — | — | — | The lit ring's tone; neutral is the primary's. |
| `disabled` | `boolean` | `false` | — | — | — | It answers nothing and shows faded (out of ammunition, not yet learnt). |

```ogt
struct ActionButtonDesc {
  doc "A touch action button (jump, fire, a skill): a disc with a rim (part ActionButton.face, flagged primary, pressed and disabled, which a style dresses), its icon on it; a label wider than it with no icon widens it to a capsule instead. A further touch holds it only when it went down on it, so a thumb steering a stick never fires it by sliding over."
  desc true
  field size f32 "Its side; 0 is 64 (never under 44)." {
    unit cssPx
    min 0
  }
  field primary bool "The primary button (one per screen)."
  field icon String "An icon of the atlas by name, instead of the label; empty is the label."
  field customize Option.Customize "The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit)."
  field round bool "Kept a disc when its label is wider than it (it grows as a disc, a pedal) instead of widening to a capsule; a button with an icon, a short label or a charge is a disc anyway."
  field charge Option.f32 "A ring inside its edge, lit clockwise from the top to this (0 to 1: a boost's charge, a brake's pressure), with a faint track for the rest. It follows the pad's outline, so a pad with a charge is drawn round. null draws no ring."
  field chargeTone Tone "The lit ring's tone; neutral is the primary's."
  field disabled bool "It answers nothing and shows faded (out of ammunition, not yet learnt)."
}
```

<sub>Module `ui` line 1123</sub>
