// Scratch: compile the changed post modules (assembled as post.rs does) in Chrome on the phone; print each module's errors.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";

const dir = "C:/Users/win/src/origin-engine-android/crates/og-render/shaders";
const w = (...f: string[]) => readFileSync(join(dir, "fullscreen.wgsl"), "utf8") + f.map((x) => readFileSync(join(dir, x), "utf8")).join("");
const modules: Record<string, string> = {
  post: w("hdr_lpm.wgsl", "post.wgsl", "bloom_chain.wgsl"),
  taa: w("taa_uniform.wgsl", "taa.wgsl"),
  fxaa: w("fxaa.wgsl"),
  stylise: w("stylise.wgsl"),
  "fx post": w("fxpost.wgsl", "ssgi.wgsl", "fxpost_gather.wgsl"),
  "hdr transfer": w("hdr_transfer.wgsl"),
};
const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>wgsl</title>", { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/`, []);
  await page.until("document.readyState === 'complete'", 15000);
  const out = await page.eval<Record<string, string>>(`(async () => {
    const d = await (await navigator.gpu.requestAdapter()).requestDevice();
    const r = {};
    for (const [k, code] of Object.entries(${JSON.stringify(modules)})) {
      const m = d.createShaderModule({ code });
      const e = (await m.getCompilationInfo()).messages.filter((x) => x.type === "error");
      r[k] = e.length ? e.slice(0, 3).map((x) => x.lineNum + ": " + x.message).join(" | ") : "ok";
    }
    return r;
  })()`);
  console.log(JSON.stringify(out, null, 1));
} finally {
  await browser.close();
  server.stop();
}
