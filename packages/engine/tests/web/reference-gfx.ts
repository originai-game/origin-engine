// The browser tiers of og-tests' own gfx scenes: crates/og-tests-web (the
// scenes of crates/og-tests/src/scenes.rs compiled to wasm32, their assets
// compiled in) rendered in headless Chrome on WebGPU (or WebGL2 with
// --webgl) at 1280×720, each scene drawn exactly as the native harness
// draws it (`og_tests::gfx::render_built`: its frames, its update before
// each, the offscreen target read back), and compared with
// crates/og-tests/references/<tier>/ (`webgpu` or `webgl2`) when that frame
// is committed, else with `metal`, under the scene's own tolerance
// (crates/og-tests/src/compare.rs: a channel off by more than 32/255 is an
// outlier; by default fail above 0.5 % outliers or an RGB RMSE above 0.01).
// Every frame is also compared with its metal reference, reported and kept
// in the JSON whether or not it is the frame the run is judged against.
// og-ui's reference scenes ride along as `og-ui/<name>` (sample-screen,
// text-glow, glass, widget-set: crates/og-ui/tests/support/scenes.rs, the
// code og-ui's native gfx tests draw them with), judged the same way against
// crates/og-ui/references/<tier>/<name>.png, else metal.
//
//   bun tests/web/reference-gfx.ts [--webgl] [--record] [--scene a,b] [--time <frames>] [--cpu-culling] [--perf-out <dir>] [--out <dir>] [--chrome <path>]
//
// A scene with no reference on the tier (`taa` on WebGL2, `msaa` on
// WebGPU: scenes::Applies) is listed as not applying, not rendered.
// --record writes the frames as references/<tier>/ instead of comparing.
// --time <n> then times the scenes og-tests' perf suite times natively
// (crates/og-tests/tests/perf.rs): after the capture, 7 runs of n frames,
// each run ending in a capture whose readback waits for the GPU to finish
// them; the best run per frame lands in
// crates/og-tests/perf/<tier>-chrome-<adapter>/<scene>.json beside the
// native baselines, with the median wall time of one render call (the CPU
// cost of encoding and submitting a frame) and the last frame's counts.
// The perf-only city scenes (CITY: city-street, city-aerial) are timed only
// when --scene names them. --cpu-culling culls on the CPU on WebGPU too
// (og-render's Culling::Cpu; WebGL2 always does); --perf-out writes the
// timing JSON to <dir>/<tier>[-cpu-culling]/ instead (evidence runs).
// Writes <out>/<tier>/<scene>.png, <scene>.diff.png on a mismatch and
// reference-gfx-<tier>.json (default dist/test-gfx/out). Exits 1 when a
// scene does not match or does not render. Needs Chrome (cdp.ts),
// wasm-bindgen at the Cargo.lock version and the wasm32 target.

import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { decodePng, encodePng, type Image } from "../../src/node/png.ts";
import { createRequire } from "node:module";

const { cpus, loadavg } = createRequire(import.meta.url)("node:os") as { cpus(): { model: string }[]; loadavg(): number[] };

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};
declare function atob(data: string): string;

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const record = args.includes("--record");
const work = join(pkg, "dist", "test-gfx");
const out = resolve(opt("--out") ?? join(work, "out"));
const chrome = chromePath(args);
const refs = join(repo, "crates", "og-tests", "references");
/** Where a scene's references live: og-ui's under crates/og-ui/references. */
function referencesOf(scene: string): { root: string; name: string } {
  return scene.startsWith("og-ui/")
    ? { root: join(repo, "crates", "og-ui", "references"), name: scene.slice("og-ui/".length) }
    : { root: refs, name: scene };
}
/** The scenes crates/og-tests/tests/perf.rs times natively. */
// The scenes crates/og-tests/tests/perf.rs times natively (tilemap-256 is
// built for timing only, not in scenes::SCENES), timed the same way: 7 runs.
const PERF = ["gltf-reference", "spheres-20k", "point-spot-lights", "showcase", "sprites-20k", "lights-256", "tilemap-256"];
const RUNS = 7;
/** Perf-only scenes (scenes::build, not SCENES), timed when --scene names them. */
const CITY = ["city-street", "city-aerial"];
const cpuCulling = args.includes("--cpu-culling");
const perfOut = opt("--perf-out");

