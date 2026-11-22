<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `items` guide: Items: containers, stacks, pickups, crafting and loot

How og.systems.inventory keeps items: containers of slots, stacks limited by the item table, moves that are all or nothing, pickups taken through a sensor, crafting and loot tables. Read before building a backpack, a chest, a shop, a crafting bench or loot drops. The item rows themselves are tables (`data`); sensors are `physics` §5; the screens are `panels/backpack`, `panels/crafting` and `panels/loot`.

The API: `og api show inventory`.

<a id="guide-items-1"></a>
### `items §1` Containers and stacks

List the systems in the game, and keep the items as a table (`data`) whose
component names a stack limit:

```ts
const Item = defineComponent("Item", { maxStack: u32(), name: str(32) });
og.game({ systems: [...og.systems.inventory, ...mySystems] });
```

- An **og.Container** has `slots` slot entities, placed as its children on
  the tick it appears; a move outside a system (GameDesc.start, a command)
  places them at once. Each slot is an **og.ItemStack**: a TableRow `item`
  and a `count`, 0 for an empty slot. og.inventory.slots lists them in
  slot order: map them to a backpack's tiles.
- A stack holds at most its row's limit: the field of the item's component
  that the container's `stack` names (`stack: "maxStack"`). A row whose
  field is 0, or a container with no `stack`, holds none of it: there is no
  default, and two containers may name different fields (a quiver of 99
  arrows, a bag of 20).
- og.inventory.add fills the stacks of that row in slot order, then the
  empty slots; remove takes from the last slot back. move takes from one
  slot into a container (another, or its own), split moves part of a stack
  into the first empty slot, merge moves as much as fits from one slot onto
  another (what a dropped tile does: the backpack reports `onMove`). Every
  one plans the slots it changes and writes them only when the whole move
  fits: refused, both containers are as they were.
- Lowering `slots` keeps the slots past it, holding what they hold; they
  take nothing new.
- A system that moves items lists og.Container and og.ItemStack in its
  writes; one that rolls loot lists og.Rng.

Typical numbers: 20 to 40 slots for a backpack, stack limits of 1 for gear,
20 to 99 for materials and consumables.

<a id="guide-items-2"></a>
### `items §2` Pickups, crafting and loot

- An **og.Pickup** is an og.ItemStack lying in the world with a sensor
  og.Collider on its entity. The **og.Collector** inside it (its own
  collider) takes it into its `container`, whole, or with `partial` as much
  as fits; og.PickedUp says how many, and an empty pickup is despawned.
  Several collectors inside one pickup are served in entity order.
- og.inventory.craft takes the inputs and gives the outputs as one move:
  short of an input, or without room for an output once the inputs are
  gone, it changes nothing and names why. The recipes are the game's (a
  table of them, read as a component with lists of TableRow and counts).
- og.inventory.roll draws a loot table on og.Rng: an entry per draw in
  proportion to its weight, a count from its min to its max. The same seed
  draws the same loot; put it where the game wants with add, or spawn it
  as pickups.

<a id="guide-items-3"></a>
### `items §3` Recipes

A pouch of 3 slots that holds 5 of a gem a stack: 16 gems do not fit, 15
do, and a ring crafted from two gems has no slot to go to, so nothing changes.

```ts recipe
import { TableRow, defineComponent, defineSystem, og, u32 } from "@origingame/engine";
import type { Entity, Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const Item = defineComponent("Item", { maxStack: u32(), gem: TableRow() });

const ITEMS = `type table
version 1
---
row "gem" {
  maxStack 5
}
row "ring" {
  maxStack 1
}
`;

const row = (world: World, id: string) => must(og.table.row(world, "items.ogt", id));

const Check = defineSystem("Check", { reads: [], writes: [og.Container, og.ItemStack] }, (_q, _time, world) => {
  const pouch = world.find("pouch") as Entity;
  const gem = row(world, "gem");
  og.assert.always(og.inventory.count(world, pouch, gem) + 2 * og.inventory.count(world, pouch, row(world, "ring")) === 15, "gems are kept", {});
  if (og.inventory.count(world, pouch, gem) === 15) {
    // Three slots, all gems: no room for the ring, so nothing changes.
    og.assert.always(!og.inventory.craft(world, pouch, { inputs: [{ item: gem, count: 2 }], outputs: [{ item: row(world, "ring"), count: 1 }] }).ok, "no room for the ring", {});
    must(og.inventory.remove(world, pouch, gem, 5));
    must(og.inventory.add(world, pouch, gem, 5));
    og.assert.reachable("the pouch was full and refused the craft", {});
  }
});

export default og.game({
  systems: [...og.systems.inventory, Check],
  start(world) {
    must(og.table.parse(world, "items.ogt", ITEMS, Item));
    const pouch = must(world.spawn({ name: "pouch" }));
    must(world.insert(pouch, og.Container, { slots: 3, stack: "maxStack" }));
    const gem = row(world, "gem");
    og.assert.always(!og.inventory.add(world, pouch, gem, 16).ok, "16 gems do not fit in 3 stacks of 5", {});
    must(og.inventory.add(world, pouch, gem, 15));
  },
});
```
