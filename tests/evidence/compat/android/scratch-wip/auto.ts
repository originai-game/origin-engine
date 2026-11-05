// Scratch: fx-showcase on the phone with the player's default display settings (auto quality on):
// each second from the tap, ticks, frames, the measured display rate and the scale auto quality drew with.
import { resolve } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const dist = resolve("C:/Users/win/src/origin-engine-android/packages/engine/dist/test-reference/fx-showcase");
const seconds = Number(process.argv[2] ?? 40);
const station = process.argv[3];
const server = serve({ dist, isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const log: string[] = [];
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log, { touch: true });
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
if (box) {
  await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box.x, y: box.y }] });
  await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
}
const t0 = performance.now();
await page.until(`${mark("og:start")} > 0`, 15_000);
if (station) await page.eval(`__runtime_probe__.commands(${JSON.stringify(`0.05:station:${station}`)})`);
await page.eval("__og_timings__(true)");
for (let s = 1; s <= seconds; s++) {
  await Bun.sleep(1000);
  const tick = await page.eval<number>("__runtime_probe__.tick()");
  const t = await page.eval<{ frames: number; intervals: number[] }>("__og_timings__(true)");
  const f = (await page.eval<any>("__runtime_probe__.state()")).facts ?? {};
  const d = f.display ?? {};
  const iv = [...(t.intervals ?? [])].sort((a, b) => a - b);
  console.log(JSON.stringify({ t: Math.round((performance.now() - t0) / 1000), tick, frames: t.frames, interval: +(iv[iv.length >> 1] ?? 0).toFixed(1), displayHz: +(d.displayHz ?? 0).toFixed(1), drawnScale: d.renderScale, steps: d.presetSteps, scene: `${d.sceneWidth}x${d.sceneHeight}`, frameMs: +(d.frameMs ?? 0).toFixed(1), station: f.station, pipelines: JSON.stringify((await page.eval<any>("__runtime_probe__.boot"))?.pipelines ?? null), vb: f.capabilities?.visibilityBuffer }));
}
console.log("--- log", log.length); console.log(log.map((l) => l.slice(0, 400)).join("\n"));
await browser.close();
server.stop();
process.exit(0);
