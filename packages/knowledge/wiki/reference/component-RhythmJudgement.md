<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmJudgement"></a>
### `RhythmJudgement`

The play field's centre: the running combo as the hero figure with its word (Combo broken when it breaks, Full combo or All perfect while the run holds one), and under it the last hit's judgement, each in its own lit colour, with Early or Late and the offset. A new hit id pops the word and the count again. The game owns timing windows, judging and counting; this only shows what it was told. Read only; not a live region.

`import { RhythmJudgement } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/judgement.tsx`).

| Prop | Type | What |
|---|---|---|
| `judgement?` | `string` | The last hit's judgement as the game judged it: perfect, great, good, bad, miss, or none (nothing shown); on data-judgement for the pack, and always a word. Default "none". |
| `label` | `string` | The judgement's word as the game names it (Critical, Marvelous); empty takes the kit's word for judgement. |
| `timing` | `string` | early or late under the word (Early, Late); empty hides it. |
| `offset` | `string` | The hit's offset as the game words it (+4 ms, −12 ms), beside the timing; empty hides it. |
| `hit` | `number` | The hit's id: a new value draws the word afresh (each note judged), so the word and the combo pop again. |
| `combo` | `number` | The running combo as the game counts it; 0 hides it. |
| `broken` | `boolean` | The combo just broke: Combo broken shows in its place. |
| `mark` | `string` | The run is still on course for a mark: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `great`, `good`, `miss`, `overflow`.
