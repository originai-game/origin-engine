<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Reveal"></a>
### `Reveal`

A reward reveal: a kicker naming the source over the title; sealed, the prize large over a gold glow with the game's words under it and Open; revealing, a spinner and Skip; revealed, each reward in a socket of its rarity (its count in a corner, a dot when it is new), each bursting in as it shows with a reveal of its rarity's light, with the rarity in words over its name, and Continue. Rewards stay hidden until the game says they are revealed; a reward is a button that reports inspect with its key. The game owns the roll, the timing and the rewards; the panel reports open, skip, inspect, continue and retry.

`import { Reveal } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/reveal.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Rewards. |
| `kicker?` | `string` | A small line over the title naming where the rewards come from (Expedition rewards); empty hides it. |
| `body?` | `string` | Optional explanation under the sealed prize. |
| `icon?` | `string` | The sealed prize's icon or picture (a chest, a pouch); empty uses the bag. |
| `state?` | `string` | Game-owned sealed/loading/revealing/revealed. No roll, timer or reward mutation runs here. Default "sealed". |
| `rewards?` | `ReadonlyArray<{ id: string; name: string; icon: string; amount: string; rarity: string; fresh: boolean }>` | Keyed { id, name, icon, amount, rarity, fresh } rewards, shown only once revealed. rarity is common/uncommon/rare/epic/legendary/mythic, in words and as the socket's colour; amount (×3) sits in the socket's corner; fresh marks a new one with a dot. Default []. |
| `canSkip?` | `boolean` | Offer skip while revealing; the game decides what skipping does. Default true. |
| `continueLabel?` | `string` | Continue wording; empty uses Continue. |
| `disabled?` | `boolean` | Prevent opening, skipping, inspecting and continuing. Default false. |
| `error?` | `string` | A failed reveal with retry; rewards stay hidden until the game resolves it. |
| `onContinue?` | `(e: UiEvent, key?: string) => void` | The continue event, with the row's key where it comes from a row. |
| `onInspect?` | `(e: UiEvent, key?: string) => void` | The inspect event, with the row's key where it comes from a row. |
| `onOpen?` | `(e: UiEvent, key?: string) => void` | The open event, with the row's key where it comes from a row. |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Slot.frame`.

Fixtures: `default`, `loading`, `revealing`, `revealed`, `empty`, `error`.

Actions: `open` (Open the sealed rewards), `skip` (Finish the reveal early), `inspect` (Inspect a revealed reward by its key), `continue` (Continue after the reveal), `retry` (Retry the reveal).
