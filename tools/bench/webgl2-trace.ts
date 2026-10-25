// The per-frame accounting of tools/bench/threejs-compare.ts's engine on the
// WebGL2 tier: what one `frame` call (one tick, one render) asks of WebGL,
// counted exactly in the engine Worker, and the frame's time there.
//
//   bun tools/bench/webgl2-trace.ts [--only reference|spheres] [--mode webgl2|webgpu]
//       [--frames 120] [--warm 60] [--counts-only | --times-only] [--sequence <calls.json>]
//       [--gap <ms>] [--profile <out.cpuprofile>] [--shot <out.png>]
//       [--dist <dir>] [--prebuilt] [--no-shim] [--out <result.json>] [--chrome <path>]
//
// Counts: every WebGL2RenderingContext method is wrapped in the engine
// Worker before its first line (cdp.ts prepareWorkers); the calls of
// `--frames` frames after `--warm` are divided by the frame count, with the
// bytes of bufferSubData and texSubImage*. `--sequence` writes one more
// frame's calls in order, objects named by first appearance ("#12"):
// packages/engine/tests/fixtures/webgl2-frame/ is such a frame. Times: a
// second, unwrapped page times `--frames` frame round trips (as
// threejs-compare.ts does); `--gap` pauses before each, so the GPU process
// has drained and the round trip is the Worker's own work; `--profile`
// writes a V8 CPU profile of the Worker over them. `--shot` only renders
// `--warm` frames and saves a screenshot, to compare two builds' frames
// pixel for pixel. `--prebuilt` serves `--dist` as it is (another
// revision's build), instead of building the project's web half into it.
//
// On a machine without a GPU, Chrome draws WebGL with SwiftShader or
// llvmpipe: times are only comparable with a base build on the same
// machine, counts are exact anywhere. TRACE_CHROME_FLAGS adds Chrome flags
// (an orb: `--use-angle=gl-egl` for llvmpipe, whose GL compiles the
// engine's shaders where Chrome 156's SwiftShader refuses one).
//
// Needs packages/engine/dist/web-publish (scripts/build.sh, or the 3D
// publish wasm bound there); cooks the project when it has no cook. Serves
// without isolation, so the single-threaded variant runs.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../packages/engine/src/node/web-build.ts";
import { Browser, chromePath } from "../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../packages/engine/tests/web/serve.ts";
import { ogBin } from "../../packages/engine/tests/support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const FRAMES = Number(opt("--frames", "120"));
const WARM = Number(opt("--warm", "60"));
const ONLY = opt("--only", "reference");
const MODE = opt("--mode", "webgl2");
const PROFILE = opt("--profile", "");
const W = 1280, H = 720;
const project = ONLY === "spheres" ? join(here, "spheres") : join(repo, "crates", "og-tests", "reference-scene");
const chrome = chromePath(argv.includes("--chrome") ? ["--chrome", opt("--chrome", "")] : []);

// Runs in the engine Worker before its first line.
const SHIM = `(() => {
  const counts = Object.create(null);
  const bytes = Object.create(null);
  // --sequence: one frame's calls in order, objects by a per-object id.
  let seq = null;
  const ids = new WeakMap();
  let nextId = 1;
  const arg = (v) => (v === null || v === undefined || typeof v !== "object" ? v : ArrayBuffer.isView(v) ? "bytes:" + v.byteLength : (ids.has(v) || ids.set(v, "#" + nextId++), ids.get(v)));
  const size = (v) => (v && typeof v.byteLength === "number" ? v.byteLength : 0);
  const wrap = (proto, prefix) => {
    if (!proto) return;
    for (const k of Object.getOwnPropertyNames(proto)) {
      const d = Object.getOwnPropertyDescriptor(proto, k);
      if (!d || typeof d.value !== "function" || k === "constructor") continue;
      const f = d.value;
      const name = prefix + k;
      proto[k] = function (...a) {
        counts[name] = (counts[name] || 0) + 1;
        if (seq) seq.push([name, ...a.map(arg)]);
        if (k === "bufferSubData") bytes[name] = (bytes[name] || 0) + (a[4] ? a[4] * (a[2].BYTES_PER_ELEMENT || 1) : size(a[2]) - (a[3] || 0) * ((a[2] && a[2].BYTES_PER_ELEMENT) || 1));
        else if (k === "bufferData") bytes[name] = (bytes[name] || 0) + (typeof a[1] === "number" ? a[1] : size(a[1]));
        else if (k.startsWith("texSubImage") || k.startsWith("texImage") || k.startsWith("compressedTex")) { const v = a.find((x) => x && typeof x === "object" && typeof x.byteLength === "number"); bytes[name] = (bytes[name] || 0) + size(v); }
        return f.apply(this, a);
      };
    }
  };
  const g = globalThis;
  wrap(g.WebGL2RenderingContext && g.WebGL2RenderingContext.prototype, "gl.");
  g.__glc__ = {
    reset() { for (const k in counts) delete counts[k]; for (const k in bytes) delete bytes[k]; },
    read() { return { counts: { ...counts }, bytes: { ...bytes } }; },
    record() { seq = []; },
    sequence() { const s = seq; seq = null; return s; },
  };
})();`;

