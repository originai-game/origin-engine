// og.motion on the web never holds a tick, and its captures reproduce
// (docs/design.md §2.4, crates/og-runtime/src/bind/motion/present.rs).
//
// motion-feet's course (tests/fixtures/motion-feet: two agents walking,
// running, jumping, turning, posing) runs in headless Chrome on WebGPU from
// its dev web export on the GPU backend (the default), one `frame` call a
// tick, --ticks ticks (default 420), twice: once as the GPU answers and
// once with every og.motion readback (the `og-motion readback` buffer's
// mapAsync, patched by a module the engine Worker imports before game.js)
// held back --delay ms (default 600). Checked:
// - every frame call takes a tick: the probe's gate never holds one for
//   og.motion, prompt or late (the late run's Presenter falls behind the
//   world instead, `dump_motion`'s `late`);
// - the world hash is the same on every tick in both runs;
// - a probe capture every --every ticks (default 60) is the same PNG in
//   both runs: a capture waits for every result due by its tick.
//
// --realtime <s> instead plays the course paced like a page (a frame every
// 16 ms, the wall time since the last as its elapsed) for <s> seconds and
// reports any frames whose tick the gate held, on --backend gpu (default)
// or cpu.
//
//   bun tests/web/motion-ticks.ts [--ticks N] [--delay ms] [--every N] [--realtime s [--backend gpu|cpu]] [--chrome <path>]
//
// Needs Chrome (cdp.ts), a GPU and the motion model
// (bun scripts/motion-model.ts ensure). Writes dist/test-motion-ticks/motion-ticks.json;
// exits 1 on a mismatch.

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { cookProject } from "../support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tests", "fixtures", "motion-feet");
const TICKS = Number(opt("--ticks") ?? 420);
const DELAY = Number(opt("--delay") ?? 600);
const REALTIME = Number(opt("--realtime") ?? 0);
const BACKEND = opt("--backend") ?? "gpu";
const EVERY = Number(opt("--every") ?? 60);
const [WIDTH, HEIGHT] = [640, 360];
const work = join(pkg, "dist", "test-motion-ticks");
mkdirSync(work, { recursive: true });

type Tick = { tick: number; hash: string };
type Shot = { tick: number; png: string };
type Late = { behind: number; ticks: number; most: number };

cookProject(project, repo);
const dist = join(work, "motion-feet");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><title>motion ticks</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${WIDTH}px;height:${HEIGHT}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/motion.html": html } });
const browser = await Browser.from(args, { chrome: chromePath(args), width: WIDTH, height: HEIGHT, reverse: [server.port] });

/**
 * The engine Worker with og.motion's readbacks landing `ms` later: a module
 * that patches WebGPU's mapAsync, then imports game.js.
 */
function delayedWorker(ms: number): string {
  writeFileSync(
    join(dist, "motion-delay.js"),
    `const map = GPUBuffer.prototype.mapAsync;
GPUBuffer.prototype.mapAsync = function (...a) {
  const p = map.apply(this, a);
  if (this.label !== "og-motion readback") return p;
  return p.then((v) => new Promise((r) => setTimeout(() => r(v), ${ms})));
};
`,
  );
  writeFileSync(join(dist, "motion-worker.js"), `import "./motion-delay.js";\nimport "./game.js";\n`);
  return "motion-worker.js";
}

/** The page with its engine started; `delay` ms on og.motion's readbacks. */
async function start(delay: number, log: string[]) {
  const tab = await browser.open("about:blank", log);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  const worker = delay > 0 ? `&worker=${delayedWorker(delay)}` : "";
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/motion.html?profile=${settingsOf(project).profile}${worker}` });
  await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
  const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}\n${(await tab.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-20).join("\n")}`);
  return tab;
}

/** One tick a frame to TICKS: the frames that took no tick (held), each tick's world hash, a capture every EVERY ticks. */
const runTicks = `(async () => {
  const shell = globalThis.__og_shell__;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const ticks = [], shots = [];
  let held = 0, tick = 0;
  while (tick < ${TICKS}) {
    const a = await shell.call("frame", 1000 / 60);
    if (!a || a.ok === false) throw new Error("frame refused at tick " + tick + ": " + JSON.stringify(a));
    if (a.tick <= tick) { held++; continue; }
    tick = a.tick;
    const s = await shell.probe("snapshot");
    ticks.push({ tick: s.tick, hash: s.hash });
    if (tick % ${EVERY} === 0) {
      let shot = null;
      for (let tries = 0; !shot && tries < 5000; tries++) {
        shot = await shell.probe("capture", { width: ${WIDTH}, height: ${HEIGHT} });
        if (!shot) await sleep(2);
      }
      if (!shot) throw new Error("no capture at tick " + tick);
      shots.push({ tick, png: shot.png });
    }
  }
  await shell.call("probe", "commands", "0:dump_motion:motion.json");
  let dump;
  for (let tries = 0; !dump && tries < 5000; tries++) {
    await shell.call("frame", 0);
    dump = (await shell.call("outputs")).find((o) => o.action === "dump_motion");
    if (!dump) await sleep(2);
  }
  const motion = dump ? JSON.parse(dump.data) : null;
  return { ticks, shots, held, late: motion && motion.model.late, state: motion && motion.model.state, batches: motion ? motion.model.batches : 0 };
})()`;

