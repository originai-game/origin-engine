// Scratch: one fx-showcase export on the phone, traced from navigation: first frame, the tap after <tapDelay> s,
// the stall after it (tick frozen, longest frame gap), boot.pipelines 20 s after the first frame, the longest
// HandleDawnCommands slices, and every pipeline the Worker asked for (label, method, phone time).
//   bun trace-cmp.ts <dist> <out.json> [tapDelaySeconds=1]
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const dist = resolve(process.argv[2]!);
const outFile = process.argv[3]!;
const tapDelay = Number(process.argv[4] ?? 1);
const flush = process.argv[5] === "flush";
const server = serve({ dist, isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });

// The Worker's pipeline calls, recorded before its own script runs (auto-attach, paused at start).
const HOOK = `(() => {
  const p = self.GPUDevice && GPUDevice.prototype; if (!p || self.__pl) return;
  self.__pl = [];
  for (const m of ["createRenderPipeline", "createRenderPipelineAsync", "createComputePipeline", "createComputePipelineAsync"]) {
    const f = p[m];
    p[m] = function (d) { self.__pl.push({ m, label: d && d.label, at: Date.now() }); const dev = this; if (self.__flush && !self.__flushing) { self.__flushing = true; setTimeout(() => { self.__flushing = false; dev.queue.submit([]); }, 0); } return f.apply(this, arguments); };
  }
  const sm = p.createShaderModule;
  p.createShaderModule = function (d) { self.__pl.push({ m: "createShaderModule", label: d && d.label, at: Date.now(), bytes: d && d.code && d.code.length }); return sm.apply(this, arguments); };
  const seen = new WeakSet();
  for (const E of [self.GPURenderPassEncoder, self.GPUComputePassEncoder, self.GPURenderBundleEncoder]) {
    if (!E) continue;
    const set = E.prototype.setPipeline;
    E.prototype.setPipeline = function (pl) { if (pl && typeof pl === "object" && !seen.has(pl)) { seen.add(pl); self.__pl.push({ m: "firstUse", label: pl.label, at: Date.now() }); } return set.apply(this, arguments); };
  }
})()`;
const workers: string[] = [];
let done: (h: string) => void;
const complete = new Promise<string>((r) => (done = r));
const consoleLines: string[] = [];
browser.on((method, params) => {
  if (method === "Runtime.consoleAPICalled") { const a = (params["args"] as { value?: unknown }[] | undefined) ?? []; const t = a.map((x) => String(x.value ?? "")).join(" "); if (t.includes("[og] pipelines")) consoleLines.push(t); }
  if (method === "Tracing.tracingComplete") done(params["stream"] as string);
  if (method === "Target.attachedToTarget") {
    console.error("attached", JSON.stringify(params["targetInfo"]).slice(0, 200), params["waitingForDebugger"]);
    const info = params["targetInfo"] as { type: string };
    const sid = params["sessionId"] as string;
    if (info.type === "worker" || info.type === "dedicated_worker") {
      workers.push(sid);
      void (async () => {
        await browser.send("Runtime.enable", {}, sid).catch(() => {});
        await browser.send("Runtime.runIfWaitingForDebugger", {}, sid);
        for (let i = 0; i < 200; i++) {
          const r = await browser.send<{ result: { value?: unknown } }>("Runtime.evaluate", { expression: `${HOOK}, self.__flush = ${flush}, !!self.__pl`, returnByValue: true }, sid).catch(() => null);
          if (r?.result.value === true) return console.error(`worker hooked after ${i} tries`);
          await Bun.sleep(5);
        }
        console.error("worker hook failed");
      })();
    } else void browser.send("Runtime.runIfWaitingForDebugger", {}, sid).catch(() => {});
  }
});
await browser.send("Tracing.start", { transferMode: "ReturnAsStream", traceConfig: { includedCategories: ["gpu", "dawn", "disabled-by-default-gpu.dawn", "gpu.service", "toplevel", "blink.user_timing", "devtools.timeline", "disabled-by-default-gpu.service", "gpu.dawn", "disabled-by-default-dawn", "disabled-by-default-gpu_decoder", "skia.gpu", "viz"] } });
const log: string[] = [];
const page = await browser.open("about:blank", log, { touch: true });
await page.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: true, flatten: true });
await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
const firstFrame = await page.eval<{ ms: number; at: number }>(`({ ms: Math.round(${mark("og:first-frame")}), at: Math.round(performance.timeOrigin + ${mark("og:first-frame")}) })`);
const ffHost = performance.now();
await Bun.sleep(tapDelay * 1000);
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
const tapAt = await page.eval<number>("Date.now()");
if (box) {
  await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box.x, y: box.y }] });
  await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
}
// From the tap: the tick every 250 ms for 16 s (the longest stretch it stays frozen, and when it first moves), and og:start.
const t1 = performance.now();
const tick0 = await page.eval<number>("__runtime_probe__.tick()");
let last = tick0, since = t1, frozen = 0, firstMoveMs = -1, startMs = -1;
await page.eval("__og_timings__(true)");
while (performance.now() - t1 < 16_000) {
  const t = await page.eval<number>("__runtime_probe__.tick()");
  const now = performance.now();
  if (t !== last) { if (firstMoveMs < 0) firstMoveMs = Math.round(now - t1); last = t; since = now; } else frozen = Math.max(frozen, now - since);
  if (startMs < 0 && (await page.eval<number>(`${mark("og:start")}`)) > 0) startMs = Math.round(now - t1);
  await Bun.sleep(250);
}
const timings = await page.eval<{ frames: number; intervals: number[] }>("__og_timings__()");
const longestGap = Math.round(Math.max(0, ...(timings.intervals ?? [])));
const wait = 20_000 - (performance.now() - ffHost);
if (wait > 0) await Bun.sleep(wait);
const pipelines = await page.eval<unknown>("__runtime_probe__.boot.pipelines");
const late = 50_000 - (performance.now() - ffHost);
if (late > 0) await Bun.sleep(late);
const pipelinesLate = await page.eval<unknown>("__runtime_probe__.boot.pipelines");
const calls: { m: string; label: string; at: number }[] = [];
for (const sid of workers) {
  try {
    const r = await browser.send<{ result: { value?: typeof calls } }>("Runtime.evaluate", { expression: "self.__pl || []", returnByValue: true }, sid);
    calls.push(...(r.result.value ?? []));
  } catch { /* gone */ }
}
await browser.send("Tracing.end");
const stream = await complete;
const chunks: string[] = [];
for (;;) {
  const r = await browser.send<{ data: string; eof: boolean; base64Encoded?: boolean }>("IO.read", { handle: stream, size: 1 << 20 });
  chunks.push(r.base64Encoded ? Buffer.from(r.data, "base64").toString() : r.data);
  if (r.eof) break;
}
const raw = chunks.join("");
writeFileSync(outFile.replace(/\.json$/, ".trace.json"), raw);
const parsed = JSON.parse(raw);
const events = (parsed.traceEvents ?? parsed) as { name: string; ph: string; dur?: number; ts: number; args?: Record<string, unknown> }[];
// Align the trace's clock to the phone's wall clock with the first-frame user-timing mark.
const ffEvent = events.find((e) => e.name === "og:first-frame");
const toWall = (ts: number) => (ffEvent ? firstFrame.at + (ts - ffEvent.ts) / 1000 : NaN);
const dawn = events
  .filter((e) => e.name === "WebGPUDecoderImpl::HandleDawnCommands" && (e.dur ?? 0) > 100_000)
  .sort((a, b) => b.dur! - a.dur!)
  .slice(0, 8)
  .map((e) => ({ s: +(e.dur! / 1e6).toFixed(2), fromTap: +((toWall(e.ts) - tapAt) / 1000).toFixed(2), fromFirstFrame: +((toWall(e.ts) - firstFrame.at) / 1000).toFixed(2), bytes: e.args?.["bytes"] }));