const enginePage = await Bun.build({ entrypoints: [join(here, "engine.page.ts")], target: "browser", format: "esm" });
if (!enginePage.success) throw new Error(enginePage.logs.map(String).join("\n"));
const pageJs = await enginePage.outputs[0]!.text();
const html = `<!doctype html><html><head><meta charset="utf-8"><title>engine trace</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/bench-page.js"></script></body></html>`;
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  mkdirSync(join(project, "assets"), { recursive: true });
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = opt("--dist", join(repo, "target", "tmp", "webgl2-trace", ONLY));
// --prebuilt: time a build made earlier (another revision's), as it is.
if (!argv.includes("--prebuilt")) {
  const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
  if (!built.ok) throw new Error(built.reason);
}
writeFileSync(join(dist, "bench-page.js"), pageJs);
const server = serve({ dist, isolate: false, extra: { "/bench.html": html } });
const url = `http://127.0.0.1:${server.port}/bench.html${MODE === "webgl2" ? "?webgl=1" : ""}`;

const FRAME = `(async () => { const a = await globalThis.__og_shell__.call("frame", 1000 / 60); if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a)); })()`;

// A shader the software rasterizer refuses at random (an empty info log)
// is tried again in a fresh Chrome, as threejs-compare.ts does.
async function open(shim: boolean) {
  for (let attempt = 1; ; attempt++) {
    const o = await openOnce(shim);
    try {
      for (let i = 0; i < 3; i++) await o.page.eval(FRAME);
      return o;
    } catch (e) {
      await o.browser.close();
      process.stderr.write(`attempt ${attempt}: ${e instanceof Error ? e.message.split("\n")[0] : String(e)}\n`);
      if (attempt >= 5) throw e;
    }
  }
}

async function openOnce(shim: boolean) {
  const browser = await Browser.launch({ chrome, width: W, height: H, flags: ["--disable-frame-rate-limit", ...(process.env.TRACE_CHROME_FLAGS ?? "").split(" ").filter(Boolean)] });
  const log: string[] = [];
  const page = await browser.open("about:blank", log);
  let engine: string | null = null;
  await page.prepareWorkers(async (w, session) => {
    if (!w.url.endsWith("/game.js")) return;
    engine = session;
    if (shim) await page.evalWorker(session, SHIM);
  });
  await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await page.send("Page.navigate", { url });
  await page.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 180_000);
  const failed = await page.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}\n${log.slice(-20).join("\n")}`);
  if (!engine) throw new Error("no engine Worker attached");
  return { browser, page, engine: engine as string, log };
}

// --shot <png>: only a screenshot after exactly --warm frames (in a Chrome
// with the frame-rate limit on, so the capture lands), to compare two
// builds' frames pixel for pixel.
const SHOT = opt("--shot", "");
if (SHOT) {
  const browser = await Browser.launch({ chrome, width: W, height: H, flags: (process.env.TRACE_CHROME_FLAGS ?? "").split(" ").filter(Boolean) });
  const page = await browser.open("about:blank", []);
  await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await page.send("Page.navigate", { url });
  await page.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 180_000);
  const failed = await page.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}`);
  for (let i = 0; i < WARM; i++) await page.eval(FRAME);
  writeFileSync(SHOT, await page.screenshot());
  await browser.close();
  server.stop();
  process.exit(0);
}

