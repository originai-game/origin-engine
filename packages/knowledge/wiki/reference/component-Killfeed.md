<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Killfeed"></a>
### `Killfeed`

Eliminations at the screen's top corner, newest first: each row the killer, the weapon icon and name, and the victim, a headshot tag, names in their side's colour on frosted glass, an edge at the screen's side lit for the player's own rows. Read only.

`import { Killfeed } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/killfeed.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<KillfeedRow>` | The eliminations to show, newest first. The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms. |

Each `KillfeedRow`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `killer` | `string` | The killer's name. |
| `victim` | `string` | The victim's name. |
| `weapon?` | `string` | The weapon's name. |
| `kind?` | `string` | The weapon's class (rifle, smg, pistol, shotgun, sniper, launcher or melee); its glyph is drawn from it, empty for the weapon's name in words. |
| `headshot?` | `boolean` |  |
| `self?` | `boolean` | The player took part (the row's edge is lit and the player's name takes the HUD's accent). |
| `team?` | `string` | Ally or enemy, the killer's side (the victim takes the other); empty for FFA neutral ink. |

Pack slots: `Chip.frame`, `Feed.name`, `Feed.row`, `Feed.row.self`, `Feed.weapon`.

Fixtures: `default`, `words`, `long`.
