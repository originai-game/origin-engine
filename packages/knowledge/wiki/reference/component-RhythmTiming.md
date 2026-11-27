<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmTiming"></a>
### `RhythmTiming`

How the run is being judged, at a corner of the field: the tally so far, each judgement in its colour, and under it the timing scale from Early to Late with the beat at its centre and the latest hits as ticks, the newest brightest. The game judges and counts; read only.

`import { RhythmTiming } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/timing.tsx`).

| Prop | Type | What |
|---|---|---|
| `counts` | `ReadonlyArray<{ id: string; judgement: string; label: string; count: number }>` | The judgement tally so far in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; its colour, on data-judgement), label its word as the game names it, count how many. |
| `hits` | `ReadonlyArray<{ id: string; judgement: string; offset: number; age: number }>` | The latest hits on the timing scale, keyed rows of { id, judgement, offset, age }: offset from −1 (earliest the game judges) through 0 (on the beat) to 1 (latest); age from 0 (this hit) to 1 (about to drop), which fades it. The game keeps the list short (eight or so). |

Pack slots: `Plate.frame`.

Fixtures: `default`, `early`, `empty`, `overflow`.