// 1. Counts (--times-only skips them).
let tier = "", total = 0;
let perFrame: Record<string, number> = {}, bytesPerFrame: Record<string, number> = {};
if (!argv.includes("--times-only")) {
  const counted = await open(!argv.includes("--no-shim"));
  tier = await counted.page.eval<string>("globalThis.__og_shell__.tier");
  for (let i = 0; i < WARM; i++) await counted.page.eval(FRAME);
  await counted.page.evalWorker(counted.engine, "globalThis.__glc__ && globalThis.__glc__.reset()");
  for (let i = 0; i < FRAMES; i++) await counted.page.eval(FRAME);
  const raw = await counted.page.evalWorker<{ counts: Record<string, number>; bytes: Record<string, number> }>(counted.engine, "globalThis.__glc__ ? globalThis.__glc__.read() : { counts: {}, bytes: {} }");
  const SEQUENCE = opt("--sequence", "");
  if (SEQUENCE) {
    await counted.page.evalWorker(counted.engine, "globalThis.__glc__.record()");
    await counted.page.eval(FRAME);
    writeFileSync(SEQUENCE, JSON.stringify(await counted.page.evalWorker(counted.engine, "globalThis.__glc__.sequence()")));
  }
  await counted.browser.close();
  perFrame = Object.fromEntries(Object.entries(raw.counts).sort((a, b) => b[1] - a[1]).map(([k, v]) => [k, Math.round((v / FRAMES) * 100) / 100]));
  bytesPerFrame = Object.fromEntries(Object.entries(raw.bytes).map(([k, v]) => [k, Math.round(v / FRAMES)]));
  total = Object.values(raw.counts).reduce((a, b) => a + b, 0) / FRAMES;

  console.error(JSON.stringify({ tier, callsPerFrame: total, perFrame, bytesPerFrame }));
  if (argv.includes("--counts-only")) process.exit(0);
}

// 2. Times, unwrapped.
const timed = await open(false);
tier ||= await timed.page.eval<string>("globalThis.__og_shell__.tier");
for (let i = 0; i < WARM; i++) await timed.page.eval(FRAME);
if (PROFILE) {
  await timed.browser.send("Profiler.enable", {}, timed.engine);
  await timed.browser.send("Profiler.setSamplingInterval", { interval: 100 }, timed.engine);
  await timed.browser.send("Profiler.start", {}, timed.engine);
}
// --gap <ms>: a pause before each timed frame, so the GPU process has
// drained and the round trip is the Worker's own work (a software
// rasterizer otherwise bounds every frame).
const GAP = Number(opt("--gap", "0"));
const each = await timed.page.eval<number[]>(`(async () => { const each = []; for (let i = 0; i < ${FRAMES}; i++) { if (${GAP}) await new Promise((r) => setTimeout(r, ${GAP})); const t = performance.now(); await ${FRAME}; each.push(performance.now() - t); } return each; })()`);
if (PROFILE) {
  const r = await timed.browser.send<{ profile: unknown }>("Profiler.stop", {}, timed.engine);
  writeFileSync(PROFILE, JSON.stringify(r.profile));
}
await timed.browser.close();
server.stop();
const sorted = [...each].sort((a, b) => a - b);
const r3 = (v: number) => Math.round(v * 1000) / 1000;
const result = {
  scene: ONLY,
  tier,
  frames: FRAMES,
  callsPerFrame: Math.round(total * 100) / 100,
  perFrame,
  bytesPerFrame,
  gapMs: GAP,
  ms: { p50: r3(sorted[Math.floor(sorted.length / 2)]!), p90: r3(sorted[Math.floor(sorted.length * 0.9)]!), mean: r3(each.reduce((a, b) => a + b, 0) / each.length), min: r3(sorted[0]!) },
};
const outFile = opt("--out", "");
if (outFile) writeFileSync(outFile, JSON.stringify(result, null, 1) + "\n");
console.log(JSON.stringify(result, null, 1));
