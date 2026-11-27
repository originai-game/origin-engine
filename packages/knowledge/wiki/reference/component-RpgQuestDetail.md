<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgQuestDetail"></a>
### `RpgQuestDetail`

One quest: its kind in gold small capitals over its name on the heading band (over the scenery, shaded into the glass, when the game gives one), the story so far, its objectives as stages with their tasks on a rail under them, and its rewards as cut chips, each with its icon in an octagon socket (one with a detail, a unique reward, lit gold). A done objective is struck with its diamond filled in the success colour; the first stage not done is lit gold. The body scrolls inside the frame's height, fading out above the actions (Show on map, Track or Stop tracking), which stay at its foot; they report locate and track. Track is the primary action; once the quest is tracked it reads Stop tracking on a quiet secondary button. With neither action it is read only.

`import { RpgQuestDetail } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/quest-detail.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The quest's name, in the heading band. |
| `summary` | `string` | Its story so far, as the game writes it. |
| `objectives?` | `ReadonlyArray<{ id: string; text: string; done: boolean; optional: boolean; step: number }>` | Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so. |
| `rewards?` | `ReadonlyArray<{ id: string; icon: string; label: string; detail: string }>` | Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it. |
| `label` | `string` | What kind of quest it is, as the game words it (Main quest), over the name; empty hides it. |
| `tracked` | `boolean` | The HUD follows this quest: its action reads Stop tracking. |
| `canTrack` | `boolean` | Show the Track (or Stop tracking) action, which reports track. |
| `canLocate` | `boolean` | Show the Show on map action, which reports locate. |
| `scenery` | `Texture \| 0` | A texture beside the rewards (og.load.texture), with its own space; 0 is none. |
| `onTrack?` | `(e: UiEvent) => void` | Track or Stop tracking. |
| `onLocate?` | `(e: UiEvent) => void` | Show on map. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `long`.