/** Paced like a page: the frames whose tick the gate held, and for how long. */
const runRealtime = (seconds: number, backend: string) => `(async () => {
  const shell = globalThis.__og_shell__;
  await shell.call("probe", "commands", "0:motion_backend:${backend}");
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const episodes = [];
  let last = performance.now();
  const end = last + ${seconds * 1000};
  let frames = 0, heldSince = null, heldFrames = 0, tick = 0;
  while (performance.now() < end) {
    await sleep(16);
    const now = performance.now();
    const a = await shell.call("frame", now - last);
    last = now;
    if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
    frames++;
    // A frame that took no tick: the gate held it (a frame every 16 ms owes about one).
    if (a.ticks === 0 && a.tick === tick) {
      if (heldSince === null) heldSince = now;
      heldFrames++;
    } else if (heldSince !== null) {
      episodes.push({ atTick: tick, ms: +(now - heldSince).toFixed(1), frames: heldFrames });
      heldSince = null; heldFrames = 0;
    }
    tick = a.tick;
  }
  return { frames, ticks: tick, episodes };
})()`;

const log: string[] = [];
const result: Record<string, unknown> = { ticks: TICKS, delay: DELAY };
let failures = 0;
try {
  if (REALTIME > 0) {
    const tab = await start(0, log);
    const r = await tab.eval<{ frames: number; ticks: number; episodes: Array<{ atTick: number; ms: number; frames: number }> }>(runRealtime(REALTIME, BACKEND));
    result["realtime"] = { backend: BACKEND, ...r };
    const ms = r.episodes.map((e) => e.ms).sort((a, b) => a - b);
    console.log(`realtime ${REALTIME} s on ${BACKEND}: ${r.frames} frames, ${r.ticks} ticks; ${r.episodes.length} held episodes, ${r.episodes.reduce((n, e) => n + e.frames, 0)} frames; ms median ${ms[ms.length >> 1] ?? 0}, max ${ms[ms.length - 1] ?? 0}`);
    console.log(JSON.stringify(r.episodes));
  } else {
    type Run = { ticks: Tick[]; shots: Shot[]; held: number; late: Late | null; state: string; batches: number };
    const runs: Record<string, Run> = {};
    for (const [name, delay] of [["gpu", 0], ["gpu late", DELAY]] as const) {
      const tab = await start(delay, log);
      const run = await tab.eval<Run>(runTicks);
      runs[name] = run;
      console.log(`${name} (readbacks +${delay} ms): ${run.ticks.length} ticks, ${run.held} frames held, ${run.shots.length} captures, ${run.batches} batches; behind the world on ${run.late?.ticks ?? "?"} ticks, at most ${run.late?.most ?? "?"}`);
    }
    const fail = (why: string) => {
      failures++;
      console.log(`FAIL ${why}`);
    };
    const prompt = runs["gpu"]!;
    const late = runs["gpu late"]!;
    for (const [name, run] of Object.entries(runs)) {
      if (run.state !== "gpu") fail(`${name}: og.motion ran on ${run.state}, not gpu`);
      if (run.batches < 4) fail(`${name}: ${run.batches} batches in ${TICKS} ticks: og.motion is not posing the course`);
      if (run.held !== 0) fail(`${name}: the gate held ${run.held} frames for og.motion`);
      else console.log(`ok   ${name}: no frame held, every frame call took a tick`);
    }
    if ((late.late?.ticks ?? 0) < 30) fail(`gpu late: the Presenter was behind on ${late.late?.ticks} ticks; the readbacks were not late`);
    const world = late.ticks.findIndex((t, i) => t.tick !== prompt.ticks[i]?.tick || t.hash !== prompt.ticks[i]?.hash);
    if (world >= 0 || late.ticks.length !== prompt.ticks.length) fail(`gpu late: the world parts from the prompt run at tick ${late.ticks[world]?.tick ?? "end"}`);
    else console.log(`ok   gpu late: ${late.ticks.length} ticks' world hashes equal the prompt run's`);
    const shot = late.shots.findIndex((s, i) => s.tick !== prompt.shots[i]?.tick || s.png !== prompt.shots[i]?.png);
    if (shot >= 0 || late.shots.length !== prompt.shots.length) fail(`gpu late: the capture at tick ${late.shots[shot]?.tick ?? "end"} differs from the prompt run's`);
    else console.log(`ok   gpu late: ${late.shots.length} captures (ticks ${late.shots.map((s) => s.tick).join(", ")}) are the prompt run's PNGs`);
    result["runs"] = Object.fromEntries(Object.entries(runs).map(([k, v]) => [k, { held: v.held, late: v.late, batches: v.batches, captures: v.shots.map((s) => s.tick) }]));
  }
  result["engine"] = log.slice(-40);
} catch (e) {
  failures++;
  result["error"] = String(e instanceof Error ? e.stack : e);
  console.log(`FAIL ${result["error"]}\n${log.slice(-20).join("\n")}`);
} finally {
  await browser.close();
  server.stop();
}
writeFileSync(join(work, "motion-ticks.json"), JSON.stringify(result, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} og.motion's ticks on the web; ${join(work, "motion-ticks.json")}`);
process.exit(failures === 0 ? 0 : 1);
