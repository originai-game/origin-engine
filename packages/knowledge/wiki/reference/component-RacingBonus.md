<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingBonus"></a>
### `RacingBonus`

Feedback above the road at the top centre: a meter (the slipstream) as a word over a bar in the HUD's accent, and the points a drift or a chain earns, large and slanted, with what earns them in gold under it. The game decides the meter and every figure; read only.

`import { RacingBonus } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/bonus.tsx`).

| Prop | Type | What |
|---|---|---|
| `meter` | `string` | A meter's name as the game words it (Slipstream, Drafting), over a bar in the HUD's accent; empty hides the meter. |
| `value` | `number` | The meter's share, 0 to 1. |
| `points` | `string` | The points being earned now as the game words them (+2,450), large and slanted; empty hides them. |
| `detail` | `string` | What earns them as the game words it (Drift · Near miss ×3), in gold under the points; empty hides it. |
| `at` | `number` | The chain's id: a new value replays the points' entrance. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `meter`, `points`, `overflow`.
