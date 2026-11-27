// A bag (4 slots) carried by a walker, a chest (3 slots) and three pickups
// on the walker's path (README.md). The walker crosses the pickups' sensors
// and collects them into the bag; the probe's commands move, split, merge,
// craft and roll against the two containers. Every tick the Check system
// counts every item held anywhere, in slots and lying in the world.
//
//   og run tests/fixtures/inventory-chest --headless --commands "2:dump_world;3:quit"
import { defineResource, defineSystem, og, u32 } from "@origingame/engine";
import type { Entity, Result, World } from "@origingame/engine";
import { ItemRow } from "./schema/items.ts";
export { ItemRow } from "./schema/items.ts";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** What the game read back: counts by row in each container, and the commands' answers. */
export const Facts = defineResource("ChestFacts", {
  bagCoin: u32(),
  bagOre: u32(),
  bagIngot: u32(),
  chestOre: u32(),
  chestSword: u32(),
  lying: u32(),
  picked: u32(),
  refused: u32(),
  rolled: u32(),
  rollSum: u32(),
  split: u32(),
  merged: u32(),
});

const row = (world: World, id: string) => must(og.table.row(world, "base/items", id));

/** The walker's x on tick `t`: from -3 to 3 over a second, across the pickups at -2, 0 and 2. */
export const walk = (tick: number): number => Math.min(3, -3 + 0.1 * tick);

const Walk = defineSystem("Walk", { reads: [og.Collector], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const position = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) position[i * 3] = walk(time.tick);
  }
});

/** Every coin, ore, ingot and sword held anywhere: in a slot or lying as a pickup. */
export function facts(world: World): Record<string, number> {
  const f = world.resource(Facts);
  if (!f.ok) return {};
  const v = f.value;
  return {
    bagCoin: v.bagCoin, bagOre: v.bagOre, bagIngot: v.bagIngot, chestOre: v.chestOre, chestSword: v.chestSword,
    lying: v.lying, picked: v.picked, refused: v.refused, rolled: v.rolled, rollSum: v.rollSum, split: v.split, merged: v.merged,
  };
}

const Check = defineSystem(
  "Check",
  { reads: [og.PickedUp, og.Pickup, og.Parent], writes: [Facts, og.Container, og.ItemStack] },
  (_q, _time, world) => {
    const f = must(world.resource(Facts));
    const bag = world.find("bag") as Entity;
    const chest = world.find("chest") as Entity;
    const coin = row(world, "coin");
    const ore = row(world, "ore");
    for (const e of world.events(og.PickedUp)) f.picked += e.count;
    f.bagCoin = og.inventory.count(world, bag, coin);
    f.bagOre = og.inventory.count(world, bag, ore);
    f.bagIngot = og.inventory.count(world, bag, row(world, "ingot"));
    f.chestOre = og.inventory.count(world, chest, ore);
    f.chestSword = og.inventory.count(world, chest, row(world, "sword"));
    const held = (name: string) => {
      const p = world.find(name);
      return p === null ? 0 : must(world.get(p, og.ItemStack)).count;
    };
    const lyingCoin = held("coins");
    const lyingOre = held("ore-a") + held("ore-b");
    f.lying = lyingCoin + lyingOre;
    // Nothing is made or lost but by craft: 60 coins, and ore + 2 per ingot is 24.
    og.assert.always(f.bagCoin + lyingCoin === 60, "coins are kept", { f: facts(world) });
    og.assert.always(f.bagOre + f.chestOre + 2 * f.bagIngot + lyingOre === 24, "ore is kept but by crafting", { f: facts(world) });
    for (const c of [bag, chest]) {
      for (const s of og.inventory.slots(world, c)) {
        const st = must(world.get(s, og.ItemStack));
        const limit = st.count > 0 ? must(og.table.get(world, st.item, ItemRow)).maxStack : 0;
        og.assert.always(st.count <= limit, "no stack past its row's limit", { slot: st.slot, count: st.count, limit });
        og.assert.always(must(world.get(s, og.Parent)).entity === c, "a slot is its container's child", {});
      }
    }
    og.assert.sometimes(f.picked === 80, "everything lying was picked up");
  },
);

function pickup(world: World, name: string, x: number, item: string, count: number): void {
  const e = must(world.spawn({ name, position: { x, y: 0.25, z: 0 } }));
  must(world.insert(e, og.Collider, { shape: "box", size: { x: 0.4, y: 0.4, z: 0.4 }, sensor: true }));
  must(world.insert(e, og.ItemStack, { item: row(world, item), count }));
  must(world.insert(e, og.Pickup, { partial: true }));
}

