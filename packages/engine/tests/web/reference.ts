// The browser reference tiers of the effect and overlay frames:
// templates/fx-showcase's five stations (every E1 effect, each under the
// screen-space HUD and og.ui badge the overlay pass draws) rendered in
// headless Chrome on WebGPU (or the WebGL2 fallback with --webgl) at
// 1280×720, DPR 1, from the dev web export's engine Worker, and compared
// with templates/fx-showcase/references/<tier>/ (`webgpu` or `webgl2`) when
// that tier is committed, else with `metal`, under og-tests' rule
// (crates/og-tests/src/compare.rs): a channel off by more than 32/255 is an
// outlier; fail above 0.5 % outliers or an RGB RMSE above 0.01.
//
// The page (reference.page.ts) starts the export's game.js with no gate and
// no clock; the harness drives it as `og run --commands` drives the native
// host, one `frame` of one tick and one render at a time, so TAA, motion
// blur and particles see every frame the native references saw. The
// stations are the playtest's (tests/frames.og.ts): the same `station:`
// commands at the same seconds, each frame captured at its screenshot's tick.
// First the Worker gets `call("probe", "commands", {…})` with an object and
// must answer the probe's refusal (it once trapped the wasm), then runs them.
//
//   bun tests/web/reference.ts [--webgl] [--record] [--out <dir>] [--chrome <path>]
//   make web-reference                        both tiers
//
// --record writes the frames as references/<tier>/ instead of comparing.
// Writes <out>/<tier>/<station>.png, <station>.diff.rgba on a mismatch (raw
// RGBA, differences x4, outliers red) and reference-<tier>.json (default
// dist/test-reference/out; on an error, with the engine's last log lines).
// Exits 1 when a station does not match, the engine does not start or the
// probe does not refuse. Needs Chrome (cdp.ts: a missing one is an error).

import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";
import { decodePng, type Image } from "../support/png.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "fx-showcase");
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const record = args.includes("--record");
const work = join(pkg, "dist", "test-reference");
const out = resolve(opt("--out") ?? join(work, "out"));
const chrome = chromePath(args);
const TICK_RATE = 60;
const WIDTH = 1280;
const HEIGHT = 720;

/** The playtest's stations (tests/frames.og.ts): switched at `at` seconds, captured at tick `tick`, the tick its `screenshot` runs at. */
const STATIONS = [
  { name: "post", at: 0, tick: 60 },
  { name: "lens", at: 1.1, tick: 120 },
  { name: "fx", at: 2.1, tick: 240 },
  { name: "2d", at: 4.1, tick: 300 },
  { name: "stylised", at: 5.1, tick: 420 },
] as const;
const SCRIPT = STATIONS.filter((s) => s.at > 0).map((s) => `${s.at}:station:${s.name}`).join(";");

/** og-tests' rule (crates/og-tests/src/compare.rs `Tolerance::default`). */
const TOLERANCE = { channel: 32, outliers: 0.005, rmse: 0.01 };
function compare(a: Image, b: Image): { outliers: number; rmse: number; diff: Uint8Array } {
  const n = a.width * a.height;
  const diff = new Uint8Array(n * 4);
  let outliers = 0;
  let sum = 0;
  for (let p = 0; p < n; p++) {
    let max = 0;
    for (let k = 0; k < 3; k++) {
      const d = Math.abs(a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!);
      max = Math.max(max, d);
      sum += (d / 255) ** 2;
      diff[p * 4 + k] = Math.min(255, d * 4);
    }
    diff[p * 4 + 3] = 255;
    if (max > TOLERANCE.channel) {
      outliers++;
      diff.set([255, 0, 0, 255], p * 4);
    }
  }
  return { outliers: outliers / n, rmse: Math.sqrt(sum / (n * 3)), diff };
}

if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = join(work, "fx-showcase");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);

