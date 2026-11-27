<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaSpectator"></a>
### `ArenaSpectator`

A spectator's controls: the subjects the game offers to follow, the one followed, previous and next, free camera and Leave. It reports intents; the game moves the camera.

`import { ArenaSpectator } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/spectator.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Spectating. |
| `detail?` | `string` | Match status, broadcast delay or disabled reason, as the game words it. |
| `targets?` | `ReadonlyArray<SpectatorTarget>` | The subjects the game offers. follow reports their stable id. |
| `selected?` | `string` | The followed subject id. Empty means no chosen subject; freeCamera takes precedence over its selected state. |
| `freeCamera?` | `boolean` | Free camera active; choosing a subject only reports follow, leaving the game to change camera mode. |
| `canFree?` | `boolean` | Show and enable the free camera action; false omits it. Default true. |
| `canCycle?` | `boolean` | Enable previous and next. The game owns cycling order and wrapping. Default true. |
| `canLeave?` | `boolean` | Enable Leave; false keeps the exit visible but disabled. Default true. |
| `loading?` | `boolean` | Waiting for subjects; hides stale rows and disables camera actions, but not Leave. |
| `onFollow?` | `(e: UiEvent, id: string) => void` | Follow a subject. |
| `onPrevious?` | `(e: UiEvent) => void` | Previous subject. |
| `onNext?` | `(e: UiEvent) => void` | Next subject. |
| `onFree?` | `(e: UiEvent) => void` | Free camera. |
| `onLeave?` | `(e: UiEvent) => void` | Leave. |

Each `SpectatorTarget`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | What a reader calls them. |
| `team?` | `string` | Their side as the game words it; empty hides it. |
| `detail?` | `string` | A line about them (Alive, Respawning · 4s, Disconnected). |
| `disabled?` | `boolean` | The game decides which subjects can be followed. |

Pack slots: `Button.plate`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `free`, `disabled`, `loading`, `empty`, `overflow`.
