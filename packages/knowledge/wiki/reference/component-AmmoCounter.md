<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-AmmoCounter"></a>
### `AmmoCounter`

Ammunition at the screen's bottom corner: the reserve weapon on a small plate, the magazine as the HUD's largest figure with the reserve beside it, a tick per round under the count (lit while loaded), and the fire mode as pips beside the weapon's name; the labels sit on HUD plates. Low and empty change the count's ink and say so; a reload shows its progress. The game owns every count; read only. The ticks' mask and the lean (skewX) have no TSX surface: the ticks are separate views, the lean is dropped.

`import { AmmoCounter } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ammo-counter.tsx`).

| Prop | Type | What |
|---|---|---|
| `weapon` | `string` | The weapon's name. |
| `mode?` | `string` | Its fire mode: auto, burst or single (beside the name in words, and as three leaning pips: all lit for auto, two for burst, one for single). Default auto. |
| `magazine` | `number` | Rounds in the magazine. |
| `capacity` | `number` | Rounds a full magazine holds: a leaning tick per round under the count, lit for each round left (a continuous strip past 60). |
| `reserve` | `number` | Rounds carried beyond the magazine. |
| `reloading?` | `number` | The reload's progress, 0 to 1; negative while not reloading. Default -1. |
| `low?` | `number` | The fraction of capacity at or under which the count turns low (0.25). Default 0.25. |
| `secondary?` | `string` | The weapon held in reserve, on a small plate over the readout; empty hides it. |
| `secondaryAmmo?` | `number` | The reserve weapon's rounds; negative hides the figure. Default -1. |
| `secondaryKey?` | `string` | The input that swaps to it as the game names it (2), on a key cap; empty hides it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `low`, `empty`, `reloading`, `belt`, `overflow`.
