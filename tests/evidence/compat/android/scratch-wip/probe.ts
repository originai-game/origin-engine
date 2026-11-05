// Scratch: open fx-showcase's test-reference export on the phone and dump the console and probe state.
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const dist = (await import("node:path")).resolve("C:/Users/win/src/origin-engine-android/packages/engine/dist/test-reference/fx-showcase");
const server = serve({ dist, isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const log: string[] = [];
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log);
await Bun.sleep(8_000);
const box = await page.eval<{ x: number; y: number } | null>(`(() => { const b = document.querySelector("#og-gate button"); if (!b || document.getElementById("og-gate").hidden) return null; const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
if (box) { await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box.x, y: box.y }] }); await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] }); }
await Bun.sleep(30_000);
console.log("state", JSON.stringify(await page.eval(`({ probe: typeof __runtime_probe__, marks: performance.getEntriesByType("mark").map(m => m.name + "@" + Math.round(m.startTime)), gate: !!document.querySelector("#og-gate"), body: document.body.innerText.slice(0, 300) })`)));
try { const f = (await page.eval<any>(`__runtime_probe__.state()`)).facts ?? {}; console.log("caps", JSON.stringify({ vb: f.capabilities?.visibilityBuffer, refused: f.capabilities?.visibilityBufferRefused, tier: f.capabilities?.tier })); } catch (e) { console.log("no state", String(e).slice(0, 300)); }
console.log(log.join("\n").slice(0, 6000));
await browser.close?.();
server.stop?.();
process.exit(0);
