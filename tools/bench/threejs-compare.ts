// The design §7 comparison with three.js, in headless Chrome for Testing:
// the same scenes in the engine (WebGPU and the WebGL2 fallback) and in
// three.js 0.180 (WebGLRenderer; WebGPURenderer for the sphere field), on a
// 1280×720 canvas at DPR 1 with vsync and the frame-rate limit off.
//
//   bun tools/bench/threejs-compare.ts [--frames 600] [--warm 60] [--rounds 2]
//       [--only complex|reference|spheres] [--mode <text>] [--shots] [--shot-frames 120] [--shot-wait 30]
//       [--out <dir>] [--chrome <path>]
//
// Scenes:
// - complex: tools/bench/complex, the scene that judges §7's 1.5x budget
//   (WebGPU the primary tier, WebGL2 reported): a product-scale level of a
//   320 m textured ground with a lake, 5,261 props from 19 models in 60
//   materials, 48 skinned characters on 14 clips, the sun's cascades and
//   four shadowed local lights, 20 particle emitters, blended water, glass
//   and ghosts, AO, bloom, ACES and the tier's antialiasing; and
//   three-complex.js, its three.js rebuild from the same assets and the
//   same src/layout.js (InstancedMesh per 40 m tile, or per part over the
//   whole level; WebGLRenderer with SMAA, WebGPURenderer with TRAA).
//   Its assets are made by complex/tools/make-assets.ts on first use;
// - reference: crates/og-tests/reference-scene (500 turning objects in four
//   PBR materials, 20 skinned walkers, a shadowed sun and spot, sky IBL,
//   AO, bloom, ACES, TAA), and three-reference.js, its three.js rebuild
//   (instanced and one Mesh per object; SMAA for TAA);
// - spheres: tools/bench/spheres (20,000 spheres, each moved every tick by
//   a TS system), and three-spheres.js (one Mesh each, and InstancedMesh).
//
// What a frame time is, on both sides: the wall time from one frame to the
// next when frames run back to back, each advancing the simulation exactly
// 1/60 s and rendering once. three.js: the interval between consecutive
// requestAnimationFrame callbacks (no vsync, no frame-rate limit, so the
// next callback comes as soon as the browser has presented the last frame).
// The engine: the round trip of one `frame` call to the engine Worker (one
// tick and one render, as packages/engine/bench/lanes.ts times it): its
// own requestAnimationFrame runs the fixed 60 Hz tick on wall time, so with
// frames uncapped it would tick on fewer frames than it renders and do less
// work per frame than three.js. A frame call's GPU work is submitted, not
// waited for; over hundreds of back-to-back frames the GPU's queue throttles
// submission, so `sustained` (all frames' wall time over their count) is
// bounded by the GPU on both sides. `warm` frames before the timed ones are
// not counted. Engine builds are `og build --publish`'s web half, served
// cross-origin isolated (so the shell takes the lanes variant, as the
// product page does on an isolated host); the three.js pages are served the
// same way. Each run gets its own Chrome; the modes are interleaved round by
// round, and every run records the machine's load average (`uptime`).
//
// Needs packages/engine/dist (scripts/build.sh) and `bun install
// --frozen-lockfile` in tools/bench. Writes <out>/threejs-comparison.json
// and a screenshot per mode (default tests/evidence/P4/web-host-tools/).

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../packages/engine/src/node/web-build.ts";
import { Browser, chromePath, type Page } from "../../packages/engine/tests/web/cdp.ts";
import { serve, TYPES } from "../../packages/engine/tests/web/serve.ts";
import { ogBin } from "../../packages/engine/tests/support/og-bin.ts";