function run(cmd: string[]): string {
  const r = Bun.spawnSync(cmd, { cwd: repo, stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")}: ${new TextDecoder().decode(r.stderr).slice(-4000)}`);
  return new TextDecoder().decode(r.stdout);
}

// The module: the wasm profile (what ships; the scenes are timed in it),
// then wasm-bindgen for the browser (no wasm-opt: nothing ships). Its own
// target dir: og-tests-web unifies og-render's features differently from the
// gate's wasm-build, and cargo keeps one fingerprint per unit whatever its
// features, so sharing target/wasm32-unknown-unknown would rebuild og-render
// for both every time (scripts/check.sh wasm_build).
const t0 = performance.now();
const targetDir = join(process.env.CARGO_TARGET_DIR ? resolve(repo, process.env.CARGO_TARGET_DIR) : join(repo, "target"), "wasm-gfx");
const messages = run(["cargo", "build", "--profile", "wasm", "--target", "wasm32-unknown-unknown", "--target-dir", targetDir, "-p", "og-tests-web", "--lib", "--message-format=json-render-diagnostics"]);
const raw = messages
  .split("\n")
  .filter((l) => l.includes('"reason":"compiler-artifact"') && l.includes("og_tests_web"))
  .flatMap((l) => (JSON.parse(l) as { filenames: string[] }).filenames)
  .find((f) => f.endsWith(".wasm"));
if (!raw) throw new Error("cargo built no og_tests_web.wasm");
const lock = /name = "wasm-bindgen"\nversion = "([^"]+)"/.exec(readFileSync(join(repo, "Cargo.lock"), "utf8"))?.[1];
const have = run(["wasm-bindgen", "--version"]).trim().split(" ")[1];
if (lock !== have) throw new Error(`wasm-bindgen ${have} does not match Cargo.lock ${lock} (scripts/install-tools.sh)`);
const dist = join(work, "site");
run(["wasm-bindgen", "--target", "web", "--out-dir", join(dist, "pkg"), "--out-name", "og-tests", raw]);
const buildMs = Math.round(performance.now() - t0);

const html = `<!doctype html><html><head><meta charset="utf-8"><title>og-tests gfx</title></head><body>
<script type="module">
import init, * as gfx from "/game/pkg/og-tests.js";
try { await init(); globalThis.__gfx = gfx; } catch (e) { globalThis.__gfx_error = String(e && e.stack || e); }
</script></body></html>`;
const server = serve({ dist, isolate: false, extra: { "/gfx.html": html } });
const browser = await Browser.launch({ chrome, width: 1280, height: 720 });
const log: string[] = [];
const dir = join(out, tier);
mkdirSync(dir, { recursive: true });

type Opened = { tier: string; adapter: string; antialias: string; applies: boolean; frames: number; width: number; height: number; tolerance: { channel: number; outliers: number; rmse: number } };

function compare(a: Image, b: Image, channel: number): { outliers: number; rmse: number; diff: Uint8Array } {
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
    if (max > channel) {
      outliers++;
      diff.set([255, 0, 0, 255], p * 4);
    }
  }
  return { outliers: outliers / n, rmse: Math.sqrt(sum / (n * 3)), diff };
}

