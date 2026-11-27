<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaScoreboard"></a>
### `ArenaScoreboard`

A two-team scoreboard: each team's name and score and its players' rows, kills, deaths, assists and score or score alone, side by side or one above the other. Read only; the game orders the rows.

`import { ArenaScoreboard } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/scoreboard.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The board's heading; empty takes the kit's (Scoreboard). |
| `columns?` | `string` | What each row shows: kda (kills, deaths, assists and score) or score (score only). |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `scoreA?` | `string` | The first team's score as the game words it (rounds, points); empty hides it. |
| `scoreB?` | `string` | The second team's score as the game words it; empty hides it. |
| `rows?` | `ReadonlyArray<ScoreboardRow>` | Every player in the game's order. |
| `split?` | `boolean` | The two teams stand side by side; otherwise one above the other. |

Each `ScoreboardRow`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The player's key. |
| `team?` | `string` | a or b. |
| `name?` | `string` | What a reader calls them. |
| `kills?` | `number` | Kills as the game counts them. |
| `deaths?` | `number` | Deaths as the game counts them. |
| `assists?` | `number` | Assists as the game counts them. |
| `score?` | `number` | The score as the game counts it. |
| `local?` | `boolean` | The local player's row (You, the row is current). |
| `disconnected?` | `boolean` | The row is dimmed and named Disconnected. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.muted`, `Row.frame.selected`.

Fixtures: `default`, `score`, `disconnected`, `overflow`.
