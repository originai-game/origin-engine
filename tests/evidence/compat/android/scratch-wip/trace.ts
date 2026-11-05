// Scratch: a Chrome trace on the phone across fx-showcase's tap and the stall after it; prints the longest slices.
import { resolve } from "node:path";
import { writeFileSync } from "node:fs";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const dist = resolve("C:/Users/win/src/origin-engine-android/packages/engine/dist/test-reference/fx-showcase");
const server = serve({ dist, isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const log: string[] = [];
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log, { touch: true });
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
const chunks: string[] = [];
let done: (h: string) => void;
const complete = new Promise<string>((r) => (done = r));
browser.on((method, params) => {
  if (method === "Tracing.tracingComplete") done(params["stream"] as string);
});
await browser.send("Tracing.start", { transferMode: "ReturnAsStream", traceConfig: { includedCategories: ["gpu", "dawn", "disabled-by-default-gpu.dawn", "gpu.service", "gpu.angle", "viz", "toplevel", "v8.execute", "devtools.timeline", "disabled-by-default-devtools.timeline.frame"] } });
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
if (box) {
  await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box.x, y: box.y }] });
  await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
}
const ticks: string[] = [];
for (let s = 0; s < 18; s++) {
  await Bun.sleep(1000);
  ticks.push(`${s + 1}s tick ${await page.eval<number>("__runtime_probe__.tick()")}`);
}
await browser.send("Tracing.end");
const stream = await complete;
for (;;) {
  const r = await browser.send<{ data: string; eof: boolean; base64Encoded?: boolean }>("IO.read", { handle: stream, size: 1 << 20 });
  chunks.push(r.base64Encoded ? Buffer.from(r.data, "base64").toString() : r.data);
  if (r.eof) break;
}
const raw = chunks.join("");
writeFileSync(process.argv[2]!, raw);
const events = (JSON.parse(raw).traceEvents ?? JSON.parse(raw)) as { name: string; cat: string; ph: string; dur?: number; ts: number; pid: number; tid: number; args?: unknown }[];
const names = new Map<number, string>();
for (const e of events) if (e.ph === "M" && e.name === "process_name") names.set(e.pid, (e.args as { name: string }).name);
const t0 = Math.min(...events.filter((e) => e.ts > 0).map((e) => e.ts));
const long = events.filter((e) => e.ph === "X" && (e.dur ?? 0) > 300_000).sort((a, b) => b.dur! - a.dur!).slice(0, 25);
console.log(ticks.join(" | "));
for (const e of long) console.log(`${((e.ts - t0) / 1e6).toFixed(2)}s +${(e.dur! / 1e6).toFixed(2)}s ${names.get(e.pid) ?? e.pid}/${e.tid} [${e.cat}] ${e.name} ${JSON.stringify(e.args ?? {}).slice(0, 160)}`);
await browser.close();
server.stop();
process.exit(0);