interface Hasher {
  update(data: string | Uint8Array): Hasher;
  digest(encoding: "hex"): string;
}
declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  serve(o: { port: number; hostname?: string; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
  CryptoHasher: new (algorithm: string) => Hasher;
};

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const out = opt("--out", join(repo, "tests", "evidence", "P4", "web-host-tools"));
const FRAMES = Number(opt("--frames", "600"));
const WARM = Number(opt("--warm", "60"));
const ROUNDS = Number(opt("--rounds", "2"));
const ONLY = opt("--only", "");
/** --shots: only the screenshots (no timing). */
const SHOTS_ONLY = argv.includes("--shots");
/** --mode <text>: only the modes whose name contains it (e.g. `engine`, `WebGL2`). */
const MODE = opt("--mode", "");
/** --shot-frames N, --shot-wait S: frames played before a screenshot (120) and how long to wait for it (30 s); fewer and longer on a CPU-only machine. */
const SHOT_FRAMES = Number(opt("--shot-frames", "120"));
const SHOT_WAIT = Number(opt("--shot-wait", "30"));
const W = 1280, H = 720;
mkdirSync(out, { recursive: true });
const chrome = chromePath(argv.includes("--chrome") ? ["--chrome", opt("--chrome", "")] : []);
/** BENCH_CHROME_FLAGS: more Chrome flags (an orb without a GPU: `--use-angle=gl-egl`, llvmpipe's GL, since SwiftShader refuses the engine's msdf-text shader on WebGL2). */
const extraFlags = (process.env.BENCH_CHROME_FLAGS ?? "").split(" ").filter(Boolean);

const SCENES = {
  complex: { project: join(here, "complex"), three: "complex" },
  reference: { project: join(repo, "crates", "og-tests", "reference-scene"), three: "reference" },
  spheres: { project: join(here, "spheres"), three: "spheres" },
} as const;
type SceneName = keyof typeof SCENES;

/** One mode: who renders the scene and how. */
interface Mode {
  readonly scene: SceneName;
  readonly name: string;
  readonly side: "engine" | "three";
  /** The page's query. */
  readonly query: string;
}
const MODES: Mode[] = [
  { scene: "complex", name: "engine WebGPU", side: "engine", query: "" },
  { scene: "complex", name: "engine WebGL2", side: "engine", query: "?webgl=1" },
  { scene: "complex", name: "three WebGL2, InstancedMesh per tile", side: "three", query: "?scene=complex" },
  { scene: "complex", name: "three WebGL2, InstancedMesh per part", side: "three", query: "?scene=complex&tiles=0" },
  { scene: "complex", name: "three WebGPU, InstancedMesh per tile", side: "three", query: "?scene=complex&webgpu=1" },
  { scene: "reference", name: "engine WebGPU", side: "engine", query: "" },
  { scene: "reference", name: "engine WebGL2", side: "engine", query: "?webgl=1" },
  { scene: "reference", name: "three WebGL2, InstancedMesh", side: "three", query: "?scene=reference" },
  { scene: "reference", name: "three WebGL2, one Mesh each", side: "three", query: "?scene=reference&naive=1" },
  { scene: "spheres", name: "engine WebGPU", side: "engine", query: "" },
  { scene: "spheres", name: "engine WebGL2", side: "engine", query: "?webgl=1" },
  { scene: "spheres", name: "three WebGL2, InstancedMesh", side: "three", query: "?scene=spheres" },
  { scene: "spheres", name: "three WebGL2, one Mesh each", side: "three", query: "?scene=spheres&naive=1" },
  { scene: "spheres", name: "three WebGPU, InstancedMesh", side: "three", query: "?scene=spheres&webgpu=1" },
].filter((m) => (!ONLY || m.scene === ONLY) && (!MODE || m.name.toLowerCase().includes(MODE.toLowerCase())));

const r3 = (v: number) => Math.round(v * 1000) / 1000;
const q = (s: number[], p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
function stats(xs: number[]) {
  const s = [...xs].sort((a, b) => a - b);
  return { p50: r3(q(s, 0.5)), p95: r3(q(s, 0.95)), p99: r3(q(s, 0.99)), max: r3(s.at(-1)!), mean: r3(xs.reduce((a, b) => a + b, 0) / xs.length) };
}
const load = () => new TextDecoder().decode(Bun.spawnSync(["uptime"]).stdout).trim().replace(/.*load averages?: /, "");

// ── the engine builds ───────────────────────────────────────────────────
const enginePage = await Bun.build({ entrypoints: [join(here, "engine.page.ts")], target: "browser", format: "esm" });
if (!enginePage.success) throw new Error(enginePage.logs.map(String).join("\n"));
const pageJs = await enginePage.outputs[0]!.text();
const engineHtml = `<!doctype html><html><head><meta charset="utf-8"><title>engine bench</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/bench-page.js"></script></body></html>`;
const dists: Partial<Record<SceneName, string>> = {};
for (const scene of new Set(MODES.map((m) => m.scene))) {
  const { project } = SCENES[scene];
  // The complex scene's assets are generated (complex/tools/make-assets.ts, not committed): made again
  // when the generator or the layout it reads changed since assets/.made, then cooked again.
  const maker = join(project, "tools", "make-assets.ts");
  let stale = false;
  if (existsSync(maker)) {
    const stamp = new Bun.CryptoHasher("sha256").update(readFileSync(maker)).update(readFileSync(join(project, "src", "layout.js"))).digest("hex");
    const made = join(project, "assets", ".made");
    if (!existsSync(made) || readFileSync(made, "utf8").trim() !== stamp) {
      process.stderr.write(`making ${scene}'s assets (${maker})\n`);
      const r = Bun.spawnSync([process.execPath, maker], { cwd: repo, stdout: "ignore", stderr: "pipe" });
      if (r.exitCode !== 0) throw new Error(`making ${scene}'s assets: ${new TextDecoder().decode(r.stderr)}`);
      writeFileSync(made, stamp + "\n");
      stale = true;
    }
  }
  if (stale || !existsSync(join(project, "cooked", ".cook.ogt"))) {
    // og cook wants an assets/ even when the game makes everything in code (the sphere field).
    mkdirSync(join(project, "assets"), { recursive: true });
    const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
    if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
  }
  const dist = join(repo, "target", "tmp", "threejs-bench", scene);
  process.stderr.write(`building ${scene} (og build --publish's web half) into ${dist}\n`);
  const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
  if (!built.ok) throw new Error(`${scene}: ${built.reason}`);
  if (!existsSync(join(dist, "origin-engine-lanes_bg.wasm"))) throw new Error(`${scene}: the publish build has no lanes variant`);
  writeFileSync(join(dist, "bench-page.js"), pageJs);
  dists[scene] = dist;
}

// ── the three.js pages: tools/bench under /three/, the walker under /scene/ ──
const isolation = { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "cross-origin", "cache-control": "no-store" };
const threeServer = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const url = new URL(req.url);
    let path: string | null = null;
    if (url.pathname === "/three-page.html") path = join(here, "three-page.html");
    else if (url.pathname.startsWith("/three/")) path = normalize(join(here, decodeURIComponent(url.pathname.slice("/three/".length))));
    else if (url.pathname === "/scene/walker.glb") path = join(SCENES.reference.project, "assets", "models", "walker.glb");
    if (!path || !path.startsWith(repo) || !(await Bun.file(path).exists())) return new Response("not found", { status: 404, headers: isolation });
    const ext = path.slice(path.lastIndexOf("."));
    return new Response(Bun.file(path), { headers: { ...isolation, "content-type": TYPES[ext] ?? (ext === ".glb" ? "model/gltf-binary" : "application/octet-stream") } });
  },
});

