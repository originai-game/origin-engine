// Scratch: how long a compositor frame (CDP Page.captureScreenshot) takes at points after og:first-frame, behind the gate.
//   bun shots.ts <dist> <second,...>
import { resolve } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const server = serve({ dist: resolve(process.argv[2]!), isolate: true });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, [], { touch: true });
const mark = (n: string) => `(performance.getEntriesByName(${JSON.stringify(n)})[0]?.startTime ?? 0)`;
await page.until(`typeof __runtime_probe__ === "object" && ${mark("og:first-frame")} > 0`, 90_000);
const ff = performance.now();
const out: string[] = [];
for (const at of process.argv[3]!.split(",").map(Number)) {
  const wait = at * 1000 - (performance.now() - ff);
  if (wait > 0) await Bun.sleep(wait);
  const t0 = performance.now();
  await page.send("Page.captureScreenshot", { format: "jpeg", quality: 30 });
  out.push(`ff+${((t0 - ff) / 1000).toFixed(1)}s: ${Math.round(performance.now() - t0)} ms`);
}
console.log(out.join(" | "));
await browser.close();
server.stop();
process.exit(0);
