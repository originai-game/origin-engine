// GPU pass timing on the WebGL2 tier (the probe's `gpuTimestamps`
// capability and ProbeTimings.gpuPasses), in headless Chrome, twice:
//
// - on a context with EXT_disjoint_timer_query_webgl2: the dev engine
//   (dist/web) reports gpuTimestamps=true and gpuPasses named as the frame's
//   render-graph passes (ProbeTimings.passes), every query read after it was
//   available;
// - on a context without it (Firefox, Safari, and Chrome on SwiftShader,
//   which this machine runs): gpuTimestamps=false and no gpuPasses.
//
// Where Chrome does not expose the extension (no GPU), the first run gives
// the engine Worker's WebGL2 contexts the stand-in of timer-shim.ts, which
// has the extension as ANGLE's Metal backend does (TIME_ELAPSED_EXT only)
// and whose clock is the Worker's, so its numbers say nothing of a GPU; a
// real GPU's come from tools/bench/gpu-passes.ts
// (tests/evidence/debt/webgl2-gpu-timing/NOTES.md). Where Chrome has the
// extension, the first run uses it as it is and the second is skipped.
//
//   bun tests/web/gpu-timing.ts [--web <dir>] [--frames N] [--chrome <path>]
//
// --web serves another engine build's dist/web (default packages/engine's).
// Needs `bun run build` first (dist/web). Exits 1 when a check fails.

import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { TIMER_SHIM } from "./timer-shim.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  serve(o: { port: number; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const web = resolve(opt("--web") ?? join(pkg, "dist", "web"));
const frames = opt("--frames") ?? "40";
const chrome = chromePath();
if (!existsSync(join(web, "origin-engine_bg.wasm"))) {
  console.error(`no engine at ${web}: run \`bun run build\` in packages/engine first`);
  process.exit(1);
}

const built: Record<string, string> = {};
for (const entry of ["gpu-timing.page.ts", "gpu-timing.worker.ts"]) {
  const r = await Bun.build({ entrypoints: [join(here, entry)], target: "browser", format: "esm" });
  if (!r.success) throw new Error(r.logs.map(String).join("\n"));
  built[`/${entry.replace(/\.ts$/, ".js")}`] = await r.outputs[0]!.text();
}
const html = `<!doctype html><html><head><meta charset="utf-8"><title>gpu timing</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:640px;height:360px}</style></head>
<body><canvas></canvas><script type="module" src="./gpu-timing.page.js"></script></body></html>`;
let resolveResult: (v: unknown) => void = () => {};
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const url = new URL(req.url);
    const headers: Record<string, string> = { "cache-control": "no-store", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" };
    if (url.pathname === "/result") {
      resolveResult(await req.json());
      return new Response("ok", { headers });
    }
    if (url.pathname === "/") return new Response(html, { headers: { ...headers, "content-type": "text/html" } });
    if (built[url.pathname]) return new Response(built[url.pathname], { headers: { ...headers, "content-type": "text/javascript" } });
    if (url.pathname.startsWith("/web/")) {
      const file = Bun.file(join(web, url.pathname.slice("/web/".length)));
      if (await file.exists()) {
        const type = url.pathname.endsWith(".wasm") ? "application/wasm" : "text/javascript";
        return new Response(file, { headers: { ...headers, "content-type": type } });
      }
    }
    return new Response("not found", { status: 404, headers });
  },
});

type Timing = { name: string; ms: number };
type Run = {
  tier: string;
  timings: { passes: Timing[]; gpuPasses: Timing[] };
  caps: { gpuTimestamps: boolean; tier: string };
  /** Whether the browser's own WebGL2 contexts list the extension (the page's, never shimmed). */
  native: boolean;
  /** The stand-in's counts (timer-shim.ts), where it stood in. */
  shim: Record<string, number> | null;
};

