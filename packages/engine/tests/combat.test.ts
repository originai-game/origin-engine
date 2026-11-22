// og.systems.combat on the native host, tick by tick, with the
// combat-arena fixture's game (a light-light-heavy combo into a dummy).
//
// Ways it could fail, and the check that catches each:
// - a link taken outside [open, close), or a press consumed by a closed
//   link → the early press chains exactly on the window's first tick, the
//   late one never starts light 2
// - a hit's damage taken from the playing move rather than the swing's →
//   light 1's damage lands although the game cut it before the hit was read
// - the cancel window read off by one → cancelable on cancelOpen, not before
// - knockback towards the attacker → Damaged.knockback points from the
//   fighter to the dummy
// - a hit stop advancing moves → tick frozen at time scale 0
// - state outside the components → a restore mid-swing ends on the same hash
import { beforeAll, expect, test } from "bun:test";
import { og, type Entity, type World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const project = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "combat-arena");
const fixture = (await import(join(project, "src", "main.ts"))) as {
  default: unknown;
  facts: (world: World) => Record<string, number>;
  LIGHT: { startup: number; active: number; recovery: number; cancelOpen: number };
  WINDOW: { open: number; close: number };
};

beforeAll(() => {
  loadNativeHost();
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function arena() {
  const engine = must(createEngine(fixture.default as GameToken, { root: project }));
  const world = engine.world;
  const fighter = world.find("fighter")!;
  const attack = () => must(world.get(fighter, og.Attack));
  const moveName = () => ["light1", "light2", "heavy"].find((n) => world.find(n) === attack().move) ?? "none";
  const key = (k: string, action: "down" | "up") => engine.probe.input({ kind: "key", key: k, action } as never);
  // A press in one tick: down before it, up before the next.
  const tap = (k: string) => {
    key(k, "down");
    must(engine.advance(1));
    key(k, "up");
  };
  /** Advance until light 1 has played `t` ticks (tick 0 is the tick it started). */
  const startLight = () => {
    must(engine.advance(2));
    tap("KeyJ");
    expect(moveName()).toBe("light1");
    expect(attack().tick).toBe(0);
  };
  const until = (t: number) => {
    while (attack().tick < t) must(engine.advance(1));
    expect(attack().tick).toBe(t);
  };
  return { engine, world, attack, moveName, tap, startLight, until };
}

test("a press one tick before the window chains on its first tick; one at its close does not", () => {
  const { engine, attack, moveName, tap, startLight, until } = arena();
  startLight();
  until(fixture.WINDOW.open - 2);
  tap("KeyJ"); // light 1's tick open - 1: buffered
  expect(moveName()).toBe("light1");
  expect(attack().tick).toBe(fixture.WINDOW.open - 1);
  must(engine.advance(1));
  expect(moveName()).toBe("light2");
  expect(attack().tick).toBe(0);

  // light 2's window closes at WINDOW.close: a press there waits in its
  // buffer and starts no heavy (a new combo once idle is the starter's).
  until(fixture.WINDOW.close - 1);
  tap("KeyJ");
  expect(attack().tick).toBe(fixture.WINDOW.close);
  for (let i = 0; i < 40; i++) {
    must(engine.advance(1));
    expect(moveName()).not.toBe("heavy");
  }
  engine.destroy();
});

test("a hit read after its move was cut still takes its damage, knocked back away from the fighter", () => {
  const { engine, world, attack, startLight, until } = arena();
  const dummy = world.find("dummy")!;
  const damaged: { amount: number; knockback: { z: number }; move: Entity }[] = [];
  startLight();
  until(fixture.LIGHT.startup); // its first active tick: the blade strikes
  expect(attack().phase).toBe("active");
  // The game cuts the move before the hit is read.
  must(world.set(world.find("fighter")!, og.Attack, { move: 0 as Entity }));
  for (let i = 0; i < 3; i++) {
    must(engine.advance(1));
    for (const d of world.events(og.Damaged)) damaged.push(d as never);
  }
  expect(attack().phase).toBe("idle");
  expect(attack().swingMove).toBe(world.find("light1")!);
  expect(damaged.length).toBe(1);
  expect(damaged[0]!.move).toBe(world.find("light1")!);
  expect(damaged[0]!.amount).toBe(10);
  expect(damaged[0]!.knockback.z).toBeGreaterThan(0);
  expect(must(world.get(dummy, og.Health)).value).toBe(30);
  // Cut, its blade is quiet: nothing more strikes.
  must(engine.advance(10));
  expect(must(world.get(dummy, og.Health)).value).toBe(30);
  engine.destroy();
});

test("cancelable from cancelOpen; a hit stop holds the move", () => {
  const { engine, world, attack, startLight, until } = arena();
  startLight();
  until(fixture.LIGHT.startup);
  expect(attack().phase).toBe("active");
  expect(attack().cancelable).toBe(false);
  until(fixture.LIGHT.cancelOpen - 1);
  expect(attack().cancelable).toBe(false);
  until(fixture.LIGHT.cancelOpen);
  expect(attack().cancelable).toBe(true);
  must(og.time.setScale(world, 0));
  must(engine.advance(5));
  expect(attack().tick).toBe(fixture.LIGHT.cancelOpen);
  must(og.time.setScale(world, 1));
  must(engine.advance(1));
  expect(attack().tick).toBe(fixture.LIGHT.cancelOpen + 1);
  engine.destroy();
});

test("a restore mid-swing ends on the hash of the run that did not save", () => {
  const { engine, world, tap, startLight, until } = arena();
  startLight();
  until(fixture.LIGHT.startup + 1); // active, its hit not read yet
  const save = must(world.snapshot());
  tap("KeyJ");
  must(engine.advance(60));
  const straight = { hash: engine.hash(), facts: fixture.facts(world) };
  expect(straight.facts.damaged).toBeGreaterThan(0);
  must(world.restore(save));
  tap("KeyJ");
  must(engine.advance(60));
  expect({ hash: engine.hash(), facts: fixture.facts(world) }).toEqual(straight);
  engine.destroy();
});
