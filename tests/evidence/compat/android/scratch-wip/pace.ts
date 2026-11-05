// Scratch: main-thread rAF rate against the engine's frame rate during play, on an export.
import { resolve } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";
const server = serve({ dist: resolve(process.argv[2]!), isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, [], { touch: true });
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
const raf = `new Promise((r) => { const t = []; const f = (x) => { t.push(x); if (x - t[0] < 3000) requestAnimationFrame(f); else r(+((t.length - 1) / ((t.at(-1) - t[0]) / 1000)).toFixed(1)); }; requestAnimationFrame(f); })`;
const both = async (label: string) => {
  await page.eval("__og_timings__(true)"); const t0 = performance.now();
  const r = await page.eval<number>(raf);
  const t = await page.eval<{ frames: number; intervals: number[] }>("__og_timings__()");
  const iv = [...t.intervals].sort((a, b) => a - b);
  console.log(label, JSON.stringify({ pageRaf: r, engineFps: +(t.frames / ((performance.now() - t0) / 1000)).toFixed(1), engineMedianMs: +(iv[iv.length >> 1] ?? 0).toFixed(2), display: (await page.eval<any>("__runtime_probe__.state()")).facts?.display?.displayHz }));
};
await both("gate");
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
if (box) { await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [box] }); await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] }); }
for (const s of [3, 15, 40]) { await Bun.sleep(s === 3 ? 3000 : s === 15 ? 9000 : 22000); await both(`play+${s}s`); }
await browser.close(); server.stop(); process.exit(0);
