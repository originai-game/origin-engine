// The WebGL calls one frame of the reference scene sends out of wasm on the
// WebGL2 tier, in headless Chrome: fewer than MAX_CALLS_PER_FRAME. This holds
// the carried wgpu-hal's second patch (third_party/wgpu-hal/ORIGIN.md, its
// GLES queue skipping GL calls that set what is already set): upstream
// wgpu-hal 30.0.1 sends 3,797 a frame, the patched one about 1,335
// (tests/evidence/debt3/wgpu-hal-vendor/NOTES.md).
//
// The calls are counted as tools/bench/webgl2-trace.ts counts them, in the
// engine Worker before its first line (cdp.ts prepareWorkers), but in front
// of the JS state filter (src/web/gl-state.ts) rather than behind it: the
// filter drops most restated calls itself, so behind it the patched and the
// upstream hal differ by a few percent (732 against 761), while in front of
// it they differ by almost three times. The count behind the filter (what
// reaches WebGL) is printed too, and checked to be no more than the one in
// front of it.
//
// The reference scene (crates/og-tests/reference-scene) is built with
// `og build --publish`'s web half and served without isolation (the
// single-threaded module, as webgl2-trace.ts does); WARM frames are driven
// one `frame` call at a time, then FRAMES more are counted.
//
//   bun tests/web/gl-calls.ts [--frames N] [--warm N] [--out <dir>] [--chrome <path>]
//
// Needs dist/web-publish (scripts/build.sh). On a machine without a GPU,
// Chrome 156's SwiftShader refuses one of the scene's shaders on WebGL2:
// TRACE_CHROME_FLAGS="--use-angle=gl-egl" draws with llvmpipe's GL instead
// (the counts are the same on any GPU). Writes <out>/gl-calls.json (default
// dist/test-gl-calls/out). Exits 1 when a check fails.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { cookProject } from "../support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
};

/** The ceiling on calls a frame in front of the JS filter: upstream wgpu-hal 30.0.1 sends 3,797. */
export const MAX_CALLS_PER_FRAME = 1_500;

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const work = join(pkg, "dist", "test-gl-calls");
const out = resolve(opt("--out") ?? join(work, "out"));
mkdirSync(out, { recursive: true });
const chrome = chromePath(args);
const FRAMES = Number(opt("--frames") ?? "60");
const WARM = Number(opt("--warm") ?? "60");
const W = 1280, H = 720;

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(ok && detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 600)}` : ""}`);
}

// Runs in the engine Worker before its first line. Two counters:
// - behind the filter: every WebGL2RenderingContext.prototype method,
//   wrapped before the filter reads them (as webgl2-trace.ts does);
// - in front of it: each WebGL2 context the engine opens is remembered, and
//   when counting starts a prototype that counts every method is put between
//   the context and the filter's own (gl-state.ts puts its methods on a
//   prototype between the context and WebGL2RenderingContext's), so every
//   call wasm makes on the context passes through it once.
const COUNTER = `(() => {
  const behind = Object.create(null);
  const front = Object.create(null);
  const proto = globalThis.WebGL2RenderingContext && WebGL2RenderingContext.prototype;
  if (!proto) return;
  const names = [];
  for (const k of Object.getOwnPropertyNames(proto)) {
    const d = Object.getOwnPropertyDescriptor(proto, k);
    if (!d || typeof d.value !== "function" || k === "constructor") continue;
    names.push(k);
    const f = d.value;
    proto[k] = function (...a) { behind[k] = (behind[k] || 0) + 1; return f.apply(this, a); };
  }
  const contexts = [];
  const getContext = OffscreenCanvas.prototype.getContext;
  OffscreenCanvas.prototype.getContext = function (id, ...rest) {
    const ctx = getContext.call(this, id, ...rest);
    if (id === "webgl2" && ctx && !contexts.includes(ctx)) contexts.push(ctx);
    return ctx;
  };
  const counted = new WeakSet();
  globalThis.__glc__ = {
    start() {
      for (const ctx of contexts) {
        if (counted.has(ctx)) continue;
        counted.add(ctx);
        const next = Object.getPrototypeOf(ctx);
        const layer = Object.create(next);
        for (const k of names) {
          Object.defineProperty(layer, k, { configurable: true, writable: true, value: function (...a) { front[k] = (front[k] || 0) + 1; return next[k].apply(this, a); } });
        }
        Object.setPrototypeOf(ctx, layer);
      }
      for (const k in behind) delete behind[k];
      for (const k in front) delete front[k];
      return { contexts: contexts.length, filtered: contexts.map((c) => Object.getPrototypeOf(Object.getPrototypeOf(c)) !== proto) };
    },
    read() { return { front: { ...front }, behind: { ...behind } }; },
  };
})();`;

