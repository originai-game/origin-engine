// A remote url() image's EXIF orientation on the web host (docs/ui.md §3.3
// Remote images): the engine Worker's fetcher (src/web/boot.ts
// installRemoteFetcher) must hand the engine the texels as the file stores
// them and the orientation its metadata names, as the native host does
// (crates/og-runtime/tests/remote_images.rs, the EXIF-6 JPEG), so CSS
// image-orientation turns it the same on both hosts. createImageBitmap
// always turns a JPEG upright, so the fetcher turns it back.
//
// In headless Chrome, a page on 127.0.0.1 loads dist/web's glue (whose
// og_web_image_orientation is og-runtime's own reader) and the bundled
// fetcher, and fetches from an art server on localhost (another origin,
// CORS `*`) a 16 × 8 JPEG, red on its stored left half and blue on its
// right, tagged EXIF 6 (stored turned a quarter; upright it is 8 × 16). It
// must arrive as 16 × 8 with orientation 6 and red left, blue right. A
// fetcher that hands over createImageBitmap's pixels gives 8 × 16 at
// orientation 1 (or 6, turned twice); one that turns the wrong way puts
// blue on the left.
//
//   bun tests/web/remote-images.ts [--chrome <path>]
//
// Needs Chrome (cdp.ts) and dist/web (`bun run build`). Exits 1 on a failure.

import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  serve(o: { port: number; hostname: string; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
};

const args = process.argv.slice(2);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const dist = join(pkg, "dist", "web");
const glue = join(dist, "origin-engine.js");
if (!existsSync(glue) || !readFileSync(glue, "utf8").includes("og_web_image_orientation")) {
  throw new Error(`${glue} has no og_web_image_orientation: build the web host (\`bun run build\` in packages/engine)`);
}
const built = await Bun.build({ entrypoints: [join(pkg, "src", "web", "boot.ts")], target: "browser", format: "esm" });
if (!built.success) throw new Error(`bundling boot.ts: ${built.logs.map(String).join("\n")}`);
// Inlined in the page's module (a module script must not be served as HTML).
const boot = (await built.outputs[0]!.text()).replaceAll("</script", "<\\/script");

// The art server: what the page uploads, served back to any origin.
const art = new Map<string, Uint8Array>();
const artServer = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const path = new URL(req.url).pathname;
    const cors = { "access-control-allow-origin": "*", "access-control-allow-methods": "GET, PUT", "access-control-allow-headers": "content-type" };
    if (req.method === "OPTIONS") return new Response(null, { status: 204, headers: cors });
    if (req.method === "PUT") {
      art.set(path, new Uint8Array(await req.arrayBuffer()));
      return new Response("ok", { headers: cors });
    }
    const bytes = art.get(path);
    return bytes ? new Response(new Blob([new Uint8Array(bytes)]), { headers: { ...cors, "content-type": "image/jpeg" } }) : new Response("not found", { status: 404, headers: cors });
  },
});

const page = `<!doctype html><html><head><meta charset="utf-8"><title>remote images</title></head><body>
<script type="module">
import init, * as glue from "/game/origin-engine.js";
${boot}
try {
  await init();
  globalThis.__glue = glue;
  globalThis.__install = installRemoteFetcher;
  globalThis.__ready = true;
} catch (e) { globalThis.__error = String(e && e.stack || e); }
</script></body></html>`;
const server = serve({ dist, isolate: false, extra: { "/remote.html": page } });
const browser = await Browser.launch({ chrome: chromePath(args), width: 640, height: 480 });
const log: string[] = [];
let failures = 0;
const check = (ok: boolean, what: string) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures++;
};
try {
  const tab = await browser.open(`http://127.0.0.1:${server.port}/remote.html`, log);
  await tab.until("globalThis.__ready || globalThis.__error", 60_000);
  const failed = await tab.eval<string | undefined>("globalThis.__error");
  if (failed) throw new Error(`the page did not load: ${failed}`);
  type Answer = { width: number; height: number; orientation: number; left: number[]; right: number[]; glueReads: number } | { error: string };
  const answer = await tab.eval<Answer>(`(async () => {
    // The stored image: 16 x 8, red left, blue right, as a JPEG with an
    // APP1 Exif segment naming orientation 6 (the native test's with_exif).
    const canvas = new OffscreenCanvas(16, 8);
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "#ff0000"; ctx.fillRect(0, 0, 8, 8);
    ctx.fillStyle = "#0000ff"; ctx.fillRect(8, 0, 8, 8);
    const jpeg = new Uint8Array(await (await canvas.convertToBlob({ type: "image/jpeg", quality: 1 })).arrayBuffer());
    const app1 = [0xff, 0xe1, 0x00, 0x22, ...new TextEncoder().encode("Exif"), 0, 0,
      0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00,
      0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 6, 0, 0, 0, 0, 0, 0, 0];
    const tagged = new Uint8Array([...jpeg.slice(0, 2), ...app1, ...jpeg.slice(2)]);
    const url = "http://localhost:${artServer.port}/turned.jpg";
    await fetch(url, { method: "PUT", body: tagged });
    const glueReads = globalThis.__glue.og_web_image_orientation(tagged);
    return await new Promise((done) => {
      const installed = globalThis.__install({
        og_web_set_remote_fetcher: (f) => f(1, url, false, "", ""),
        og_web_image_orientation: (b) => globalThis.__glue.og_web_image_orientation(b),
        og_web_put_image: (key, width, height, rgba, orientation) => {
          const px = (x, y) => Array.from(rgba.slice((y * width + x) * 4, (y * width + x) * 4 + 4));
          done({ width, height, orientation, left: px(2, 4), right: px(13, 4), glueReads });
        },
        og_web_fail_image: (key, reason) => done({ error: reason }),
      }, () => Promise.reject(new Error("no SVG here")));
      if (!installed) done({ error: "installRemoteFetcher found no createImageBitmap or OffscreenCanvas" });
    });
  })()`);
  if ("error" in answer) throw new Error(`the fetch failed: ${answer.error}`);
  console.log(JSON.stringify(answer));
  check(answer.glueReads === 6, `the wasm's reader finds EXIF 6 in the fetched bytes (got ${answer.glueReads})`);
  check(answer.width === 16 && answer.height === 8, `handed over as stored, 16 x 8 (got ${answer.width} x ${answer.height})`);
  check(answer.orientation === 6, `with its orientation, 6 (got ${answer.orientation})`);
  const red = (p: number[]) => p[0]! > 200 && p[2]! < 60;
  const blue = (p: number[]) => p[2]! > 200 && p[0]! < 60;
  check(red(answer.left) && blue(answer.right), `red on the stored left, blue on the right (got ${answer.left} and ${answer.right})`);
} catch (e) {
  failures++;
  console.error(e instanceof Error ? e.stack : String(e));
  if (log.length > 0) console.error(log.join("\n"));
} finally {
  await browser.close();
  server.stop();
  artServer.stop();
}
process.exit(failures === 0 ? 0 : 1);
