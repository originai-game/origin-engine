// Saves that survive game and engine updates: a game's own save version and
// migrations (GameDesc.saveVersion, .migrations, SaveEdit), the engine's
// refusal of obsolete engine envelopes,
// and selections (SnapshotDesc.with / .resources).
//
// Ways it could fail, and the check that catches each:
// - a renamed field restored at its default (the balance zeroed) → `coins`
// - a split type dropped, or a value not transformed → `health`, `mana`, `run`
// - a failing migration half-applied → `unchanged` and its reason
// - a migrated world that is not the one the new version builds → its hash
// - a save of a newer game version taken as if it were current → refused
// - an obsolete engine envelope mutating the world → explicit refusal
// - a selection that touches what it did not save → the untagged entity
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineResource, defineSystem, u32 } from "@origingame/engine";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import game, { Run, Wallet, start } from "./fixtures/saves/game.ts";

const here = dirname(fileURLToPath(import.meta.url));
const tmp = mkdtempSync(join(tmpdir(), "og-save-versions-"));

beforeAll(() => {
  loadNativeHost();
});

/** tests/support/save-versions.ts in its own process (a bundle declares its types once). */
function version(...args: string[]): Record<string, any> {
  const r = Bun.spawnSync(["bun", join(here, "support", "save-versions.ts"), ...args], { cwd: join(here, "..") });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr) + out);
  return JSON.parse(out.trim().split("\n").at(-1)!);
}

function v1Save(): string {
  const file = join(tmp, "v1.save");
  writeFileSync(file, version("--version", "1").save);
  return file;
}

test("a version 1 save restores through the game's migration: renamed, split, transformed, added", () => {
  const r = version("--version", "2", "--save", v1Save(), "--dump");
  expect(r.restored).toEqual({ ok: true, value: null });
  expect(r.tick).toBe(3);
  // gold → coins kept every balance; Stats split into Health and Mana on
  // the entities that had it; Run's score in tenths; the chest added.
  expect(r.entities).toEqual([
    { wallet: { coins: 100 }, health: { hp: 10 }, mana: { mp: 20 }, chest: null },
    { wallet: { coins: 101 }, health: null, mana: null, chest: null },
    { wallet: { coins: 102 }, health: { hp: 12 }, mana: { mp: 22 }, chest: null },
    { wallet: { coins: 103 }, health: null, mana: null, chest: null },
    { wallet: null, health: null, mana: null, chest: { gold: 5 } },
  ]);
  expect(r.run).toEqual({ tenths: 420 });
  // Record for record the world version 2 builds from the same state.
  const built = version("--version", "2", "--built", "--dump");
  expect(r.dump).toEqual(built.dump);
});

test("a migrated world hashes as the one the new version builds, and the same save migrates to the same world", () => {
  // Without the split every step keeps which types sit together, so the
  // two worlds are the same to the byte (a split puts Health and Mana in
  // one archetype at once, which inserts one at a time never do).
  const migrated = version("--version", "2", "--keep-stats", "--save", v1Save());
  const built = version("--version", "2", "--keep-stats", "--built");
  expect(migrated.restored.ok).toBeTrue();
  expect(migrated.entities[0].stats).toEqual({ hp: 10, mp: 20 });
  expect(migrated.hash).toBe(built.hash);
  // Restoring the same save twice migrates to the same world.
  expect(version("--version", "2", "--save", v1Save()).hash).toBe(version("--version", "2", "--save", v1Save()).hash);
});

test("a failing migration refuses the restore with its reason and leaves the world as it was", () => {
  const r = version("--version", "2", "--save", v1Save(), "--fail");
  expect(r.restored).toEqual({ ok: false, reason: "the save migration from version 1 refused: the test's migration refuses" });
  expect(r.unchanged).toBeTrue();
});

test("a save of a newer game version is refused by the older game", () => {
  const file = join(tmp, "v2.save");
  const v2 = version("--version", "2", "--built", "--write");
  writeFileSync(file, v2.save.value);
  const r = version("--version", "1", "--save", file);
  expect(r.restored.ok).toBeFalse();
  expect(r.restored.reason).toContain("save version 2");
  expect(r.unchanged).toBeTrue();
});

test("an old engine save is refused without conversion or mutation", () => {
  const save = readFileSync(join(here, "fixtures", "saves", "0.7.7.save"), "utf8");
  const made = createEngine(game as unknown as GameToken);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  try {
    const before = engine.hash();
    expect(engine.world.restore(save)).toEqual({
      ok: false,
      reason: "save format 1 is incompatible with engine save format 3; save migration is not supported",
    });
    expect(engine.hash()).toBe(before);
  } finally { engine.destroy(); }
});

