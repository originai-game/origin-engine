// Uncooked loads at run time (og.load.imageData, og.load.gltfData, G3-4):
// an image or a model from bytes, held to its budget, put in place at a tick boundary and journaled,
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
// - a glTF over its triangles, or with an image over textureSize, read
//   anyway → refused at the call, naming the limit; what the reader cannot
//   decode (Draco, another file) drawn wrong → refused by name
// - a glTF put in place without its nodes, materials or textures → its
//   info, its spawned Model and the bounds its child node moves agree
// - a web-timed glTF completion replayed when this host finishes it →
//   pending until its recorded tick, then ready
// - a released asset still holding its texels and vertices → the counted
//   heap gives back what the loads took (the GPU side: uncooked-release)
// - a released pending load completing anyway → never journaled, failed
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
import { gridGlb, quadGlb } from "./fixtures/uncooked/glb.ts";
import type { Asset } from "@origingame/engine";

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

test("a glTF over its budget, or holding what the reader does not decode, is refused at the call by name", () => {
  const made = createEngine(game);
  if (!made.ok) throw new Error(made.reason);
  const world = made.value.world as unknown as World;
  const reason = (bytes: Uint8Array, desc: unknown = {}) => {
    const r = og.load.gltfData(world, "creations/kite.glb", bytes, desc as never);
    return r.ok ? "accepted" : r.reason;
  };
  // Two triangles a node, but the budget counts each mesh once.
  expect(reason(quadGlb(image), { budget: { triangles: 1 } })).toBe(
    "creations/kite.glb: 2 triangles is over LoadBudget.triangles (1); simplify the model (a decimate modifier, fewer subdivisions) or raise the budget",
  );
  expect(reason(quadGlb(image), { budget: { textureSize: 2 } })).toContain("creations/kite.glb: image 0: 3x2 is over LoadBudget.textureSize (2)");
  expect(reason(quadGlb(image), { budget: { bytes: 100 } })).toMatch(/creations\/kite\.glb: \d+ bytes is over LoadBudget\.bytes \(100\)/);
  expect(reason(quadGlb(image, { extensionsRequired: ["KHR_draco_mesh_compression"], extensionsUsed: ["KHR_draco_mesh_compression"] }))).toContain("Draco");
  expect(reason(quadGlb(image, { buffers: [{ uri: "kite.bin", byteLength: 4 }] }))).toContain("pack the model as a .glb");
  expect(reason(quadGlb(new TextEncoder().encode("GIF89a\x03\x00\x02\x00")))).toContain("image 0: not a PNG or JPEG");
  expect(reason(new TextEncoder().encode("solid kite\nendsolid"))).toContain("not a glTF");
  expect(reason(quadGlb(image))).toBe("accepted");
});

test("natively a glTF is in place at the boundary after its call, drawn as og.load.gltf's model would be", () => {
  const go = () => {
    const made = createEngine(game);
    if (!made.ok) throw new Error(made.reason);
    const world = made.value.world as unknown as World;
    const r = og.load.gltfData(world, "creations/kite.glb", quadGlb(image), {});
    if (!r.ok) throw new Error(r.reason);
    expect(og.load.status(world, r.value)).toBe("pending");
    expect(made.value.advance(1).ok).toBe(true);
    return { engine: made.value, world, asset: r.value as Asset };
  };
  const { engine, world, asset } = go();
  expect([og.load.status(world, asset), og.load.error(world, asset)]).toEqual(["ready", null]);
  // The image was request 0 (the game's start), the model request 1.
  expect(og.load.completions(world)).toEqual([{ request: 0, tick: 0, ok: true }, { request: 1, tick: 0, ok: true }]);
  const info = og.load.gltfInfo(world, asset);
  if (!info.ok) throw new Error(info.reason);
  // One primitive drawn by two nodes: one geometry, one material.
  expect([info.value.geometries.length, info.value.materials.length, info.value.scenes]).toEqual([1, 1, 1]);
  const e = world.instantiate(asset, { name: "kite" });
  if (!e.ok) throw new Error(e.reason);
  expect(engine.advance(1).ok).toBe(true);
  // Both nodes draw: the quad at the root and again 5 m along x (its texture coordinates with it),
  // and nothing between them.
  const hit = (x: number) => {
    const r = og.render.raycast(world, { origin: { x, y: 0.25, z: 5 }, direction: { x: 0, y: 0, z: -1 } }, { first: true });
    if (!r.ok) throw new Error(r.reason);
    return r.value.map((h) => [h.entity === e.value, h.point.x, h.distance, h.uv.x, h.uv.y]);
  };
  expect(hit(0.5)).toEqual([[true, 0.5, 5, 0.5, 0.25]]);
  expect(hit(5.75)).toEqual([[true, 5.75, 5, 0.75, 0.25]]);
  expect(hit(3)).toEqual([]);
  // Not registered under its name: og.load.gltf of it is the cooked path's, refused here.
  expect(og.load.gltf(world, "creations/kite.glb").ok).toBe(false);
  // The same every run.
  const again = go();
  expect(again.engine.hash()).toBe(go().engine.hash());
});

