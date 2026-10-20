// A CPU profile of the engine Worker drawing tools/bench/spheres in headless
// Chrome, and its frame and probe timings: where the sphere field's frame
// goes on the web (owed row 50, docs/phase-1.md).
//
//   bun tools/bench/profile-spheres.ts [--count 20000] [--frames 120] [--webgl]
//       [--single] [--names] [--top 40] [--chrome <path>] [--json <file>]
//
// --single serves the page without cross-origin isolation (the shell takes
// the single-threaded wasm); --names binds the publish wasm again without
// wasm-opt, so the profile names Rust functions (the timings of such a run
// are not the shipping module's). Needs packages/engine/dist (scripts/build.sh).
// Prints the frame timings, the probe's step and render medians, and the
// functions with the most self time in the engine Worker.

import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
const COUNT = Number(opt("--count", "20000"));
const FRAMES = Number(opt("--frames", "120"));
const TOP = Number(opt("--top", "40"));
const WEBGL = argv.includes("--webgl");
const SINGLE = argv.includes("--single");
const NAMES = argv.includes("--names");
const W = 1280, H = 720;
const chrome = chromePath(argv.includes("--chrome") ? ["--chrome", opt("--chrome", "")] : []);

// The project with its field's size set (a constant of the game).
const work = join(repo, "target", "tmp", "profile-spheres");
const project = join(work, `spheres-${COUNT}`);
if (!existsSync(project)) {
  cpSync(join(here, "spheres"), project, { recursive: true, filter: (p) => !p.includes("/cooked") && !p.includes("/og-out") });
  const main = join(project, "src", "main.ts");
  writeFileSync(main, readFileSync(main, "utf8").replace("export const COUNT = 20_000;", `export const COUNT = ${COUNT};`));
}
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  mkdirSync(join(project, "assets"), { recursive: true });
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = join(work, `dist-${COUNT}`);
rmSync(dist, { recursive: true, force: true });
const built = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
if (NAMES) {
  // The same publish modules bound again from cargo's output, without wasm-opt (names kept).
  const target = process.env.CARGO_TARGET_DIR ?? join(repo, "target");
  for (const [raw, name] of [
    [join(target, "wasm-publish", "wasm32-unknown-unknown", "wasm", "og_host_web.wasm"), "origin-engine"],
    [join(target, "wasm-mt", "wasm32-unknown-unknown", "wasm", "og_host_web.wasm"), "origin-engine-lanes"],
  ] as const) {
    if (!existsSync(raw)) throw new Error(`${raw} is missing: build the web host first`);
    const tmp = join(work, "bindgen", name);
    rmSync(tmp, { recursive: true, force: true });
    const r = Bun.spawnSync(["wasm-bindgen", "--target", "web", "--keep-debug", "--out-dir", tmp, "--out-name", name, raw], { stdout: "ignore", stderr: "pipe" });
    if (r.exitCode !== 0) throw new Error(`wasm-bindgen ${raw}: ${new TextDecoder().decode(r.stderr)}`);
    cpSync(join(tmp, `${name}_bg.wasm`), join(dist, `${name}_bg.wasm`));
    cpSync(join(tmp, `${name}.js`), join(dist, `${name}.js`));
  }
}
const page = await Bun.build({ entrypoints: [join(here, "engine.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "bench-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><title>spheres profile</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/bench-page.js"></script></body></html>`;
const server = serve({ dist, isolate: !SINGLE, extra: { "/bench.html": html } });
const browser = await Browser.launch({ chrome, width: W, height: H, flags: ["--disable-frame-rate-limit"] });

type Node = { id: number; callFrame: { functionName: string; url: string }; hitCount?: number; children?: number[] };
type Profile = { nodes: Node[]; samples: number[]; timeDeltas: number[]; startTime: number; endTime: number };
const workers = new Map<string, string>();
browser.on((method, params) => {
  if (method !== "Target.attachedToTarget") return;
  const info = params["targetInfo"] as { type: string; url: string };
  if (info.type === "worker") workers.set(params["sessionId"] as string, info.url);
});
try {
  const tab = await browser.open("about:blank", []);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/bench.html${WEBGL ? "?webgl=1" : ""}` });
  await tab.until("!!(globalThis.__og_shell__ || globalThis.__og_shell_error__)", 180_000);
  const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(failed);
  const engineSession = [...workers].find(([, url]) => url.endsWith("/game.js"))?.[0];
  if (!engineSession) throw new Error(`no engine Worker among ${[...workers.values()].join(", ")}`);
  const frames = (n: number) => tab.eval<{ each: number[]; step: number[]; render: number[]; total: number }>(`(async () => {
    const shell = globalThis.__og_shell__;
    const each = [], step = [], render = [];
    const t0 = performance.now();
    for (let i = 0; i < ${n}; i++) {
      const t = performance.now();
      const a = await shell.call("frame", 1000 / 60);
      if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
      each.push(performance.now() - t);
      const s = await shell.probe("state");
      step.push(s.timings.step); render.push(s.timings.render);
    }
    return { each, step, render, total: performance.now() - t0 };
  })()`);
  await frames(30);
  const timed = await frames(FRAMES);
  await browser.send("Profiler.enable", {}, engineSession);
  await browser.send("Profiler.setSamplingInterval", { interval: 100 }, engineSession);
  await browser.send("Profiler.start", {}, engineSession);
  await frames(Math.max(30, FRAMES / 2));
  const { profile } = await browser.send<{ profile: Profile }>("Profiler.stop", {}, engineSession);
  const med = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)]!;
  const info = await tab.eval<Record<string, unknown>>(`({ tier: __og_shell__.tier, lanes: __og_shell__.ready.lanes, isolated: crossOriginIsolated })`);
  // Self and inclusive time per function name.
  const byId = new Map(profile.nodes.map((n) => [n.id, n]));
  const parent = new Map<number, number>();
  for (const n of profile.nodes) for (const c of n.children ?? []) parent.set(c, n.id);
  const self = new Map<string, number>(), incl = new Map<string, number>();
  let total = 0;
  for (let i = 0; i < profile.samples.length; i++) {
    const dt = profile.timeDeltas[i] ?? 0;
    let id: number | undefined = profile.samples[i]!;
    const leaf = byId.get(id)!.callFrame.functionName || "(anonymous)";
    if (leaf === "(idle)" || leaf === "(program)") continue;
    total += dt;
    self.set(leaf, (self.get(leaf) ?? 0) + dt);
    const seen = new Set<string>();
    while (id !== undefined) {
      const name = byId.get(id)!.callFrame.functionName || "(anonymous)";
      if (!seen.has(name)) { seen.add(name); incl.set(name, (incl.get(name) ?? 0) + dt); }
      id = parent.get(id);
    }
  }
  const fmt = (m: Map<string, number>) => [...m].sort((a, b) => b[1] - a[1]).slice(0, TOP).map(([n, us]) => `${(us / 1000).toFixed(1).padStart(9)} ms ${((100 * us) / total).toFixed(1).padStart(5)} %  ${n.slice(0, 160)}`).join("\n");
  const report = {
    count: COUNT, frames: FRAMES, info, names: NAMES,
    frameMs: med(timed.each), sustainedMs: timed.total / FRAMES, stepMs: med(timed.step), renderMs: med(timed.render),
    profiledMs: total / 1000,
  };
  console.log(JSON.stringify(report));
  console.log(`self (busy ${(total / 1000).toFixed(1)} ms):\n${fmt(self)}\ninclusive:\n${fmt(incl)}`);
  if (argv.includes("--json")) writeFileSync(opt("--json", ""), JSON.stringify({ report, profile }) + "\n");
} finally {
  await browser.close();
  server.stop();
}
