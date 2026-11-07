// og.motion at fixed ticks on the web (docs/design.md §2.4): a GPU batch's
// frames are taken at its due tick whenever its readback lands. On the web,
// which cannot wait for the GPU, the probe's tick gate holds the next tick
// until og.motion's step has its batch (crates/og-runtime/src/motion/present.rs).
//
// motion-feet's course (tests/fixtures/motion-feet: two agents walking,
// running, jumping, turning, posing) runs in headless Chrome on WebGPU from
// its dev web export, one tick per `frame` call, --ticks ticks (default
// 420), twice: as fast as the GPU answers, then with every og.motion
// readback (the `og-motion readback` buffer's mapAsync, patched by a module
// the engine Worker imports before game.js) held back --delay ms (default
// 600), past its due tick at the harness's pace. Each tick's snapshot
// (asked again while its presentation hash is empty: og.motion's step at
// that tick still waits) is read, and checked:
// - the delayed run's world and presentation hash equal the prompt run's
//   on every tick (a batch decoded when its readback lands, the old
//   arrival-driven path, parts them at the first late batch);
// - both runs' world hashes equal the native host's on every tick
//   (tests/support/presentation-capture.ts --mode ticks), so a hold never
//   drops, repeats or changes a tick;
// - both runs' og.motion schedule equals native's: `dump_motion`'s `taken`
//   (the tick each batch was taken at, and its due tick) and each agent's
//   frames shown, starved and warming up.
// Presentation hashes are compared within the web host only: the model's
// GPU floats are the backend compiler's, so across hosts they are not
// expected to match.
//
// --realtime <s> instead plays the course paced like a page (a frame every
// 16 ms, the wall time since the last as its elapsed) for <s> seconds and
// reports the frames whose tick the gate held, and for how long: what the
// wait costs a player (no comparison).
//
//   bun tests/web/motion-ticks.ts [--ticks N] [--delay ms] [--realtime s] [--chrome <path>]
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
const [WIDTH, HEIGHT] = [640, 360];
const work = join(pkg, "dist", "test-motion-ticks");
mkdirSync(work, { recursive: true });

type Tick = { tick: number; hash: string; presentation: string };
type Motion = { model: { taken: Array<[number, number]>; agents: Array<{ entity: number; framesShown: number; framesStarved: number; framesWarmup: number }> } } | null;
/** What of `dump_motion` is the same on every host: when each batch was taken (and due), each agent's frames shown, starved and warming up. */
const schedule = (m: Motion) => m && { taken: m.model.taken, agents: m.model.agents.map(({ entity, framesShown, framesStarved, framesWarmup }) => ({ entity, framesShown, framesStarved, framesWarmup })) };
/** The ticks that waited for og.motion's step (the gate holding the next tick, or the tick's snapshot asked again), and for how long in all and at most. */
type Held = { ticks: number; ms: number; longest: number; frames: number; retries: number };

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

/** One tick a frame to TICKS, each tick's hashes once og.motion's step there has finished. */
const runTicks = `(async () => {
  const shell = globalThis.__og_shell__;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const ticks = [];
  const held = { frames: 0, ms: 0, longest: 0, ticks: 0, retries: 0 };
  let tick = 0;
  while (tick < ${TICKS}) {
    let frames = 0, retries = 0, waited = 0;
    for (;;) {
      const a = await shell.call("frame", 1000 / 60);
      if (!a || a.ok === false) throw new Error("frame refused at tick " + tick + ": " + JSON.stringify(a));
      if (a.tick > tick) { tick = a.tick; break; }
      frames++;
      const t = performance.now();
      await sleep(2);
      waited += performance.now() - t;
    }
    let s = await shell.probe("snapshot");
    for (; s && s.presentation === "" && retries < 5000; retries++) {
      const t = performance.now();
      await sleep(2);
      waited += performance.now() - t;
      s = await shell.probe("snapshot");
    }
    if (frames + retries > 0) {
      held.ticks++; held.frames += frames; held.retries += retries; held.ms += waited; held.longest = Math.max(held.longest, waited);
    }
    ticks.push({ tick: s.tick, hash: s.hash, presentation: s.presentation });
  }
  await shell.call("probe", "commands", "0:dump_motion:motion.json");
  await shell.call("frame", 0);
  const dump = (await shell.call("outputs")).find((o) => o.action === "dump_motion");
  return { ticks, held, motion: dump ? JSON.parse(dump.data) : null };
})()`;

