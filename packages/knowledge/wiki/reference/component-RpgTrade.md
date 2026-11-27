<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgTrade"></a>
### `RpgTrade`

A merchant: a band with the merchant's name, the Buy / Sell switch and the purse; an alert when a deal failed; the goods as rows (the item's icon in a socket, its rarity on the row's edge and in its name, its stock or why it cannot be had, and its price, a discount struck before it); and at the foot the selected good with its total, the balance after, and Buy or Sell. The game decides every price and verdict; the panel reports choose, switchMode, buy and sell, each with the good's key.

Dropped against the .svelte: the purse coin's radial sheen (a flat gold disc stands in; radial gradients have no surface here), the socket's rarity rim (the rarity tints the socket's ground instead), and the chosen row's wash gradient and inset rim (a flat wash and the leading edge stand in; a cut element takes one bg colour and no box shadow).

`import { RpgTrade } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/trade.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The merchant or trade title; empty uses Trade. |
| `mode?` | `string` | buy or sell. The game changes this in response to switchMode and supplies that side's goods and prices. Default "buy". |
| `wallet?` | `string` | The player's balance as the game words it; empty hides it. |
| `goods?` | `ReadonlyArray<{ id: string; name: string; icon: string; stock: string; description: string; price: string; was: string; after: string; available: boolean; reason: string; rarity: string }>` | Keyed { id, name, icon, stock, description, price, was, after, available, reason, rarity }. stock, price (the total), was (the struck price before a discount, empty none) and after (the balance once paid, empty hides it) are game-formatted strings; available includes balance, capacity, quantity and legality checks; rarity (common … mythic, empty common) colours the row's edge, the socket and the name. Default []. |
| `selected?` | `string` | The inspected good's id. Unavailable goods remain selectable so their restriction can be read. |
| `loading?` | `boolean` | Goods are loading; selection and transactions are disabled. Default false. |
| `busy?` | `boolean` | A transaction is pending; selection, mode switching and transactions are disabled. Default false. |
| `error?` | `string` | The game's transaction failure, kept visible alongside the selection. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onSell?` | `(e: UiEvent, key?: string) => void` | The sell event, with the row's key where it comes from a row. |
| `onSwitchMode?` | `(e: UiEvent, key?: string) => void` | The switchMode event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `empty`, `loading`, `sell`, `unavailable`.
