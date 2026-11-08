// A presentation checkpoint on the web (docs/design.md §2.4): the display
// frames drawn after a restore are the uninterrupted run's.
//
// Each case runs its fixture in headless Chrome on WebGPU from its dev web
// export, one tick a frame, each display frame read as presented (the
// canvas, through CDP's Page.captureScreenshot): to --fork-at, a snapshot, the display frames on
// to the end (pass A), another branch of 24 ticks from the fork without the
// inputs, the fork restored and the same ticks drawn again (pass B), as
// tests/support/presentation-capture.ts --mode display does natively.
// - Within the web host, every frame of B equals A's byte for byte.
// - Against native: each frame of A is compared with the native pass's
//   under the cross-platform tolerance (png.ts CROSS_TOLERANCE), since
//   Metal through wgpu and Chrome's Dawn compile the shaders apart; not
//   for motion-feet, whose og.motion poses are the GPU's on each host.
//
// Cases: vehicle-terrain mid-slide (TAA, motion vectors), terrain-stream
// before its tiles stream in (tile arrival and fades) and motion-feet
// mid-course (og.motion's Presenter). motion-feet settles every tick as a
// fork check does (a small probe capture waits for each og.motion result
// due by the tick), then draws the frame read.
//
//   bun tests/web/presentation-fork.ts [--case vehicle-terrain|terrain-stream|motion-feet] [--chrome <path>]
//
// Needs Chrome (cdp.ts), a GPU and the motion model for motion-feet. Writes
// dist/test-presentation-fork/presentation-fork.json; exits 1 on a mismatch.

import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { createHash } from "node:crypto";
import { compareFrames, CROSS_TOLERANCE, decodePng, type Image } from "../support/png.ts";
import { cookProject } from "../support/og-bin.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const work = join(pkg, "dist", "test-presentation-fork");
mkdirSync(work, { recursive: true });

type Case = { name: string; ticks: number; forkAt: number; width: number; height: number; inputs: string; settle?: boolean };
const CASES: Case[] = [
  { name: "vehicle-terrain", ticks: 156, forkAt: 110, width: 960, height: 540, inputs: "12:key_down:KeyW;120:key_down:KeyD;120:key_down:Space;138:key_up:Space" },
  { name: "terrain-stream", ticks: 110, forkAt: 20, width: 960, height: 540, inputs: "" },
  { name: "motion-feet", ticks: 360, forkAt: 300, width: 640, height: 360, inputs: "", settle: true },
];
const only = opt("--case");
const cases = CASES.filter((c) => !only || c.name === only);
if (cases.length === 0) throw new Error(`--case is one of ${CASES.map((c) => c.name).join(", ")}`);

/** The page's helpers: `__fork.step(pressing)` one tick, `__fork.snapshot()`, `__fork.restore(id, tick)`. */
const pageHelpers = (c: Case) => `(() => {
  const shell = globalThis.__og_shell__;
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const inputs = new Map();
  for (const step of ${JSON.stringify(c.inputs)}.split(";").filter(Boolean)) {
    const at = step.indexOf(":");
    const t = Number(step.slice(0, at));
    inputs.set(t, [...(inputs.get(t) ?? []), "0:" + step.slice(at + 1)]);
  }
  const f = { tick: 0 };
  f.start = async () => { f.tick = (await shell.call("frame", 0)).tick; return f.tick; };
  // One tick: the inputs due at it, then frames until it has stepped (og.motion's gate may hold one).
  f.step = async (pressing) => {
    const due = pressing ? inputs.get(f.tick) : undefined;
    if (due) await shell.probe("commands", { script: due.join(";") });
    // Settled: the tick stepped without a frame, so no frame before the
    // settle feeds TAA's history.
    if (${c.settle ? "true" : "false"}) f.tick = (await shell.call("advance", 1)).tick ?? f.tick + 1;
    for (; !${c.settle ? "true" : "false"};) {
      const a = await shell.call("frame", 1000 / 60);
      if (!a || a.ok === false) throw new Error("frame refused at tick " + f.tick + ": " + JSON.stringify(a));
      if (a.tick > f.tick) { f.tick = a.tick; break; }
      await sleep(2);
    }
    if (${c.settle ? "true" : "false"}) {
      // Settled, then the tick drawn again without stepping.
      let shot = null;
      for (let i = 0; !shot && i < 5000; i++) {
        shot = await shell.probe("capture", { width: 64, height: 36 });
        if (!shot) await sleep(2);
      }
      if (!shot) throw new Error("no settled capture at tick " + f.tick);
      await shell.call("frame", 0);
    }
    return f.tick;
  };
  f.snapshot = async () => {
    for (let i = 0; i < 5000; i++) {
      const s = await shell.probe("snapshot");
      if (s && s.presentation !== "") return s;
      await sleep(2);
    }
    throw new Error("no snapshot at tick " + f.tick);
  };
  f.restore = async (id, tick) => {
    if (!(await shell.probe("restore", { id }))) throw new Error("restore refused");
    f.tick = tick;
    return tick;
  };
  globalThis.__fork = f;
  return true;
})()`;

const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
const pageJs = await page.outputs[0]!.text();

