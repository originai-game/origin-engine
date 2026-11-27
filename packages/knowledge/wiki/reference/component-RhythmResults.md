<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmResults"></a>
### `RhythmResults`

After the song: the grade, Cleared or Failed, the score with the old best and New best, Full combo or All perfect, max combo and accuracy, and the judgement tally. Retry (disabled when the game refuses it) and Next (also Escape, pad B) report once. The game computes every figure, grade and verdict.

`import { RhythmResults } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/results.tsx`).

| Prop | Type | What |
|---|---|---|
| `song` | `string` | The song's title as the heading; empty takes the kit's (Results). |
| `grade` | `string` | The grade the game awarded (S, AA, C); empty hides it. |
| `cleared` | `boolean` | The game's verdict: Cleared, or Failed (data-cleared). |
| `score` | `number` | The run's score as the game counts it, shown with grouping. |
| `previous` | `number` | The player's best before this run (Best …); 0 hides it. |
| `newBest` | `boolean` | The game says this run beat the best: New best shows by the score. |
| `maxCombo` | `number` | The longest combo of the run. |
| `accuracy` | `string` | The accuracy as the game words it (97.42%); empty hides it. |
| `mark` | `string` | A run mark the game awarded: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. |
| `counts` | `ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>` | The judgement tally in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; on data-judgement), label its word as the game names it, count how many. |
| `canRetry` | `boolean` | Retry is offered (false: disabled). |
| `onRetry?` | `(e: UiEvent) => void` | Retry; silent when the game refuses it. |
| `onNext?` | `(e: UiEvent) => void` | Next; also Escape and the back action (pad B). |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `failed`, `fullCombo`, `allPerfect`, `disabled`, `empty`, `overflow`.
