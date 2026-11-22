// A game that uses no optional module, on the split publish module
// (docs/web-export.md §Modules), with a module host that throws on any load:
// a call from the engine into a module placeholder is a module-free game
// downloading a module, and fails here naming it. The game makes meshes
// (the generated geometries), draws them, and runs ticks, frames (no
// renderer outside a browser: the tick side of a frame), a snapshot, a hash
// and a restore. Prints JSON { tick, modules }.
//
// --terrain: the game also places an og.Terrain, so the frames call into
// the 3D world's module (world3d), the one module the host loads; any other
// still fails.
//
//   bun tests/support/module-free.ts --profile 3d|2d [--terrain]

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";
import type { ModuleHost } from "../../src/web/split.ts";

const argv = process.argv.slice(2);
const profile = argv.includes("--profile") ? argv[argv.indexOf("--profile") + 1]! : "3d";
if (profile !== "3d" && profile !== "2d") throw new Error(`--profile ${profile}: 3d or 2d`);
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const dir = join(pkg, "dist", "web-publish");
const terrain = argv.includes("--terrain");
const refuse = (name: string): never => {
  throw new Error(`a game using no optional module called into engine module ${name}`);
};
const { fileModuleHost } = await import("../../src/node/modules.ts");
const files = fileModuleHost(dir);
const host: ModuleHost = {
  load: async (name, imports) => (terrain && name === "world3d" ? files.load(name, imports) : refuse(name)),
  loadSync: (name, imports) => (terrain && name === "world3d" ? files.loadSync!(name, imports) : refuse(name)),
};
const { loadWebHost, loadedEngineModules } = await import("../../src/web/host.ts");
const glue = await import(join(dir, "origin-engine.js"));
await loadWebHost(glue, readFileSync(join(dir, "origin-engine_bg.wasm")), undefined, host);

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const game = og.game({
  tick: 60,
  seed: 3,
  start: (world) => {
    const material = must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.4, b: 0.2 } }));
    const shapes = [
      must(og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 })),
      must(og.render.planeGeometry(world, { width: 4, height: 4 })),
      must(og.render.sphereGeometry(world, { radius: 0.5 })),
    ];
    shapes.forEach((geometry, i) => {
      const e = must(world.spawn({ name: `shape-${i}`, position: { x: i * 2, y: 0.5, z: 0 } }));
      world.insert(e, og.Mesh, { geometry, material });
    });
    // A terrain call (refused: it names no loaded terrain) runs og.terrain's binding, in world3d.
    if (terrain && og.terrain.bounds(world, 0 as never).ok) throw new Error("og.terrain.bounds answered an unknown terrain");
  },
}) as unknown as GameToken;
const engine = must(createEngine(game, profile === "2d" ? { project: 'type project\nversion 1\n---\ncreate {\n  form "2d"\n}\n' } : {}));
must(engine.advance(30));
for (let i = 0; i < 30; i++) engine.frame(1000 / 60);
const snapshot = must(engine.world.snapshot());
engine.hash();
must(engine.world.restore(snapshot));
must(engine.advance(30));
console.log(JSON.stringify({ tick: engine.world.tick, modules: loadedEngineModules() }));
process.exit(0);
