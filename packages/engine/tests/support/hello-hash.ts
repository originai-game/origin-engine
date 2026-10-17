// One host per process (the runtime binds one): the design's hello world on
// the host named by --host (node: the napi addon; web: the wasm module, run
// here outside a browser), 10,000 entities through its own spawn_wave
// command, then --ticks fixed steps. Prints JSON { host, tick, hash, ... }.
//
//   bun tests/support/hello-hash.ts --host node|web [--ticks 600] [--seed S]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const which = opt("--host", "node");
const ticks = Number(opt("--ticks", "600"));
const seed = argv.includes("--seed") ? Number(opt("--seed", "0")) : undefined;
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}
const game = (await import("../../examples/hello.ts")).default as unknown as GameToken;
const created = createEngine(game, seed === undefined ? {} : { seed });
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const hash0 = engine.hash();
const spawned = engine.command("spawn_wave", ["10000"]);
if (!spawned.ok) throw new Error(spawned.reason);
const t0 = performance.now();
const advanced = engine.advance(ticks);
const ms = performance.now() - t0;
if (!advanced.ok) throw new Error(advanced.reason);
const dump = engine.probe.dump({ what: "world" }) as { hash: string; entities: number };
console.log(JSON.stringify({ host: which, tick: advanced.value, hash: dump.hash, hashAtLoad: hash0, entities: dump.entities, advanceMs: +ms.toFixed(2) }));