const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
const result: Record<string, unknown> = { tier, buildMs, scenes: {} as Record<string, unknown> };
const rows = result["scenes"] as Record<string, Record<string, unknown>>;
let failures = 0;
const t1 = performance.now();
// A scene that panics inside the module (a wgpu validation error, say)
// leaves its promise unsettled and the module unusable: a scene fails at
// the page's first uncaught exception after it started, or after SCENE_MS,
// and after a failure the next scene starts on a fresh page.
const SCENE_MS = 120_000;
function bounded<T>(p: Promise<T>, what: string): Promise<T> {
  const from = log.length;
  let timer: ReturnType<typeof setInterval> | undefined;
  const t0 = performance.now();
  const late = new Promise<never>((_, reject) => {
    timer = setInterval(() => {
      if (log.slice(from).some((l) => l.startsWith("exception:"))) reject(new Error(`${what} threw in the page`));
      else if (performance.now() - t0 > SCENE_MS) reject(new Error(`${what} did not finish in ${SCENE_MS / 1000} s`));
    }, 100);
  });
  return Promise.race([p, late]).finally(() => clearInterval(timer));
}
async function page() {
  const t = await browser.open(`http://127.0.0.1:${server.port}/gfx.html`, log);
  await t.until("globalThis.__gfx || globalThis.__gfx_error", 60_000);
  const failed = await t.eval<string | undefined>("globalThis.__gfx_error");
  if (failed) throw new Error(`the module did not load: ${failed}`);
  if (cpuCulling) await t.eval("globalThis.__gfx.og_gfx_cpu_culling(true)");
  return t;
}
try {
  let tab = await page();
  const all = [...(await tab.eval<string[]>("__gfx.og_gfx_scenes()")), ...(await tab.eval<string[]>("__gfx.og_gfx_ui_scenes()"))];
  const only = opt("--scene")?.split(",");
  const timed = Number(opt("--time") ?? 0);
  const scenes = only ?? (timed > 0 ? [...all, ...PERF.filter((p) => !all.includes(p))] : all);
  for (const s of scenes) if (!all.includes(s) && !PERF.includes(s) && !CITY.includes(s)) throw new Error(`no scene ${s} (og-tests knows ${all.join(", ")})`);
  for (const scene of scenes) {
    const row: Record<string, unknown> = {};
    rows[scene] = row;
    const ts = performance.now();
    const seen = log.length;
    try {
      // Open, draw the scene's frames, read the last one back (base64 RGBA).
      const shot = await bounded(tab.eval<{ opened: Opened; rgba?: string; renderMs?: number }>(`(async () => {
        const gfx = globalThis.__gfx;
        const opened = JSON.parse(await gfx.og_gfx_open(${JSON.stringify(scene)}, ${webgl}, new OffscreenCanvas(1, 1)));
        if (!opened.applies) { gfx.og_gfx_close(); return { opened }; }
        const t = performance.now();
        gfx.og_gfx_render(opened.frames);
        const px = await gfx.og_gfx_capture();
        const renderMs = performance.now() - t;
        let bin = "";
        for (let i = 0; i < px.length; i += 0x8000) bin += String.fromCharCode.apply(null, px.subarray(i, i + 0x8000));
        return { opened, rgba: btoa(bin), renderMs };
      })()`), scene);
      const o = shot.opened;
      Object.assign(row, { tier: o.tier, adapter: o.adapter, antialias: o.antialias, frames: o.frames });
      result["adapter"] ??= o.adapter;
      if (o.tier !== tier) throw new Error(`rendered on ${o.tier}, not ${tier}`);
      if (!o.applies) {
        row["applies"] = false;
        console.log(`--   ${scene} (${tier}): no reference on this tier (scenes::Applies)`);
        continue;
      }
      row["renderMs"] = +shot.renderMs!.toFixed(1);
      const img: Image = { width: o.width, height: o.height, rgba: Uint8Array.from(atob(shot.rgba!), (c) => c.charCodeAt(0)) };
      const png = encodePng(img);
      const file = join(dir, `${scene}.png`);
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, png);
      const where = referencesOf(scene);
      const metal = join(where.root, "metal", `${where.name}.png`);
      if (existsSync(metal)) {
        const m = compare(img, decodePng(new Uint8Array(readFileSync(metal))), o.tolerance.channel);
        row["metal"] = { outliers: +(m.outliers * 100).toFixed(3), rmse: +m.rmse.toFixed(5), ok: m.outliers <= o.tolerance.outliers && m.rmse <= o.tolerance.rmse };
      }
      if (!all.includes(scene)) {
        console.log(`--   ${scene} (${tier}): timed only (not in scenes::SCENES, no reference)`);
      } else if (record) {
        mkdirSync(join(where.root, tier), { recursive: true });
        const to = join(where.root, tier, `${where.name}.png`);
        copyFileSync(file, to);
        console.log(`rec  ${scene} (${tier}) -> ${to.slice(repo.length + 1)}; against metal ${JSON.stringify(row["metal"] ?? "none")}`);
      } else {
        const own = join(where.root, tier, `${where.name}.png`);
        const against = existsSync(own) ? own : metal;
        if (!existsSync(against)) throw new Error(`no reference at ${own} or ${metal}; record one with --record --scene ${scene}`);
        const d = compare(img, decodePng(new Uint8Array(readFileSync(against))), o.tolerance.channel);
        const ok = d.outliers <= o.tolerance.outliers && d.rmse <= o.tolerance.rmse;
        Object.assign(row, { against: against.slice(repo.length + 1), outliers: +(d.outliers * 100).toFixed(3), rmse: +d.rmse.toFixed(5), ok });
        if (!ok) {
          failures++;
          writeFileSync(join(dir, `${scene}.diff.png`), encodePng({ width: img.width, height: img.height, rgba: d.diff }));
        }
        console.log(
          `${ok ? "ok  " : "FAIL"} ${scene} (${tier}): ${(d.outliers * 100).toFixed(3)} % outliers (≤ ${(o.tolerance.outliers * 100).toFixed(1)} %), RMSE ${d.rmse.toFixed(5)} (≤ ${o.tolerance.rmse}) against ${against.slice(repo.length + 1)}` +
            (against !== metal && row["metal"] ? `; metal: RMSE ${(row["metal"] as { rmse: number }).rmse}` : ""),
        );
      }
      if (timed > 0 && (PERF.includes(scene) || CITY.includes(scene))) {
        const t = await bounded(tab.eval<{ runs: number[]; each: number[]; stats: Record<string, unknown> }>(`(async () => {
          const gfx = globalThis.__gfx;
          const runs = [], each = [];
          let stats = "{}";
          gfx.og_gfx_render(10);
          await gfx.og_gfx_capture();
          for (let r = 0; r < ${RUNS}; r++) {
            const start = performance.now();
            for (let i = 0; i < ${timed}; i++) {
              const t = performance.now();
              stats = gfx.og_gfx_render(1);
              each.push(performance.now() - t);
            }
            await gfx.og_gfx_capture();
            runs.push((performance.now() - start) / ${timed});
          }
          return { runs, each, stats: JSON.parse(stats) };
        })()`), `timing ${scene}`);
        const runs = [...t.runs].sort((a, b) => a - b).map((v) => +v.toFixed(3));
        const each = [...t.each].sort((a, b) => a - b);
        const perf = {
          scene,
          tier,
          browser: "Chrome for Testing (cdp.ts)",
          adapter: o.adapter,
          culling: cpuCulling || webgl ? "cpu" : "auto",
          last_frame: t.stats,
          width: o.width,
          height: o.height,
          frames_per_run: timed,
          runs_ms: runs,
          frame_ms: runs[0],
          submit_median_ms: +each[Math.floor(each.length / 2)]!.toFixed(3),
          load: loadavg().map((l) => +l.toFixed(1)),
          note: "each run ends in one capture (its readback waits for the GPU to finish the run's frames)",
        };
        // Chrome names no WebGPU adapter and ANGLE's string is long: the
        // directory names the machine (its CPU, which on Apple silicon is
        // the GPU's name too), the adapter string stays in the file.
        const pdir = perfOut
          ? join(resolve(perfOut), `${tier}${cpuCulling && !webgl ? "-cpu-culling" : ""}`)
          : join(repo, "crates", "og-tests", "perf", `${tier}-chrome-${slug(cpus()[0]?.model ?? "") || "unknown"}`);
        mkdirSync(pdir, { recursive: true });
        writeFileSync(join(pdir, `${scene}.json`), JSON.stringify(perf, null, 2) + "\n");
        row["frameMs"] = perf.frame_ms;
        console.log(`time ${scene} (${tier}): ${perf.frame_ms} ms per frame with the GPU (best of ${RUNS} x ${timed}), submit median ${perf.submit_median_ms} ms, last frame ${JSON.stringify(t.stats)} -> ${pdir.slice(repo.length + 1)}`);
      }
      await tab.eval("globalThis.__gfx.og_gfx_close()");
    } catch (e) {
      failures++;
      // What the page logged meanwhile names the cause (a panic's message).
      const said = log.slice(seen).filter((l) => l.includes("panicked:")).join("\n");
      row["error"] = String(e instanceof Error ? e.message : e) + (said ? `\n${said}` : "");
      console.log(`FAIL ${scene} (${tier}): ${row["error"]}`);
      tab = await page();
    } finally {
      row["ms"] = Math.round(performance.now() - ts);
    }
  }
} catch (e) {
  failures++;
  result["error"] = String(e instanceof Error ? e.stack : e);
  console.log(`FAIL ${result["error"]}`);
} finally {
  await browser.close();
  server.stop();
}
result["ms"] = Math.round(performance.now() - t1);
result["log"] = log.slice(-40);
writeFileSync(join(out, `reference-gfx-${tier}.json`), JSON.stringify(result, null, 2) + "\n");
const n = Object.keys(rows).length;
console.log(`${failures === 0 ? "PASS" : "FAIL"} og-tests gfx on ${tier}: ${n} scenes in ${result["ms"]} ms (module built in ${buildMs} ms); ${join(out, `reference-gfx-${tier}.json`)}`);
process.exit(failures === 0 ? 0 : 1);
