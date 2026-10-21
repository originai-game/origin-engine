// A game's saves on the native host (World.snapshot, World.restore), and the
// same save on both hosts (tests/support/save-round.ts, one process per host).
//
// Ways it could fail, and the test that catches each:
// - a restore does not bring the world back exactly (the hash differs), or
//   the restored world does not go on as the saved one did → "a save
//   restored between ticks hashes as the saved world …"
// - inside a system the snapshot panics on the system's own columns, the
//   restore lands mid-tick, the restored world runs the saving tick's
//   systems before the saver again (meridian-loop FINDINGS #12), or a bad
//   save is not refused at the call →
//   "inside a system a snapshot is taken at once and a restore waits …"
// - the session goes back with the save (the player's volume, the probe's
//   scheduled steps jumping with og.Time) → "a restore keeps the session …"
// - the text is not what the docs say, or garbage is restored → "a save is
//   text …"
// - the hosts write different saves for one world, or one host's save does
//   not restore on the other to the same hash, or a publish build lacks it →
//   "one world saves to the same text on both hosts …"
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

const STALE_WASM = process.env.OG_WASM_STALE === "1";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const SvPos = defineComponent("SvPos", { x: f32(), v: f32(1) });
const SvRun = defineResource("SvRun", { coins: u32(), marker: u32(), saveAt: u32(), loadAt: u32() });

let saved: string | null = null;
let bad = "";
let refusals: string[] = [];

// Moves every SvPos and draws coins from og.Rng: a world whose next ticks depend on all of it.
const SvMove = defineSystem("SvMove", { writes: [SvPos, SvRun, og.Rng] }, (q, time, world) => {
  for (const a of q.archetypes()) {
    const p = a.column(SvPos);
    for (let i = 0; i < a.count; i++) p.x[i] = p.x[i]! + p.v[i]! * time.dt;
  }
  const run = world.resource(SvRun);
  const rng = world.resource(og.Rng);
  if (run.ok && rng.ok) run.value.coins += rng.value.integer(0, 3);
});

// Saves and loads at the ticks SvRun names (0: never), from inside a system (as a game's lantern or Continue button would).
const SvSaveLoad = defineSystem("SvSaveLoad", { reads: [SvPos], writes: [SvRun], after: ["SvMove"] }, (q, time, world) => {
  for (const a of q.archetypes()) a.column(SvPos); // holds its columns while it saves
  const run = world.resource(SvRun);
  if (!run.ok) return;
  if (run.value.saveAt > 0 && run.value.saveAt === time.tick) {
    const s = world.snapshot();
    if (!s.ok) throw new Error(s.reason);
    saved = s.value;
  }
  if (run.value.loadAt > 0 && run.value.loadAt === time.tick && saved) {
    const refused = world.restore(bad);
    if (!refused.ok) refusals.push(refused.reason);
    const r = world.restore(saved);
    if (!r.ok) throw new Error(r.reason);
  }
});

const spawn = (world: World) => {
  for (let i = 0; i < 3; i++) {
    const e = world.spawn({ name: `sv-${i}` });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, SvPos, { x: i, v: i + 1 });
  }
};

const game = og.game({ tick: 60, seed: 11, systems: [SvMove, SvSaveLoad], resources: [SvRun], start: spawn });

