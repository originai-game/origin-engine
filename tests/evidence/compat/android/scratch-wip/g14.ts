// Scratch: G1-4 on the phone (frame cap, render scale, auto quality, battery
// saver), boot, memory and thermals, on fx-showcase's dev export with a
// temporary `display:` command and display/capabilities in its facts.
import { writeFileSync } from "node:fs";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const dist = (await import("node:path")).resolve("C:/Users/win/src/origin-engine-android/packages/engine/dist/test-reference/fx-showcase");
const out = process.argv[2]!;
const STRESS_S = Number(process.argv[3] ?? 90);
const server = serve({ dist, isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const log: string[] = [];
const sh = (cmd: string) => new TextDecoder().decode(Bun.spawnSync(["adb", "shell", cmd]).stdout);
const thermal = () => {
  const t: Record<string, number> = {};
  for (const m of sh("dumpsys thermalservice").matchAll(/mValue=([-\d.E]+), mType=-?\d+, mName=([A-Za-z0-9_-]+)/g)) t[m[2]!] = +Number(m[1]).toFixed(1);
  const status = /Thermal Status: (\d+)/.exec(sh("dumpsys thermalservice"))?.[1];
  return { status: Number(status), skin: t["VIRTUAL-SKIN-LEGACY"], soc: t["soc_therm"], big: t["BIG"], mid: t["MID"], little: t["LITTLE"], battery: t["battery"] };
};
const result: Record<string, unknown> = { at: new Date().toISOString(), device: browser.device };
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log, { touch: true });
const t0 = performance.now();
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
result["boot"] = await page.eval(`({ firstFrameMs: Math.round(${mark("og:first-frame")}), readyMs: Math.round(${mark("og:ready")}), boot: __runtime_probe__.boot, viewport: [innerWidth, innerHeight, devicePixelRatio] })`);
console.log("boot", JSON.stringify(result["boot"]).slice(0, 400), "harness", Math.round(performance.now() - t0), "ms");
// The play gate: a real tap.
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
if (box) {
  await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box.x, y: box.y }] });
  await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
}
await page.until(`${mark("og:start")} > 0`, 15_000);
const state = () => page.eval<{ facts?: Record<string, any> } & Record<string, any>>("__runtime_probe__.state()");
const facts = async () => { const s = await state(); return (s.facts ?? s.game ?? s) as Record<string, any>; };
const run = (script: string) => page.eval(`__runtime_probe__.commands(${JSON.stringify(script)})`);
const stats = (a: number[]) => { const s = [...a].sort((x, y) => x - y); const q = (p: number) => +(s[Math.min(s.length - 1, Math.floor(p * s.length))] ?? 0).toFixed(2); return { n: a.length, median: q(0.5), p5: q(0.05), p95: q(0.95), p99: q(0.99) }; };
async function window(seconds: number) {
  await page.eval("__og_timings__(true)");
  const tick0 = await page.eval<number>("__runtime_probe__.tick()");
  const began = performance.now();
  await Bun.sleep(seconds * 1000);
  const t = await page.eval<{ frames: number; intervals: number[]; work: number[] }>("__og_timings__()");
  const el = (performance.now() - began) / 1000;
  const ticks = (await page.eval<number>("__runtime_probe__.tick()")) - tick0;
  return { fps: +(t.frames / el).toFixed(1), ticksPerSecond: +(ticks / el).toFixed(1), interval: stats(t.intervals), work: stats(t.work) };
}
await Bun.sleep(40000); // past the compiles the tap flushes (sync warm-up, about 23 s on PowerVR)
const f0 = await facts();
result["capabilities"] = f0["capabilities"];
result["displayDefault"] = f0["display"];
console.log("capabilities.visibilityBuffer", f0["capabilities"]?.visibilityBuffer, JSON.stringify(f0["capabilities"]?.visibilityBufferRefused ?? null).slice(0, 160));
console.log("display", JSON.stringify(f0["display"]).slice(0, 500));
result["thermalStart"] = thermal();
const caps: Record<string, unknown> = {};
for (const cap of ["display", "fps60", "fps30"]) {
  await run(`0.05:display:${cap}:${process.env.G1_SCALE ?? 1}:off`);
  await Bun.sleep(1500);
  caps[cap] = { ...(await window(6)), display: (await facts())["display"] };
  console.log("cap", cap, JSON.stringify(caps[cap]).slice(0, 300));
}
result["frameCap"] = caps;
await run(`0.05:display:display:0.5:off`);
await Bun.sleep(1500);
result["renderScaleHalf"] = { ...(await window(4)), display: (await facts())["display"], canvas: await page.eval(`[document.querySelector("canvas").width, document.querySelector("canvas").height]`) };
writeFileSync(`${out}/scale-0.5.png`, Buffer.from((await page.send<{ data: string }>("Page.captureScreenshot", { format: "png" })).data, "base64"));
console.log("scale 0.5", JSON.stringify(result["renderScaleHalf"]).slice(0, 300));
await run(`0.05:display:auto:1:off:on`);
await Bun.sleep(1500);
result["batterySaver"] = { ...(await window(6)), display: (await facts())["display"] };
console.log("saver", JSON.stringify(result["batterySaver"]).slice(0, 300));
// Auto quality under load: render scale 2 (DPR 2.625 page), auto on, min 0.5, sustained, with thermals.
await run(`0.05:display:display:2:on:off:0.5`);
const samples: unknown[] = [];
const began = performance.now();
for (let s = 0; (performance.now() - began) / 1000 < STRESS_S; s++) {
  const w = await window(5);
  const d = (await facts())["display"];
  const row = { t: Math.round((performance.now() - began) / 1000), fps: w.fps, interval: w.interval.median, p95: w.interval.p95, work: w.work.median, scale: d?.scale, step: d?.step, displayMs: d?.displayMs, thermal: thermal() };
  samples.push(row);
  console.log("auto", JSON.stringify(row));
}
result["autoQuality"] = samples;
result["memory"] = await page.eval("__runtime_probe__.memory()");
result["chromeMeminfo"] = sh("dumpsys meminfo com.android.chrome | grep -E 'TOTAL PSS|TOTAL RSS|Graphics|GL mtrack|EGL mtrack|Native Heap' | head -8");
result["gpuMem"] = sh("dumpsys gpu --gpumem 2>/dev/null | head -20");
result["thermalEnd"] = thermal();
result["errors"] = log.filter((l) => /error|exception/i.test(l)).slice(0, 30);
writeFileSync(`${out}/g14.json`, JSON.stringify(result, null, 1));
console.log("memory", JSON.stringify(result["memory"]).slice(0, 400));
console.log("thermal", JSON.stringify(result["thermalStart"]), "->", JSON.stringify(result["thermalEnd"]));
await browser.close();
server.stop();