const project = join(repo, "crates", "og-tests", "reference-scene");
cookProject(project, repo);
const dist = join(work, "build");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "gl-calls.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "gl-calls-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><title>gl calls</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/gl-calls-page.js"></script></body></html>`;
const server = serve({ dist, isolate: false, extra: { "/gl-calls.html": html } });

const FRAME = `(async () => { const a = await globalThis.__og_shell__.call("frame", 1000 / 60); if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a)); })()`;
const sum = (o: Record<string, number>) => Object.values(o).reduce((a, b) => a + b, 0);
const perFrame = (o: Record<string, number>) =>
  Object.fromEntries(Object.entries(o).sort((a, b) => b[1] - a[1]).map(([k, v]) => [k, Math.round((v / FRAMES) * 100) / 100]));

const flags = ["--disable-frame-rate-limit", ...(process.env.TRACE_CHROME_FLAGS ?? "").split(" ").filter(Boolean)];
const browser = await Browser.launch({ chrome, width: W, height: H, flags });
const log: string[] = [];
let result: Record<string, unknown> = {};
try {
  const tab = await browser.open("about:blank", log);
  let engine: string | null = null;
  await tab.prepareWorkers(async (w, session) => {
    if (!w.url.endsWith("/game.js")) return;
    engine = session;
    await tab.evalWorker(session, COUNTER);
  });
  await tab.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/gl-calls.html` });
  await tab.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 180_000);
  const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}\n${log.slice(-20).join("\n")}`);
  if (!engine) throw new Error("no engine Worker attached");
  const tier = await tab.eval<string>("globalThis.__og_shell__.tier");
  check(tier === "webgl2", "the page runs the WebGL2 tier", tier);
  for (let i = 0; i < WARM; i++) await tab.eval(FRAME);
  const started = await tab.evalWorker<{ contexts: number; filtered: boolean[] }>(engine, "globalThis.__glc__.start()");
  check(started.contexts > 0 && started.filtered.every(Boolean), "the engine draws through the JS state filter", started);
  for (let i = 0; i < FRAMES; i++) await tab.eval(FRAME);
  const raw = await tab.evalWorker<{ front: Record<string, number>; behind: Record<string, number> }>(engine, "globalThis.__glc__.read()");
  const front = sum(raw.front) / FRAMES;
  const behind = sum(raw.behind) / FRAMES;
  const draws = ["drawArrays", "drawElements", "drawArraysInstanced", "drawElementsInstanced", "drawRangeElements"].reduce((a, k) => a + (raw.front[k] ?? 0), 0) / FRAMES;
  // A counter that saw nothing would pass the ceiling: the scene's 41 draws must be there.
  check(draws >= 30, `the counter sees the scene's draws (${draws} a frame)`, draws);
  check(front < MAX_CALLS_PER_FRAME, `${Math.round(front)} calls a frame leave wasm, fewer than ${MAX_CALLS_PER_FRAME} (upstream wgpu-hal: 3,797)`, perFrame(raw.front));
  check(behind <= front, `${Math.round(behind)} of them reach WebGL behind the JS filter`, { front, behind });
  result = { tier, frames: FRAMES, warm: WARM, callsPerFrame: { front, behind, draws }, front: perFrame(raw.front), behind: perFrame(raw.behind) };
} catch (e) {
  check(false, "runs", { error: String(e), log: log.slice(-20) });
} finally {
  await browser.close();
  server.stop();
}
writeFileSync(join(out, "gl-calls.json"), JSON.stringify({ max: MAX_CALLS_PER_FRAME, ...result, checks }, null, 1) + "\n");
console.log(`${checks.length - failures} of ${checks.length} checks passed (${join(out, "gl-calls.json")})`);
process.exit(failures === 0 ? 0 : 1);
