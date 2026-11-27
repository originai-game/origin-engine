<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmScoreHud"></a>
### `RhythmScoreHud`

The run's figures over the play field, set against the screen's end: the score as the hero figure, the accuracy with the grade it is on course for, and the player's best with the gap to it. The game owns every figure; read only. Pair it with RhythmGauge for life and fever and RhythmSongProgress for the song's progress.

`import { RhythmScoreHud } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/score-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `score` | `number` | The score as the game counts it, shown with grouping. |
| `accuracy` | `string` | The accuracy as the game words it (98.42%); empty hides it. |
| `grade` | `string` | The grade the run is on course for as the game names it (S, AA); empty hides it. |
| `best?` | `number` | The player's best on this chart; 0 hides it. Ahead or behind it shows as a signed gap beside it (on data-trend). Default 0. |

Pack slots: `Chip.frame`, `Count.roll`, `Plate.frame`.

Fixtures: `default`, `ahead`, `first`, `start`, `overflow`.
