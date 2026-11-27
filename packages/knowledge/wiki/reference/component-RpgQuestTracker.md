<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgQuestTracker"></a>
### `RpgQuestTracker`

The tracked quest on the HUD: a heading in gold, the quest's name, then its objectives, the done ones struck through, the one in play lit with its progress on a tag and the upcoming ones dim. Read only; the journal owns choosing what to track.

`import { RpgQuestTracker } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/quest-tracker.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | The tracker's heading as the game words it (Side quest); empty takes the component's Main quest. |
| `name` | `string` | The tracked quest's name; empty hides the tracker. |
| `steps` | `ReadonlyArray<{ id: string; text: string; done: boolean; tag?: string }>` | Its objectives in order, keyed rows of { id, text, done, tag }: done ones are struck and dimmed with a filled diamond; the first not done is the step in play (lit, on data-current); the ones after it are upcoming (dim); tag its progress as the game words it (1 / 3) on a tag, empty for none. A phone shows the step in play alone. |
| `distance` | `string` | How far its marker is, as the game words it (312 m), on a tag beside the name; empty hides it. |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `done`, `overflow`.
