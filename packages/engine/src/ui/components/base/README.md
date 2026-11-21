# base: built-in components

Small building blocks for items, identity, state and progress: slots,
avatars, currency, pips, empty and error states, list rows, steps,
spinners, skeletons and ribbons. They display game-owned data; they do not
consume items, advance cooldowns, grant levels or infer presence. A state
is worded as well as coloured (presence under the name, a step's state, a
locked slot's word).

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `Avatar` | `base/avatar` | A portrait in a round well. |
| `Currency` | `base/currency` | A resource readout (icon, name, amount, capacity, rate), not a button; it sends nothing. |
| `EmptyState` | `base/empty-state` | Where content would be, there is a recess. |
| `ErrorState` | `base/error-state` | EmptyState's recess with the failure on a disc tinted and ringed in danger (quieter than the action it leads to). |
| `ItemSlot` | `base/item-slot` | An item in a square slot: its icon, count, key, cooldown sweep and a rim in its rarity's colour; reports choose. |
| `ListItem` | `base/list-item` | One line of a list. |
| `Pips` | `base/pips` | A counted resource on one line. |
| `Ribbon` | `base/ribbon` | A strip across a corner of whatever it lies over (or a stamp across its middle). |
| `SkeletonGroup` | `base/skeleton-group` | One band of light crosses the whole group. |
| `Spinner` | `base/spinner` | A ring that is always there and an arc that runs round it. |
| `Steps` | `base/steps` | A rail of numbered marks. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only base/` draws every member's fixtures under every pack.