test("a selection saves and restores only the tagged entities and the chosen resources", () => {
  const Keep = defineComponent("Keep", {});
  const Settings = defineResource("Settings", { volume: u32() });
  const Score = defineResource("Score", { points: u32() });
  const made = createEngine(og.game({ tick: 60, seed: 7, resources: [Run, Settings, Score], start }) as unknown as GameToken);
  if (!made.ok) throw new Error(made.reason);
  const e = made.value;
  const w = e.world;
  const kept = w.spawn({ name: "hero", bare: true });
  if (!kept.ok) throw new Error(kept.reason);
  w.insert(kept.value, Keep, {});
  w.insert(kept.value, Wallet, { gold: 7 });
  const settings = w.resource(Settings);
  const score = w.resource(Score);
  if (!settings.ok || !score.ok) throw new Error("resources");
  settings.value.volume = 3;
  score.value.points = 11;
  // A name given between ticks reaches the world when the next tick starts.
  e.advance(1);
  const save = w.snapshot({ with: [Keep], resources: [Settings] });
  if (!save.ok) throw new Error(save.reason);
  // A whole-world save of the same world is larger: a selection holds no
  // coin, no body, no UI.
  const whole = w.snapshot();
  if (!whole.ok) throw new Error(whole.reason);
  expect(save.value.length).toBeLessThan(whole.value.length);

  // Changed after the save: the hero, both resources, an untagged coin.
  w.insert(kept.value, Wallet, { gold: 99 });
  settings.value.volume = 9;
  score.value.points = 50;
  const coin = w.find("coin-1")!;
  w.insert(coin, Wallet, { gold: 1234 });
  e.advance(2);

  expect<unknown>(w.restore(save.value)).toEqual({ ok: true, value: null });
  const hero = w.find("hero")!;
  const wallet = w.get(hero, Wallet);
  expect(wallet.ok && wallet.value.gold).toBe(7);
  const settingsNow = w.resource(Settings);
  if (!settingsNow.ok) throw new Error(settingsNow.reason);
  expect(settingsNow.value.volume).toBe(3);
  // What the selection did not hold stays as it is now.
  const scoreNow = w.resource(Score);
  if (!scoreNow.ok) throw new Error(scoreNow.reason);
  expect(scoreNow.value.points).toBe(50);
  const c = w.get(coin, Wallet);
  expect(c.ok && c.value.gold).toBe(1234);
});

test("a save holds the names given since the last tick began, between ticks and by an earlier system", () => {
  // A restore drops the names still queued for the world, so a name not in
  // the save is lost for good.
  const Mark = defineComponent("NamesMark", { n: u32() });
  let mid = "";
  const Spawner = defineSystem("NamesSpawner", { writes: [Mark] }, (_q, time, world) => {
    if (time.tick === 2) world.spawn({ name: "made-in-system", bare: true });
  });
  const Saver = defineSystem("NamesSaver", { reads: [Mark], after: ["NamesSpawner"] }, (_q, time, world) => {
    if (time.tick === 2) {
      const s = world.snapshot();
      if (s.ok) mid = s.value;
    }
  });
  const named = og.game({ tick: 60, seed: 7, systems: [Spawner, Saver] });
  const make = () => {
    const made = createEngine(named as unknown as GameToken);
    if (!made.ok) throw new Error(made.reason);
    return made.value;
  };
  const live = make();
  const hero = live.world.spawn({ name: "between", bare: true });
  if (!hero.ok) throw new Error(hero.reason);
  const between = live.world.snapshot();
  if (!between.ok) throw new Error(between.reason);
  live.advance(4);
  expect(mid).not.toBe("");

  const a = make();
  expect(a.world.restore(between.value).ok).toBeTrue();
  a.advance(1);
  expect(a.world.find("between")).toBe(hero.value);

  // Restored mid-tick, the next step completes tick 2: the system's
  // entity is in the save with its name, as in the world that saved it.
  const b = make();
  b.advance(1);
  expect(b.world.restore(mid).ok).toBeTrue();
  b.advance(2);
  expect(b.world.find("made-in-system")).not.toBeNull();
  expect(b.world.find("made-in-system")).toBe(live.world.find("made-in-system"));
  // From the next tick on it is the world that saved (at the restoring
  // tick itself the session's input-action block is the restoring engine's).
  b.advance(1);
  live.advance(1);
  expect(b.world.tick).toBe(live.world.tick);
  expect(b.hash()).toBe(live.hash());
});
