<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyTower"></a>
### `StrategyTower`

A tower's inspector: its picture, name and level, its current stats with the value after the next upgrade where the game gives one, the upgrade paths (tier of tiers, the next tier's cost, a state word: affordable, too costly, locked with the reason, maxed) and Sell with what it returns. Reports upgrade (the path's key; only an affordable path is enabled) and sell. The game owns prices, refunds, stats and what is unlocked.

`import { StrategyTower } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/tower.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The tower's name; empty takes the kit's (Tower). |
| `icon` | `string` | An icon of the atlas for the tower, drawn when portrait is 0. |
| `portrait` | `Texture \| 0` | A texture of the tower (og.load.texture); 0 draws icon. |
| `level` | `number` | The tower's level as the game counts it; 0 hides it. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: string; next: string }>` | Its current stats, keyed rows of { id, label, value, next }: value as the game words it (42); next the value after the upgrade in focus (51), shown after an arrow, empty none. |
| `paths?` | `ReadonlyArray<{ id: string; name: string; icon: string; detail: string; tier: number; tiers: number; cost: string; state: string; reason: string }>` | Upgrade paths, keyed rows of { id, name, icon, detail, tier, tiers, cost, state, reason }: tier of tiers bought; cost the next tier's price as the game words it (250 gold); state affordable, unaffordable, locked or maxed (data-state and a word; only affordable can be bought); reason why it is locked (Requires tier 3 on another path), empty none. |
| `sellText` | `string` | What selling returns, as the game words it (+180 gold), on the Sell button. |
| `canSell?` | `boolean` | The game allows selling now; false disables Sell. |
| `onUpgrade?` | `(e: UiEvent, id: string) => void` | Upgrade. |
| `onSell?` | `(e: UiEvent) => void` | Sell. |

Pack slots: `Button.plate`, `Button.plate.…`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `maxed`, `disabled`, `empty`, `overflow`.