function start(): Engine {
  const r = createEngine(game as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function run(engine: Engine): { coins: number; marker: number; saveAt: number; loadAt: number } {
  const r = engine.world.resource(SvRun);
  if (!r.ok) throw new Error(r.reason);
  return r.value as never;
}

function x(engine: Engine, name: string): number {
  const got = engine.world.get(engine.world.find(name) as Entity, SvPos);
  if (!got.ok) throw new Error(got.reason);
  return (got.value as { x: number }).x;
}

beforeAll(() => {
  loadNativeHost();
});

test("a save restored between ticks hashes as the saved world, and the world goes on as if it never left", () => {
  const engine = start();
  engine.advance(30);
  const save = engine.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  const hash = engine.hash();
  const coins = run(engine).coins;
  engine.advance(20);
  const extra = engine.world.spawn({ name: "extra" });
  expect(extra.ok).toBeTrue();
  expect(engine.hash()).not.toBe(hash);

  const restored = engine.world.restore(save.value);
  expect(restored).toEqual({ ok: true, value: null });
  expect(engine.hash()).toBe(hash);
  expect(engine.world.tick).toBe(30);
  expect(run(engine).coins).toBe(coins);
  // The names came back with their entities; the one spawned since is gone.
  expect(engine.world.find("extra")).toBeNull();
  expect(x(engine, "sv-2")).toBeCloseTo(2 + 3 * 30 / 60, 4);

  // Ten more ticks from the save equal forty ticks of a world that never saved.
  engine.advance(10);
  const straight = start();
  straight.advance(40);
  expect(engine.hash()).toBe(straight.hash());
  // The save's size: three entities and a few resources.
  console.log(`save: ${save.value.length} characters for a world of ${engine.world.query().ok ? "3 entities" : "?"}`);
  engine.destroy();
  straight.destroy();
});

test("inside a system a snapshot is taken at once and a restore waits for the tick's end, refusing a bad save at the call", () => {
  saved = null;
  bad = "not a save";
  refusals = [];
  const engine = start();
  run(engine).saveAt = 10;
  run(engine).loadAt = 25;
  engine.advance(11); // ticks 0..10: saved inside tick 10, after SvMove moved
  expect(saved).not.toBeNull();
  const atSave = x(engine, "sv-1");
  engine.advance(15); // ticks 11..25: tick 25 asks for the restore
  expect(refusals.length).toBe(1);
  expect(refusals[0]).toContain("world.restore takes the text world.snapshot returned");
  // Applied when tick 25 ended: the world is the one SvSaveLoad saw in tick 10.
  expect(engine.world.tick).toBe(10);
  expect(x(engine, "sv-1")).toBeCloseTo(atSave, 5);
  expect(run(engine).saveAt).toBe(10);
  expect(engine.world.find("sv-0")).not.toBeNull();
  // It goes on from there: the next step completes tick 10 as it went on
  // when it saved (the systems after SvSaveLoad: none of the game's, so it
  // neither moves again nor saves again), then tick 11 moves the world.
  saved = null;
  engine.advance(1);
  expect(engine.world.tick).toBe(11);
  expect(x(engine, "sv-1")).toBeCloseTo(atSave, 5);
  expect(saved).toBeNull();
  engine.advance(1);
  expect(x(engine, "sv-1")).toBeGreaterThan(atSave);
  expect(engine.takeLog().filter((l) => l.warning).map((l) => l.text)).toEqual([]);
  engine.destroy();
});

test("a restore keeps the session: the player's volumes, and the scheduled steps keep their delays though og.Time jumps", () => {
  const engine = start();
  engine.advance(10);
  const save = engine.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  engine.advance(20); // tick 30
  const audio = engine.world.resource(og.Audio);
  if (!audio.ok) throw new Error(audio.reason);
  audio.value.master = 0.25;
  // Due at the start of tick 45, after 15 more ticks.
  expect(og.commands.schedule(engine.world as never, "0.25:set:SvRun.marker=7").ok).toBeTrue();
  expect(engine.world.restore(save.value).ok).toBeTrue();
  expect(engine.world.tick).toBe(10);
  expect(audio.value.master).toBeCloseTo(0.25, 6);
  // Still 15 ticks after it was scheduled: at the start of tick 25 now.
  engine.advance(15);
  expect(run(engine).marker).toBe(0);
  engine.advance(1);
  expect(run(engine).marker).toBe(7);
  engine.destroy();
});

test("a save is text behind og-save:1:, and anything else is refused with nothing changed", () => {
  const engine = start();
  engine.advance(5);
  const save = engine.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  expect(save.value.startsWith("og-save:1:")).toBeTrue();
  expect(save.value).toMatch(/^og-save:1:[A-Za-z0-9+/]+=*$/);
  // A JSON value as is: what og.platform.storeSet takes.
  expect(JSON.parse(JSON.stringify({ world: save.value })).world).toBe(save.value);
  const hash = engine.hash();
  for (const [text, says] of [
    ["{\"coins\":3}", "not a save"],
    ["og-save:7:AAAA", "newer engine"],
    [save.value.slice(0, -12) + "AAAAAAAAAAAA", "damaged"],
  ] as const) {
    const r = engine.world.restore(text);
    expect(r.ok).toBeFalse();
    if (!r.ok) expect(r.reason).toContain(says);
  }
  expect(engine.hash()).toBe(hash);
  engine.destroy();
});

type Round = { host: string; save: string; hash: string; restored?: { ok: boolean; hash: string; tick: number } };

function round(host: string, ...args: string[]): Round {
  const r = Bun.spawnSync(["bun", "tests/support/save-round.ts", "--host", host, ...args], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Round;
}

test.skipIf(STALE_WASM)("one world saves to the same text on both hosts, publish included, and each host's save restores on the other to the saved hash", () => {
  const node = round("node");
  const web = round("web");
  const publish = round("publish");
  expect(web.save).toBe(node.save);
  expect(publish.save).toBe(node.save);
  expect(web.hash).toBe(node.hash);
  const dir = mkdtempSync(join(tmpdir(), "og-save-"));
  for (const [from, to] of [[node, "web"], [web, "node"], [node, "publish"]] as const) {
    const file = join(dir, `${from.host}.txt`);
    writeFileSync(file, from.save);
    const back = round(to, "--restore", file);
    expect(back.restored?.ok).toBeTrue();
    expect(back.restored?.tick).toBe(30);
    // The publish build has no world dump to hash.
    if (to !== "publish") expect(back.restored?.hash).toBe(node.hash);
  }
});
