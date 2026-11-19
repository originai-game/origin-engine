// Which engine functions a game that uses no optional module runs
// (scripts/wasm-split.ts): loads a module instrumented by `wasm-split
// --instrument` with its glue, creates a game of one entity, ticks it,
// snapshots, hashes and restores it, ticks again, and writes the module's
// profile. Those functions stay in the split primary, so such a game loads
// no split module.
//
//   bun scripts/wasm-profile.ts <glue.js> <instrumented.wasm> <profile out>

import { readFileSync, writeFileSync } from "node:fs";
import { og } from "@origingame/engine";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { loadWebHost } from "../src/web/host.ts";

const [glueFile, wasmFile, out] = process.argv.slice(2);
if (!glueFile || !wasmFile || !out) throw new Error("usage: bun scripts/wasm-profile.ts <glue.js> <instrumented.wasm> <profile out>");
let exports: WebAssembly.Exports | null = null;
const instantiate = WebAssembly.instantiate;
(WebAssembly as { instantiate: unknown }).instantiate = async (source: BufferSource, imports?: WebAssembly.Imports) => {
  const r = (await instantiate(source, imports)) as unknown as WebAssembly.WebAssemblyInstantiatedSource;
  exports = r.instance.exports;
  return r;
};
const glue = await import(glueFile);
// A lanes module (wasm atomics) runs on the engine thread alone here.
await loadWebHost(glue, readFileSync(wasmFile));
const created = createEngine(og.game({ tick: 60, seed: 1, start: (w) => void w.spawn({ name: "a", position: { x: 1, y: 2, z: 3 } }) }) as unknown as GameToken);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
engine.advance(30);
const snapshot = engine.world.snapshot();
if (!snapshot.ok) throw new Error(snapshot.reason);
engine.hash();
const restored = engine.world.restore(snapshot.value);
if (!restored.ok) throw new Error(restored.reason);
engine.advance(30);
const ex = exports as unknown as { memory: WebAssembly.Memory; __write_profile: (at: number, length: number) => number };
const pages = 16;
const at = ex.memory.grow(pages) * 65536;
const length = ex.__write_profile(at, pages * 65536);
writeFileSync(out, new Uint8Array(ex.memory.buffer, at, length).slice());
process.exit(0);
