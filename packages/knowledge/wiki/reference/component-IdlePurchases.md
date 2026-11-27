<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IdlePurchases"></a>
### `IdlePurchases`

The upgrade shop of an idle game: a ×1 / ×10 / ×100 / Max choice of how many to buy (chooseamount, the amount's key), then one row per upgrade with its icon, name, effect, level, how many the chosen amount buys and what that costs, and Buy (buy, the upgrade's key). The game prices every amount and decides what can be bought: Buy is enabled only when the row is affordable and neither maxed nor locked, and each refused state is a word (Not enough, Maxed, Locked). It fills its height and scrolls.

`import { IdlePurchases } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/purchases.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The list's heading; empty takes the group's (Upgrades). |
| `amounts?` | `ReadonlyArray<{ id: string; label: string }>` | The buy amounts offered, keyed rows of { id, label }: label the amount as the game words it (it wins; empty reads the key max as Max and any other key as ×key: ×1, ×10, ×100). An empty list hides the choice. |
| `amount` | `string` | The buy amount chosen, one of the amounts' keys. |
| `upgrades?` | `ReadonlyArray<{ id: string; icon: string; name: string; effect: string; level: number; count: string; cost: string; costIcon: string; affordable: boolean; maxed: boolean; locked: boolean }>` | The purchases in the game's order, keyed rows of { id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }: effect what it does as the game words it; level its level (0 hides it); count how many the chosen amount buys, as the game words it (×37; empty hides it); cost the price of that many, formatted; costIcon the currency's icon; affordable the player can pay it (else named Not enough); maxed nothing more to buy (named Maxed); locked not yet available (named Locked). Buy is offered only when affordable and neither maxed nor locked. |
| `onBuy?` | `(e: UiEvent, key?: string) => void` | The buy event, with the row's key where it comes from a row. |
| `onChooseamount?` | `(e: UiEvent, key?: string) => void` | The chooseamount event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`, `Slot.frame`, `Tab.indicator`.

Fixtures: `default`, `max`, `disabled`, `maxed`, `empty`, `overflow`.
