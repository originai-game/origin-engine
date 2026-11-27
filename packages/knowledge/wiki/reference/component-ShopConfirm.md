<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShopConfirm"></a>
### `ShopConfirm`

The purchase confirmation the game shows after a shop's buy: a small heading over the offer's name, its art in a socket of its rarity beside its detail, and a ledger with one group per currency: the price (a discount's old price struck before it, and named: 700 Gems, was 1,000), then the balance before and after, or, for a currency the player lacks, Not enough and how much more it takes. The game decides whether it may be bought and what it costs; the panel reports confirm (Buy) and cancel (Cancel, or Escape and pad B anywhere in it). Enter and pad A press the focused button, Cancel first, so no key buys by accident. A refused purchase disables Buy and says why (the short currency's row, or the game's reason over the buttons); a failed one shows the game's error on a strip and leaves Buy to try again. While busy Buy is disabled. The ledger scrolls when it must; the buttons stay at the foot. It is content, not a dialog: the host that shows it (the built-in confirm screen, a sheet at the foot of a phone, or the game's own modal) owns modality and focus.

`import { ShopConfirm } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/shop-confirm.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading, a small line over the offer's name; empty takes the group's (Confirm purchase). |
| `name` | `string` | The offer's name. |
| `icon` | `string` | The offer's icon in the style's or the game's atlas. |
| `rarity?` | `string` | common, uncommon, rare, epic, legendary or mythic: the socket the icon sits in; empty is common. |
| `detail` | `string` | What the offer gives, as the game words it; empty hides it. |
| `prices` | `ReadonlyArray<{ id: string; icon: string; label: string; amount: string; was: string; after: string; short: boolean; missing: string }>` | What it costs, one group of rows per currency: keyed rows of { id, icon, label, amount, was, after, short, missing }: amount, was (struck, empty none) and after (the balance before and after, 3,200 → 2,500; empty hides it) as the game formats them; label the currency's name; short marks a currency the player lacks, and missing how much more it takes (1,600), shown in place of the balance. |
| `canBuy?` | `boolean` | The game allows the purchase; false disables Buy. |
| `reason?` | `string` | Why the purchase is refused when no price says it, as the game words it (Requires level 40); shown over the buttons while canBuy is false. |
| `busy?` | `boolean` | The purchase is in flight: Buy is disabled and says it is working. |
| `error` | `string` | Why the last attempt failed, as the game words it; empty is none. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Slot.frame`.

Fixtures: `default`, `disabled`, `refused`, `loading`, `error`, `overflow`.

Actions: `back` (Cancel).
