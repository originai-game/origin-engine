<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyQueue"></a>
### `StrategyQueue`

A production queue as one row: what produces in small capitals, the order in production as a larger tile with a dark sweep for what is still to run and its time left, the orders waiting as smaller tiles (a stack's count on a tag), the free places as empty tiles, the count over the limit, and Paused, Blocked with the reason, or Queue full as words. Reports cancel (a tile's key); the game owns costs, times, refunds and the limit.

`import { StrategyQueue } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/queue.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | What produces, as the game words it (Barracks queue); empty takes the kit's (Production). |
| `items?` | `ReadonlyArray<{ id: string; name: string; picture: Texture \| 0; icon: string; count: number; progress: number; timeText: string; state: string; reason: string }>` | The queue in the game's order, first in production, keyed rows of { id, name, picture, icon, count, progress, timeText, state, reason }: name for a reader; picture a texture (og.load.texture), 0 draws icon from the atlas; count a stack of the same order (×2, shown above 1); progress 0 to 1 done, a dark sweep over the first order's art for what is still to run; timeText its time left as the game words it (0:08), on the first order's tile, empty none; state building, waiting, paused or blocked (data-state; paused and blocked are also words after the tiles); reason why it is blocked (Need 3 supply), empty none. |
| `queued` | `number` | Orders queued, as the game counts them (stacks included), shown over limit on a tag. |
| `limit` | `number` | The most the queue holds: its free places are drawn as empty tiles after the orders; 0 hides the count and the free places. |
| `full` | `boolean` | The game says no more can be queued: the Queue full word on the count's tag and data-full on the frame. |
| `onCancel?` | `(e: UiEvent, id: string) => void` | Cancel the order (a slot's key). |

Pack slots: `Button.plate`, `Chip.frame`.

Fixtures: `default`, `blocked`, `empty`, `full`, `paused`, `overflow`.
