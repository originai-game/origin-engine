// The GPU time of each render-graph pass of tools/bench/threejs-compare.ts's
// §7 reference scene in headless Chrome, on either tier: a non-publish
// build (a publish build times no pass on the GPU), driven one `frame` at
// a time through engine.page.ts as threejs-compare.ts does, with the run's
// trace on. Every GPU frame the trace holds after `--warm` (the timer reads
// a frame back a frame or two late, and skips one while its readbacks are
// all in flight) gives each pass its milliseconds; printed per pass as
// mean, p50 and p95 with the frame's total, and the page's WebGL2 timer
// extension (EXT_disjoint_timer_query_webgl2, and its TIMESTAMP_EXT counter
// bits: 0 on ANGLE's Metal and D3D11 backends).
//
//   bun tools/bench/gpu-passes.ts [--mode webgl2|webgpu] [--frames 600] [--warm 60]
//       [--gap <ms>] [--timer-shim] [--out <result.json>] [--chrome <path>]
//
// --gap pauses after each frame (default 4 ms) so the Worker's event loop
// runs: WebGL makes a query's result available only after the task that
// ended it. --timer-shim gives the engine Worker the stand-in extension of
// packages/engine/tests/web/timer-shim.ts where Chrome has none (no GPU):
// it proves the path, its numbers are the Worker's clock, not a GPU's.
// TRACE_CHROME_FLAGS adds Chrome flags (an orb: `--use-angle=gl-egl` for
// llvmpipe, whose GL compiles the engine's shaders where SwiftShader may
// refuse one).
//
// Needs packages/engine/dist/web (the dev engine: `bun run build` in
// packages/engine); cooks the project when it has no cook. Serves without
// isolation, so the single-threaded engine runs.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../packages/engine/src/node/web-build.ts";
import { Browser, chromePath } from "../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../packages/engine/tests/web/serve.ts";
import { TIMER_SHIM } from "../../packages/engine/tests/web/timer-shim.ts";
import { ogBin } from "../../packages/engine/tests/support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const FRAMES = Number(opt("--frames", "600"));
const WARM = Number(opt("--warm", "60"));
const GAP = Number(opt("--gap", "4"));
const MODE = opt("--mode", "webgl2");
const W = 1280, H = 720;
const project = join(repo, "crates", "og-tests", "reference-scene");
const chrome = chromePath(argv.includes("--chrome") ? ["--chrome", opt("--chrome", "")] : []);

const enginePage = await Bun.build({ entrypoints: [join(here, "engine.page.ts")], target: "browser", format: "esm" });
if (!enginePage.success) throw new Error(enginePage.logs.map(String).join("\n"));
const html = `<!doctype html><html><head><meta charset="utf-8"><title>engine gpu passes</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/bench-page.js"></script></body></html>`;
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  mkdirSync(join(project, "assets"), { recursive: true });
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = join(repo, "target", "tmp", "gpu-passes");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
writeFileSync(join(dist, "bench-page.js"), await enginePage.outputs[0]!.text());
const server = serve({ dist, isolate: false, extra: { "/bench.html": html } });