test("a web-timed glTF completion replays at its recorded tick: pending until it, ready from it", () => {
  const made = createEngine(game);
  if (!made.ok) throw new Error(made.reason);
  const world = made.value.world as unknown as World;
  const journal = [{ request: 0, tick: 0, ok: true }, { request: 1, tick: 4, ok: true }];
  expect(og.load.replayCompletions(world, journal as never)).toEqual({ ok: true });
  const r = og.load.gltfData(world, "creations/kite.glb", quadGlb(image), {});
  if (!r.ok) throw new Error(r.reason);
  const seen: string[] = [];
  for (let i = 0; i < 6; i++) {
    expect(made.value.advance(1).ok).toBe(true);
    seen.push(og.load.status(world, r.value));
  }
  // The boundary before tick 4 is the fifth advance's.
  expect(seen).toEqual(["pending", "pending", "pending", "pending", "ready", "ready"]);
  expect(og.load.completions(world)).toEqual(journal);
});

test("og.load.release gives back the memory an uncooked image and model took, and drops a pending load unjournaled", () => {
  const made = createEngine(game);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  const world = engine.world as unknown as World;
  const live = () => {
    const m = engine.probe.memory() as { heap: { live: number } | null } | null;
    if (!m?.heap) throw new Error("this host counts no heap");
    return m.heap.live;
  };
  expect(engine.advance(1).ok).toBe(true);
  const before = live();
  // 4 MiB of texels and a grid of 131,072 triangles (66,049 vertices).
  const size = 1024;
  const texels = new Uint8Array(size * size * 4).map((_, i) => (i * 7) & 0xff);
  const picture = og.load.imageData(world, "drawings/wall.png", encodePng({ width: size, height: size, rgba: texels }), {});
  const grid = og.load.gltfData(world, "creations/field.glb", gridGlb(256), {});
  if (!picture.ok || !grid.ok) throw new Error("refused");
  expect(engine.advance(1).ok).toBe(true);
  expect([og.load.status(world, picture.value), og.load.status(world, grid.value)]).toEqual(["ready", "ready"]);
  const loaded = live();
  expect(loaded - before).toBeGreaterThan(8_000_000);
  for (const a of [picture.value, grid.value]) expect(og.load.release(world, a)).toEqual({ ok: true });
  expect(engine.advance(1).ok).toBe(true);
  // What stays is the handles' bookkeeping, not the content.
  expect(loaded - live()).toBeGreaterThan(0.95 * (loaded - before));
  expect(og.load.status(world, grid.value)).toBe("failed");
  expect(og.load.error(world, grid.value)).toBe("creations/field.glb was released (og.load.release)");
  const again = world.instantiate(grid.value, {});
  expect(!again.ok && again.reason).toBe("creations/field.glb was released (og.load.release)");
  expect(!og.load.imageInfo(world, picture.value).ok).toBe(true);
  expect(og.load.release(world, grid.value)).toEqual({ ok: true });
  // A pending load released before its boundary never lands and is never journaled.
  const pending = og.load.imageData(world, "drawings/late.png", image, {});
  if (!pending.ok) throw new Error(pending.reason);
  expect(og.load.release(world, pending.value)).toEqual({ ok: true });
  expect(engine.advance(2).ok).toBe(true);
  expect(og.load.status(world, pending.value)).toBe("failed");
  expect(og.load.completions(world).map((c) => c.request)).toEqual([0, 1, 2]);
});
