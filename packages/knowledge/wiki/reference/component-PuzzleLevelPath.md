<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PuzzleLevelPath"></a>
### `PuzzleLevelPath`

A scrolling level path over the game's own map art, which the component leaves at full brightness: chapters in order, each a glass header with its name, its stars and a gold line of them, then its levels as discs on a road, each at the place across the path the game gives (x). The road is lit gold up to the level to play and pale past it. A disc shows its number, a lock when it is locked, its stars on a dark capsule at its foot once done, and the Play plate under the current one, the road running on behind it; the chosen one is ringed and says Selected, an open one says Open. Reports choose (a level's key; a locked level is disabled and sends nothing) and, with X or pad Y on the focused disc, details. The game decides what is open and counts the stars. The path opens scrolled to the level to play, each chapter's header held at the top while its levels scroll under it.

`import { PuzzleLevelPath } from "@origingame/engine/ui/components"` (kit `puzzle`, `packages/engine/src/ui/components/puzzle/level-path.tsx`).

| Prop | Type | What |
|---|---|---|
| `chapters?` | `ReadonlyArray<{ id: string; label: string; earned: number; total: number }>` | The chapters in path order, keyed rows of { id, label, earned, total }: earned and total the chapter's stars as the game counts them. |
| `levels?` | `ReadonlyArray<{ id: string; chapter: string; number: number; state: string; stars: number; maxStars: number; x: number }>` | The levels in path order, keyed rows of { id, chapter, number, state, stars, maxStars, x }: chapter a chapter's id; state locked, open, current or done; stars earned (0 to 3) of maxStars (1 to 3); x the node's place across the path, 0 (start) to 1 (end), as the game lays the path out. |
| `selected` | `string` | The level the player has picked (its details shown elsewhere); empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onDetails?` | `(e: UiEvent, key?: string) => void` | The details event, with the row's key where it comes from a row. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `selected`, `locked`, `overflow`.

Actions: `details` (Play).
