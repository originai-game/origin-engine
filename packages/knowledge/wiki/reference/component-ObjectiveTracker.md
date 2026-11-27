<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ObjectiveTracker"></a>
### `ObjectiveTracker`

The tracked objectives over the scene, with no plate: the quest in gold small capitals, then each objective with a gold diamond (filled once done, its words struck and dimmed) over a thin leaning bar in a dark well, cut into cells when the game counts its steps. The first not done is lit; on a phone only it is shown. Read only.

`import { ObjectiveTracker } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/objective-tracker.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The quest or chapter. |
| `objectives` | `ReadonlyArray<{ id: string; label: string; progress: number; done: boolean; steps: number }>` | A keyed collection of { id, label, progress (0–1), done, steps }: steps, when above 1, cuts the bar into that many cells (2 / 5 is five); 0 or absent is one bar. |

Pack slots: `Plate.frame`.

Fixtures: `default`.
