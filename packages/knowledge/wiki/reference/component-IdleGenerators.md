<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IdleGenerators"></a>
### `IdleGenerators`

The generator list of an idle game: one cut row per producer with its art in a socket and its level on a hexagon at the socket's foot, its name and how many are owned, its yield in green, its running cycle as a leaning bar with the time left, and its Auto and boost tags. The game runs the cycles and owns every number; the list reports choose (a row's key) and the chosen row is washed in the accent. A locked row is disabled and greyed, and says Locked with what unlocks it. It fills its height and scrolls, its foot fading while there is more.

`import { IdleGenerators } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/generators.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Generators). |
| `generators?` | `ReadonlyArray<{ id: string; icon: string; name: string; owned: number; rate: string; cycle: number; cycleText: string; level: number; boost: string; auto: boolean; locked: boolean }>` | The producers in the game's order, keyed rows of { id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }: owned how many the player has (a number, compacted; 0 hides it); rate what it yields as the game words it (+1.2K/s; on a locked row what unlocks it); cycle the running cycle 0 to 1 (negative hides the bar); cycleText the time left as the game words it (empty shows the percent); level its level (0 hides it); boost a multiplier as the game words it (×2; empty hides it); auto it runs by itself (named Auto); locked not yet available (named Locked, disabled). |
| `selected` | `string` | The generator the player last chose, selected; empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`, `Slot.frame`.

Fixtures: `default`, `empty`, `disabled`, `selected`, `overflow`.
