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

// og.Projectile: shots that are their own owners, without an og.Attack.
// - damage taken from the owner's move only → the three shots take none
// - one owner's swing shared by every shot → the second and third do nothing
// - pierce counted on a hit that took no damage → the bullet dies on the
//   graze ring and never reaches the target
// - a spent shot left alive → it strikes the third target
// - projectiles listed as attackers → 1,500 of them throw
// - i-frames ignored for a shot → the second shot in the window takes damage
function shots(build: (world: World) => void) {
  const engine = must(createEngine(og.game({ tick: 60, seed: 1, systems: [...og.systems.combat], start: build } as never) as GameToken, {}));
  const world = engine.world;
  const damaged: { target: Entity; source: Entity; amount: number; move: Entity }[] = [];
  let hits = 0;
  const step = (n: number) => {
    for (let i = 0; i < n; i++) {
      must(engine.advance(1));
      hits += world.events(og.HitEvent).length;
      for (const d of world.events(og.Damaged)) damaged.push(d as never);
    }
  };
  return { engine, world, damaged, step, hits: () => hits };
}

/** A target of team 2 at `y`, with og.Health unless `health` is negative. */
function target(world: World, name: string, y: number, health = -1, hurt?: object): Entity {
  const e = must(world.spawn({ name, position: { x: 0, y, z: 0 } }));
  must(world.insert(e, og.HurtVolume, { shape: "sphere", radius: 0.5, team: 2 }));
  if (health >= 0) must(world.insert(e, og.Health, { value: health }));
  if (hurt) must(world.insert(e, og.Hurt, hurt));
  return e;
}

/** A shot of team 1 at `y`, as its own owner. */
function shot(world: World, name: string, y: number, projectile: object): Entity {
  const e = must(world.spawn({ name, position: { x: 0, y, z: 0 } }));
  must(world.insert(e, og.HitVolume, { shape: "sphere", radius: 0.1, active: true, team: 1 }));
  must(world.insert(e, og.Projectile, projectile));
  return e;
}

/** Moves every shot named in `names` up by `dy`. */
function fly(world: World, names: readonly string[], dy: number) {
  for (const n of names) {
    const e = world.find(n);
    if (e === null) continue;
    const t = must(world.get(e, og.Transform));
    must(world.set(e, og.Transform, { position: { ...t.position, y: t.position.y + dy } }));
  }
}

test("three projectiles through a target each strike it once", () => {
  const names = ["s0", "s1", "s2"];
  const { engine, world, damaged, step, hits } = shots((w) => {
    const ship = must(w.spawn({ name: "ship", position: { x: 0, y: -50, z: 0 } }));
    target(w, "drone", 5, 10);
    names.forEach((n, i) => shot(w, n, -i, { damage: 1, shooter: ship, pierce: 0 }));
  });
  for (let i = 0; i < 60; i++) {
    fly(world, names, 0.25);
    step(1);
  }
  expect(hits()).toBe(3);
  expect(damaged.length).toBe(3);
  expect(damaged.every((d) => d.source === world.find("ship") && d.move === 0 && d.amount === 1)).toBe(true);
  expect(must(world.get(world.find("drone")!, og.Health)).value).toBe(7);
  // Pierce 0: never spent.
  expect(names.every((n) => world.find(n) !== null)).toBe(true);
  engine.destroy();
});

test("pierce 2 through three targets strikes two, then the shot is spent", () => {
  const { engine, world, damaged, step } = shots((w) => {
    for (let i = 0; i < 3; i++) target(w, `t${i}`, 3 + 3 * i, 5);
    shot(w, "bolt", 0, { damage: 2, pierce: 2 });
  });
  const bolt = world.find("bolt")!;
  for (let i = 0; i < 80; i++) {
    fly(world, ["bolt"], 0.25);
    step(1);
  }
  expect(damaged.map((d) => d.target)).toEqual([world.find("t0")!, world.find("t1")!]);
  expect(damaged[0]!.source).toBe(bolt); // no shooter: the projectile itself
  expect(world.find("bolt")).toBeNull();
  expect(must(world.get(world.find("t2")!, og.Health)).value).toBe(5);
  engine.destroy();
});

test("a strike that takes no damage spends nothing: through a graze volume, then into a target", () => {
  const { engine, world, damaged, step, hits } = shots((w) => {
    target(w, "graze", 3); // no og.Health
    target(w, "core", 7, 4);
    shot(w, "bullet", 0, { damage: 1, pierce: 1 });
  });
  const bullet = world.find("bullet")!;
  let afterGraze = -1;
  for (let i = 0; i < 60 && world.find("bullet") !== null; i++) {
    fly(world, ["bullet"], 0.25);
    step(1);
    if (hits() === 1 && afterGraze < 0 && damaged.length === 0) afterGraze = must(world.get(bullet, og.Projectile)).struck;
  }
  expect(afterGraze).toBe(0);
  expect(hits()).toBe(2);
  expect(damaged.map((d) => d.target)).toEqual([world.find("core")!]);
  expect(world.find("bullet")).toBeNull();
  engine.destroy();
});

test("a second projectile inside the target's invulnerability does nothing and passes on", () => {
  const { engine, world, damaged, step } = shots((w) => {
    target(w, "drone", 3, 10, { invulnerableTicks: 30 });
    shot(w, "a", 0, { damage: 1, pierce: 1 });
    shot(w, "b", -1, { damage: 1, pierce: 1 });
  });
  for (let i = 0; i < 40; i++) {
    fly(world, ["a", "b"], 0.25);
    step(1);
  }
  expect(damaged.length).toBe(1);
  expect(must(world.get(world.find("drone")!, og.Health)).value).toBe(9);
  expect(world.find("a")).toBeNull();
  // Blocked by the i-frames, it is not spent.
  expect(must(world.get(world.find("b")!, og.Projectile)).struck).toBe(0);
  engine.destroy();
});

test("1,500 live projectiles do not throw, and each strikes", () => {
  const { engine, world, damaged, step } = shots((w) => {
    target(w, "boss", 0, 10_000);
    for (let i = 0; i < 1500; i++) shot(w, "", 0, { damage: 1, pierce: 1 });
  });
  step(3);
  expect(damaged.length).toBe(1500);
  expect(must(world.get(world.find("boss")!, og.Health)).value).toBe(8500);
  expect(must(world.resource(og.CombatAttackers)).attackers.length).toBe(0);
  engine.destroy();
});
