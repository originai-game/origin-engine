<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `inventory` module

The records og.systems.inventory carries out, and the moves a game makes with them: an og.Container holds `slots` slot entities (its children, each an og.ItemStack of a table row and a count), a stack holds at most its row's stack limit, read from the item table, and every move, split, merge and craft is all or nothing. An og.Pickup lying in the world is taken by the og.Collector that enters its sensor. Every number is the game's: a container of 0 slots holds nothing.

- **TypeScript**: functions are the namespace `og.inventory`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module](index.md).

```ogt
module inventory {
  doc "The records og.systems.inventory carries out, and the moves a game makes with them: an og.Container holds `slots` slot entities (its children, each an og.ItemStack of a table row and a count), a stack holds at most its row's stack limit, read from the item table, and every move, split, merge and craft is all or nothing. An og.Pickup lying in the world is taken by the og.Collector that enters its sensor. Every number is the game's: a container of 0 slots holds nothing."
  # declarations follow, each on its own page
```

<sub>Module `inventory` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.Container`](inventory/record-Container.md) | component |  | Holds items in `slots` slot entities: og.systems.inventory places them as its children (og.Parent), each an og.ItemStack numbered by its slot, on the tick it appears; a move outside a system (GameDes… |
| [`og.ItemStack`](inventory/record-ItemStack.md) | component |  | Some of one table row: a slot of an og.Container (a child of it) or, with og.Pickup, a pickup lying in the world. |
| [`og.Pickup`](inventory/record-Pickup.md) | component |  | An og.ItemStack lying in the world, with a sensor og.Collider on its entity: og.systems.inventory moves it into the og.Container of an og.Collector inside the sensor, in entity order, and despawns it… |
| [`og.Collector`](inventory/record-Collector.md) | component |  | What takes an og.Pickup whose sensor it is inside (its entity's collider): into `container`. |
| [`og.PickedUp`](inventory/record-PickedUp.md) | event |  | An og.Collector took some of an og.Pickup: sent by og.systems.inventory the tick it was moved. |
| [`og.InventoryEntities`](inventory/record-InventoryEntities.md) | resource |  | Every og.Container and og.Pickup in entity order, written by og.systems.inventory's first systems each tick for its last. |
| [`ItemCount`](inventory/type-ItemCount.md) | struct |  | Some of one row. |
| [`LootEntry`](inventory/type-LootEntry.md) | struct |  | One line of a loot table. |
| [`RollDesc`](inventory/type-RollDesc.md) | struct |  | What og.inventory.roll draws. |
| [`CraftDesc`](inventory/type-CraftDesc.md) | struct |  | What og.inventory.craft takes and gives. |
| [`og.inventory.slots`](inventory/fn-slots.md) | function | `(world: World, container: Entity): readonly Entity[]` | A container's slots in slot order (og.ItemStack entities, empty ones too): what a backpack screen lists. |
| [`og.inventory.count`](inventory/fn-count.md) | function | `(world: World, container: Entity, item: TableRow): number` | How many of a row a container holds, over all its stacks. |
| [`og.inventory.canFit`](inventory/fn-canFit.md) | function | `(world: World, container: Entity, item: TableRow, count: number): boolean` | Whether `count` of a row fit in a container now: onto its stacks of that row up to the row's stack limit, then into its empty slots. |
| [`og.inventory.add`](inventory/fn-add.md) | function | `(world: World, container: Entity, item: TableRow, count: number): Result<void>` | Put `count` of a row in a container: onto its stacks of that row in slot order up to the stack limit, then into its empty slots in order. |
| [`og.inventory.remove`](inventory/fn-remove.md) | function | `(world: World, container: Entity, item: TableRow, count: number): Result<void>` | Take `count` of a row from a container: from its stacks of that row from the last slot back, so the first slots stay full. |
| [`og.inventory.move`](inventory/fn-move.md) | function | `(world: World, slot: Entity, to: Entity, count: number): Result<void>` | Move `count` from a slot into another container (or its own): onto that container's stacks of the row, then into its empty slots. |
| [`og.inventory.split`](inventory/fn-split.md) | function | `(world: World, slot: Entity, count: number): Result<Entity>` | Move `count` of a slot's stack into the first empty slot of its container. |
| [`og.inventory.merge`](inventory/fn-merge.md) | function | `(world: World, from: Entity, into: Entity): Result<number>` | Move as much of one slot into another as fits: into an empty slot, or onto a stack of the same row up to its limit; the rest stays. |
| [`og.inventory.craft`](inventory/fn-craft.md) | function | `(world: World, container: Entity, desc: CraftDesc): Result<void>` | Take the inputs from a container and put the outputs in it, as one move: refused, changing nothing, when an input is short or the outputs do not fit once the inputs are taken. |
| [`og.inventory.roll`](inventory/fn-roll.md) | function | `(world: World, desc: RollDesc): readonly ItemCount[]` | Draw from a loot table on og.Rng: each draw picks an entry in proportion to its weight (og.Rng.weighted) and a count from min to max (og.Rng.integer). |