export default og.game({
  tick: 60,
  seed: 7,
  resources: [Facts],
  systems: [Walk, ...og.systems.inventory, Check],
  start(world) {
    must(og.table.load(world, "base/items", ItemRow));
    const bag = must(world.spawn({ name: "bag" }));
    must(world.insert(bag, og.Container, { slots: 4, stack: "maxStack" }));
    const chest = must(world.spawn({ name: "chest" }));
    must(world.insert(chest, og.Container, { slots: 3, stack: "maxStack" }));
    must(og.inventory.add(world, chest, row(world, "sword"), 1));
    must(og.inventory.add(world, chest, row(world, "ore"), 4));
    const walker = must(world.spawn({ name: "walker", position: { x: walk(0), y: 0.25, z: 0 } }));
    must(world.insert(walker, og.RigidBody, { kind: "kinematic" }));
    must(world.insert(walker, og.Collider, { shape: "box", size: { x: 0.5, y: 0.5, z: 0.5 } }));
    must(world.insert(walker, og.Collector, { container: bag }));
    // 60 coins (stacks of 50 and 10), then 12 ore and 8 ore (two stacks of
    // 10): the bag's four slots are full.
    pickup(world, "coins", -2, "coin", 60);
    pickup(world, "ore-a", 0, "ore", 12);
    pickup(world, "ore-b", 2, "ore", 8);
  },
  commands: {
    // The chest's sword into the full bag: no slot takes it, so neither container changes.
    overfill: (world) => {
      const chest = world.find("chest")!;
      const sword = og.inventory.slots(world, chest).find((s) => must(world.get(s, og.ItemStack)).item === row(world, "sword"))!;
      const r = og.inventory.move(world, sword, world.find("bag")!, 1);
      if (!r.ok) must(world.resource(Facts)).refused += 1;
    },
    // 6 ore from a bag stack onto the chest's 4: one stack of 10.
    deposit: (world) => {
      const bag = world.find("bag")!;
      const ore = og.inventory.slots(world, bag).find((s) => must(world.get(s, og.ItemStack)).item === row(world, "ore"))!;
      must(og.inventory.move(world, ore, world.find("chest")!, 6));
    },
    // The chest's 10 ore split 3 off into its empty slot, then merged back.
    split: (world) => {
      const chest = world.find("chest")!;
      const ore = og.inventory.slots(world, chest).find((s) => must(world.get(s, og.ItemStack)).item === row(world, "ore"))!;
      const to = must(og.inventory.split(world, ore, 3));
      must(world.resource(Facts)).split = must(world.get(to, og.ItemStack)).count;
      must(world.resource(Facts)).merged = must(og.inventory.merge(world, to, ore));
    },
    // The bag's first ore stack, whole, into the chest's empty slot.
    stash: (world) => {
      const bag = world.find("bag")!;
      const ore = og.inventory.slots(world, bag).find((s) => must(world.get(s, og.ItemStack)).item === row(world, "ore"))!;
      must(og.inventory.move(world, ore, world.find("chest")!, must(world.get(ore, og.ItemStack)).count));
    },
    // Two ore make an ingot: refused while the bag has no slot for it, done once one is free.
    smelt: (world) => {
      const bag = world.find("bag")!;
      const desc = { inputs: [{ item: row(world, "ore"), count: 2 }], outputs: [{ item: row(world, "ingot"), count: 1 }] };
      const r = og.inventory.craft(world, bag, desc);
      if (!r.ok) must(world.resource(Facts)).refused += 1;
    },
    // Five draws of a loot table on og.Rng: their sum is recorded, nothing is added.
    roll: (world) => {
      const loot = og.inventory.roll(world, {
        rolls: 5,
        entries: [
          { item: row(world, "coin"), weight: 3, min: 1, max: 10 },
          { item: row(world, "ore"), weight: 1, min: 2, max: 2 },
          { item: row(world, "sword"), weight: 0, min: 1, max: 1 },
        ],
      });
      const f = must(world.resource(Facts));
      f.rolled = loot.length;
      f.rollSum = loot.reduce((n, l) => n + l.count, 0);
      og.assert.always(loot.every((l) => l.item !== row(world, "sword")), "a zero weight is never drawn", {});
    },
  },
  facts,
});
