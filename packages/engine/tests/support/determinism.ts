import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "../../src/index.ts";
import { createEngine } from "../../src/runtime.ts";

const host = process.argv[2];
const pkg = join(dirname(fileURLToPath(import.meta.url)), "../..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else if (host === "web") {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const dir = join(pkg, "dist/web");
  await loadWebHost(await import(join(dir, "origin-engine.js")), readFileSync(join(dir, "origin-engine_bg.wasm")));
} else throw new Error(`unknown host ${host}`);
const created = createEngine(og.game({}));
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
try {
  engine.probe.commands("0:dump_determinism:digests.json");
  const advanced = engine.advance(1);
  if (!advanced.ok) throw new Error(advanced.reason);
  const output = engine.takeOutputs().find(o => o.file === "digests.json");
  if (!output) throw new Error("development host omitted determinism digests");
  // One JSON line, as the other host probes print (hello-hash.ts).
  console.log(JSON.stringify(JSON.parse(output.data)));
} finally {
  engine.destroy();
}
