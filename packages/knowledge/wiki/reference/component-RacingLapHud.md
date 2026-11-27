<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingLapHud"></a>
### `RacingLapHud`

The race's place and clock at the top left: the place large and slanted over the field's size, the lap on a lit plate and the places gained beside it; under them the lap time running, the last and best laps and the split delta on a green or red tag. Every figure and time comes from the game, formatted; read only.

`import { RacingLapHud } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/lap-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `lap` | `number` | The lap the player is on, from 1; 0 before the start shows a dash. |
| `laps` | `number` | Laps in the race; 0 hides the lap's total (an open session). |
| `position` | `number` | The player's place, from 1; 0 shows a dash. |
| `racers` | `number` | Cars in the race; 0 hides the place. |
| `gained` | `number` | Places gained since the start as the game counts them: positive reads Up n places, negative Down n places (on data-trend), 0 hides it. |
| `current` | `string` | The lap time running now, as the game formats it (1:24.618); empty shows the blank time. |
| `best` | `string` | The player's best lap, formatted, in the best lap's violet; empty shows the blank time. |
| `last` | `string` | The last lap, formatted; empty shows the blank time. |
| `sector` | `string` | The split's name as the game words it (Sector 2); shown beside the delta. |
| `split` | `string` | The split delta as the game words it (−0.412, +1.204), on a green tag ahead and a red one behind; empty hides the row. |
| `splitState?` | `string` | ahead, behind or even: the delta's sense, on data-state and read with it. Default "even". |
| `freshBest` | `boolean` | The last lap was a new best: a Best lap tag stands on the last row. |
| `finalLap` | `boolean` | The player is on the final lap: the lap plate reads Final lap (data-final). |
| `timeLeft?` | `number` | Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it. Default -1. |
| `timeTotal` | `number` | The checkpoint's whole time, for the timer's ring; 0 draws none. |
| `warnAt?` | `number` | Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out. Default 10. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `leading`, `last`, `grid`, `checkpoint`, `overflow`.
