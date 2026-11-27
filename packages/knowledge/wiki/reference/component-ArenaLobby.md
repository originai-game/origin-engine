<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaLobby"></a>
### `ArenaLobby`

A match lobby: its heading and detail line, the seats in the game's order (open seats offer Invite), the local player's Ready up or Cancel ready, and Start match for the host. The game decides who is ready and when the match can start.

`import { ArenaLobby } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/lobby.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The lobby's heading (the match's name); empty takes the kit's (Lobby). |
| `detail?` | `string` | A line beside the heading as the game words it (Ranked · 2 v 2, Waiting for the host); empty hides it. |
| `slots?` | `ReadonlyArray<LobbySlot>` | The lobby's seats in the game's order. |
| `canInvite?` | `boolean` | Open seats offer Invite (false: they read Open slot only). |
| `isReady?` | `boolean` | The local player is ready: the button reads Cancel ready (unready) instead of Ready up (ready). |
| `canReady?` | `boolean` | The ready button is offered (false: disabled). |
| `isHost?` | `boolean` | The local player hosts: Start match is shown. |
| `canStart?` | `boolean` | Start match is offered (false: disabled; the game decides when the match can start). |
| `onInvite?` | `(e: UiEvent, id: string) => void` | Invite to an open seat. |
| `onReady?` | `(e: UiEvent) => void` | Ready up. |
| `onUnready?` | `(e: UiEvent) => void` | Cancel ready. |
| `onStart?` | `(e: UiEvent) => void` | The host starts the match. |

Each `LobbySlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The seat's key, reported back with invite. |
| `name?` | `string` | The seat's player; empty when open. |
| `portrait?` | `Texture \| 0` | The seat's portrait texture (0 shows the icon). |
| `icon?` | `string` | The seat's player's icon, shown while no portrait is set. |
| `open?` | `boolean` | The seat is free. |
| `host?` | `boolean` | The seat's player hosts. |
| `ready?` | `boolean` | The seat's player is ready, in words (Ready or Not ready). |
| `local?` | `boolean` | The seat is the local player's. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.muted`, `Row.frame.selected`.

Fixtures: `default`, `full`, `guest`, `disabled`, `overflow`.
