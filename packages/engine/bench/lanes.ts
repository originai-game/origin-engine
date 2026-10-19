// Frame times of the reference scene (docs/design.md §7,
// crates/og-tests/reference-scene) with lanes off and on (phase 3, lanes).
//
//   bun bench/lanes.ts --native [--frames 600] [--size 1280x720]
//       natively (Metal on macOS): each mode in its own process, lanes set by
//       OG_LANES (off: no pool; unset: one thread a core, at most 8)
//   bun bench/lanes.ts --native --crowd [--frames 600]
//       the same with tests/support/lanes-crowd.ts added (2000 more boxes, 160
//       more walkers, 48 falling crates), where every lane has work to split
//   bun bench/lanes.ts --native --lanes on|off   one native run, JSON to stdout
//   bun bench/lanes.ts --chrome [--frames 600] [--chrome-path <path>]
//       in headless Chrome over WebGPU, the scene's publish build (`og build
//       --publish`, isolation "preferred", so it ships the lanes variant) in
//       three modes: not isolated (the shell falls back to single-threaded),
//       isolated with the lanes variant withheld (single-threaded), and
//       isolated with lanes (the shell's own choice)
//
// A native sample is engine.frame(16.667 ms) plus waiting for the GPU
// (og_node_wait_gpu): the tick (systems, animation, physics), extraction,
// recording and GPU work. A Chrome sample is the page's round trip of one
// `frame` call to the engine Worker (the same work, submitted but not waited
// for: the publish build has no capture to wait on), and `sustained` is the
// wall time of all the frames over their count. 60 warm-up frames are not
// counted. Native runs need a release addon (native/build-profile) and end
// with the world hash, which must be the same with lanes off and on; a publish
// build has no world hash (the probe's dump is stripped), so the Chrome modes
// compare the RGBA of a capture after the timed frames instead. Results go to
// --out (default tests/evidence/P3/lanes/).

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost, native } from "../src/node/index.ts";
import { bundleGame } from "../src/node/build.ts";
import { settingsOf, webBuild } from "../src/node/web-build.ts";
import { Browser, chromePath } from "../tests/web/cdp.ts";
import { serve } from "../tests/web/serve.ts";
import { decodePng } from "../tests/support/png.ts";
import { createHash } from "node:crypto";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { crowd } from "../tests/support/lanes-crowd.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const scene = join(repo, "crates", "og-tests", "reference-scene");
const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const out = opt("--out", join(repo, "tests", "evidence", "P3", "lanes"));
const [W, H] = opt("--size", "1280x720").split("x").map(Number) as [number, number];
const FRAMES = Number(opt("--frames", "600"));
const WARM = 60;
/** --crowd: the scene plus tests/support/lanes-crowd.ts, so every lane splits. */
const CROWD = argv.includes("--crowd");

