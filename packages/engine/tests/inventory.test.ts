// og.systems.inventory on the native host, with the inventory-chest
// fixture's game (a walker collecting pickups into a 4-slot bag beside a
// 3-slot chest, items.ogt's stack limits).
//
// Ways it could fail, and the check that catches each:
// - a move that changes one side before finding the other has no room →
//   a refused move and a refused craft leave the world hash unchanged
// - split or merge that loses or doubles items → counts before and after
// - slots kept outside the components → a restore ends on the hash of the
//   run that did not save, and every slot is its container's child again
// - loot drawn off og.Rng's stream → two engines with one seed roll the same
// - a lowered slot count that drops or fills slots past it → they keep
//   what they hold and take nothing new
import { beforeAll, expect, test } from "bun:test";
import { og, type Entity, type World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const project = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "inventory-chest");
const fixture = (await import(join(project, "src", "main.ts"))) as typeof import("../../../tests/fixtures/inventory-chest/src/main.ts");

beforeAll(() => {
  loadNativeHost();
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function chest() {
  const engine = must(createEngine(fixture.default as GameToken, { root: project }));
  const world = engine.world as World;
  const row = (id: string) => must(og.table.row(world, "items.ogt", id));
  const bag = world.find("bag")! as Entity;
  const box = world.find("chest")! as Entity;
  const step = (n = 1) => must(engine.advance(n));
  const stacks = (c: Entity) => og.inventory.slots(world, c).map((s) => must(world.get(s, og.ItemStack))).map(({ item, count }) => ({ item, count }));
  // Past the last pickup (x = 2 at tick 50): the bag is full.
  step(60);
  return { engine, world, row, bag, box, step, stacks };
}

test("a refused move and a refused craft change nothing, down to the world hash", () => {
  const { engine, world, row, bag, box } = chest();
  expect(og.inventory.count(world, bag, row("ore"))).toBe(20);
  const hash = engine.hash();
  const sword = og.inventory.slots(world, box)[0]!;
  const moved = og.inventory.move(world, sword, bag, 1);
  expect(moved.ok).toBe(false);
  const craft = og.inventory.craft(world, bag, { inputs: [{ item: row("ore"), count: 2 }], outputs: [{ item: row("ingot"), count: 1 }] });
  expect(craft.ok ? "" : craft.reason).toContain("do not fit");
  const short = og.inventory.craft(world, bag, { inputs: [{ item: row("ore"), count: 21 }], outputs: [] });
  expect(short.ok ? "" : short.reason).toContain("short");
  expect(og.inventory.add(world, box, row("ore"), 17).ok).toBe(false); // 6 on the stack, 10 in the empty slot
  expect(og.inventory.canFit(world, box, row("ore"), 16)).toBe(true);
  expect(engine.hash()).toBe(hash);
  engine.destroy();
});

test("split and merge keep every item; merge stops at the stack limit", () => {
  const { engine, world, row, box, stacks } = chest();
  const ore = og.inventory.slots(world, box)[1]!;
  const to = must(og.inventory.split(world, ore, 3));
  expect(stacks(box)).toEqual([{ item: row("sword"), count: 1 }, { item: row("ore"), count: 1 }, { item: row("ore"), count: 3 }]);
  expect(og.inventory.split(world, ore, 1).ok).toBe(false); // a stack of 1 does not split
  expect(must(og.inventory.merge(world, to, ore))).toBe(3);
  expect(stacks(box)[1]).toEqual({ item: row("ore"), count: 4 });
  must(og.inventory.add(world, box, row("ore"), 9)); // the stack to 10, 3 in the last slot
  expect(must(og.inventory.merge(world, og.inventory.slots(world, box)[2]!, ore))).toBe(0);
  expect(must(og.inventory.merge(world, og.inventory.slots(world, box)[2]!, og.inventory.slots(world, box)[0]!))).toBe(0); // another row
  expect(og.inventory.count(world, box, row("ore"))).toBe(13);
  engine.destroy();
});

test("a restore ends on the hash of the run that did not save, every slot its container's child", () => {
  const { engine, world, row, bag, box, step } = chest();
  const save = must(world.snapshot());
  const finish = () => {
    must(og.inventory.move(world, og.inventory.slots(world, bag)[2]!, box, 10));
    must(og.inventory.craft(world, bag, { inputs: [{ item: row("ore"), count: 2 }], outputs: [{ item: row("ingot"), count: 1 }] }));
    step(5);
    return { hash: engine.hash(), facts: fixture.facts(world) };
  };
  const straight = finish();
  expect(straight.facts).toMatchObject({ bagOre: 8, chestOre: 14, bagIngot: 1 });
  must(world.restore(save));
  expect(og.inventory.count(world, bag, row("ore"))).toBe(20);
  for (const c of [bag, box]) for (const s of og.inventory.slots(world, c)) expect(must(world.get(s, og.Parent)).entity).toBe(c);
  expect(finish()).toEqual(straight);
  engine.destroy();
});

test("one seed rolls one loot; a zero weight is never drawn", () => {
  const roll = () => {
    const { engine, world, row } = chest();
    const loot = og.inventory.roll(world, {
      rolls: 20,
      entries: [
        { item: row("coin"), weight: 3, min: 1, max: 10 },
        { item: row("ore"), weight: 1, min: 2, max: 2 },
        { item: row("sword"), weight: 0, min: 1, max: 1 },
      ],
    });
    const out = { loot, coin: row("coin"), ore: row("ore") };
    engine.destroy();
    return out;
  };
  const a = roll();
  expect(a.loot.length).toBe(20);
  expect(new Set(a.loot.map((l) => l.item))).toEqual(new Set([a.coin, a.ore]));
  for (const l of a.loot) expect(l.item === a.ore ? l.count === 2 : l.count >= 1 && l.count <= 10).toBe(true);
  expect(roll().loot).toEqual(a.loot);
});

test("a lowered slot count keeps the slots past it, which take nothing new", () => {
  const { engine, world, row, box, stacks } = chest();
  must(world.set(box, og.Container, { slots: 1 }));
  expect(stacks(box)).toEqual([{ item: row("sword"), count: 1 }, { item: row("ore"), count: 4 }, { item: 0 as never, count: 0 }]);
  expect(og.inventory.add(world, box, row("ore"), 1).ok).toBe(false);
  must(og.inventory.remove(world, box, row("ore"), 4));
  expect(og.inventory.count(world, box, row("ore"))).toBe(0);
  engine.destroy();
});