const browser = await Browser.launch({ chrome, width: W, height: H, flags: ["--disable-frame-rate-limit", ...(process.env.TRACE_CHROME_FLAGS ?? "").split(" ").filter(Boolean)] });
const log: string[] = [];
const page = await browser.open("about:blank", log);
await page.prepareWorkers(async (w, session) => {
  if (w.url.endsWith("/game.js") && argv.includes("--timer-shim")) await page.evalWorker(session, TIMER_SHIM);
});
await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/bench.html${MODE === "webgl2" ? "?webgl=1" : ""}` });
await page.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 180_000);
const failed = await page.eval<string | undefined>("globalThis.__og_shell_error__");
if (failed) throw new Error(`the engine did not start: ${failed}\n${log.slice(-20).join("\n")}`);

// The page's own WebGL2 context: what this Chrome's timer extension offers.
const timer = await page.eval<{ extension: boolean; timestampBits: number | null; elapsedBits: number | null }>(`(() => {
  const gl = document.createElement("canvas").getContext("webgl2");
  const ext = gl && gl.getExtension("EXT_disjoint_timer_query_webgl2");
  return { extension: !!ext, timestampBits: ext ? gl.getQuery(ext.TIMESTAMP_EXT, ext.QUERY_COUNTER_BITS_EXT) : null, elapsedBits: ext ? gl.getQuery(ext.TIME_ELAPSED_EXT, ext.QUERY_COUNTER_BITS_EXT) : null };
})()`);
const FRAME = `(async () => { const a = await globalThis.__og_shell__.call("frame", 1000 / 60); if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a)); await new Promise((r) => setTimeout(r, ${GAP})); })()`;
for (let i = 0; i < WARM; i++) await page.eval(FRAME);
await page.eval(`globalThis.__og_shell__.call("trace", "start")`);
for (let i = 0; i < FRAMES; i++) await page.eval(FRAME);
type Event = { name: string; cat?: string; ph: string; dur?: number; args?: Record<string, unknown> };
const doc = await page.eval<{ traceEvents: Event[] }>(`globalThis.__og_shell__.call("trace", "take")`);
const state = await page.eval<{ timings: { gpuPasses: Array<{ name: string; ms: number }> } }>(`globalThis.__og_shell__.call("probe", "state")`);
const tier = await page.eval<string>("globalThis.__og_shell__.tier");
await page.eval(`globalThis.__og_shell__.call("trace", "stop")`);
await browser.close();
server.stop();

// The gpu thread: each frame's span ("gpu frame N") and its passes after it, in order.
const gpu = doc.traceEvents.filter((e) => e.ph === "X" && e.cat === "gpu");
const frames: Array<{ frame: string; ms: number; passes: Map<string, number> }> = [];
for (const e of gpu) {
  if (e.name.startsWith("gpu frame")) frames.push({ frame: e.name, ms: (e.dur ?? 0) / 1000, passes: new Map() });
  else frames.at(-1)?.passes.set(e.name, (frames.at(-1)!.passes.get(e.name) ?? 0) + (e.dur ?? 0) / 1000);
}
const stat = (xs: number[]) => {
  const s = [...xs].sort((a, b) => a - b);
  const at = (q: number) => s[Math.min(s.length - 1, Math.floor(q * s.length))] ?? 0;
  return { mean: +(xs.reduce((a, b) => a + b, 0) / Math.max(1, xs.length)).toFixed(4), p50: +at(0.5).toFixed(4), p95: +at(0.95).toFixed(4) };
};
const names = [...new Set(frames.flatMap((f) => [...f.passes.keys()]))];
const total = stat(frames.map((f) => f.ms));
const passes = names.map((name) => {
  const s = stat(frames.map((f) => f.passes.get(name) ?? 0));
  return { name, ...s, share: total.mean > 0 ? +((s.mean / total.mean) * 100).toFixed(2) : 0 };
});
const result = { tier, mode: MODE, chrome, timerShim: argv.includes("--timer-shim"), timer, frames: FRAMES, warm: WARM, gpuFrames: frames.length, total, passes, lastProbeGpuPasses: state.timings.gpuPasses };
console.log(`${tier}: ${frames.length} GPU frames of ${FRAMES}; frame ${total.mean} ms mean, ${total.p50} p50, ${total.p95} p95; extension ${timer.extension} (TIMESTAMP_EXT ${timer.timestampBits} bits, TIME_ELAPSED_EXT ${timer.elapsedBits})${result.timerShim ? "; stand-in timer (Worker clock, not a GPU's)" : ""}`);
for (const p of passes) console.log(`  ${p.name.padEnd(18)} ${p.mean.toFixed(4)} ms mean  ${p.p50.toFixed(4)} p50  ${p.p95.toFixed(4)} p95  ${p.share}%`);
const OUT = opt("--out", "");
if (OUT) writeFileSync(OUT, JSON.stringify(result, null, 2) + "\n");
