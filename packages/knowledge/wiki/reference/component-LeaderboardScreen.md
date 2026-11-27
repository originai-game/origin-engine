<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-LeaderboardScreen"></a>
### `LeaderboardScreen`

A leaderboard: global, friends and league tabs, the period, the top rows with the player's own marked, the player's row pinned, and the league's promotion and demotion lines. The game owns the rows.

`import { LeaderboardScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/leaderboard.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `tab` | `string` | global, friends or league. |
| `period` | `string` | The period, as text (This week). |
| `rows` | `ReadonlyArray<{ id: string; name: string; rank: number; score: number; texture: Texture \| 0; you: boolean }>` | The top rows: { rank, name, score, you (bool), texture }. |
| `selected` | `string` | The stable collection key selected now. |
| `me` | `Record<string, any>` | The player's row: { rank, name, score }. |
| `league` | `Record<string, any>` | The league: { name, promote (rank), demote (rank), ends (seconds) }. |
| `board?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the board snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onPeriod?` | `(e: UiEvent, key?: string) => void` | The period event, with the row's key where it comes from a row. |
| `onRow?` | `(e: UiEvent, key?: string) => void` | The row event, with the row's key where it comes from a row. |
| `onTab?` | `(e: UiEvent, key?: string) => void` | A tab pressed, with its key (global, friends or league): the game sets `tab`. |

Pack slots: `Chip.frame`, `List.cascade`, `List.item`, `Panel.frame`, `Plate.frame`, `Row.frame`, `Row.frame.selected`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Profile), `back` (Back), `tabNext` (Next tab), `tabPrev` (Previous tab).
