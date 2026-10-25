// Run with Bun from the repository root after a release build and og cook.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { loadavg } from "node:os";
import { loadNativeHost, native } from "../../../../packages/engine/src/node/index.ts";
import { bundleGame } from "../../../../packages/engine/src/node/build.ts";
import { createEngine } from "../../../../packages/engine/src/runtime.ts";
import { decodePng } from "../../../../packages/engine/tests/support/png.ts";

const project = resolve(process.argv[2]);
const out = resolve(process.argv[3]);
mkdirSync(out, { recursive: true });
const must = (r) => { if (!r.ok) throw new Error(r.reason); return r.value; };
const quantiles = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  return { p50: s[Math.floor(s.length * 0.5)], p95: s[Math.floor(s.length * 0.95)], max: s.at(-1) };
};
loadNativeHost();
const bundle = resolve("target/tmp/r4-capture.js");
must(await bundleGame(`${project}/src/main.ts`, bundle, "native"));
const { default: game } = await import(bundle);
const engine = must(createEngine(game, { root: project, project: readFileSync(`${project}/project.ogt`, "utf8"), deferStart: true }));
const tier = must(JSON.parse(native().og_node_attach_offscreen(engine.id, 1280, 720)));
must(engine.advance(0));
const samples = [];
try {
  for (let i = 0; i < 60; i++) must(engine.frame(1000 / 60));
  for (let i = -3; i < 30; i++) {
    must(engine.frame(1000 / 60));
    const load = loadavg();
    if (load[0] >= 8) throw new Error(`idle limit exceeded: ${load}`);
    const t0 = performance.now();
    const shot = engine.probe.capture(1280, 720);
    const ms = performance.now() - t0;
    const png = shot?.png;
    if (typeof png !== "string") throw new Error(`capture returned ${JSON.stringify(png)}`);
    const bytes = Buffer.from(png, "base64");
    const image = decodePng(bytes);
    if (image.width !== 1280 || image.height !== 720) throw new Error("wrong capture size");
    if (i >= 0) samples.push({ ms, load, bytes: bytes.length, tick: engine.probe.state().tick });
    if (i === 29) writeFileSync(`${out}/capture.png`, bytes);
  }
  writeFileSync(`${out}/capture.json`, JSON.stringify({ project, tier, method: "30 synchronous probe.capture calls, after 60 warm frames and 3 discarded captures; one frame between calls; wall clock includes extraction, rendering, GPU readback, PNG/base64 and ABI parsing, excludes PNG validation and file write", budgetMs: 30, summary: quantiles(samples.map((s) => s.ms)), samples }, null, 2));
} finally {
  engine.destroy();
}
