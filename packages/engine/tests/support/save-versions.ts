// One version of a game for tests/save-versions.test.ts, a process each (a
// bundle declares its types once): `--version 1` writes a save of its world,
// `--version 2` restores one through its migration and prints what the
// world holds, `--version 2 --built` builds that same state directly.
//
// Version 1 keeps Wallet { gold } and Stats { hp, mp } and a Run resource
// { score }; version 2 renamed `gold` to `coins`, split Stats into Health
// and Mana, keeps Run's score in tenths and adds a chest the player is owed.
//
//   bun tests/support/save-versions.ts --version 1|2 [--built] [--save <file>]
//     [--fail] [--keep-stats] [--dump] [--write]
//
// --fail adds a step that refuses; --keep-stats keeps Stats (no split);
// --dump prints every type's records; --write prints a save of the world;
// `--version 1 --save` restores a (newer) save into the version 1 game.

import { readFileSync } from "node:fs";
import { og, defineComponent, defineResource, f32, u32 } from "@origingame/engine";
import type { Entity, Result, SaveEdit, World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

(await import("../../src/node/index.ts")).loadNativeHost();
const argv = process.argv.slice(2);
const opt = (name: string) => (argv.includes(name) ? (argv[argv.indexOf(name) + 1] ?? "") : "");
const version = Number(opt("--version"));
const out = (v: unknown) => process.stdout.write(JSON.stringify(v) + "\n");

if (version === 1) {
  const Wallet = defineComponent("Wallet", { gold: u32() });
  const Stats = defineComponent("Stats", { hp: f32(), mp: f32() });
  const Run = defineResource("Run", { score: u32() });
  const game = og.game({
    saveVersion: 1,
    resources: [Run],
    start: (world: World) => {
      for (let i = 0; i < 4; i++) {
        const e = world.spawn({ name: `p${i}`, bare: true });
        if (!e.ok) throw new Error(e.reason);
        world.insert(e.value, Wallet, { gold: 100 + i });
        if (i % 2 === 0) world.insert(e.value, Stats, { hp: 10 + i, mp: 20 + i });
      }
      const run = world.resource(Run);
      if (run.ok) run.value.score = 42;
    },
  });
  const engine = createEngine(game as unknown as GameToken);
  if (!engine.ok) throw new Error(engine.reason);
  engine.value.advance(3);
  if (opt("--save")) {
    // A save of a later version than this game's: refused, nothing changed.
    const before = engine.value.hash();
    const restored = engine.value.world.restore(readFileSync(opt("--save"), "utf8"));
    out({ restored, unchanged: engine.value.hash() === before });
    process.exit(0);
  }
  const save = engine.value.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  out({ save: save.value });
} else {
  const Wallet = defineComponent("Wallet", { coins: u32() });
  // --keep-stats: version 2 keeps Stats as it was (no split), so every
  // step keeps which types sit together and the built world's archetypes
  // are the migrated one's: the two hash the same.
  const keep = argv.includes("--keep-stats");
  const Stats = keep ? defineComponent("Stats", { hp: f32(), mp: f32() }) : null;
  const Health = defineComponent("Health", { hp: f32() });
  const Mana = defineComponent("Mana", { mp: f32() });
  const Chest = defineComponent("Chest", { gold: u32() });
  const Run = defineResource("Run", { tenths: u32() });
  const fail = argv.includes("--fail");
  const migrate = (save: SaveEdit): Result<void> => {
    const steps = [
      () => save.renameField("Wallet", "gold", "coins"),
      () => (keep ? ({ ok: true as const, value: undefined }) : save.split("Stats", (v) => ({ Health: { hp: v.hp }, Mana: { mp: v.mp } }))),
      () => save.transform("Run", (v) => ({ tenths: (v.score as number) * 10 })),
      () => save.spawn({ name: "chest", components: { Chest: { gold: 5 } } }),
      () => (fail ? { ok: false as const, reason: "the test's migration refuses" } : { ok: true as const, value: undefined }),
    ];
    for (const step of steps) {
      const r = step() as Result<unknown>;
      if (!r.ok) return r;
    }
    return { ok: true, value: undefined };
  };
  const built = argv.includes("--built");
  const game = og.game({
    saveVersion: 2,
    resources: [Run],
    migrations: [{ from: 1, migrate }],
    start: (world: World) => {
      if (!built) return;
      for (let i = 0; i < 4; i++) {
        const e = world.spawn({ name: `p${i}`, bare: true });
        if (!e.ok) throw new Error(e.reason);
        world.insert(e.value, Wallet, { coins: 100 + i });
        if (i % 2 === 0 && Stats) world.insert(e.value, Stats, { hp: 10 + i, mp: 20 + i });
        else if (i % 2 === 0) {
          world.insert(e.value, Health, { hp: 10 + i });
          world.insert(e.value, Mana, { mp: 20 + i });
        }
      }
      const chest = world.spawn({ name: "chest", bare: true });
      if (chest.ok) world.insert(chest.value, Chest, { gold: 5 });
      const run = world.resource(Run);
      if (run.ok) run.value.tenths = 420;
    },
  });
  const engine = createEngine(game as unknown as GameToken);
  if (!engine.ok) throw new Error(engine.reason);
  const world = engine.value.world;
  let restored: Result<void> = { ok: true, value: undefined };
  // The built world runs to the saved tick (3), as the restored one is at it.
  if (built) engine.value.advance(3);
  else {
    const before = engine.value.hash();
    restored = world.restore(readFileSync(opt("--save"), "utf8"));
    if (!restored.ok) {
      out({ restored, unchanged: engine.value.hash() === before });
      process.exit(0);
    }
  }
  const of = (name: string) => {
    const e = world.find(name) as Entity;
    const get = (c: typeof Wallet | typeof Health | typeof Mana | typeof Chest) => {
      const v = world.get(e, c);
      return v.ok ? v.value : null;
    };
    const stats = Stats && world.get(e, Stats);
    return { wallet: get(Wallet), health: get(Health), mana: get(Mana), chest: get(Chest), ...(stats ? { stats: stats.ok ? stats.value : null } : {}) };
  };
  const run = world.resource(Run);
  out({
    restored,
    entities: ["p0", "p1", "p2", "p3", "chest"].map(of),
    run: run.ok ? { tenths: run.value.tenths } : null,
    tick: world.tick,
    hash: engine.value.hash(),
    ...(argv.includes("--dump") ? { dump: dump(world) } : {}),
    ...(argv.includes("--write") ? { save: world.snapshot() } : {}),
  });
}

function dump(world: World) {
  const snap = world.snapshot();
  if (!snap.ok) return snap.reason;
  const e = world.openSave(snap.value);
  if (!e.ok) return e.reason;
  const out: Record<string, unknown> = {};
  for (const t of e.value.types()) out[t.name] = e.value.records(t.name);
  e.value.close();
  return out;
}