const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><title>reference</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${WIDTH}px;height:${HEIGHT}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/reference.html": html } });
const browser = await Browser.launch({ chrome, width: WIDTH, height: HEIGHT });
const log: string[] = [];
const dir = join(out, tier);
mkdirSync(dir, { recursive: true });
const own = join(project, "references", tier);
const against = record ? own : existsSync(join(own, "post.png")) ? own : join(project, "references", "metal");
const result: Record<string, unknown> = { tier, against: against.slice(repo.length + 1), tolerance: TOLERANCE, stations: {} as Record<string, unknown> };
let failures = 0;
const t0 = performance.now();
let tab: Awaited<ReturnType<typeof browser.open>> | undefined;
try {
  tab = await browser.open("about:blank", log);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  const profile = settingsOf(project).profile;
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/reference.html?profile=${profile}${webgl ? "&webgl=1" : ""}` });
  await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
  const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}\n${(await tab.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-20).join("\n")}`);
  result["renderer"] = await tab.eval<string>("__og_shell__.tier");
  // A wrong argument through the Worker's call is answered with the probe's
  // error (it used to trap the wasm), and the same Worker runs the stations.
  const refused = await tab.eval<string>(`__og_shell__.call("probe", "commands", { script: ${JSON.stringify(SCRIPT)} }).then((v) => "answered " + JSON.stringify(v), (e) => String(e && e.message || e))`);
  if (refused !== "probe.commands: script must be a string, not object") throw new Error(`commands with an object: ${refused}`);
  result["refused"] = refused;
  const scheduled = await tab.eval<{ scheduled?: number } | null>(`__og_shell__.call("probe", "commands", ${JSON.stringify(SCRIPT)})`);
  if (!scheduled?.scheduled) throw new Error(`the station script was not scheduled: ${JSON.stringify(scheduled)}`);
  // One tick and one rendered frame per call, as og run does natively, so
  // TAA, motion blur and particles see every frame the native references saw.
  const frames = await tab.eval<number>(`(async () => {
    const shell = globalThis.__og_shell__;
    const ticks = ${JSON.stringify(STATIONS.map((s) => s.tick))};
    const shots = [];
    let tick = 0;
    for (const want of ticks) {
      while (tick < want) {
        const answer = await shell.call("frame", 1000 / ${TICK_RATE});
        if (!answer || answer.ok === false) throw new Error("frame refused at tick " + tick + ": " + JSON.stringify(answer));
        tick = answer.tick ?? tick + 1;
      }
      globalThis.__og_shots__ = shots;
      shots.push(await shell.probe("capture", { width: ${WIDTH}, height: ${HEIGHT} }));
    }
    globalThis.__og_shots__ = shots;
    return tick;
  })()`);
  result["ticks"] = frames;
  const shots = await tab.eval<Array<{ png: string; width: number; height: number; tick: number } | null>>("globalThis.__og_shots__");
  for (const [i, s] of STATIONS.entries()) {
    const shot = shots[i];
    if (!shot?.png) throw new Error(`${s.name}: the capture answered ${JSON.stringify(shot)?.slice(0, 200)}`);
    const bytes = Uint8Array.from(atob(shot.png), (c) => c.charCodeAt(0));
    const file = join(dir, `${s.name}.png`);
    writeFileSync(file, bytes);
    const row: Record<string, unknown> = { tick: shot.tick, size: `${shot.width}x${shot.height}` };
    (result["stations"] as Record<string, unknown>)[s.name] = row;
    if (shot.width !== WIDTH || shot.height !== HEIGHT) {
      failures++;
      row["error"] = `captured ${shot.width}x${shot.height}, not ${WIDTH}x${HEIGHT}`;
      console.log(`FAIL ${s.name}: ${row["error"]}`);
      continue;
    }
    if (record) {
      mkdirSync(own, { recursive: true });
      copyFileSync(file, join(own, `${s.name}.png`));
      console.log(`recorded ${join(own, `${s.name}.png`)} (tick ${shot.tick})`);
      continue;
    }
    const d = compare(decodePng(bytes), decodePng(new Uint8Array(readFileSync(join(against, `${s.name}.png`)))));
    const ok = d.outliers <= TOLERANCE.outliers && d.rmse <= TOLERANCE.rmse;
    Object.assign(row, { outliers: +(d.outliers * 100).toFixed(3), rmse: +d.rmse.toFixed(5), ok });
    if (!ok) {
      failures++;
      // magick -size 1280x720 -depth 8 rgba:<station>.diff.rgba diff.png
      writeFileSync(join(dir, `${s.name}.diff.rgba`), d.diff);
    }
    console.log(`${ok ? "ok  " : "FAIL"} ${s.name} (${tier}, tick ${shot.tick}): ${(d.outliers * 100).toFixed(3)} % outliers (≤ 0.5 %), RMSE ${d.rmse.toFixed(5)} (≤ 0.01) against ${against.slice(repo.length + 1)}`);
  }
} catch (e) {
  failures++;
  // The engine's own log says why a started engine stopped answering.
  const engine = await tab?.eval<string[]>("globalThis.__og_log__ ?? []").catch(() => []) ?? [];
  result["error"] = [String(e instanceof Error ? e.stack : e), ...engine.slice(-20)].join("\n");
  console.log(`FAIL ${result["error"]}`);
} finally {
  await browser.close();
  server.stop();
}
result["ms"] = Math.round(performance.now() - t0);
result["log"] = log.slice(-40);
writeFileSync(join(out, `reference-${tier}.json`), JSON.stringify(result, null, 2) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} ${tier} references: ${STATIONS.length} stations in ${result["ms"]} ms; ${join(out, `reference-${tier}.json`)}`);
process.exit(failures === 0 ? 0 : 1);
