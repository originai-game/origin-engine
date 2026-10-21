// The probe's `memory` on the host named by --host (node: the napi addon;
// web: the wasm module, outside a browser, as the engine Worker runs it),
// after the hello world's 10,000 entities and 60 ticks: the same shape on
// both, each with the figures its host can read. On the web it is also the
// engine Worker's `memory` call (`og_web_memory`), which a publish page
// answers without the probe's dev members. Prints JSON { host, memory, worker }.
//
//   bun tests/support/probe-memory.ts --host node|web

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[process.argv.indexOf("--host") + 1];
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
let glue: { og_web_memory?(engine: number): string } | null = null;
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue as never, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}
const created = createEngine((await import("../../examples/hello.ts")).default as unknown as GameToken);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const spawned = engine.command("spawn_wave", ["10000"]);
if (!spawned.ok) throw new Error(spawned.reason);
const advanced = engine.advance(60);
if (!advanced.ok) throw new Error(advanced.reason);
const memory = (engine.probe as unknown as { memory(): unknown }).memory();
const worker = glue?.og_web_memory ? JSON.parse(glue.og_web_memory(engine.id)) : null;
console.log(JSON.stringify({ host, memory, worker }));
