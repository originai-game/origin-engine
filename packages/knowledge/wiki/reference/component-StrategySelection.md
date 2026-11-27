<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategySelection"></a>
### `StrategySelection`

The selection panel at the foot: the portrait in a tile with a lit edge, its health as a bar and in figures under it; the name large with how many are selected, its class line in small capitals, what it is doing and its stats on tags; and every unit selected as a card with its health, the focused one lit. Reports choose (a card's key: show that unit or subgroup). The game owns every figure and what the panel shows.

`import { StrategySelection } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/selection.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The selected unit's or building's name, or the focused subgroup's (Men-at-Arms); empty shows Nothing selected. |
| `count` | `number` | How many of it are selected (×12, shown above 1). |
| `kind` | `string` | unit or building (data-kind on the frame). |
| `line` | `string` | Its class line as the game words it (Heavy infantry · Veteran II), under the name; empty none. |
| `status` | `string` | What it is doing as the game words it (Holding, Under construction), on a tag before the stats; empty none. |
| `portrait` | `Texture \| 0` | Its portrait (og.load.texture) in a tile with a lit edge; 0 draws icon instead. |
| `icon` | `string` | An icon of the atlas drawn when portrait is 0. |
| `health` | `number` | Its health now (the subgroup's together), under the portrait as a bar and in figures. |
| `healthMax?` | `number` | Its whole health; 0 hides the bar and the figures. Default 1. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: string }>` | Its stats on tags, keyed rows of { id, label, value }: label short as the game words it (ATK), value its figure (12). |
| `units?` | `ReadonlyArray<{ id: string; name: string; kind: string; portrait: Texture \| 0; icon: string; count: number; health: number; healthMax: number }>` | Each unit or subgroup selected, as cards in the game's order (shown when there are two or more; two rows, then they scroll), keyed rows of { id, name, kind, portrait, icon, count, health, healthMax }: portrait a texture, 0 draws icon from the atlas; count shown above 1 on a tag; health over healthMax a bar on the card, green, gold when hurt and red when low (data-tone). |
| `focused` | `string` | The card whose unit or subgroup the panel shows (its edge lit, aria-selected); empty none. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a card (its key: show that unit or subgroup). |

Pack slots: `Button.plate`, `Chip.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `single`, `empty`, `groups`, `dense`, `overflow`.
