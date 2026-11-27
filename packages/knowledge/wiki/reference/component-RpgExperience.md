<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgExperience"></a>
### `RpgExperience`

The experience line across the screen's foot: a thin violet line lit to the share of the way to the next level, with the level and that share in small capitals over its start, inset from the screen's edge as the HUD is. Read only.

`import { RpgExperience } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/experience.tsx`).

| Prop | Type | What |
|---|---|---|
| `level` | `number` | The level now; 0 hides the caption's level. |
| `value` | `number` | The share of the way to the next level, 0 to 1. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `full`.
