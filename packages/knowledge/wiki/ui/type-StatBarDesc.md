<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatBarDesc"></a>

### `StatBarDesc` (desc struct)

A health or resource bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | The value. |
| `max` | `number` | `100` | — | — | — | Its maximum. |
| `kind` | `MeterKind` | `"health"` | — | — | — | What it measures (its colour): health turns warn and danger as it falls. |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own instead of the kind's. |
| `segment` | `number` | `0` | ≥ 0 | — | — | Value per segment: the bar is blocks of this much (Overwatch's 25-point blocks); 0 is one bar. With style pips, the value per pip (0 is 1). |
| `shield` | `number` | `0` | ≥ 0 | — | — | Shield or armour on top of the value, drawn as a row of cells over the bar (Apex's armour). |
| `shieldMax` | `number` | `0` | ≥ 0 | — | — | The most shield; 0 hides the row. |
| `shieldCell` | `number` | `0` | ≥ 0 | — | — | Shield per cell; 0 is one continuous strip. |
| `shieldTier` | `Rarity` | `"none"` | — | — | — | The shield's tier colour (white, blue, purple, gold, red as Apex's); none is the info colour. |
| `heal` | `number` | `0` | ≥ 0 | — | — | Healing on its way, a pale preview past the fill. |
| `absorb` | `number` | `0` | ≥ 0 | — | — | An absorb on the health (World of Warcraft's, Diablo IV's): a white-blue overlay past the fill, laid back over its end when the bar is full. |
| `recoverable` | `number` | `0` | ≥ 0 | — | — | Damage that can still come back (a fighting game's grey health): drawn between the value and this. |
| `ghost` | `boolean` | `true` | — | — | — | A drop leaves a pale chunk that holds, then drains to the new value (the default). |
| `label` | `string` | `""` | — | — | — | A short name at its start (HP); empty is none. |
| `showValue` | `boolean` | `false` | — | — | — | The value as text at its end (72 / 100). |
| `valueInside` | `boolean` | `false` | — | — | — | The value inside the track at its end, over the fill, so the bar keeps its full length (a unit frame's health); the track grows to hold the text. |
| `valueText` | `string` | `""` | — | — | — | The value as text instead (72%); empty formats value / max. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas before it, as tall as the line of the text it stands beside at least (its label's, or iconStep's), never only the track's height. |
| `iconStep` | `TypeStepName \| null` | `null` | — | — | — | The type step of the text the icon stands beside, when that is a label of the game's outside the bar (a row's name at body): the icon is that step's line tall. Null is the bar's own label step. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 240 (grows with the surface). |
| `thickness` | `number` | `0` | ≥ 0 | `cssPx` | — | Its height; 0 is 14. |
| `low` | `number` | `0.3` | — | `ratio` | — | Below this fraction a health bar is low (warn). |
| `critical` | `number` | `0.15` | — | `ratio` | — | Below this fraction it is critical: danger, and it pulses. |
| `reverse` | `boolean` | `false` | — | — | — | Fills from the end (the right player's bar in a fighting game). |
| `style` | `StatBarStyle` | `"flat"` | — | — | — | Its shape. |
| `anchor` | `Anchor \| null` | `null` | — | — | — | Opens its own anchored group there; null builds it in place. |
| `name` | `string` | `""` | — | — | — | What a screen reader calls it; empty is the label, else Health. |
| `bare` | `boolean` | `false` | — | — | — | Draw no plate: by default the bar sits on a soft HUD plate, so it reads over a bright sky (the plate is the group's when it already has one). |

```ogt
struct StatBarDesc {
  doc "A health or resource bar."
  desc true
  field value f32 "The value."
  field max f32 "Its maximum." {
    default 100
  }
  field kind MeterKind "What it measures (its colour): health turns warn and danger as it falls."
  field color Option.Paint "A colour of its own instead of the kind's."
  field segment f32 "Value per segment: the bar is blocks of this much (Overwatch's 25-point blocks); 0 is one bar. With style pips, the value per pip (0 is 1)." {
    min 0
  }
  field shield f32 "Shield or armour on top of the value, drawn as a row of cells over the bar (Apex's armour)." {
    min 0
  }
  field shieldMax f32 "The most shield; 0 hides the row." {
    min 0
  }
  field shieldCell f32 "Shield per cell; 0 is one continuous strip." {
    min 0
  }
  field shieldTier Rarity "The shield's tier colour (white, blue, purple, gold, red as Apex's); none is the info colour."
  field heal f32 "Healing on its way, a pale preview past the fill." {
    min 0
  }
  field absorb f32 "An absorb on the health (World of Warcraft's, Diablo IV's): a white-blue overlay past the fill, laid back over its end when the bar is full." {
    min 0
  }
  field recoverable f32 "Damage that can still come back (a fighting game's grey health): drawn between the value and this." {
    min 0
  }
  field ghost bool "A drop leaves a pale chunk that holds, then drains to the new value (the default)." {
    default true
  }
  field label String "A short name at its start (HP); empty is none."
  field showValue bool "The value as text at its end (72 / 100)."
  field valueInside bool "The value inside the track at its end, over the fill, so the bar keeps its full length (a unit frame's health); the track grows to hold the text."
  field valueText String "The value as text instead (72%); empty formats value / max."
  field icon String "An icon of the atlas before it, as tall as the line of the text it stands beside at least (its label's, or iconStep's), never only the track's height."
  field iconStep Option.TypeStepName "The type step of the text the icon stands beside, when that is a label of the game's outside the bar (a row's name at body): the icon is that step's line tall. Null is the bar's own label step."
  field width Option.Size "Its width; null is 240 (grows with the surface)."
  field thickness f32 "Its height; 0 is 14." {
    min 0
    unit cssPx
  }
  field low f32 "Below this fraction a health bar is low (warn)." {
    default 0.3
    unit ratio
  }
  field critical f32 "Below this fraction it is critical: danger, and it pulses." {
    default 0.15
    unit ratio
  }
  field reverse bool "Fills from the end (the right player's bar in a fighting game)."
  field style StatBarStyle "Its shape."
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place."
  field name String "What a screen reader calls it; empty is the label, else Health."
  field bare bool "Draw no plate: by default the bar sits on a soft HUD plate, so it reads over a bright sky (the plate is the group's when it already has one)."
}
```

<sub>`api.ogt` line 21654</sub>
