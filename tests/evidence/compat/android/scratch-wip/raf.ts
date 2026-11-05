// Scratch: requestAnimationFrame's rate in Chrome on the phone, idle and while touch moves arrive.
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";

const server = Bun.serve({ port: 0, fetch: () => new Response(`<!doctype html><title>raf</title><canvas id=c width=400 height=400></canvas><script>
const ctx = c.getContext("2d"); let n = 0;
const loop = () => { ctx.fillStyle = "hsl(" + (n++ % 360) + ",80%,50%)"; ctx.fillRect(0, 0, 400, 400); requestAnimationFrame(loop); }; loop();
window.measure = (ms) => new Promise((r) => { const t = []; const f = (x) => { t.push(x); if (x - t[0] < ms) requestAnimationFrame(f); else { const d = t.slice(1).map((v, i) => v - t[i]).sort((a, b) => a - b); r({ fps: +((t.length - 1) / ((t.at(-1) - t[0]) / 1000)).toFixed(1), median: +d[d.length >> 1].toFixed(2) }); } }; requestAnimationFrame(f); });
</script>`, { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/`, [], { touch: true });
  await page.until("typeof measure === 'function'", 15000);
  await Bun.sleep(3000);
  console.log("idle", JSON.stringify(await page.eval("measure(3000)")));
  let on = true;
  const drag = (async () => {
    await page.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: 100, y: 600 }] });
    for (let i = 0; on; i++) { await page.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: 100 + (i % 50), y: 600 }] }); await Bun.sleep(30); }
    await page.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  })();
  await Bun.sleep(500);
  console.log("touching", JSON.stringify(await page.eval("measure(3000)")));
  on = false; await drag;
  await Bun.sleep(4000);
  console.log("idle again", JSON.stringify(await page.eval("measure(3000)")));
} finally {
  await browser.close();
  server.stop();
}
