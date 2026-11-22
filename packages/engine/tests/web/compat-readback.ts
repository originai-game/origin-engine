// The browser's WebGPU compatibility mode, checked without presenting: the
// final pass's first web-compat leg (docs/ci.md). Linux Chrome's opengles
// adapter crashes its GPU process at a canvas's first present, so this page
// never configures one: it asks for a compatibility adapter, draws a
// triangle into a texture, copies it to a buffer and reads it back, and
// checks the pixels and the device's errors.
//
//   bun tests/web/compat-readback.ts --chrome <path> [--runs n]
//
// Prints one JSON line a run and exits non-zero when a run finds a
// validation error, a lost device, a core adapter or device (not
// compatibility mode) or wrong pixels.
import { Browser } from "./cdp.ts";

declare const Bun: { serve(o: { port: number; hostname: string; fetch(r: Request): Response }): { port: number; stop(): void } };

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const chrome = opt("--chrome");
if (!chrome) throw new Error("compat-readback.ts: --chrome <path> is required");
const runs = Number(opt("--runs") ?? 1);

const page = `<!doctype html><script type="module">
const out = { errors: [], lost: null };
try {
  const a = await navigator.gpu.requestAdapter({ featureLevel: "compatibility" });
  if (!a) throw new Error("no adapter");
  out.adapter = { vendor: a.info.vendor, architecture: a.info.architecture, description: a.info.description, core: a.features.has("core-features-and-limits") };
  const d = await a.requestDevice();
  out.device = { core: d.features.has("core-features-and-limits"), maxTextureDimension2D: d.limits.maxTextureDimension2D, maxColorAttachments: d.limits.maxColorAttachments, maxStorageBuffersInVertexStage: d.limits.maxStorageBuffersInVertexStage };
  d.lost.then((i) => { out.lost = i.reason + ": " + i.message; });
  d.onuncapturederror = (e) => out.errors.push(e.error.message);
  const m = d.createShaderModule({ code: \`
    @vertex fn v(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
      let p = array(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
      return vec4f(p[i], 0.0, 1.0);
    }
    @fragment fn f(@builtin(position) p: vec4f) -> @location(0) vec4f {
      return vec4f(select(0.0, 1.0, p.x < 32.0), select(0.0, 1.0, p.x >= 32.0), 0.25, 1.0);
    }\` });
  d.pushErrorScope("validation");
  const pipeline = await d.createRenderPipelineAsync({ layout: "auto", vertex: { module: m }, fragment: { module: m, targets: [{ format: "rgba8unorm" }] } });
  const W = 64, H = 64;
  const tex = d.createTexture({ size: [W, H], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
  const buf = d.createBuffer({ size: W * H * 4, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
  const e = d.createCommandEncoder();
  const pass = e.beginRenderPass({ colorAttachments: [{ view: tex.createView(), loadOp: "clear", storeOp: "store", clearValue: [0, 0, 0, 1] }] });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  e.copyTextureToBuffer({ texture: tex }, { buffer: buf, bytesPerRow: W * 4 }, [W, H]);
  d.queue.submit([e.finish()]);
  const scoped = await d.popErrorScope();
  if (scoped) out.errors.push(scoped.message);
  await buf.mapAsync(GPUMapMode.READ);
  const px = new Uint8Array(buf.getMappedRange());
  const at = (x, y) => Array.from(px.subarray((y * W + x) * 4, (y * W + x) * 4 + 4));
  out.pixels = { left: at(8, 32), right: at(56, 32) };
  const near = (a, b) => a.every((v, i) => Math.abs(v - b[i]) <= 2);
  out.pixelsOk = near(out.pixels.left, [255, 0, 64, 255]) && near(out.pixels.right, [0, 255, 64, 255]);
  buf.unmap();
  await d.queue.onSubmittedWorkDone();
} catch (x) {
  out.error = String(x && x.message || x);
}
window.result = JSON.stringify(out);
</script>`;

const server = Bun.serve({ port: 0, hostname: "127.0.0.1", fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
let failed = false;
for (let i = 0; i < runs; i++) {
  const browser = await Browser.launch({ chrome });
  try {
    const tab = await browser.open(`http://127.0.0.1:${server.port}/`);
    await tab.until("globalThis.result", 60_000);
    const r = JSON.parse(await tab.eval<string>("result")) as { error?: string; errors: string[]; lost: string | null; adapter?: { core: boolean }; device?: { core: boolean }; pixelsOk?: boolean };
    const findings = [
      r.error && `error: ${r.error}`,
      r.errors.length > 0 && `${r.errors.length} validation error(s)`,
      r.lost && `device lost: ${r.lost}`,
      r.adapter?.core !== false && "not a compatibility adapter (core-features-and-limits)",
      r.device?.core !== false && "not a compatibility device (core-features-and-limits)",
      r.pixelsOk !== true && "pixels read back wrong",
    ].filter(Boolean);
    if (findings.length) failed = true;
    console.log(JSON.stringify({ run: i, ok: findings.length === 0, findings, ...r }));
  } catch (e) {
    failed = true;
    console.log(JSON.stringify({ run: i, ok: false, findings: [`harness: ${e instanceof Error ? e.message : String(e)}`] }));
  } finally {
    await browser.close();
  }
}
server.stop();
process.exit(failed ? 1 : 0);