interface Run {
  scene: SceneName;
  mode: string;
  round: number;
  load: string;
  info: Record<string, unknown>;
  frames: number;
  ms: ReturnType<typeof stats>;
  sustainedMs: number;
  fps: number;
  shot: string;
}

async function runEngine(page: Page, m: Mode): Promise<{ each: number[]; total: number; info: Record<string, unknown> }> {
  await page.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 120_000);
  const failed = await page.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}\n${(await page.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-20).join("\n")}`);
  // The loop runs in the page; the harness polls its progress, so a stall is seen, not waited on.
  await page.eval(`void (globalThis.__bench__ = { done: 0, result: null, error: null, promise: (async () => {
    const shell = globalThis.__og_shell__;
    const frame = async () => {
      const a = await shell.call("frame", 1000 / 60);
      if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
      globalThis.__bench__.done++;
    };
    for (let i = 0; i < ${WARM}; i++) await frame();
    const each = [];
    const t0 = performance.now();
    let last = t0;
    for (let i = 0; i < ${FRAMES}; i++) {
      await frame();
      const now = performance.now();
      each.push(now - last);
      last = now;
    }
    const total = performance.now() - t0;
    // One more frame so the screenshot shows a presented frame of the run.
    await frame();
    globalThis.__bench__.result = { each, total, info: { tier: shell.tier, lanes: shell.ready.lanes, isolated: globalThis.crossOriginIsolated === true, scene: ${JSON.stringify(m.scene)} } };
  })().catch((e) => { globalThis.__bench__.error = String(e && e.stack || e); }) })`);
  let seen = -1, since = performance.now();
  for (;;) {
    const s = await page.eval<{ done: number; result: { each: number[]; total: number; info: Record<string, unknown> } | null; error: string | null }>("({ done: __bench__.done, result: __bench__.result, error: __bench__.error })");
    if (s.error) throw new Error(s.error);
    if (s.result) return s.result;
    if (s.done !== seen) {
      seen = s.done;
      since = performance.now();
    } else if (performance.now() - since > 60_000) throw new Error(`the engine answered no frame for 60 s after ${s.done} frames; log: ${(await page.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-10).join(" | ")}`);
    await new Promise((r) => setTimeout(r, 250));
  }
}

async function runThree(page: Page): Promise<{ each: number[]; total: number; info: Record<string, unknown> }> {
  await page.until("!!(globalThis.__bench_ready__ || globalThis.__bench_error__)", 120_000);
  const failed = await page.eval<string | undefined>("globalThis.__bench_error__");
  if (failed) throw new Error(`the three.js page failed: ${failed}`);
  const r = await page.eval<{ each: number[]; total: number }>(`__bench_run__(${WARM}, ${FRAMES})`);
  return { ...r, info: { ...(await page.eval<Record<string, unknown>>("globalThis.__bench_info__")), isolated: await page.eval<boolean>("crossOriginIsolated") } };
}

const shotName = (m: Mode) => `threejs-${m.scene}-${m.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}.png`;
const pageUrl = (m: Mode) => (m.side === "engine" ? `http://127.0.0.1:${servers[m.scene]!.port}/bench.html${m.query}` : `http://127.0.0.1:${threeServer.port}/three-page.html${m.query}`);

