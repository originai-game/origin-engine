// Capture/readback ownership under a delayed GPU and a busy Worker. Runs
// the real wasm renderer on an OffscreenCanvas; no reference frames.
//   bun tests/web/capture.ts [--chrome <path>]
import { cpSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; outdir: string; target: string; format: string }): Promise<{ success: boolean; logs: unknown[] }>;
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const builtAt = process.argv.indexOf("--built");
const work = builtAt < 0 ? join(pkg, "dist", "test-capture") : resolve(process.argv[builtAt + 1]!);
mkdirSync(work, { recursive: true });
if (builtAt < 0) {
writeFileSync(join(work, "worker.ts"), `
import { startWorker } from ${JSON.stringify(join(pkg, "src/web/worker.ts"))};
import { og, defineSystem } from ${JSON.stringify(join(pkg, "src/index.ts"))};
let mode = "normal";
addEventListener("message", (e) => { if (e.data.captureMode) mode = e.data.captureMode; });
const requestDevice = GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice = async function (...args) {
  const device = await requestDevice.apply(this, args);
  device.lost.then((loss) => self.postMessage({ captureDeviceLost: loss.message }));
  return device;
};
const raf = self.requestAnimationFrame.bind(self);
self.requestAnimationFrame = (f) => mode === "hidden" ? 0 : raf(f);
const map = GPUBuffer.prototype.mapAsync;
GPUBuffer.prototype.mapAsync = function (...args) {
  const p = map.apply(this, args);
  // Delay only explicit 160 × 90 probe captures, not the fallback
  // canvas's 256 × 144 presentation readbacks on the same device.
  if (this.label !== "capture" || this.size !== Math.ceil(160 * 4 / 256) * 256 * 90 || mode === "normal") return p;
  const policy = mode;
  return p.then(async (v) => {
    if (policy === "fail") throw new Error("injected capture mapping failure");
    if (policy === "hang") return new Promise(() => {});
    await new Promise((r) => setTimeout(r, 300));
    // Contention in the same event loop that delivers mapAsync and RPC.
    const end = performance.now() + 25;
    while (performance.now() < end) {}
    return v;
  });
};
const draw = defineSystem("CaptureTick", {}, (_q, _t, w) => {
  if (w.tick < 2) og.render.sprite(w, { position: { x: 0, y: 0, z: -5 }, width: 3, height: 3 });
});
startWorker(og.game({ systems: [draw] }));
`);
writeFileSync(join(work, "page.ts"), `
import { Shell } from ${JSON.stringify(join(pkg, "src/web/shell.ts"))};
const worker = new Worker("/game/worker.js", { type: "module" });
globalThis.captureWorker = worker;
globalThis.captureLosses = [];
worker.addEventListener("message", ({data}) => { if (data.captureDeviceLost) captureLosses.push(data.captureDeviceLost); });
Shell.start({ canvas: document.querySelector("canvas"), worker,
  glue: "/game/origin-engine.js", wasm: "/game/origin-engine_bg.wasm",
  autoplay: false, prime: false, dprCap: 1,
}).then((shell) => { globalThis.shell = shell; }).catch((e) => { globalThis.failed = String(e); });
`);
const built = await Bun.build({ entrypoints: [join(work, "worker.ts"), join(work, "page.ts")], outdir: work, target: "browser", format: "esm" });
if (!built.success) throw new Error(built.logs.map(String).join("\n"));
// The glue may import snippets beside itself.
cpSync(join(pkg, "dist/web"), work, { recursive: true });
}
// Serve the development wasm beside this test's bundled Worker.
const server = serve({ dist: work, isolate: !process.argv.includes("--no-isolation"), extra: {
  "/capture.html": '<!doctype html><canvas width="256" height="144"></canvas><script type="module" src="/game/page.js"></script>',
} });
const browser = await Browser.launch({ chrome: chromePath(process.argv), width: 160, height: 90 });
const log: string[] = [];
let failures = 0;
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/capture.html`, log);
  await page.until("globalThis.shell || globalThis.failed", 90_000);
  await page.until("globalThis.shell?.boot || globalThis.failed", 90_000);
  const result = await page.eval<Record<string, unknown>>(`(async () => {
    if (globalThis.failed) throw new Error(globalThis.failed);
    const s = globalThis.shell, worker = globalThis.captureWorker;
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    const mode = async (captureMode) => { worker.postMessage({ captureMode }); await s.call("hash"); };
    const shot = () => s.probe("capture", { width: 160, height: 90, fit: 0 });
    const errors = {};
    await mode("delay");
    await s.call("advance", 1);
    const first = await shot();
    // A command/direct probe can leave a readback pending while ticks run.
    const pending = await s.call("probe", "capture", 160, 90);
    await s.call("advance", 3);
    await sleep(400);
    let stale = null;
    try { stale = await shot(); } catch (e) { errors.stale = String(e); }
    const current = await shot();
    await mode("fail");
    try { await shot(); } catch (e) { errors.mapping = String(e); }
    await mode("delay");
    const recovered = await shot();
    const bootLosses = captureLosses.slice();
    await mode("normal");
    s.play(true);
    const lost = await s.call("loseDevice");
    const deadline = performance.now() + 10000;
    let display = await s.call("display");
    while ((display?.deviceLosses ?? 0) === 0 && performance.now() < deadline) {
      await sleep(50);
      display = await s.call("display");
    }
    const replacement = await s.probe("capture", { width: 158, height: 88, fit: 0 });
    s.play(false);
    await mode("hang");
    try { await shot(); } catch (e) { errors.deadline = String(e); }
    await mode("hidden");
    s.play(true);
    await s.call("hash");
    // A readback needs GPU/event-loop progress, not a new surface frame.
    const hidden = await Promise.race([s.probe("capture", { width: 159, height: 89, fit: 0 }), sleep(1500).then(() => null)]);
    s.play(false);
    return { pending, first: { tick: first?.tick }, stale: stale && { tick: stale.tick, previousPixels: stale.png === first.png },
      current: { tick: current?.tick, changed: current?.png !== first?.png },
      recovered: { tick: recovered?.tick, same: recovered?.png === current?.png }, hidden: !!hidden?.png,
      boot: !!s.boot, losses: bootLosses,
      replacement: { lost, devices: display?.deviceLosses, tick: replacement?.tick, png: !!replacement?.png }, errors };
  })()`);
  console.log(JSON.stringify(result, null, 2));
  const errors = result.errors as Record<string, string>;
  for (const [ok, label] of [
    [result.boot === true && (result.losses as unknown[]).length === 0, "a complete canvas frame boots and captures without losing its device"],
    [result.pending === null && /tick 1.*tick 4/.test(errors.stale ?? ""), "a tick-1 readback cannot be returned as tick 4"],
    [(result.current as { tick: number; changed: boolean }).tick === 4 && (result.current as { changed: boolean }).changed, "a fresh tick-4 capture contains the current scene"],
    [/readback:.*map/i.test(errors.mapping ?? ""), "a failed mapping reports its cause instead of restarting"],
    [(result.recovered as { same: boolean }).same, "a subsequent capture succeeds after a mapping error"],
    [(result.replacement as { lost: boolean; devices: number; tick: number; png: boolean }).lost
      && (result.replacement as { devices: number }).devices === 1
      && (result.replacement as { tick: number }).tick > 4
      && (result.replacement as { png: boolean }).png, "a lost device is replaced while ticks continue and capture recovers"],
    [result.hidden === true, "a playing capture completes without presentation callbacks"],
    [/readback.*20000 ms/.test(errors.deadline ?? ""), "a readback deadline is an error, never null success"],
  ] as const) {
    console.log(`${ok ? "ok  " : "FAIL"} ${label}`);
    if (!ok) failures++;
  }
} catch (e) {
  failures++;
  console.error(e, log.slice(-20));
} finally {
  await browser.close();
  server.stop(true);
}
process.exitCode = failures ? 1 : 0;
