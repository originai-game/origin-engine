<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RhythmSongProgress"></a>
### `RhythmSongProgress`

The song's progress as a thin lit line along the screen's top edge, and the time played over the length riding under its played end. The game keeps time; read only.

`import { RhythmSongProgress } from "@origingame/engine/ui/components"` (kit `rhythm`, `packages/engine/src/ui/components/rhythm/song-progress.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | How far through the song, 0 to 1. |
| `elapsed` | `string` | The time played as the game words it (2:07); empty hides the time. |
| `length` | `string` | The song's length as the game words it (3:21), after the time played. |

Fixtures: `default`, `start`, `end`, `bare`, `overflow`.
