<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudTimer"></a>
### `HudTimer`

A HUD clock on its plate: a caption, the seconds in the HUD's large figure, and for a countdown with a total a ring of what is left, which warns and then turns red near zero; paused, the time dims and breathes. Read only.

`import { HudTimer } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hud-timer.tsx`).

| Prop | Type | What |
|---|---|---|
| `seconds` | `number` | Seconds on the clock. |
| `total?` | `number` | A countdown's whole time: a ring of what is left beside it (0: none). |
| `countdown?` | `boolean` | Counts down (warns and turns red near zero); false is a stopwatch. |
| `warnAt?` | `number` | Seconds left when it turns warn; a third of it is critical. |
| `paused?` | `boolean` | Paused: the time dims and breathes. |
| `label?` | `string` | A caption over it (Round 2); empty is none. |

Pack slots: `Plate.frame`.

Fixtures: `default`.
