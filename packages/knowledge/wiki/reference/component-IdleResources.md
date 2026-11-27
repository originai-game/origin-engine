<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IdleResources"></a>
### `IdleResources`

The resource bar of an idle game: one cut chip per currency with its art in a socket, its name, the amount large and the income under it, green when it rises. A store at its cap gets a gold rim and a Full tag. The game owns every number: a formatted value or rate it passes wins, else the bare number is compacted (12.3K) and counts to each new amount. While loading each chip keeps its art and name and shows a placeholder, which a pack may animate; the row's name says it is loading. It reports no intents. The chips wrap onto more lines when they do not fit.

`import { IdleResources } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/resources.tsx`).

| Prop | Type | What |
|---|---|---|
| `currencies?` | `ReadonlyArray<{ id: string; icon: string; name: string; amount: number; value: string; rate: string; perSecond: number; full: boolean }>` | The currencies in the game's order, keyed rows of { id, icon, name, amount, value, rate, perSecond, full }: value the amount as the game formats it (it wins; empty shows amount, a number, compacted: 12.3K, counting to each new amount); rate the income as the game words it (+1.2K/s; it wins; empty shows perSecond, a number per second, compacted; 0 hides it); perSecond also sets data-trend (up, down, flat) whether or not rate is given, so pass it with a worded rate too; full the store is at its cap, named Full. |
| `loading?` | `boolean` | The amounts are on their way: each row keeps its icon and name and shows a placeholder for its figures. |

Pack slots: `Count.roll`, `Slot.frame`.

Fixtures: `default`, `loading`, `full`, `draining`, `empty`, `overflow`.
