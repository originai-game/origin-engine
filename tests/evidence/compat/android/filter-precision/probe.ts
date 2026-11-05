// How finely the GPU blends a bilinear sample, outside the engine.
//
//   bun tests/evidence/compat/android/filter-precision/probe.ts [--android [serial] | --swiftshader]
//
// A 2x1 texture, texel 0 black and texel 1 white, sampled with a linear
// filter at u = (0.5 + f) / 2 for f from 0 to 1: the exact answer is f. A
// compute pass writes each sample to a buffer, read back as f32. Printed
// per format (rgba8unorm, rgba16float): the distinct values returned (the
// filter's weight steps), the largest error against f, and the answers at
// the texel centre itself and a hair either side of it, where the engine's
// full-screen passes sample. Without --android the page runs in this
// machine's Chrome (cdp.ts: --chrome or $CHROME), the control.
import { Browser, chromePath } from "../../../../../packages/engine/tests/web/cdp.ts";

const args = process.argv.slice(2);
const android = args.includes("--android");
const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>filter</title>", { headers: { "content-type": "text/html" } }) });
// --swiftshader: desktop Chrome on SwiftShader, its CPU WebGPU adapter.
const flags = args.includes("--swiftshader") ? ["--use-webgpu-adapter=swiftshader", "--enable-unsafe-swiftshader"] : [];
const browser = await Browser.from(args, { chrome: android ? "" : chromePath(args), flags, reverse: [server.port] });
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/`, []);
  await page.until("document.readyState === 'complete'", 15000);
  const out = await page.eval<Record<string, unknown>>(`(async () => {
    const a = await navigator.gpu.requestAdapter();
    const d = await a.requestDevice();
    const N = 1024;
    // f = i / N for i < N, then the texel centre offset by -1e-4, -1e-5, 0, 1e-5, 1e-4 texels.
    const fs = [...Array.from({ length: N + 1 }, (_, i) => i / N), -1e-4, -1e-5, 0, 1e-5, 1e-4];
    const result = { adapter: a.info.vendor + " " + a.info.architecture };
    for (const format of ["rgba8unorm", "rgba16float"]) {
      const tex = d.createTexture({ size: [2, 1], format, usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST });
      const texel = format === "rgba8unorm" ? new Uint8Array([0, 0, 0, 255, 255, 255, 255, 255]) : new Uint16Array([0, 0, 0, 0x3c00, 0x3c00, 0x3c00, 0x3c00, 0x3c00]);
      d.queue.writeTexture({ texture: tex }, texel, { bytesPerRow: 256 }, [2, 1]);
      const coords = d.createBuffer({ size: fs.length * 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
      d.queue.writeBuffer(coords, 0, new Float32Array(fs.map((f) => (0.5 + f) / 2)));
      const values = d.createBuffer({ size: fs.length * 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC });
      const read = d.createBuffer({ size: fs.length * 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
      const m = d.createShaderModule({ code: \`
        @group(0) @binding(0) var t: texture_2d<f32>;
        @group(0) @binding(1) var s: sampler;
        @group(0) @binding(2) var<storage, read> u: array<f32>;
        @group(0) @binding(3) var<storage, read_write> v: array<f32>;
        @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3u) {
          if (id.x >= arrayLength(&u)) { return; }
          v[id.x] = textureSampleLevel(t, s, vec2f(u[id.x], 0.5), 0.0).r;
        }\` });
      const p = d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: "main" } });
      const bg = d.createBindGroup({ layout: p.getBindGroupLayout(0), entries: [
        { binding: 0, resource: tex.createView() },
        { binding: 1, resource: d.createSampler({ magFilter: "linear", minFilter: "linear" }) },
        { binding: 2, resource: { buffer: coords } },
        { binding: 3, resource: { buffer: values } },
      ] });
      const e = d.createCommandEncoder();
      const pass = e.beginComputePass();
      pass.setPipeline(p); pass.setBindGroup(0, bg); pass.dispatchWorkgroups(Math.ceil(fs.length / 64)); pass.end();
      e.copyBufferToBuffer(values, 0, read, 0, fs.length * 4);
      d.queue.submit([e.finish()]);
      await read.mapAsync(GPUMapMode.READ);
      const got = Array.from(new Float32Array(read.getMappedRange()));
      const sweep = got.slice(0, N + 1);
      const steps = new Set(sweep.map((x) => x.toFixed(7))).size;
      const worst = Math.max(...sweep.map((x, i) => Math.abs(x - i / N)));
      result[format] = { distinctValues: steps, worstError: +worst.toFixed(6), atCentre: Object.fromEntries(fs.slice(N + 1).map((f, i) => [String(f), got[N + 1 + i]])) };
    }
    // A full-screen pass, 320x200 (the og-ui glass frame), samples a 320x200 stripe texture
    // (4 white texels, 4 black) with a linear filter: uv from the triangle's interpolated output,
    // and uv from @builtin(position) / size. Exact either way: every texel back unchanged.
    {
      const W = 320, H = 200;
      const src = d.createTexture({ size: [W, H], format: "rgba16float", usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST });
      const row = new Uint16Array(W * 4);
      for (let x = 0; x < W; x++) if (x % 8 < 4) row.fill(0x3c00, x * 4, x * 4 + 4); else row.fill(0, x * 4, x * 4 + 3), (row[x * 4 + 3] = 0x3c00);
      const all = new Uint16Array(W * 4 * H); for (let y = 0; y < H; y++) all.set(row, y * W * 4);
      d.queue.writeTexture({ texture: src }, all, { bytesPerRow: W * 8 }, [W, H]);
      const m = d.createShaderModule({ code: \`
        @group(0) @binding(0) var t: texture_2d<f32>;
        @group(0) @binding(1) var s: sampler;
        struct V { @builtin(position) pos: vec4f, @location(0) uv: vec2f };
        @vertex fn vs(@builtin(vertex_index) i: u32) -> V {
          let p = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
          return V(vec4f(p * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0), p);
        }
        @fragment fn interpolated(v: V) -> @location(0) vec4f { return textureSampleLevel(t, s, v.uv, 0.0); }
        @fragment fn from_position(v: V) -> @location(0) vec4f { return textureSampleLevel(t, s, v.pos.xy / vec2f(${"$"}{W}.0, ${"$"}{H}.0), 0.0); }\` });
      const sampler = d.createSampler({ magFilter: "linear", minFilter: "linear" });
      for (const entry of ["interpolated", "from_position"]) {
        const target = d.createTexture({ size: [W, H], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
        const p = d.createRenderPipeline({ layout: "auto", vertex: { module: m, entryPoint: "vs" }, fragment: { module: m, entryPoint: entry, targets: [{ format: "rgba8unorm" }] } });
        const bg = d.createBindGroup({ layout: p.getBindGroupLayout(0), entries: [{ binding: 0, resource: src.createView() }, { binding: 1, resource: sampler }] });
        const buf = d.createBuffer({ size: 1280 * H, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
        const e = d.createCommandEncoder();
        const pass = e.beginRenderPass({ colorAttachments: [{ view: target.createView(), loadOp: "clear", storeOp: "store", clearValue: [0, 0, 0, 1] }] });
        pass.setPipeline(p); pass.setBindGroup(0, bg); pass.draw(3); pass.end();
        e.copyTextureToBuffer({ texture: target }, { buffer: buf, bytesPerRow: 1280 }, [W, H]);
        d.queue.submit([e.finish()]);
        await buf.mapAsync(GPUMapMode.READ);
        const px = new Uint8Array(buf.getMappedRange());
        let wrong = 0, worst = 0; const cols = new Set();
        for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
          const want = x % 8 < 4 ? 255 : 0, got = px[y * 1280 + x * 4];
          if (got !== want) { wrong++; worst = Math.max(worst, Math.abs(got - want)); cols.add(x); }
        }
        result["pass-" + entry] = { wrongPixels: wrong, worst, columns: [...cols].slice(0, 12), row0: Array.from({ length: 16 }, (_, x) => px[x * 4]).join(" ") };
      }
    }
    return result;
  })()`);
  console.log(JSON.stringify(out, null, 1));
} finally {
  await browser.close();
  server.stop();
}
