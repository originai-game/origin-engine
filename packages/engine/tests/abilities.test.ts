// og.systems.abilities on the native host, with the abilities fixture's
// game (a hero with a dash of two charges, a fireball that costs mana,
// armour and speed stats).
//
// Ways it could fail, and the check that catches each:
// - a recharge counted from the tick after the use, or one late → ready on
//   tick T + R exactly, not T + R - 1
// - both charges returning at once → one back after R, the other after 2R
// - cooldowns counting through a hit stop → held while it lasts
// - modifiers applied in the order they came in → every insertion order of
//   an add, a percent and a multiply gives one value, and so do adds whose
//   float sum depends on their order
// - a tie under strongest replacing the first, or maxStacks 0 adding
//   stacks → the first stays with the second's ticks; no stack is added
// - state outside the components → a restore ends on the hash of the run
//   that did not save
import { beforeAll, expect, test } from "bun:test";
import { og, type Entity, type World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const project = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "abilities");
const fixture = (await import(join(project, "src", "main.ts"))) as typeof import("../../../tests/fixtures/abilities/src/main.ts");

beforeAll(() => {
  loadNativeHost();
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function hero() {
  const engine = must(createEngine(fixture.default as GameToken, { root: project }));
  const world = engine.world as World;
  const find = (name: string) => world.find(name)! as Entity;
  const step = (n = 1) => must(engine.advance(n));
  const ability = (name: string) => must(world.get(find(name), og.Ability));
  const value = (e: Entity) => must(world.get(e, og.Stat)).value;
  step(2);
  return { engine, world, find, step, ability, value };
}

test("a charge is back R ticks after its use, the next R after that, and a hit stop holds them", () => {
  const { engine, world, find, step, ability } = hero();
  const dash = find("dash");
  must(og.abilities.use(world, dash));
  expect(og.abilities.use(world, dash).ok).toBe(false); // its 12-tick lock
  step(12);
  must(og.abilities.use(world, dash));
  expect(ability("dash").charges).toBe(0);
  const used = og.abilities.use(world, dash);
  expect(used.ok ? "" : used.reason).toContain("no charge");
  step(47); // 59 ticks since the first use
  expect(ability("dash")).toMatchObject({ charges: 0, rechargedTicks: 59 });
  step();
  expect(ability("dash")).toMatchObject({ charges: 1, rechargedTicks: 0 });
  // A hit stop of 30 ticks: nothing counts while it lasts.
  must(og.time.hitStop(world, 30));
  step(30);
  expect(ability("dash")).toMatchObject({ charges: 1, rechargedTicks: 0 });
  step(59);
  expect(ability("dash").charges).toBe(1);
  step();
  expect(ability("dash")).toMatchObject({ charges: 2, rechargedTicks: 0 });
  expect(og.abilities.ready(world, dash)).toBe(true);
  engine.destroy();
});

test("a use under its cost is refused and changes nothing", () => {
  const { engine, world, find, step, value } = hero();
  must(world.set(find("mana"), og.Stat, { base: 19 }));
  step();
  const hash = engine.hash();
  const r = og.abilities.use(world, find("fireball"));
  expect(r.ok ? "" : r.reason).toContain("under its cost of 20");
  expect(engine.hash()).toBe(hash);
  expect(value(find("mana"))).toBe(19);
  engine.destroy();
});

test("modifiers give one value whatever order they came in", () => {
  const orders = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
  ];
  const values = (mods: [op: "add" | "percent" | "multiply", amount: number][]) =>
    orders.map((order) => {
      const { engine, world, step, value } = hero();
      const stat = must(world.spawn({ bare: true }));
      must(world.insert(stat, og.Stat, { base: 4 }));
      for (const i of order) {
        const m = must(world.spawn({ bare: true }));
        must(world.insert(m, og.Modifier, { stat, op: mods[i]![0], amount: mods[i]![1] }));
      }
      step();
      const v = value(stat);
      engine.destroy();
      return v;
    });
  // (4 + 3) × (1 + 0.5) × (1 + 1): adds, then percents, then multiplies.
  expect(values([["multiply", 1], ["add", 3], ["percent", 0.5]])).toEqual(Array(6).fill(21));
  // Summed in the order they came, 4 + 1e30 - 1e30 + 1 is 1 or 0 or 5; ascending it is always 0.
  expect(values([["add", 1e30], ["add", -1e30], ["add", 1]])).toEqual(Array(6).fill(0));
});

test("strongest keeps the first on a tie with the second's ticks; stack with maxStacks 0 adds none", () => {
  const { engine, world, find, step } = hero();
  const put = (kind: string, desc: object) => {
    const s = must(world.spawn({ parent: find("hero"), bare: true }));
    must(world.insert(s, og.Status, { kind, stacks: 1, ...desc }));
    return s;
  };
  const first = put("ward", { stacking: "strongest", strength: 4, ticks: 10 });
  step();
  const tie = put("ward", { stacking: "strongest", strength: 4, ticks: 50 });
  const capped = put("bleed", { stacking: "stack", maxStacks: 0, ticks: 30 });
  step();
  put("bleed", { stacking: "stack", maxStacks: 0, ticks: 40 });
  step();
  expect(world.alive(tie)).toBe(false);
  expect(must(world.get(first, og.Status))).toMatchObject({ strength: 4, ticks: 48 });
  expect(must(world.get(capped, og.Status))).toMatchObject({ stacks: 1, ticks: 39 });
  engine.destroy();
});

test("a restore mid-recharge with statuses on ends on the hash of the run that did not save", () => {
  const { engine, world, find, step } = hero();
  must(og.abilities.use(world, find("dash")));
  must(og.abilities.use(world, find("fireball")));
  const poison = must(world.spawn({ parent: find("hero"), bare: true }));
  must(world.insert(poison, og.Status, { kind: "poison", stacking: "stack", ticks: 40, stacks: 2, maxStacks: 3 }));
  const m = must(world.spawn({ parent: poison, bare: true }));
  must(world.insert(m, og.Modifier, { stat: find("armour"), op: "add", amount: -2 }));
  step(20);
  const save = must(world.snapshot());
  const finish = () => {
    step(100);
    return { hash: engine.hash(), facts: fixture.facts(world) };
  };
  const straight = finish();
  expect(straight.facts).toMatchObject({ dashCharges: 2, armour: 10, statuses: 0 });
  must(world.restore(save));
  expect(fixture.facts(world)).toMatchObject({ armour: 6, poisonStacks: 2 });
  expect(finish()).toEqual(straight);
  engine.destroy();
});
