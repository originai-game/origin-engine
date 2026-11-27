<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaVote"></a>
### `ArenaVote`

A map or mode vote: the options, the local vote, the time left and, once closed, the winner the game names. It reports onChoose with the option's id; the game counts the votes.

`import { ArenaVote } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/vote.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Map vote. |
| `detail?` | `string` | Voting rules or a disabled reason, supplied by the game. |
| `timeText?` | `string` | Game-owned time remaining; empty hides it. Hidden once closed. |
| `options?` | `ReadonlyArray<VoteOption>` | The choices. choose reports the stable id, never the index. |
| `selected?` | `string` | The local vote's option id; empty means no vote. A click reports choose but does not change it locally. |
| `winner?` | `string` | The winning option id, supplied by the game. Shown only when closed; empty means no announced result. |
| `canVote?` | `boolean` | Whether the local player can vote or change a vote. Default true. |
| `closed?` | `boolean` | Voting is over; options remain readable but disabled. |
| `loading?` | `boolean` | Waiting for options; hides stale options and disables voting. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose an option. |

Each `VoteOption`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What the map or choice is called. |
| `icon?` | `string` | An icon of the style's atlas; empty is none. |
| `detail?` | `string` | A line about it. |
| `votes?` | `number` | The votes so far; the game counts them. |
| `disabled?` | `boolean` | It cannot be chosen. |
| `reason?` | `string` | Why it cannot be chosen (Not installed). |

Pack slots: `Panel.frame`.

Fixtures: `default`, `closed`, `disabled`, `loading`, `empty`, `overflow`.