/** Paced like a page: the frames whose tick the gate held, and for how long. */
const runRealtime = (seconds: number) => `(async () => {
  const shell = globalThis.__og_shell__;
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
    const r = await tab.eval<{ frames: number; ticks: number; episodes: Array<{ atTick: number; ms: number; frames: number }> }>(runRealtime(REALTIME));
    result["realtime"] = r;
    const ms = r.episodes.map((e) => e.ms).sort((a, b) => a - b);
    console.log(`realtime ${REALTIME} s: ${r.frames} frames, ${r.ticks} ticks; ${r.episodes.length} held episodes, ${r.episodes.reduce((n, e) => n + e.frames, 0)} frames; ms median ${ms[ms.length >> 1] ?? 0}, max ${ms[ms.length - 1] ?? 0}`);
    console.log(JSON.stringify(r.episodes));
  } else {
    const r = Bun.spawnSync(["bun", "tests/support/presentation-capture.ts", "--project", project, "--mode", "ticks", "--ticks", String(TICKS), "--size", `${WIDTH}x${HEIGHT}`], { cwd: pkg });
    if (r.exitCode !== 0) throw new Error(`native: ${new TextDecoder().decode(r.stderr)}`);
    const ran = JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").pop()!) as { ticks: Tick[]; motion: Motion };
    const native = ran.ticks;
    const runs: Record<string, { ticks: Tick[]; held: Held; motion: Motion }> = {};
    for (const [name, delay] of [["prompt", 0], ["delayed", DELAY]] as const) {
      const tab = await start(delay, log);
      const run = await tab.eval<{ ticks: Tick[]; held: Held; motion: Motion }>(runTicks);
      runs[name] = run;
      console.log(`${name} (readbacks +${delay} ms): ${run.ticks.length} ticks; ${run.held.ticks} waited for og.motion's step, ${run.held.ms.toFixed(0)} ms in all, longest ${run.held.longest.toFixed(0)} ms (${run.held.frames} frames held, ${run.held.retries} snapshots asked again)`);
    }
    const fail = (why: string) => {
      failures++;
      console.log(`FAIL ${why}`);
    };
    const batches = ran.motion?.model.taken.length ?? 0;
    const generated = new Set(native.map((t) => t.presentation)).size;
    if (generated < TICKS / 2 || batches < 4) fail(`the native run drew ${generated} distinct presentations and took ${batches} batches in ${TICKS} ticks: og.motion is not posing the course`);
    if (ran.motion?.model.taken.some(([taken, due]) => taken > due)) fail(`a native batch was taken after its due tick: ${JSON.stringify(ran.motion.model.taken)}`);
    const prompt = runs["prompt"]!;
    const delayed = runs["delayed"]!;
    // The delay must have bitten: the gate held the tick a batch was due at.
    if (delayed.held.ticks < batches / 2) fail(`delayed: og.motion waited on ${delayed.held.ticks} ticks for ${batches} batches; the readbacks were not late`);
    // 1. On the web, when a readback lands changes nothing: every tick's hashes equal.
    const parted = delayed.ticks.findIndex((t, i) => t.tick !== prompt.ticks[i]?.tick || t.hash !== prompt.ticks[i]?.hash || t.presentation !== prompt.ticks[i]?.presentation || t.presentation === "");
    if (parted >= 0 || delayed.ticks.length !== prompt.ticks.length) fail(`delayed parts from prompt at tick ${delayed.ticks[parted]?.tick ?? "end"}: ${JSON.stringify(delayed.ticks[parted])} against ${JSON.stringify(prompt.ticks[parted])}`);
    else console.log(`ok   delayed: ${delayed.ticks.length} ticks' world and presentation hashes equal the prompt run's`);
    // 2. Each web run against native: the same world every tick, and og.motion's
    // schedule (each batch taken at its due tick, the same frames shown).
    for (const [name, run] of Object.entries(runs)) {
      const world = run.ticks.findIndex((t, i) => t.tick !== native[i]?.tick || t.hash !== native[i]?.hash);
      if (world >= 0 || run.ticks.length !== native.length) fail(`${name}: the world parts from native at tick ${run.ticks[world]?.tick ?? "end"}`);
      const a = JSON.stringify(schedule(run.motion));
      const b = JSON.stringify(schedule(ran.motion));
      if (a !== b) fail(`${name}: og.motion's schedule is not native's:\n  web    ${a}\n  native ${b}`);
      else console.log(`ok   ${name}: world hashes on ${native.length} ticks and og.motion's schedule (${batches} batches taken at their due ticks, every agent's frames) equal native's`);
    }
    result["native"] = native.length;
    result["schedule"] = schedule(ran.motion);
    result["runs"] = Object.fromEntries(Object.entries(runs).map(([k, v]) => [k, { held: v.held, ticks: v.ticks }]));
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