async function run(shim: boolean): Promise<Run> {
  // No GPU (an orb, CI): WebGL on SwiftShader, which Chrome no longer falls
  // back to by itself; with a GPU the flag changes nothing.
  const browser = await Browser.launch({ chrome, width: 640, height: 360, flags: ["--enable-unsafe-swiftshader"] });
  const log: string[] = [];
  try {
    const page: Page = await browser.open("about:blank", log);
    let engine: string | null = null;
    await page.prepareWorkers(async (worker, session) => {
      if (!worker.url.endsWith("/gpu-timing.worker.js")) return;
      engine = session;
      // The engine Worker's console and exceptions, into the run's log.
      browser.on((method, params, s) => {
        if (s !== session) return;
        if (method === "Runtime.consoleAPICalled") log.push(`worker: ${JSON.stringify(params["args"]).slice(0, 400)}`);
        else if (method === "Runtime.exceptionThrown") log.push(`worker exception: ${JSON.stringify(params["exceptionDetails"]).slice(0, 800)}`);
      });
      await browser.send("Runtime.enable", {}, session);
      if (shim) await page.evalWorker(session, TIMER_SHIM);
    });
    const result = new Promise<unknown>((r) => (resolveResult = r));
    await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/?webgl=1&frames=${frames}` });
    const answer = (await Promise.race([result, new Promise((r) => setTimeout(() => r({ error: "timed out after 120 s" }), 120_000))])) as Run & { error?: string };
    if (answer.error) throw new Error(`${answer.error}\n${log.join("\n")}`);
    if (!engine) throw new Error("the engine Worker never started");
    answer.caps = await page.evalWorker(engine, "globalThis.__og_caps__");
    answer.shim = answer.native ? null : await page.evalWorker(engine, "globalThis.__og_timer_shim__ ?? null");
    return answer;
  } finally {
    await browser.close();
  }
}

let failed = 0;
const check = (ok: boolean, what: string, got?: unknown) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${ok ? "" : `: ${JSON.stringify(got)}`}`);
  if (!ok) failed++;
};
const names = (t: Timing[]) => t.map((p) => p.name);

const timed = await run(true);
console.log(JSON.stringify({ with: timed.native ? "the browser's extension" : "the stand-in extension", ...timed }));
check(timed.tier === "webgl2" && timed.caps.tier === "webgl2", "the page runs the WebGL2 tier", [timed.tier, timed.caps.tier]);
check(timed.caps.gpuTimestamps === true, "gpuTimestamps is true where the context has the extension", timed.caps);
check(timed.timings.gpuPasses.length > 0, "gpuPasses is not empty", timed.timings.gpuPasses);
check(
  JSON.stringify(names(timed.timings.gpuPasses)) === JSON.stringify(names(timed.timings.passes)),
  "gpuPasses are the render graph's passes, in order",
  { gpu: names(timed.timings.gpuPasses), cpu: names(timed.timings.passes) },
);
check(timed.timings.gpuPasses.every((p) => Number.isFinite(p.ms) && p.ms >= 0) && timed.timings.gpuPasses.some((p) => p.ms > 0), "every pass has a time, some above 0", timed.timings.gpuPasses);
if (timed.shim) {
  check(timed.shim["begun"]! > 0 && timed.shim["begun"] === timed.shim["ended"] && timed.shim["nested"] === 0, "TIME_ELAPSED_EXT queries begun and ended in pairs, never nested", timed.shim);
  check(timed.shim["read"]! > 0 && timed.shim["readUnavailable"] === 0, "every result read once available", timed.shim);
  check(timed.shim["queryCounter"] === 0, "no queryCounterEXT (0 TIMESTAMP_EXT bits)", timed.shim);
}

if (timed.native) {
  console.log("skip: this Chrome has the extension, so there is no context without it to run");
} else {
  const plain = await run(false);
  console.log(JSON.stringify({ with: "no extension", ...plain }));
  check(plain.tier === "webgl2", "the page runs the WebGL2 tier", plain.tier);
  check(plain.caps.gpuTimestamps === false, "gpuTimestamps is false without the extension", plain.caps);
  check(plain.timings.gpuPasses.length === 0 && plain.timings.passes.length > 0, "no gpuPasses, CPU passes still timed", plain.timings);
}
server.stop();
process.exit(failed ? 1 : 0);
