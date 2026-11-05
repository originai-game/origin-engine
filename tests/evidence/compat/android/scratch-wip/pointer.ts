// Scratch: a phone tab's pointer/hover media as the runners open it, and with `touch`.
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";

const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>p</title><script>window.early = matchMedia('(pointer: coarse)').matches</script>", { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const q = `({ early, coarse: matchMedia("(pointer: coarse)").matches, fine: matchMedia("(pointer: fine)").matches, anyCoarse: matchMedia("(any-pointer: coarse)").matches, hover: matchMedia("(hover: hover)").matches, mixed: matchMedia("(pointer: coarse) and (min-width: 1px)").matches, width: matchMedia("(min-width: 300px)").matches })`;
for (const touch of [false, true]) {
  const page = await browser.open(`http://127.0.0.1:${server.port}/`, [], { touch });
  await page.until("document.readyState === 'complete'", 15000);
  console.log(touch ? "touch" : "mouse", JSON.stringify(await page.eval(q)));
}
await browser.close();
server.stop();
process.exit(0);
