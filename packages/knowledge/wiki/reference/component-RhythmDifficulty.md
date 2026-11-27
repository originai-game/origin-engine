<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmDifficulty"></a>
### `RhythmDifficulty`

Choosing a chart: one leaning tile per difficulty with its tier's colour along its top, the tier's name, its level as a large figure, the player's best grade on a tag beside the best score, and Locked or Selected in words. A tile reports choose with its key; a locked chart is disabled and reports nothing. The game owns the charts, unlocking, grades and scores.

`import { RhythmDifficulty } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/difficulty.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The heading (the song's title); empty takes the kit's (Difficulty). |
| `levels` | `ReadonlyArray<{ id: string; tier: string; label: string; rating: number; grade: string; score: string; locked: boolean }>` | The song's charts in the game's order, keyed rows of { id, tier, label, rating, grade, score, locked }: tier the chart's family (easy, normal, hard, expert, master, special; on data-tier for the pack's colour, never alone); label its name as the game words it (Hard); rating its level number; grade the player's best grade on it (S, AA; empty unplayed); score the best score as the game formats it (981,200; empty Not played); locked it cannot be played yet (disabled). |
| `selected` | `string` | The chart the game holds as chosen (selected, named Selected); empty is none. |
| `onChoose?` | `(e: UiEvent, key: string) => void` | Choose the chart; the tile's key. A locked chart is disabled and reports nothing. |

Pack slots: `Chip.frame`.

Fixtures: `default`, `selected`, `disabled`, `empty`, `overflow`.