type Stats = { p50: number; p95: number; p99: number; max: number; mean: number };
export type Run = {
  host: string; build?: string; lanes: string; threads: number; pool: string; parallelCalls: number;
  size: string; tier: string; frames: number; hash: string;
  ms: Stats; cpuMs?: Stats; stepMs?: Stats; renderMs?: Stats; sustainedMs?: number; why?: string; isolated?: boolean;
  /** Chrome: the sha256 of the RGBA of the last frame, captured after the timed frames, and its tick. */
  frame?: { tick: number; rgbaSha256: string; png: string };
  times: number[];
};

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const q = (s: number[], p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
const r3 = (v: number) => Math.round(v * 1000) / 1000;
export function stats(xs: number[]): Stats {
  const s = [...xs].sort((a, b) => a - b);
  return { p50: r3(q(s, 0.5)), p95: r3(q(s, 0.95)), p99: r3(q(s, 0.99)), max: r3(s.at(-1)!), mean: r3(xs.reduce((a, b) => a + b, 0) / xs.length) };
}

async function nativeRun(lanes: string): Promise<Run> {
  // Frame times of an unoptimised addon say nothing about the budget.
  const profileFile = join(pkg, "native", "build-profile");
  const build = existsSync(profileFile) ? readFileSync(profileFile, "utf8").trim() : "unknown";
  if (build !== "release") throw new Error(`the native addon is a ${build} build; run OG_RELEASE=1 sh scripts/build.sh first`);
  loadNativeHost();
  const bundled = join(repo, "target", "tmp", "lanes-bench", `scene-${process.pid}.js`);
  const b = await bundleGame(join(scene, "src", "main.ts"), bundled, "native");
  if (!b.ok) throw new Error(b.reason);
  const ref = (await import(bundled)) as { default: GameToken; build(world: World): void; Turn: unknown };
  const game = CROWD
    ? (og.game({ seed: 1, systems: [ref.Turn as never], render: { antialias: "taa" }, start(world: World) { ref.build(world); crowd(world); } }) as unknown as GameToken)
    : ref.default;
  const engine = must(createEngine(game, { root: scene }));
  const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  const probe = (cmd: string) => {
    engine.probe.commands(`0:${cmd}`);
    must(engine.advance(1));
    return engine.takeOutputs();
  };
  const wait = () => {
    const r = JSON.parse(native().og_node_wait_gpu(engine.id)) as { ok: boolean; reason?: string };
    if (!r.ok) throw new Error(r.reason);
  };
  const times: number[] = [], cpu: number[] = [];
  for (let i = -WARM; i < FRAMES; i++) {
    const t0 = performance.now();
    const f = engine.frame(1000 / 60);
    if (!f.ok) throw new Error(f.reason);
    const t1 = performance.now();
    wait();
    const t2 = performance.now();
    if (i >= 0) { times.push(t2 - t0); cpu.push(t1 - t0); }
  }
  const hash = engine.hash();
  const report = JSON.parse(probe("dump_lanes:lanes.json").find((o) => o.file === "lanes.json")?.data ?? "{}") as { threads?: number; pool?: string; parallelCalls?: number };
  engine.destroy();
  return {
    host: "native", build, lanes, threads: report.threads ?? 1, pool: report.pool ?? "none", parallelCalls: report.parallelCalls ?? 0,
    size: `${W}x${H}`, tier: attached.value!, frames: FRAMES, hash,
    ms: stats(times), cpuMs: stats(cpu), times: times.map(r3),
  };
}

/** The three Chrome modes: [name, served cross-origin isolated, the page offers the lanes variant]. */
const CHROME_MODES: ReadonlyArray<readonly [string, boolean, boolean]> = [
  ["not isolated", false, true],
  ["isolated, lanes off", true, false],
  ["isolated, lanes on", true, true],
];

/** The captured frame's tick, the sha256 of its RGBA and the PNG it was written to. */
function frameDigest(png: string | null, tick: number, mode: string): { tick: number; rgbaSha256: string; png: string } {
  if (!png) throw new Error(`${mode}: the capture answered no frame`);
  const bytes = Uint8Array.from(atob(png), (c) => c.charCodeAt(0));
  const name = `chrome-${mode.replace(/[^a-z]+/g, "-")}.png`;
  writeFileSync(join(out, name), bytes);
  const img = decodePng(bytes);
  return { tick, rgbaSha256: createHash("sha256").update(img.rgba).digest("hex"), png: name };
}

async function chromeRuns(): Promise<Run[]> {
  const work = join(repo, "target", "tmp", "lanes-bench");
  const dist = join(work, "web");
  const settings = settingsOf(scene);
  const built = await webBuild(scene, join(scene, "src", "main.ts"), dist, true, settings);
  if (!built.ok) throw new Error(built.reason);
  if (!existsSync(join(dist, "origin-engine-lanes_bg.wasm"))) throw new Error("the publish build has no lanes variant: build the engine with scripts/build.sh first");
  const page = await Bun.build({ entrypoints: [join(pkg, "bench", "lanes.page.ts")], target: "browser", format: "esm" });
  if (!page.success) throw new Error(page.logs.map(String).join("\n"));
  writeFileSync(join(dist, "lanes-page.js"), await page.outputs[0]!.text());
  const html = `<!doctype html><html><head><meta charset="utf-8"><title>lanes bench</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${W}px;height:${H}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/lanes-page.js"></script></body></html>`;
  const runs: Run[] = [];
  for (const [mode, isolate, offered] of CHROME_MODES) {
    const server = serve({ dist, isolate, extra: { "/lanes.html": html } });
    const browser = await Browser.launch({ chrome: chromePath(argv.includes("--chrome-path") ? ["--chrome", opt("--chrome-path", "")] : []), width: W, height: H });
    const log: string[] = [];
    try {
      const tab = await browser.open("about:blank", log);
      await tab.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
      await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/lanes.html${offered ? "" : "?lanes=0"}` });
      await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 120_000);
      const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
      if (failed) throw new Error(`${mode}: the engine did not start: ${failed}\n${(await tab.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-20).join("\n")}`);
      const r = await tab.eval<{ each: number[]; total: number; step: number[]; render: number[]; png: string | null; tick: number; lanes: { threads?: number; pool?: string; parallelCalls?: number } | null; ready: { on: boolean; threads: number; why: string }; tier: string; isolated: boolean }>(`(async () => {
        const shell = globalThis.__og_shell__;
        const frame = async () => {
          const a = await shell.call("frame", 1000 / 60);
          if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
        };
        for (let i = 0; i < ${WARM}; i++) await frame();
        const each = [], step = [], render = [];
        let total = 0;
        for (let i = 0; i < ${FRAMES}; i++) {
          const t = performance.now();
          await frame();
          const dt = performance.now() - t;
          each.push(dt);
          total += dt;
          // The engine's own timings of that frame (the probe's state, a player member).
          const s = await shell.probe("state");
          step.push(s.timings.step);
          render.push(s.timings.render);
        }
        const shot = await shell.probe("capture", { width: ${W}, height: ${H} });
        return { each, total, step, render, png: shot && shot.png, tick: shot && shot.tick, lanes: await shell.call("lanes"), ready: shell.ready.lanes, tier: shell.tier, isolated: globalThis.crossOriginIsolated === true };
      })()`);
      const run: Run = {
        host: "chrome", lanes: mode, threads: r.lanes?.threads ?? 1, pool: r.lanes?.pool ?? "none", parallelCalls: r.lanes?.parallelCalls ?? 0,
        size: `${W}x${H}`, tier: r.tier, frames: FRAMES, isolated: r.isolated, why: r.ready.on ? "" : r.ready.why,
        hash: "", frame: frameDigest(r.png, r.tick, mode),
        ms: stats(r.each), stepMs: stats(r.step), renderMs: stats(r.render), sustainedMs: r3(r.total / FRAMES), times: r.each.map(r3),
      };
      if (r.ready.on !== (isolate && offered)) throw new Error(`${mode}: lanes ${r.ready.on ? "on" : "off"} (${r.ready.why})`);
      runs.push(run);
      process.stderr.write(`chrome ${mode} (${run.threads} threads, ${run.pool}): p50 ${run.ms.p50} p95 ${run.ms.p95} p99 ${run.ms.p99} ms, sustained ${run.sustainedMs} ms; step p50 ${run.stepMs!.p50}, render p50 ${run.renderMs!.p50}; frame ${run.frame!.rgbaSha256.slice(0, 12)} at tick ${run.frame!.tick}\n`);
    } catch (e) {
      process.stderr.write(log.slice(-30).join("\n") + "\n");
      throw e;
    } finally {
      await browser.close();
      server.stop();
    }
  }
  return runs;
}

if (argv.includes("--native") && argv.includes("--lanes")) {
  const run = await nativeRun(opt("--lanes", "on"));
  process.stdout.write(JSON.stringify(run) + "\n");
} else if (argv.includes("--native")) {
  mkdirSync(out, { recursive: true });
  const runs: Run[] = [];
  for (const lanes of ["off", "on"]) {
    const env = { ...process.env, OG_LANES: lanes === "off" ? "off" : process.env.OG_LANES_ON };
    const r = Bun.spawnSync(["bun", fileURLToPath(import.meta.url), "--native", "--lanes", lanes, "--frames", String(FRAMES), "--size", `${W}x${H}`, ...(CROWD ? ["--crowd"] : [])], { cwd: pkg, env });
    if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
    const run = JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").pop()!) as Run;
    runs.push(run);
    process.stderr.write(`native lanes ${lanes} (${run.threads} threads): p50 ${run.ms.p50} p95 ${run.ms.p95} p99 ${run.ms.p99} ms, CPU p50 ${run.cpuMs?.p50}\n`);
  }
  writeFileSync(join(out, CROWD ? "native-crowd.json" : "native.json"), JSON.stringify(runs, null, 1) + "\n");
  if (new Set(runs.map((r) => r.hash)).size !== 1) {
    process.stderr.write(`FAIL: lanes off and on ended in different worlds (${runs.map((r) => r.hash).join(", ")})\n`);
    process.exit(1);
  }
  process.stderr.write(`lanes off and on ended in the same world (${runs[0]!.hash})\n`);
}
if (argv.includes("--chrome")) {
  mkdirSync(out, { recursive: true });
  const runs = await chromeRuns();
  writeFileSync(join(out, "chrome.json"), JSON.stringify(runs, null, 1) + "\n");
  const frames = new Set(runs.map((r) => r.frame!.rgbaSha256));
  if (frames.size !== 1) {
    process.stderr.write(`FAIL: the modes captured different frames (${[...frames].join(", ")})\n`);
    process.exit(1);
  }
  process.stderr.write(`the ${runs.length} modes captured the same frame at tick ${runs[0]!.frame!.tick}\n`);
}
