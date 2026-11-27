<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-LootLabel"></a>
### `LootLabel`

An item on the ground: over it a dark chip with the tier's strip on its leading edge and the name in the tier's ink (a stack's count after it); epic, legendary and mythic raise a beam. Reports pick (the chip); the game owns what is picked up.

`import { LootLabel } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/loot-label.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | The item, by its bits (0: use at). |
| `at?` | `string` | A world point it lies at without an entity (x y z). |
| `height?` | `number` | World units above it the label stands. |
| `name?` | `string` | The item's name, in its tier's ink; a long one scrolls on one line. |
| `count?` | `number` | A stack's size, shown as ×n from 2. |
| `rarity?` | `string` | Its tier: common (empty), uncommon, rare, epic or legendary: the strip on the chip's leading edge, the name's ink and the glow on the ground; epic and legendary raise a beam, a legendary wears a gold wash and rim and shimmers every 2.4 s. |
| `hovered?` | `boolean` | Aimed at or under the pointer (the chip takes :hover too): it lifts 1.06 times, its rim goes all round and the pick-up key shows. |
| `action?` | `string` | The input action whose bound key shows on a hovered chip (og.input.defineAction); empty shows none. |
| `more?` | `number` | A chip standing for this many more items the game folded into one (+n), with a dot per tier in tiers; 0 is an item's own chip. |
| `tiers?` | `ReadonlyArray<LootTier>` | The folded items' tiers, keyed rows of { id, rarity }, a dot each on a +n chip. |
| `onPick?` | `(e: UiEvent) => void` | The chip is pressed: the game owns what is picked up. |
| `maxDistance?` | `number` | Maximum world distance before hiding. |
| `scaleAt?` | `number` | Distance at which the label is drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |

Each `LootTier`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `rarity?` | `string` |  |

Pack slots: `Chip.frame`.

Fixtures: `default`, `common`, `stack`, `uncommon`, `rare`, `legendary`, `hovered`, `more`, `overflow`.
