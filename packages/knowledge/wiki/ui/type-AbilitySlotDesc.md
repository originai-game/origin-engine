<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AbilitySlotDesc"></a>

### `AbilitySlotDesc` (desc struct)

An ability, item or spell on a bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key in an ability bar (and its probe id there). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas. |
| `image` | `Texture \| null` | `null` | — | — | — | Its art (art kind ability, 1:1), filling the slot edge to edge; null draws the icon large on a gradient in a hue of its own. |
| `name` | `string` | `""` | — | — | — | What a screen reader calls it (Fireball). |
| `action` | `string` | `""` | — | — | — | The action whose glyph the hotkey cap shows on the device used last (og.ui.frameGlyph: a gameplay action's binding, else a UI action's), so it follows a rebinding and the pad the player picks up; a pad's button draws as its icon where the style has one. Empty is none. |
| `hotkey` | `string` | `""` | — | — | — | A label for the cap instead of the action's glyph (Q, LB, 1); empty shows the action's. |
| `cooldown` | `number` | `0` | ≥ 0 | `s` | — | Seconds of cooldown left; 0 is ready. |
| `cooldownMax` | `number` | `0` | ≥ 0 | `s` | — | The cooldown's length; the sweep shows cooldown over it. |
| `charges` | `number` | `0` | — | — | — | Charges ready (a count at its corner when maxCharges is over 1). |
| `maxCharges` | `number` | `0` | — | — | — | Charges it holds; 0 or 1 is none. |
| `chargeProgress` | `number` | `0` | — | `ratio` | — | How far the next charge has come, 0 to 1. |
| `cost` | `string` | `""` | — | — | — | Its cost (40): a small figure with a droplet at the bottom left corner while the slot is ready; empty is none. |
| `unaffordable` | `boolean` | `false` | — | — | — | Not enough resource: the art tinted red and the cost in danger. |
| `outOfRange` | `boolean` | `false` | — | — | — | The target is out of range: the art tinted red (the hotkey keeps its one quiet ink). |
| `active` | `boolean` | `false` | — | — | — | Toggled on or channelling: a soft glow in the primary colour that breathes and a thin edge of it. |
| `locked` | `boolean` | `false` | — | — | — | Not learnt yet: the art greyed to about a third under a filled lock. |
| `disabled` | `boolean` | `false` | — | — | — | Cannot be used now (silenced, rooted, wrong stance): the art washed out under a red slash. |
| `ultimate` | `number \| null` | `null` | — | `ratio` | — | An ultimate's charge 0 to 1: the slot is 1.15 times the others, a gold ring fills clockwise round its art with a small percent at the bottom; ready, a gold edge and glow breathe. |
| `level` | `number` | `0` | — | — | — | Ranks learnt (pips right under it). |
| `maxLevel` | `number` | `0` | — | — | — | Ranks it has; 0 draws no pips. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 56 (48 on a phone). |

```ogt
struct AbilitySlotDesc {
  doc "An ability, item or spell on a bar."
  desc true
  field key String "Its key in an ability bar (and its probe id there)."
  field icon String "An icon of the atlas."
  field image Option.Texture "Its art (art kind ability, 1:1), filling the slot edge to edge; null draws the icon large on a gradient in a hue of its own."
  field name String "What a screen reader calls it (Fireball)."
  field action String "The action whose glyph the hotkey cap shows on the device used last (og.ui.frameGlyph: a gameplay action's binding, else a UI action's), so it follows a rebinding and the pad the player picks up; a pad's button draws as its icon where the style has one. Empty is none."
  field hotkey String "A label for the cap instead of the action's glyph (Q, LB, 1); empty shows the action's."
  field cooldown f32 "Seconds of cooldown left; 0 is ready." {
    min 0
    unit s
  }
  field cooldownMax f32 "The cooldown's length; the sweep shows cooldown over it." {
    min 0
    unit s
  }
  field charges u32 "Charges ready (a count at its corner when maxCharges is over 1)."
  field maxCharges u32 "Charges it holds; 0 or 1 is none."
  field chargeProgress f32 "How far the next charge has come, 0 to 1." {
    unit ratio
  }
  field cost String "Its cost (40): a small figure with a droplet at the bottom left corner while the slot is ready; empty is none."
  field unaffordable bool "Not enough resource: the art tinted red and the cost in danger."
  field outOfRange bool "The target is out of range: the art tinted red (the hotkey keeps its one quiet ink)."
  field active bool "Toggled on or channelling: a soft glow in the primary colour that breathes and a thin edge of it."
  field locked bool "Not learnt yet: the art greyed to about a third under a filled lock."
  field disabled bool "Cannot be used now (silenced, rooted, wrong stance): the art washed out under a red slash."
  field ultimate Option.f32 "An ultimate's charge 0 to 1: the slot is 1.15 times the others, a gold ring fills clockwise round its art with a small percent at the bottom; ready, a gold edge and glow breathe." {
    unit ratio
  }
  field level u32 "Ranks learnt (pips right under it)."
  field maxLevel u32 "Ranks it has; 0 draws no pips."
  field size f32 "Its side; 0 is 56 (48 on a phone)." {
    min 0
    unit cssPx
  }
}
```

<sub>`api.ogt` line 21770</sub>
