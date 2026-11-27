<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyGroups"></a>
### `StrategyGroups`

The control groups over the selection: one small plate each with its key cap and the units it holds, the group selected now lit; a quick pick (the idle villagers) may carry a picture, its count in gold while it wants the player. Reports pick (the group's key); the game owns the groups.

`import { StrategyGroups } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/groups.tsx`).

| Prop | Type | What |
|---|---|---|
| `groups?` | `ReadonlyArray<{ id: string; label: string; key: string; icon: Texture \| 0; count: number; active: boolean; alert: boolean }>` | The control groups and quick picks (the idle villagers, the army) in the game's order, keyed rows of { id, label, key, icon, count, active, alert }: label for a reader (Group 1, Idle villagers); key the group's key as the game names it (1), on a key cap; icon a picture (og.load.texture) before the count, 0 none; count what it holds; active the group selected now, lit (aria-pressed); alert one that wants the player (idle villagers waiting), its count and edge in gold (data-alert). |
| `onPick?` | `(e: UiEvent, id: string) => void` | Pick. |

Pack slots: `Button.plate`.

Fixtures: `default`, `idle`, `overflow`.
