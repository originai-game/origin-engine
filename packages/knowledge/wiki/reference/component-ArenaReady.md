<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaReady"></a>
### `ArenaReady`

A ready check: the players asked and their answers, the time left as the game words it, and Accept and Decline for the local player. It never starts or expires a timer; the game closes the check.

`import { ArenaReady } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/ready.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Ready check. |
| `detail?` | `string` | Match or reason line; empty hides it. |
| `timeText?` | `string` | Game-owned time remaining; empty hides it. This component never starts or expires a timer. |
| `players?` | `ReadonlyArray<ReadyPlayer>` | The players asked. |
| `response?` | `string` | Local response: ready, declined or pending (empty). Both buttons remain available to change a response while canRespond. |
| `canRespond?` | `boolean` | The game allows a local response. False disables both actions. Default true. |
| `closed?` | `boolean` | The check has ended; no more responses. timeText is hidden. |
| `loading?` | `boolean` | Waiting for participants; stale rows are hidden and responses disabled. |
| `onAccept?` | `(e: UiEvent) => void` | Accept. |
| `onDecline?` | `(e: UiEvent) => void` | Decline. |

Each `ReadyPlayer`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `state?` | `string` | ready, declined, offline, or pending (empty). Order belongs to the game. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `selected`, `disabled`, `closed`, `overflow`.
