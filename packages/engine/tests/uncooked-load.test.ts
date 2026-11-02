// Uncooked loads at run time (og.load.imageData, G3-4): an image from
// bytes, held to its budget, put in place at a tick boundary and journaled,
// so a replay puts it in place at the tick the recorded run did, also when
// that run was on the web, where the browser's decode time decides the tick.
//
// Ways it could fail, and the check that catches each:
// - a load over budget decoded anyway (a 30000 px image allocating GBs) →
//   refused at the call from its header, naming the limit
// - a completion whose tick depends on the decode's speed natively → the
//   native run puts it in place at the boundary after the call, every run
// - a web-timed completion replayed when this host finishes it → the game,
//   which branches on og.load.status, sees it ready at the recorded tick,
//   and the world hashes match the recorded run tick for tick
// - a recorded failure replayed as a success → failed, saying so
import { beforeAll, expect, test } from "bun:test";
import { og, type World } from "@origingame/engine";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { encodePng } from "../src/node/png.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import uncooked, { image, Probe } from "./fixtures/uncooked/game.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const here = dirname(fileURLToPath(import.meta.url));
const game = uncooked as unknown as GameToken;

beforeAll(() => {
  loadNativeHost();
});

function run(ticks: number, journal?: unknown): { engine: Engine; hashes: string[]; probe: { readyAt: number; failedAt: number; width: number } } {
  const made = createEngine(game);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  if (journal) expect(og.load.replayCompletions(engine.world as unknown as World, journal as never)).toEqual({ ok: true });
  const hashes: string[] = [];
  for (let i = 0; i < ticks; i++) {
    const r = engine.advance(1);
    if (!r.ok) throw new Error(r.reason);
    hashes.push(engine.hash());
  }
  const p = engine.world.resource(Probe);
  if (!p.ok) throw new Error(p.reason);
  return { engine, hashes, probe: { readyAt: p.value.readyAt, failedAt: p.value.failedAt, width: p.value.width } };
}

test("an image over its budget is refused at the call, from its header, naming the limit and the fix", () => {
  const made = createEngine(game);
  if (!made.ok) throw new Error(made.reason);
  const world = made.value.world as unknown as World;
  // A PNG header claiming 30000x2: nothing is decoded.
  const huge = encodePng({ width: 1, height: 1, rgba: new Uint8Array(4) });
  new DataView(huge.buffer, huge.byteOffset).setUint32(16, 30000);
  const r = og.load.imageData(world, "huge.png", huge, {});
  expect(r.ok).toBe(false);
  expect(!r.ok && r.reason).toContain("huge.png: 30000x1 is over LoadBudget.textureSize (4096)");
  const big = og.load.imageData(world, "cat.png", image, { budget: { bytes: 10 } });
  expect(!big.ok && big.reason).toMatch(/cat\.png: \d+ bytes is over LoadBudget\.bytes \(10\); make the file smaller or raise the budget/);
  const gif = og.load.imageData(world, "a.gif", new TextEncoder().encode("GIF89a\x01\x00\x01\x00"), {});
  expect(!gif.ok && gif.reason).toContain("not a PNG or JPEG");
});

test("natively an image is in place at the boundary after its call, the same every run", () => {
  const a = run(6);
  // start runs when the engine is made; the load is in place at the next boundary, before the
  // first tick (world.tick 0), which the journal names.
  expect(a.probe).toEqual({ readyAt: 1, failedAt: 0, width: 3 });
  expect(og.load.completions(a.engine.world as unknown as World)).toEqual([{ request: 0, tick: 0, ok: true }]);
  const b = run(6);
  expect(b.hashes).toEqual(a.hashes);
});

test("a web-timed completion replays at its recorded tick: the game branching on og.load.status steps the same", () => {
  // As a web run journals it when the browser took 6 ticks longer to decode.
  const web = [{ request: 0, tick: 7, ok: true }];
  const recorded = run(12, web);
  expect(recorded.probe).toEqual({ readyAt: 8, failedAt: 0, width: 3 });
  expect(og.load.completions(recorded.engine.world as unknown as World)).toEqual(web);
  // Replayed: the same tick, the same world every tick; without the journal it differs once the live load lands.
  const replayed = run(12, og.load.completions(recorded.engine.world as unknown as World));
  expect(replayed.hashes).toEqual(recorded.hashes);
  const live = run(12);
  expect(live.probe.readyAt).toBe(1);
  expect(live.hashes[0]).not.toBe(recorded.hashes[0]);
  // A recorded failure fails at its tick, whatever this host made of the image.
  const failed = run(6, [{ request: 0, tick: 3, ok: false }]);
  expect(failed.probe).toEqual({ readyAt: 0, failedAt: 4, width: 0 });
  // The journal is for a world that has not ticked.
  const late = run(1);
  const r = og.load.replayCompletions(late.engine.world as unknown as World, web as never);
  expect(!r.ok && r.reason).toContain("has not ticked");
});

test("og replay keeps the loads journal: a trace recorded with a web session's journal compares the same, and without it diverges", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-uncooked-replay-"));
  const replay = (...args: string[]) => {
    const r = Bun.spawnSync(["bun", join(here, "..", "src", "node", "replay.ts"), "--entry", join(here, "fixtures", "uncooked", "game.ts"), ...args, "--json"], { cwd: join(here, "..") });
    return { exit: r.exitCode, out: new TextDecoder().decode(r.stdout), err: new TextDecoder().decode(r.stderr) };
  };
  writeFileSync(join(dir, "web.json"), JSON.stringify([{ request: 0, tick: 5, ok: true }]));
  const trace = join(dir, "trace.json");
  const rec = replay("--record", trace, "--commands", "10:quit", "--ticks", "10", "--loads", join(dir, "web.json"));
  expect([rec.exit, rec.err]).toEqual([0, rec.err]);
  const t = JSON.parse(readFileSync(trace, "utf8"));
  expect(t.loads).toEqual([{ request: 0, tick: 5, ok: true }]);
  const same = replay("--compare", trace);
  expect([same.exit, JSON.parse(same.out.trim().split("\n").at(-1)!).same]).toEqual([0, true]);
  // The same trace without its journal: this host puts the load in place at tick 0, so the world differs from there.
  delete t.loads;
  writeFileSync(trace, JSON.stringify(t));
  const off = replay("--compare", trace);
  expect(off.exit).toBe(3);
  expect(JSON.parse(off.out.trim().split("\n").at(-1)!).firstDivergentTick).toBeLessThanOrEqual(5);
});
