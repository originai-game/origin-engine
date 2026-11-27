<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingStandings"></a>
### `RacingStandings`

The position tower at the left edge: one leaning glass row per car in the order the game ranks them, the place in a box, the car's colour on a stripe, the name and the gap (Leader for the leader). The player's row is lit in the HUD's accent; pitting, retired and finished cars carry a word on a tag. On a phone it collapses to the leader and the rows either side of the player. Read only.

`import { RacingStandings } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/standings.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The tower's name for a reader; empty takes the kit's (Standings). Not drawn. |
| `rows?` | `ReadonlyArray<{ id: string; position: number; name: string; gap: string; color: string; player: boolean; state: string }>` | The running order as the game ranks it, keyed rows of { id, position, name, gap, color, player, state }: position the place shown in its box; gap the interval as the game words it (+1.204, +1 lap; the leader's empty gap reads Leader); color the car's or team's colour as a CSS colour, on a stripe beside the name (empty: a neutral stripe); player marks the local player's row (lit in the HUD's accent, aria-current); state racing, pitting, retired or finished, each a word on a tag (Pit, DNF, Fin), a retired row dimmed. |
| `room` | `string` | How short its place is: empty shows every row; short keeps the leader and the rows beside the player; shorter the leader and the player. |
| `hasPlayer` | `boolean` | A row is the player's. On a phone the tower keeps that row and the rows beside it, or the first three when false. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `leading`, `last`, `empty`, `single`, `finished`, `overflow`.
