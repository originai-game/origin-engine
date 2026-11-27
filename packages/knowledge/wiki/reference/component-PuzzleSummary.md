<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PuzzleSummary"></a>
### `PuzzleSummary`

The end of a level: won or failed, the level's name, the stars on the shared StarBar with the stars the game awarded in words, a new-best badge, the score breakdown the game words, and the next steps. Reports next (only when canNext), retry and menu (also Escape or pad B). The game computes the score, the stars and what comes next.

`import { PuzzleSummary } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/summary.tsx`).

| Prop | Type | What |
|---|---|---|
| `result` | `string` | How the level ended: win or lose. |
| `level` | `string` | The level's name as the game words it (Level 12); empty hides it. |
| `score` | `number` | The final score, as the game counts it. |
| `scoreMax?` | `number` | The score of the star bar's end (its last threshold). Default 1. |
| `thresholds?` | `ReadonlyArray<{ id: string; at: number }>` | The star bar's thresholds, keyed rows of { id, at }: at a fraction of scoreMax (0 to 1), as the game's star rules place them. |
| `stars` | `number` | Stars the game awarded (0 to maxStars): spoken and written beside the bar. |
| `maxStars?` | `number` | Stars the level can award. Default 3. |
| `record?` | `boolean` | The game says the score is a new best: a badge with its word. |
| `rows?` | `ReadonlyArray<{ id: string; label: string; value: string }>` | The score breakdown, keyed rows of { id, label, value }: value already worded by the game (+2,400, ×3). |
| `total` | `string` | The breakdown's total as the game words it; empty hides the total row. |
| `canNext?` | `boolean` | Next is offered (a won level with a next one). |
| `onMenu?` | `(e: UiEvent, key?: string) => void` | The menu event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`, `full`, `lose`, `empty`, `overflow`.

Actions: `back` (Map).
