// og.systems.interact on the native host, tick by tick, with the
// interact-house fixture's game (two rooms, a door on a navmesh link, two
// levers, a crate and a pressure plate).
//
// Ways it could fail, and the check that catches each:
// - a hold counted from the tick after the press, or used a tick late →
//   29 held ticks leave the door shut, the 30th opens it
// - a hold that survives letting go → progress back to 0 the tick after
// - the rules mixed up, or ties not broken by distance → nearest, centred
//   and priority each pick a different target from one place
// - a candidate behind a collider focused → the hidden lever never is
// - the carry outside the components → a restore while carrying ends on
//   the hash of the run that did not save
import { beforeAll, expect, test } from "bun:test";
import { og, type World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const project = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "interact-house");
const fixture = (await import(join(project, "src", "main.ts"))) as {
  default: unknown;
  facts: (world: World) => Record<string, number>;
  HOLD: number;
};

beforeAll(() => {
  loadNativeHost();
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function house() {
  const engine = must(createEngine(fixture.default as GameToken, { root: project }));
  const world = engine.world;
  const player = world.find("player")!;
  const facts = () => fixture.facts(world);
  const key = (k: string, action: "down" | "up") => engine.probe.input({ kind: "key", key: k, action } as never);
  const step = (n = 1) => must(engine.advance(n));
  // A press in one tick, and a tick up after it so the next is a new press.
  const tap = (k: string) => {
    key(k, "down");
    step();
    key(k, "up");
    step();
  };
  step(2);
  return { engine, world, player, facts, key, step, tap };
}

test("a hold of 29 ticks leaves the door shut; the 30th tick opens it and its link", () => {
  const { engine, facts, key, step } = house();
  expect(facts().focus).toBe(1);
  key("KeyE", "down");
  step(); // the press tick is the first held
  expect(facts().progress).toBe(1);
  step(fixture.HOLD - 2);
  expect(facts().progress).toBe(fixture.HOLD - 1);
  key("KeyE", "up");
  step();
  expect(facts()).toMatchObject({ progress: 0, doorOpen: 0, interacted: 0 });

  key("KeyE", "down");
  step(fixture.HOLD - 1);
  expect(facts()).toMatchObject({ progress: fixture.HOLD - 1, doorOpen: 0 });
  step();
  expect(facts()).toMatchObject({ progress: 0, doorOpen: 1 });
  key("KeyE", "up");
  step(); // the fixture counts og.Interacted the tick after it is sent
  expect(facts().interacted).toBe(1);
  // The guard plans through the open link and crosses it.
  step(240);
  expect(facts()).toMatchObject({ linkReached: 1, guardWest: 1 });
  engine.destroy();
});

test("nearest, centred and priority pick different targets from one place; one out of sight never", () => {
  const { engine, world, player, facts, step } = house();
  // Every direction: the crate (2 m), the lever (2.5 m), the door (2.6 m) and the hidden lever (2.5 m, behind the pillar).
  must(world.set(player, og.Interactor, { cone: 0, rule: "nearest" }));
  step();
  expect(facts().focus).toBe(4);
  must(world.set(player, og.Interactor, { rule: "centred" }));
  step();
  expect(facts().focus).toBe(1);
  must(world.set(world.find("lever")!, og.Interactable, { priority: 5 }));
  must(world.set(world.find("hiddenLever")!, og.Interactable, { priority: 9 }));
  must(world.set(player, og.Interactor, { rule: "priority" }));
  step();
  expect(facts().focus).toBe(2);
  // Without sight, the hidden lever's priority wins.
  must(world.set(player, og.Interactor, { sight: false }));
  step();
  expect(facts().focus).toBe(3);
  // A zero range focuses nothing.
  must(world.set(player, og.Interactor, { range: 0 }));
  step();
  expect(facts().focus).toBe(0);
  engine.destroy();
});

test("a restore while carrying the crate ends on the hash of the run that did not save", () => {
  const { engine, world, facts, step, tap } = house();
  for (let i = 0; i < 3; i++) tap("KeyD"); // south: the crate
  expect(facts().focus).toBe(4);
  tap("KeyE");
  step(10);
  expect(facts().holding).toBe(1);
  const save = must(world.snapshot());
  const finish = () => {
    tap("KeyE");
    step(90);
    return { hash: engine.hash(), facts: facts() };
  };
  const straight = finish();
  expect(straight.facts).toMatchObject({ holding: 0, plateOn: 1, gateOpen: 1 });
  must(world.restore(save));
  expect(facts().holding).toBe(1);
  expect(finish()).toEqual(straight);
  engine.destroy();
});