/**
 * A screenshot of each mode, in a Chrome of its own with the frame-rate limit
 * on (with it off, Page.captureScreenshot never lands): what each side drew,
 * so the scenes can be compared by eye. Not timed.
 */
async function shots(): Promise<void> {
  for (const m of MODES) {
    const browser = await Browser.launch({ chrome, width: W, height: H, flags: extraFlags });
    try {
      const page = await browser.open("about:blank", []);
      await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
      await page.send("Page.navigate", { url: pageUrl(m) });
      if (m.side === "engine") {
        await page.until("!!globalThis.__og_shell__", 120_000);
        await page.eval(`(async () => { for (let i = 0; i < ${SHOT_FRAMES}; i++) await globalThis.__og_shell__.call("frame", 1000 / 60); })()`);
        await page.eval(`void (globalThis.__keep__ = setInterval(() => globalThis.__og_shell__.call("frame", 1000 / 60).catch(() => {}), 30))`);
      } else {
        await page.until("!!(globalThis.__bench_ready__ || globalThis.__bench_error__)", 120_000);
        await page.eval(`(() => { for (let i = 0; i < ${SHOT_FRAMES}; i++) globalThis.__bench_step__(); globalThis.__keep__ = setInterval(() => globalThis.__bench_step__(), 30); })()`);
      }
      const png = await Promise.race([page.screenshot(), new Promise<null>((r) => setTimeout(() => r(null), SHOT_WAIT * 1000))]);
      if (png) writeFileSync(join(out, shotName(m)), png);
      process.stderr.write(`${m.scene} / ${m.name}: ${png ? shotName(m) : `no screenshot within ${SHOT_WAIT} s`}\n`);
    } catch (e) {
      process.stderr.write(`${m.scene} / ${m.name}: no screenshot (${e instanceof Error ? e.message : String(e)})\n`);
    } finally {
      await browser.close();
    }
  }
}

