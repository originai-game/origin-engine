<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PuzzleGoals"></a>
### `PuzzleGoals`

The in-level HUD: what limits the level (moves or time left) in a cut well, amber with a tag in words when the game says it runs low; the goals as sockets with what is left of each, its name under it and a check once met; and the score over a leaning bar with the game's star marks, lit up to its progress. It draws what the game counts; it decides nothing. Where it is given less than a phone's width (its own, not the screen's) the moves and the score share the top line and the goals wrap under them.

`import { PuzzleGoals } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/goals.tsx`).

| Prop | Type | What |
|---|---|---|
| `goals?` | `ReadonlyArray<{ id: string; icon: string; label: string; left: number; done: boolean }>` | The level's goals in order, keyed rows of { id, icon, label, left, done }: left how many are still to clear, as the game counts them; done the goal is met (a check replaces the count). |
| `counter` | `string` | What limits the level: moves (a move count) or time (a countdown). |
| `moves` | `number` | Moves left, when counter is moves. |
| `seconds` | `number` | Seconds left, when counter is time. |
| `low?` | `boolean` | The game says the moves or time are running low: the counter warns, in words as well as its dress. |
| `score` | `number` | The score so far, as the game counts it. |
| `progress?` | `number` | The score toward the level's top star, 0 to 1, for the bar under it; negative hides the bar. Default -1. |
| `stars?` | `ReadonlyArray<{ id: string; at: number }>` | The star marks on that bar, keyed rows of { id, at }: at where the star sits along the bar, 0 to 1; a star at or under progress is lit. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `time`, `low`, `lowTime`, `done`, `empty`, `overflow`.
