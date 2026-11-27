<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyWave"></a>
### `StrategyWave`

The wave readout at the top right: a plate whose edge is lit in danger while a wave comes or fights, its picture, Wave n (of N) and its name in red capitals, the countdown with a bar of the wait still to run, or Under attack or All waves cleared; the wave's enemies by kind under it, and Call early with what it pays. Reports callEarly; the game owns the timing, the wave's makeup and the reward.

`import { StrategyWave } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/wave.tsx`).

| Prop | Type | What |
|---|---|---|
| `wave` | `number` | The wave on now or coming, from 1. |
| `waves` | `number` | Waves in the level; 0 is an endless run (no total shown). |
| `name` | `string` | The wave's name as the game words it (Siege), after its number; empty none. |
| `icon` | `Texture \| 0` | The wave's picture (og.load.texture) in a cut tile before it; 0 hides it. |
| `phase` | `string` | countdown (the wave is coming), active (a wave is on) or done (every wave cleared), as the game says: data-phase; active and done stand as words in place of the countdown. |
| `next?` | `number` | Seconds to the wave while it is coming, shown as a countdown (0:42); negative hides it. Default -1. |
| `nextTotal` | `number` | The whole wait: the bar beside the countdown shows the share still to run; 0 draws no bar. |
| `warnAt?` | `number` | Seconds left when the countdown is near (data-near on the frame); 5 when left out. Default 5. |
| `finalWave` | `boolean` | The coming or current wave is the last: the Final wave badge (data-final). |
| `enemies?` | `ReadonlyArray<{ id: string; name: string; icon: string; count: number; boss: boolean; scouted: boolean }>` | The wave's enemies by kind, under the plate, keyed rows of { id, name, icon, count, boss, scouted }: icon an icon of the atlas; count shown as ×n; boss marks it with a Boss badge (data-boss); scouted false hides name and count behind Not yet scouted. Empty shows none. |
| `canCallEarly` | `boolean` | The game allows calling the wave now; false hides Call early. |
| `reward` | `string` | What calling early pays, as the game words it (+40 gold), on the button in gold; empty none. |
| `onCallEarly?` | `(e: UiEvent) => void` | Call the wave early. |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Plate.frame`, `Row.frame`, `Row.frame.muted`.

Fixtures: `default`, `incoming`, `active`, `final`, `done`, `empty`, `overflow`.
