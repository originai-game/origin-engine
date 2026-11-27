<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Shop"></a>
### `Shop`

A storefront: a band with a kicker over the title and the player's balances as chips, category tabs with their counts between the bumper caps, and a grid of offer tiles. A tile is its art in a well over a glow of its rarity, corner tags (Featured, and a badge the game words), its name and detail, and one ruled foot with its price in one or more currencies (a discount's old price struck before it, and named: 700 Gems, was 1,000). The game owns the catalogue, every price and every verdict (sold out, owned, unaffordable), and says when a purchase went through (a fountain of coins from the offer's art); the shop reports buy (a tile's key, which the game answers with a ShopConfirm), choosetab (a category's key), tabNext and tabPrev (E/Q, bumpers) and retry. A tile the game refused has no buy handler: it keeps the focus so a player can read it, is marked disabled, dims its art and says why in words on its foot. The selected tile wears corner brackets. The grid fills the height it is given and scrolls inside it; narrow, it is two columns and the tabs scroll sideways.

`import { Shop } from "@origingame/engine/ui/components"` (kit `meta`, `packages/engine/src/ui/components/meta/shop.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the shop's own: it fills its parent by default; `grow-0 basis-auto` makes it hug its offers. |
| `title` | `string` | The shop's heading; empty takes the group's (Shop). |
| `kicker?` | `string` | A small line over the title naming the place (Store, a merchant's town); empty hides it. |
| `wallet` | `ReadonlyArray<{ id: string; icon: string; label: string; amount: string }>` | The player's balances shown in the band, keyed rows of { id, icon, label, amount }: amount as the game formats it; label names the currency for a screen reader. |
| `tabs` | `ReadonlyArray<{ id: string; label: string; count: number }>` | The categories, keyed rows of { id, label, count }: count, how many offers the category holds, is shown beside its label above 0; empty shows no tab row. |
| `tab` | `string` | The category shown. |
| `offers` | `ReadonlyArray<{ id: string; name: string; icon: string; image?: Texture \| 0; detail: string; rarity: string; badge: string; badgeTone: string; featured: boolean; soldOut: boolean; owned: boolean; affordable: boolean }>` | The offers in the shown category, in the game's order: keyed rows of { id, name, icon, image?, detail, rarity, badge, badgeTone, featured, soldOut, owned, affordable }: image the offer's picture filling its art well (a render of the item), 0 or absent shows icon; detail a second line (×500, 7 days); rarity common, uncommon, rare, epic, legendary or mythic, the light behind the art (empty is common); badge a corner tag the game words (−30%, New), empty none, on gold unless badgeTone is success; featured a wide tile; soldOut, owned and affordable the game's verdicts: a sold out, owned or unaffordable (affordable false) tile keeps the focus, is marked disabled, says why in words on its foot and reports nothing. |
| `prices` | `ReadonlyArray<{ id: string; offer: string; icon: string; label: string; amount: string; was: string; short: boolean }>` | Every offer's price, one row per currency: keyed rows of { id, offer, icon, label, amount, was, short }: offer an offer's id; amount and was (the struck price before a discount, empty none) as the game formats them; label the currency's name; short marks the currency the player lacks. |
| `purchases?` | `ReadonlyArray<{ id: string; offer: string }>` | The purchases the game completed, keyed rows of { id, offer }: id the tick it went through, as text; offer the offer's id. A new row throws coins from that offer's art and pops it, with the pack's purchase sound; a kept row plays once, so drop a row once it has played. Default []. |
| `selected` | `string` | The offer the focus rests on; empty is none. |
| `loading?` | `boolean` | The catalogue is on its way: a spinner stands in. |
| `error` | `string` | Why the catalogue could not load, as the game words it; empty is none. Shows a Try again button. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChoosetab?` | `(e: UiEvent, key?: string) => void` | The choosetab event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |
| `onTabNext?` | `(e: UiEvent, key?: string) => void` | The tabNext event, with the row's key where it comes from a row. |
| `onTabPrev?` | `(e: UiEvent, key?: string) => void` | The tabPrev event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Count.roll`, `List.cascade`, `List.item`, `Panel.frame`, `Plate.frame`, `Slot.frame`, `Tab.count`.

Fixtures: `default`, `purchased`, `empty`, `loading`, `error`, `overflow`.

Actions: `tabPrev` (Previous category), `tabNext` (Next category).
