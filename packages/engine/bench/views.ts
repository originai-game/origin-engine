// tools/bench/views.ts through the real API: the same integrate over 10,000
// entities, (a) as the spike's raw loop over typed arrays in one buffer, and
// (b) as the design's hello-world Move system (examples/hello.ts, unchanged)
// running in the engine: one engine tick per sample, so dispatch, the query,
// the column views over engine memory and the tick itself are all counted.
//
//   bun bench/views.ts [--host node|web] [--entities N] [--ticks T]
//   OG_NODE_VIEWS=copy bun bench/views.ts --host node    # copy-in/copy-out views

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createEngine, type GameToken } from "../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const which = opt("--host", "node");
const N = Number(opt("--entities", "10000"));
const TICKS = Number(opt("--ticks", "2000"));
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

// (a) the raw loop, as tools/bench/views.ts measures it.
const mem = new WebAssembly.Memory({ initial: 256 });
const pos = new Float32Array(mem.buffer, 0, N * 3);
const vel = new Float32Array(mem.buffer, N * 12 + 16, N * 3);
for (let i = 0; i < N; i++) vel[i * 3 + 2] = 1;
function integrate(p: Float32Array, v: Float32Array, n: number, dt: number) {
  for (let i = 0; i < n * 3; i += 3) { p[i]! += v[i]! * dt; p[i + 1]! += v[i + 1]! * dt; p[i + 2]! += v[i + 2]! * dt; }
}
for (let i = 0; i < 100; i++) integrate(pos, vel, N, 1 / 60);
let t0 = performance.now();
for (let i = 0; i < TICKS; i++) integrate(pos, vel, N, 1 / 60);
const rawMs = (performance.now() - t0) / TICKS;

// (b) the same work as the hello world's Move system, one engine tick each.
let mode = "wasm memory";
if (which === "node") {
  const { loadNativeHost, native } = await import("../src/node/index.ts");
  loadNativeHost();
  mode = native().og_node_views_mode();
} else {
  const { loadWebHost } = await import("../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}
const game = (await import("../examples/hello.ts")).default as unknown as GameToken;
const created = createEngine(game);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
engine.command("spawn_wave", [String(N)]);
engine.advance(100);
t0 = performance.now();
const r = engine.advance(TICKS);
const tickMs = (performance.now() - t0) / TICKS;
if (!r.ok) throw new Error(r.reason);
if (which === "node") mode = (await import("../src/node/index.ts")).native().og_node_views_mode();
const dump = engine.probe.dump({ what: "world" }) as unknown as { systems: Array<{ ms: number }> };
const runtime = typeof Bun !== "undefined" ? `bun ${Bun.version}` : `node ${process.version}`;
console.log(JSON.stringify({
  host: which,
  views: mode,
  runtime,
  entities: N,
  ticks: TICKS,
  rawLoopMs: +rawMs.toFixed(4),
  engineTickMs: +tickMs.toFixed(4),
  ratio: +(tickMs / rawMs).toFixed(2),
  lastMoveSystemMs: +(dump.systems[0]?.ms ?? 0).toFixed(4),
}));
