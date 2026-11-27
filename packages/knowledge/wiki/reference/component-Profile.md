<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Profile"></a>
### `Profile`

A player profile: the avatar with level and presence, the name, handle, rank and bio, the stats the game formats and the actions it offers (friend, invite, report). Private stats, loading, an error and no profile each have their state.

`import { Profile } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/profile.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | Player display name; empty uses Player. |
| `handle` | `string` | Stable player handle or discriminator, as the game words it; empty hides it. |
| `portrait` | `Texture \| 0` | Loaded portrait texture; zero uses the base Avatar fallback. |
| `level` | `number` | Level shown by Avatar; zero hides it. |
| `status` | `string` | Avatar presence: online, away, busy or offline; empty hides it. |
| `rank` | `string` | Game-formatted rank or title; empty hides it. |
| `bio` | `string` | Player biography or activity; wraps, empty hides it. |
| `stats` | `ReadonlyArray<{ id: string; label: string; value: string; detail: string }>` | Keyed { id, label, value, detail } readouts. The game formats all values and decides their order; detail is an optional explanation. |
| `actions` | `ReadonlyArray<{ id: string; label: string; disabled: boolean; reason: string; pending: boolean }>` | Keyed { id, label, disabled, reason, pending } actions, e.g. friend, invite, edit or report. action reports the stable row id; pending disables it and reads Sending. The game owns permissions and effects. |
| `privateStats?` | `boolean` | Hide statistics and show Stats are private; no private values are mounted. |
| `loading?` | `boolean` | Loading profile data; hides all stale identity, statistics and actions behind the shared Spinner. |
| `error` | `string` | Error message; hides stale data and offers retry. Empty means no error. |
| `empty?` | `boolean` | No profile found; hides stale identity, statistics and actions. |
| `onAction?` | `(e: UiEvent, key?: string) => void` | The action event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Pack slots: `Button.plate`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `private`, `pending`, `loading`, `error`, `empty`, `no-stats`, `overflow`.
