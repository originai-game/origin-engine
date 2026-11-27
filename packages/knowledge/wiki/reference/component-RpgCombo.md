<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCombo"></a>
### `RpgCombo`

The combo counter beside the fight: the hits as a large slanted figure with its word, the window still to run as a decay bar under it and the damage multiplier the chain gives. The game owns the chain; read only.

`import { RpgCombo } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/combo.tsx`).

| Prop | Type | What |
|---|---|---|
| `count` | `number` | The hits in the chain; 0 hides the counter. |
| `word` | `string` | The word after the figure as the game words it; empty takes the component's Hits. |
| `remaining?` | `number` | The share of the chain's window still to run, 0 to 1 (the decay bar); negative hides the bar. |
| `multiplier?` | `number` | The damage multiplier the chain gives (Damage ×1.6); 0 or 1 hides it. |

Fixtures: `default`, `fading`, `empty`, `overflow`.
