<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IdleOffline"></a>
### `IdleOffline`

The welcome-back panel of an idle game: how long the player was away, large (the game's words, or its seconds as a duration), then what was gathered as sockets glowing gold from their foot, each with its art, the amount and its name. Collect reports collect; Double, the game's gold offer and shown only when it sets canDouble, reports double with what it asks under its word. The game computes the haul and applies it. Many currencies wrap and the haul scrolls.

`import { IdleOffline } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/offline.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Welcome back). |
| `away` | `string` | How long the player was away, as the game words it (it wins; empty shows seconds as a duration: 1h 2m). |
| `seconds` | `number` | How long the player was away, in seconds, when away is empty. |
| `earned?` | `ReadonlyArray<{ id: string; icon: string; name: string; amount: string }>` | What was gathered while away, keyed rows of { id, icon, name, amount }: amount as the game formats it (+12.4K). |
| `canDouble?` | `boolean` | The game offers to double the haul (an ad, a gem); false hides Double. |
| `doubleText` | `string` | What doubling costs or asks, as the game words it (Watch an ad); empty is none. |
| `onCollect?` | `(e: UiEvent, key?: string) => void` | The collect event, with the row's key where it comes from a row. |
| `onDouble?` | `(e: UiEvent, key?: string) => void` | The double event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`, `disabled`, `empty`, `overflow`.
