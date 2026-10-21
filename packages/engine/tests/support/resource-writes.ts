// One host's side of tests/resource-writes.test.ts: a game whose system
// writes 200 resource fields a tick (every kind a resource field has: f32,
// f64, u32, i32, bool, Vector3, Color, a handle, an enum and str), reading
// them back and making engine calls between the writes, as og-core §4 shows
// a game keeping its run in one resource. `--path json` writes each field
// through the engine's JSON path instead (one call per field, how a handle
// wrote before the byte layout): both paths must give one world. Prints the
// hash at tick 60, the save at tick 30 replayed to tick 60, the field
// values a command reads, and what a snapshot taken right after a command's
// write carries.
//
//   bun tests/support/resource-writes.ts --host node|web [--path bytes|json]

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineResource, defineSystem, bool, Color, Entity, f32, f64, i32, str, ToneMapping, u32, Vector3 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { boundHost, createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => (argv.includes(name) ? (argv[argv.indexOf(name) + 1] ?? "") : "");
const host = opt("--host");
const path = opt("--path") || "bytes";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}

// 120 numbers, 40 flags, 20 vectors, 10 colours, 5 handles, 3 texts and 2 enums: 200 fields.
const spec: Record<string, unknown> = {};
for (let i = 0; i < 40; i++) spec[`f${i}`] = f32(0);
for (let i = 0; i < 30; i++) spec[`d${i}`] = f64(0);
for (let i = 0; i < 30; i++) spec[`u${i}`] = u32(0);
for (let i = 0; i < 20; i++) spec[`i${i}`] = i32(0);
for (let i = 0; i < 40; i++) spec[`b${i}`] = bool(false);
for (let i = 0; i < 20; i++) spec[`v${i}`] = Vector3();
for (let i = 0; i < 10; i++) spec[`c${i}`] = Color();
for (let i = 0; i < 5; i++) spec[`h${i}`] = Entity();
for (let i = 0; i < 3; i++) spec[`s${i}`] = str(48, "");
for (let i = 0; i < 2; i++) spec[`e${i}`] = ToneMapping();
const Run = defineResource("RwRun", spec as never);
const names = Object.keys(spec);
if (names.length !== 200) throw new Error(`${names.length} fields`);

type Rec = Record<string, unknown>;
const json = (world: World, key: string, value: unknown) => {
  const answer = JSON.parse(boundHost()!.abi.og_engine_write_resource(world.id, "RwRun", JSON.stringify({ [key]: value }))) as { ok: boolean; reason?: string };
  if (!answer.ok) throw new Error(answer.reason);
};

/** The value field `k` takes on tick `t`, from what the run holds (so reads matter). */
function next(k: string, t: number, g: Rec): unknown {
  const n = Number(k.slice(1));
  switch (k[0]) {
    case "f": return (g[k] as number) * 0.97 + Math.sin(t * 0.1 + n) * 1.37;
    case "d": return (g[k] as number) + t / 3 + n * 1e-7;
    case "u": return ((g[k] as number) + t * 7 + n) % 1000;
    case "i": return (g[k] as number) - t + n;
    case "b": return !(g[k] as boolean) !== ((t + n) % 3 === 0);
    case "v": { const v = g[k] as { x: number; y: number; z: number }; return { x: v.x + 0.1, y: v.y * 0.5 + n, z: Math.cos(t + n) }; }
    case "c": return { r: ((t + n) % 10) / 10, g: 0.5, b: 1 / (t + 1) };
    case "h": return (t * 4294967296 + n) % 9007199254740991;
    case "e": return ["aces", "agx", "none"][(t + n) % 3];
    default: return `tick ${t} · 灯 ${n}`;
  }
}

const Write = defineSystem("RwWrite", { writes: [Run] }, (_q, time, world) => {
  const handle = world.resource(Run);
  if (!handle.ok) throw new Error(handle.reason);
  const g = handle.value as Rec;
  let k = 0;
  for (const name of names) {
    const value = next(name, time.tick, g);
    if (path === "json") json(world, name, value);
    else g[name] = value;
    // An engine call every few writes: a read after it must see the writes.
    if (++k % 7 === 0) world.alive(0 as never);
  }
  // Two more writes, each after a read of what was written.
  g.u0 = (g.u0 as number) + 1;
  g.f0 = (g.f0 as number) + (g.u0 as number);
});

let seen: Rec = {};
const read = (world: World) => {
  const h = world.resource(Run);
  if (h.ok) seen = { ...(h.value as Rec) };
};
let carried: unknown = null;
const write_then_snapshot = (world: World) => {
  const h = world.resource(Run);
  if (!h.ok) throw new Error(h.reason);
  (h.value as Rec).u1 = 424242;
  // The snapshot is an engine call: it must carry the write made just before it.
  const snap = world.snapshot();
  if (!snap.ok) throw new Error(snap.reason);
  (h.value as Rec).u1 = 0;
  carried = snap.value;
};

const game = og.game({ tick: 60, seed: 3, systems: [Write], resources: [Run], commands: { read, write_then_snapshot } }) as unknown as GameToken;
const created = createEngine(game);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
engine.advance(30);
const save = engine.world.snapshot();
if (!save.ok) throw new Error(save.reason);
engine.advance(30);
const hash = engine.hash();
engine.command("read", []);
const values = seen;
const replay = engine.world.restore(save.value);
if (!replay.ok) throw new Error(replay.reason);
engine.advance(30);
const replayed = engine.hash();
const w = engine.command("write_then_snapshot", []);
if (!w.ok) throw new Error(w.reason);
const restored = engine.world.restore(carried as string);
if (!restored.ok) throw new Error(restored.reason);
engine.command("read", []);
console.log(JSON.stringify({ host, path, hash, replayed, values, carriedU1: seen.u1 }));
