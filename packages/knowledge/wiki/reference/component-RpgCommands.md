<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCommands"></a>
### `RpgCommands`

A turn's commands as a row of plates, each an icon over its label; the selected one is the primary plate. Reports choose (a command's key).

`import { RpgCommands } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/commands.tsx`).

| Prop | Type | What |
|---|---|---|
| `commands?` | `ReadonlyArray<{ id: string; label: string; icon: string; enabled: boolean }>` | The commands in order, keyed rows of { id, label, icon, enabled }: label as the game words it (Attack, Skills, Spells, Items, Defend, Flee). |
| `selected` | `string` | The command the focus rests on; it takes the primary plate. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The choose event, with the command's key. |

Pack slots: `Button.plate.…`, `Panel.frame`.

Fixtures: `default`.
