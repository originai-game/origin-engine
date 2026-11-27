<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgQuestLog"></a>
### `RpgQuestLog`

The quest list, grouped as the game groups it (main, side, finished): each group a gold label in small capitals behind a short gold rule (a finished group's quiet), each quest a row with its mark. The mark is a diamond: hollow, lit gold on the quest the HUD tracks, filled in the success colour once the quest is done. The selected row (the one in detail) is washed gold from its leading edge with a gold bar on it and a chevron toward the detail; a fresh update carries a New tag. Finished quests are struck and quiet. Reports choose (a quest's key) and track (X or pad Y on the focused quest, its key). It fills its height and scrolls inside it. Dropped against the .svelte: the chosen quest's name scrolling in place when it does not fit (it ends in an ellipsis like the others), and the compact touch/narrow row height tweak (one row height everywhere).

`import { RpgQuestLog } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/quest-log.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The log's heading (Quest Journal). |
| `groups` | `ReadonlyArray<{ id: string; label: string; done: boolean }>` | The groups in order, keyed rows of { id, label, done }: done marks the group of finished quests, shown struck and quiet. |
| `quests` | `ReadonlyArray<{ id: string; group: string; name: string; tracked: boolean; fresh: boolean }>` | The quests, keyed rows of { id, group, name, tracked, fresh }: group a group's id; tracked the one the HUD follows; fresh an update the player has not read. |
| `selected` | `string` | The quest shown in detail; empty is none. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onTrack?` | `(e: UiEvent, key?: string) => void` | The track event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `overflow`.

Actions: `details` (Track).
