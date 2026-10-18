// The probe refuses an argument of the wrong type on the host named by
// --host (node: the napi addon; web: the wasm module, outside a browser, as
// the engine Worker runs it) and the engine keeps answering: commands() with
// an object where the script string goes (on the web host this used to trap
// the wasm in wasm-bindgen's string passing), then a real script, ticks and
// the hash. Prints JSON { host, refused, scheduled, tick, hash }.
//
//   bun tests/support/probe-refusal.ts --host node|web

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[process.argv.indexOf("--host") + 1];
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}
const created = createEngine((await import("../../examples/hello.ts")).default as unknown as GameToken);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const commands = engine.probe.commands as (script: unknown) => unknown;
let refused = "";
try {
  commands({ script: "0:spawn_wave:10" });
} catch (e) {
  refused = e instanceof Error ? e.message : String(e);
}
const report = engine.probe.commands("0:spawn_wave:10") as { scheduled: number } | null;
const advanced = engine.advance(30);
if (!advanced.ok) throw new Error(advanced.reason);
console.log(JSON.stringify({ host, refused, scheduled: report?.scheduled ?? 0, tick: advanced.value, hash: engine.hash() }));
