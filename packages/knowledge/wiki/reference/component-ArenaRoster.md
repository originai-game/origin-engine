<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaRoster"></a>
### `ArenaRoster`

A character select: the roster as a grid of tiles, the local player's choice selected and named under it, and who is choosing. It reports onChoose; the game decides what may be chosen.

`import { ArenaRoster } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/roster.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The screen's heading; empty takes the kit's (Choose your champion). |
| `characters?` | `ReadonlyArray<RosterCharacter>` | The roster in the game's order. |
| `selected?` | `string` | The local player's character (selected, named under the grid); empty is none. |
| `players?` | `ReadonlyArray<RosterPlayer>` | Who is choosing. |
| `compact?` | `boolean` | Its place is narrow (under about 380 px): the tiles, padding and chosen name tighten. The place decides, not the screen. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The local player chooses a character. |

Each `RosterCharacter`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The character's key, reported back with choose. |
| `name` | `string` | What a reader calls them. |
| `portrait?` | `Texture \| 0` | The character's portrait texture (og.load.texture, 0 shows the icon instead). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `role?` | `string` | A key for the pack (tank, damage, support…). |
| `roleLabel?` | `string` | The role's word. |
| `locked?` | `boolean` | The player cannot pick it yet. |
| `takenBy?` | `string` | Who already holds it (empty: free). |

Each `RosterPlayer`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The player's key. |
| `slot?` | `number` | 1 to 4, the mark P1…P4. |
| `at?` | `string` | The character id the player is on (empty: none). |
| `ready?` | `boolean` | The player has locked in (a check beside the mark). |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `none`, `full`, `disabled`, `overflow`.
