<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyCommands"></a>
### `StrategyCommands`

The command card at the bottom right: abilities, orders and buildings as square tiles four to a row, each its picture with a lit edge, the hotkey in its corner, a cooldown as a dark sweep with the seconds left, a command that cannot be paid for greyed with its price on a red tag, a disabled one greyed, the order in hand lit; the info card of the command in focus over the grid (its name and key, what it does, its price, cooldown and when it is ready, or why it cannot be used). Reports command (the tile's key; a disabled or unaffordable tile sends nothing); the game owns every cost, cooldown and rule.

`import { StrategyCommands } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/commands.tsx`).

| Prop | Type | What |
|---|---|---|
| `commands?` | `ReadonlyArray<{ id: string; name: string; icon: Texture \| 0; glyph: string; key: string; action: string; enabled: boolean; affordable: boolean; cost: string; costIcon: Texture \| 0; cooldown: number; time: string; active: boolean; detail: string; cooldownText: string; readyText: string; reason: string }>` | The command card in grid order, four to a row, keyed rows of { id, name, icon, glyph, key, action, enabled, affordable, cost, costIcon, cooldown, time, active, detail, cooldownText, readyText, reason }: name for a reader and the info card's title, empty draws an empty cell; icon the command's picture (og.load.texture), 0 draws glyph (an icon of the atlas); key the hotkey as the game names it (A), or action the input action whose bound glyph the cap shows (og.input.defineAction), both empty hide it, hidden under a touch screen; enabled false greys it and it sends nothing; affordable false greys it and shows cost on a red tag; cost and costIcon the price as the game words it (200) and the resource's picture; cooldown 0 to 1 still to run, a dark sweep over the art, with time its figure (18); active a toggle that is on or the order in hand, its edge lit; detail, cooldownText and readyText the info card's words (Cooldown 45 s, Ready in 18 s); reason why it cannot be used now (Need a Blacksmith), on the info card. |
| `focused` | `string` | The command whose info card stands over the grid (the one under the pointer, as the game decides); empty shows none. A phone shows none. |
| `onCommand?` | `(e: UiEvent, id: string) => void` | Command. |

Pack slots: `Button.plate`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `unaffordable`, `cooling`, `overflow`.