const result: Record<string, unknown> = { tolerance: CROSS_TOLERANCE, cases: {} };
let failures = 0;
const fail = (why: string) => {
  failures++;
  console.log(`FAIL ${why}`);
};
for (const c of cases) {
  const project = join(repo, "tests", "fixtures", c.name);
  cookProject(project, repo);
  const nativeDir = join(work, `${c.name}-native`);
  rmSync(nativeDir, { recursive: true, force: true });
  const n = Bun.spawnSync(["bun", "tests/support/presentation-capture.ts", "--project", project, "--mode", "display", "--ticks", String(c.ticks), "--fork-at", String(c.forkAt), "--size", `${c.width}x${c.height}`, "--frames", nativeDir, ...(c.inputs ? ["--inputs", c.inputs] : []), ...(c.settle ? ["--settle"] : [])], { cwd: pkg });
  if (n.exitCode !== 0) throw new Error(`native ${c.name}: ${new TextDecoder().decode(n.stderr)}`);
  const native = JSON.parse(new TextDecoder().decode(n.stdout).trim().split("\n").pop()!) as { checked: number; mismatches: string[] };

  const dist = join(work, c.name);
  const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
  if (!built.ok) throw new Error(built.reason);
  writeFileSync(join(dist, "reference-page.js"), pageJs);
  const html = `<!doctype html><html><head><meta charset="utf-8"><title>presentation fork</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${c.width}px;height:${c.height}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
  const server = serve({ dist, isolate: true, extra: { "/fork.html": html } });
  const browser = await Browser.from(args, { chrome: chromePath(args), width: c.width, height: c.height, reverse: [server.port] });
  const log: string[] = [];
  try {
    const tab = await browser.open("about:blank", log);
    await tab.send("Emulation.setDeviceMetricsOverride", { width: c.width, height: c.height, deviceScaleFactor: 1, mobile: false });
    await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/fork.html?profile=${settingsOf(project).profile}` });
    await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
    const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
    if (failed) throw new Error(`the engine did not start: ${failed}`);
    await tab.eval(pageHelpers(c));
    const ev = <T,>(js: string) => tab.eval<T>(js);
    let tick = await ev<number>("__fork.start()");
    // The display frame as presented: the canvas, read through the compositor (CDP).
    const shot = async () => decodePng(await tab.screenshot());
    while (tick < c.forkAt) tick = await ev<number>("__fork.step(true)");
    const fork = await ev<{ id: string; tick: number }>("__fork.snapshot()");
    const pass = async (keep: boolean) => {
      const out: Array<{ tick: number; sha: string; img?: Image }> = [];
      while (tick < c.ticks) {
        tick = await ev<number>("__fork.step(true)");
        const img = await shot();
        out.push({ tick, sha: createHash("sha256").update(img.rgba).digest("hex"), ...(keep ? { img } : {}) });
      }
      return out;
    };
    const a = await pass(true);
    tick = await ev<number>(`__fork.restore(${JSON.stringify(fork.id)}, ${fork.tick})`);
    while (tick < Math.min(c.ticks, c.forkAt + 24)) tick = await ev<number>("__fork.step(false)");
    tick = await ev<number>(`__fork.restore(${JSON.stringify(fork.id)}, ${fork.tick})`);
    const b = await pass(false);
    const r = { fork: fork.tick, a, b };
    // 1. Within the web host, byte for byte.
    const parted = r.a.filter((f, i) => f.tick !== r.b[i]?.tick || f.sha !== r.b[i]?.sha).map((f) => f.tick);
    if (parted.length > 0 || r.a.length !== r.b.length || r.a.length !== c.ticks - c.forkAt) fail(`${c.name}: web frames after the restore differ from the uninterrupted run's at ticks ${parted.join(", ")} (${r.a.length} and ${r.b.length} frames)`);
    else console.log(`ok   ${c.name}: ${r.a.length} web display frames after a restore at tick ${r.fork} equal the uninterrupted run's byte for byte`);
    // 2. Against native, under the cross-platform tolerance.
    if (native.mismatches.length > 0) fail(`${c.name}: native frames after the restore differ: ${native.mismatches.join("; ")}`);
    let worst = { tick: 0, rmse: 0, outliers: 0, bias: 0 };
    const over: number[] = [];
    for (const f of c.settle ? [] : r.a) {
      const web = f.img!;
      const nat = { width: c.width, height: c.height, rgba: new Uint8Array(readFileSync(join(nativeDir, `${f.tick}.rgba`))) };
      const d = compareFrames(web, nat, CROSS_TOLERANCE);
      if (!d.ok) over.push(f.tick);
      if (d.rmse > worst.rmse) worst = { tick: f.tick, rmse: +d.rmse.toFixed(5), outliers: +(d.outliers * 100).toFixed(3), bias: +d.bias.toFixed(5) };
    }
    if (over.length > 0) fail(`${c.name}: web frames past the cross-platform tolerance against native at ticks ${over.join(", ")} (worst ${JSON.stringify(worst)})`);
    else if (!c.settle) console.log(`ok   ${c.name}: every web frame within the cross-platform tolerance of native's (worst tick ${worst.tick}: RMSE ${worst.rmse}, ${worst.outliers} % outliers)`);
    else console.log(`ok   ${c.name}: native frames after the restore equal the uninterrupted run's (${native.checked}); poses are not compared across hosts`);
    (result["cases"] as Record<string, unknown>)[c.name] = { fork: r.fork, frames: r.a.length, parted, native: native.checked, worstAgainstNative: worst, overTolerance: over };
  } catch (e) {
    failures++;
    console.log(`FAIL ${c.name}: ${e instanceof Error ? e.stack : e}\n${log.slice(-20).join("\n")}`);
  } finally {
    await browser.close();
    server.stop();
  }
}
writeFileSync(join(work, "presentation-fork.json"), JSON.stringify(result, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} presentation checkpoints on the web; ${join(work, "presentation-fork.json")}`);
process.exit(failures === 0 ? 0 : 1);
