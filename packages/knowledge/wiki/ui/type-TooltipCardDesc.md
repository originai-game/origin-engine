<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TooltipCardDesc"></a>

### `TooltipCardDesc` (desc struct)

A rich tooltip card beside its trigger.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | The trigger is hovered or focused (its Response's hovered or focused): it shows after delayMs and stays while open, while pinned, or while the pointer is over it. |
| `target` | `string` | `""` | — | — | — | The trigger's path; a bare id names an element of the open container (built before this call). Empty places it at the pointer's last trigger rectangle of this tooltip. |
| `title` | `string` | `""` | — | — | — | The title. |
| `subtitle` | `string` | `""` | — | — | — | A line under it (Legendary sword, Passive). |
| `rarity` | `Rarity` | `"none"` | — | — | — | The title's colour and pips. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas beside the title. |
| `image` | `Texture \| null` | `null` | — | — | — | An art slot instead of the icon (item or ability icon, 1:1). |
| `body` | `string` | `""` | — | — | — | The body: rich text markup (og.ui.richText), its `[link=key]keyword[/link]` spans opening nested tooltips from links. |
| `stats` | `readonly TooltipStat[]` | `[]` | — | — | — | Stat rows. |
| `compareTitle` | `string` | `""` | — | — | — | With comparison deltas: the heading over a second card of what is equipped (Equipped); empty draws no second card. |
| `compareStats` | `readonly TooltipStat[]` | `[]` | — | — | — | The equipped item's stats, in the second card. |
| `footer` | `string` | `""` | — | — | — | A last line (Sell for 120, a flavour quote) in the muted ink. |
| `hints` | `readonly string[]` | `[]` | — | — | — | Action hints (Right-click to equip), each with its action's glyph when one is named [action:equip]. |
| `links` | `readonly TooltipLink[]` | `[]` | — | — | — | What the body's links open. |
| `delayMs` | `number` | `0` | ≥ 0 | `ms` | — | Rest before it shows; 0 is the style's slow duration. |
| `pinnable` | `boolean` | `true` | — | — | — | It locks open: after pinMs of rest a ring fills on its corner and it stays when the pointer leaves the trigger, until the pointer leaves the card (as Paradox's games do); the pin action (middle mouse, T, a pad's Y) pins at once. Nested tooltips are always pinnable. |
| `pinMs` | `number` | `0` | ≥ 0 | `ms` | — | Rest before it pins; 0 is 900. |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | Its width; 0 is 320 (300 on a phone). |
| `placement` | `Anchor` | `"right"` | — | — | — | Where it prefers to sit against the trigger: right (the end side), left, top or bottom. It sits beside the whole group the trigger is in (the panel or column, so it never covers its own trigger's panel) when there is room on either side of it, else beside the trigger; it flips and clamps to stay inside the safe area. |
| `pinLabel` | `string` | `"Pinned"` | — | — | — | The pin hint under a pinned card. |

```ogt
struct TooltipCardDesc {
  doc "A rich tooltip card beside its trigger."
  desc true
  field open bool "The trigger is hovered or focused (its Response's hovered or focused): it shows after delayMs and stays while open, while pinned, or while the pointer is over it."
  field target String "The trigger's path; a bare id names an element of the open container (built before this call). Empty places it at the pointer's last trigger rectangle of this tooltip."
  field title String "The title."
  field subtitle String "A line under it (Legendary sword, Passive)."
  field rarity Rarity "The title's colour and pips."
  field icon String "An icon of the atlas beside the title."
  field image Option.Texture "An art slot instead of the icon (item or ability icon, 1:1)."
  field body String "The body: rich text markup (og.ui.richText), its `[link=key]keyword[/link]` spans opening nested tooltips from links."
  field stats List.TooltipStat "Stat rows."
  field compareTitle String "With comparison deltas: the heading over a second card of what is equipped (Equipped); empty draws no second card."
  field compareStats List.TooltipStat "The equipped item's stats, in the second card."
  field footer String "A last line (Sell for 120, a flavour quote) in the muted ink."
  field hints List.String "Action hints (Right-click to equip), each with its action's glyph when one is named [action:equip]."
  field links List.TooltipLink "What the body's links open."
  field delayMs f32 "Rest before it shows; 0 is the style's slow duration." {
    unit ms
    min 0
  }
  field pinnable bool "It locks open: after pinMs of rest a ring fills on its corner and it stays when the pointer leaves the trigger, until the pointer leaves the card (as Paradox's games do); the pin action (middle mouse, T, a pad's Y) pins at once. Nested tooltips are always pinnable." {
    default true
  }
  field pinMs f32 "Rest before it pins; 0 is 900." {
    unit ms
    min 0
  }
  field width f32 "Its width; 0 is 320 (300 on a phone)." {
    unit cssPx
    min 0
  }
  field placement Anchor "Where it prefers to sit against the trigger: right (the end side), left, top or bottom. It sits beside the whole group the trigger is in (the panel or column, so it never covers its own trigger's panel) when there is room on either side of it, else beside the trigger; it flips and clamps to stay inside the safe area." {
    default right
  }
  field pinLabel String "The pin hint under a pinned card." {
    default "Pinned"
  }
}
```

<sub>Module `ui` line 5559</sub>
