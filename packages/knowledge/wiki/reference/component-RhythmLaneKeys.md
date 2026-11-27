<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmLaneKeys"></a>
### `RhythmLaneKeys`

The lanes' key caps under the judgement line, spread across the field's width, one per lane, each lit while the player holds it. The game reads input and names the keys; read only.

`import { RhythmLaneKeys } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/lane-keys.tsx`).

| Prop | Type | What |
|---|---|---|
| `lanes` | `ReadonlyArray<{ id: string; key: string; held: boolean }>` | The lanes left to right, keyed rows of { id, key, held }: key the input that plays the lane as the game names it (S, LB); held the player holds it now (the cap lights). |

Fixtures: `default`, `four`, `overflow`.