const rel = (at: number) => +((at - tapAt) / 1000).toFixed(2);
const before = calls.filter((c) => c.at < tapAt), after = calls.filter((c) => c.at >= tapAt);
const result = {
  dist, tapDelay, firstFrameMs: firstFrame.ms, tickFirstMovesAfterTapMs: firstMoveMs, ogStartAfterTapMs: startMs, tickFrozenMs: Math.round(frozen), longestFrameGapMs: longestGap, framesIn16s: timings.frames,
  pipelines20s: pipelines, pipelines50s: pipelinesLate, workerConsole: consoleLines, dawnSlices: dawn,
  firstUseAfterTap: after.filter((c) => c.m === "firstUse").length,
  firstUseBeforeTap: before.filter((c) => c.m === "firstUse").length,
  shaderModules: [before.filter((c) => c.m === "createShaderModule").length, after.filter((c) => c.m === "createShaderModule").length],
  calls: { total: calls.length, beforeTap: before.length, afterTap: after.length, lastBeforeTapS: before.length ? rel(before.at(-1)!.at) : null, byMethod: Object.fromEntries(["createRenderPipeline", "createRenderPipelineAsync", "createComputePipeline", "createComputePipelineAsync"].map((m) => [m, [calls.filter((c) => c.m === m && c.at < tapAt).length, calls.filter((c) => c.m === m && c.at >= tapAt).length]])) },
  afterTapLabels: after.map((c) => `${rel(c.at)}s ${c.m} ${c.label ?? "?"}`),
  lastBeforeTapLabels: before.slice(-12).map((c) => `${rel(c.at)}s ${c.m} ${c.label ?? "?"}`),
};
writeFileSync(outFile, JSON.stringify(result, null, 1));
console.log(JSON.stringify({ ...result, afterTapLabels: result.afterTapLabels.slice(0, 40), lastBeforeTapLabels: result.lastBeforeTapLabels }, null, 1));
await browser.close();
server.stop();
process.exit(0);
