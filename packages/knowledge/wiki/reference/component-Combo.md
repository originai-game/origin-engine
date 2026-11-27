<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Combo"></a>
### `Combo`

The combo readout on its HUD plate: the heading, the count and its multiplier as figures, a broken-chain cue when broken, and the window's progress while it runs. A status: its changes announce. Read only; the game owns the count and the window.

`import { Combo } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/combo.tsx`).

| Prop | Type | What |
|---|---|---|
| `count?` | `number` | The game's combo count, shown at zero too. Default 0. |
| `multiplier?` | `number` | The game's multiplier; it is not derived from count. Default 1. |
| `remaining?` | `number` | Remaining window, 0 to 1; negative hides the timer. The game advances it. Default -1. |
| `state?` | `string` | idle, active or broken; zero remaining does not determine gameplay results. Default idle. |
| `label?` | `string` | A custom heading; empty uses Combo. Default "". |
| `hideWhenIdle?` | `boolean` | Hide the plate completely while idle with zero combo. Default false. |

Pack slots: `Combo.count`, `Combo.multiplier`, `Combo.plate`, `Count.roll`.

Fixtures: `default`, `idle`, `broken`, `overflow`.
