<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Friends"></a>
### `Friends`

The friends panel: a band with the title and Add friend, the friend requests with Accept and Decline, then the friends with an avatar, the name, the status in words beside a mark (online, in game, away, offline), what they are doing and Invite. The game owns the list, presence and whether an invite may go; the panel reports add, accept and decline (a request's key), choose and invite (a friend's key). It fills its height and scrolls inside it.

`import { Friends } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/friends.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Friends). |
| `requests` | `ReadonlyArray<{ id: string; name: string; avatar: string; note: string }>` | Friend requests waiting for an answer: keyed rows of { id, name, avatar, note }: avatar an icon of the style's atlas; note a second line as the game words it (3 mutual friends), empty hides it. Empty hides the section. |
| `friends` | `ReadonlyArray<{ id: string; name: string; avatar: string; status: string; activity: string; canInvite: boolean; invited: boolean }>` | The friends in the game's order: keyed rows of { id, name, avatar, status, activity, canInvite, invited }: status online, ingame, away or offline, shown in words and by a mark; activity what they are doing (Ranked match, Last seen 2 d ago), empty hides it; canInvite the game will send an invite; invited one is pending. |
| `selected` | `string` | The friend the game marks as chosen (data-selected, aria-current); empty is none. |
| `onAccept?` | `(e: UiEvent, key?: string) => void` | The accept event, with the row's key where it comes from a row. |
| `onAdd?` | `(e: UiEvent, key?: string) => void` | The add event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onDecline?` | `(e: UiEvent, key?: string) => void` | The decline event, with the row's key where it comes from a row. |
| `onInvite?` | `(e: UiEvent, key?: string) => void` | The invite event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `requests`, `overflow`.