const runs: Run[] = [];
const servers = Object.fromEntries(Object.entries(dists).map(([scene, dist]) => [scene, serve({ dist: dist!, isolate: true, extra: { "/bench.html": engineHtml } })]));
try {
  if (SHOTS_ONLY) {
    await shots();
    process.exit(0);
  }
  for (let round = 1; round <= ROUNDS; round++) {
    for (const m of MODES) for (let attempt = 1; attempt <= 3; attempt++) {
      const browser = await Browser.launch({ chrome, width: W, height: H, flags: ["--disable-frame-rate-limit", ...extraFlags] });
      const log: string[] = [];
      const label = `${m.scene} / ${m.name} / round ${round}`;
      try {
        const page = await browser.open("about:blank", log);
        await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
        const url = pageUrl(m);
        process.stderr.write(`${label}: ${url}\n`);
        await page.send("Page.navigate", { url });
        const before = load();
        const r = m.side === "engine" ? await runEngine(page, m) : await runThree(page);
        const run: Run = {
          scene: m.scene, mode: m.name, round, load: before, info: r.info, frames: FRAMES,
          ms: stats(r.each), sustainedMs: r3(r.total / FRAMES), fps: Math.round((1000 * FRAMES) / r.total), shot: shotName(m),
        };
        runs.push(run);
        attempt = 3;
        const errors = log.filter((l) => /^(error|exception)/.test(l));
        process.stderr.write(`${label}: p50 ${run.ms.p50} p95 ${run.ms.p95} ms, sustained ${run.sustainedMs} ms (${run.fps} fps); load ${before}${errors.length ? `; console: ${errors.slice(0, 3).join(" | ")}` : ""}\n`);
      } catch (e) {
        process.stderr.write(`${label}: FAILED (attempt ${attempt} of 3) ${e instanceof Error ? e.message : String(e)}\n${log.slice(-15).join("\n")}\n`);
        // A WebGL context or WebGPU adapter refused on a loaded machine is tried again in a fresh Chrome.
        if (attempt < 3) continue;
        runs.push({ scene: m.scene, mode: m.name, round, load: load(), info: { error: e instanceof Error ? e.message : String(e) }, frames: 0, ms: { p50: NaN, p95: NaN, p99: NaN, max: NaN, mean: NaN }, sustainedMs: NaN, fps: 0, shot: "" });
      } finally {
        await browser.close();
      }
    }
  }
  await shots();
} finally {
  for (const s of Object.values(servers)) s.stop();
  threeServer.stop();
}

// Per mode: each figure's mean over the rounds; fps from the mean sustained frame time.
const mean = (xs: number[]) => {
  const v = xs.filter((x) => Number.isFinite(x));
  return v.length ? r3(v.reduce((a, b) => a + b, 0) / v.length) : NaN;
};
const summary = MODES.map((m) => {
  const mine = runs.filter((r) => r.scene === m.scene && r.mode === m.name && r.frames > 0);
  const sustainedMs = mean(mine.map((r) => r.sustainedMs));
  return { scene: m.scene, mode: m.name, p50: mean(mine.map((r) => r.ms.p50)), p95: mean(mine.map((r) => r.ms.p95)), sustainedMs, fps: Math.round(1000 / sustainedMs), rounds: mine.length };
});
writeFileSync(join(out, "threejs-comparison.json"), JSON.stringify({ chrome, frames: FRAMES, warm: WARM, rounds: ROUNDS, size: `${W}x${H}`, summary, runs }, null, 1) + "\n");
console.log("| scene | mode | p50 ms | p95 ms | sustained ms | fps |");
console.log("|---|---|---|---|---|---|");
for (const s of summary) console.log(`| ${s.scene} | ${s.mode} | ${s.p50} | ${s.p95} | ${s.sustainedMs} | ${s.fps} |`);
