<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Inbox"></a>
### `Inbox`

A mail inbox: the messages with their sender, date and unread state, the selected message's body and its attachments to claim, and Claim and Delete. Loading, a pending claim and an error each have their state. The game owns the messages and what a claim grants.

`import { Inbox } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/inbox.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Inbox. |
| `summary?` | `string` | The game's count or capacity wording, such as 2 unread · 8 / 50. |
| `messages?` | `ReadonlyArray<{ id: string; subject: string; sender: string; date: string; preview: string; body: string; unread: boolean; hasAttachments: boolean; claimable: boolean; claimed: boolean; expired: boolean; canDelete: boolean }>` | Keyed { id, subject, sender, date, preview, body, unread, hasAttachments, claimable, claimed, expired, canDelete } messages. Text is plain, not HTML. The game updates read/claim/delete facts after intents. Default []. |
| `attachments?` | `ReadonlyArray<{ id: string; message: string; name: string; icon: string; amount: string }>` | Keyed { id, message, name, icon, amount } rewards; message is the owning message id. Default []. |
| `selected?` | `string` | Selected message id, owned by the game; empty shows a choose-message prompt. |
| `loading?` | `boolean` | Loading messages; no message actions are available. Default false. |
| `busy?` | `boolean` | A claim/delete operation is pending; prevent duplicate mutations. Default false. |
| `error?` | `string` | Loading failed; show recovery advice and retry. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onClaim?` | `(e: UiEvent, key?: string) => void` | The claim event, with the row's key where it comes from a row. |
| `onDelete?` | `(e: UiEvent, key?: string) => void` | The delete event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `empty`, `loading`, `error`, `expired`.

Actions: `choose` (Read the message by stable key), `claim` (Claim the selected message's attachments by stable message key), `delete` (Request deleting the selected message by stable key), `retry` (Retry loading messages).
