// Probe: does Safari draw the same picture from drawIndexedIndirect (args
// written by a compute pass in the same command buffer, as cull_gpu.rs does)
// as from drawIndexed? A 4x4 grid of instanced quads, the instance offsets in
// a vertex buffer slice that starts past a first region (cull_gpu.rs
// `region`), read back and compared pixel by pixel. No engine code.
//
//   sh tests/evidence/compat/safari/run.sh probe tests/evidence/compat/safari/probes/indirect-draw.ts
import { Safari } from "../../../../../packages/engine/tests/web/webdriver.ts";
declare const Bun: any;
const page = `<!doctype html><html><body><script>
window.probe = async (instances, skip) => {
  const a = await navigator.gpu.requestAdapter(); const d = await a.requestDevice();
  const errs = []; d.onuncapturederror = (e) => errs.push(e.error.message);
  const W = 128;
  const code = \`
struct O { @builtin(position) p: vec4<f32>, @location(0) c: vec3<f32> };
@vertex fn vs(@location(0) v: vec2<f32>, @location(1) off: vec4<f32>) -> O {
  var o: O; o.p = vec4<f32>(v * 0.2 + off.xy, 0.5, 1.0); o.c = vec3<f32>(off.z, off.w, 1.0); return o; }
@fragment fn fs(i: O) -> @location(0) vec4<f32> { return vec4<f32>(i.c, 1.0); }
@group(0) @binding(0) var<storage, read_write> args: array<atomic<u32>>;
@compute @workgroup_size(1) fn write() {
  atomicStore(&args[0], 6u); atomicStore(&args[1], \${instances}u);
  atomicStore(&args[2], 0u); atomicStore(&args[3], 0u); atomicStore(&args[4], 0u); }\`;
  const m = d.createShaderModule({ code });
  for (const x of (await m.getCompilationInfo()).messages) errs.push(x.type + ": " + x.message);
  const verts = d.createBuffer({ size: 32, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST });
  d.queue.writeBuffer(verts, 0, new Float32Array([-1, -1, 1, -1, 1, 1, -1, 1]));
  const idx = d.createBuffer({ size: 24, usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST });
  d.queue.writeBuffer(idx, 0, new Uint32Array([0, 1, 2, 0, 2, 3]));
  // A first region of junk (huge offsets), then the real instances, as the culled regions sit.
  const inst = new Float32Array((skip + instances) * 4);
  for (let i = 0; i < skip; i++) inst.set([1e6, -1e6, 1, 0], i * 4);
  for (let i = 0; i < instances; i++) inst.set([-0.75 + (i % 4) * 0.5, -0.75 + Math.floor(i / 4) * 0.5, (i % 4) / 3, Math.floor(i / 4) / 3], (skip + i) * 4);
  const ib = d.createBuffer({ size: inst.byteLength, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
  d.queue.writeBuffer(ib, 0, inst);
  const args = d.createBuffer({ size: 64, usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.STORAGE });
  const draw = d.createRenderPipeline({ layout: "auto",
    vertex: { module: m, entryPoint: "vs", buffers: [
      { arrayStride: 8, attributes: [{ shaderLocation: 0, offset: 0, format: "float32x2" }] },
      { arrayStride: 16, stepMode: "instance", attributes: [{ shaderLocation: 1, offset: 0, format: "float32x4" }] }] },
    fragment: { module: m, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] } });
  const write = d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: "write" } });
  const wg = d.createBindGroup({ layout: write.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: args } }] });
  const shot = async (indirect) => {
    const t = d.createTexture({ size: [W, W], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
    const rb = d.createBuffer({ size: W * W * 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
    const e = d.createCommandEncoder();
    const c = e.beginComputePass(); c.setPipeline(write); c.setBindGroup(0, wg); c.dispatchWorkgroups(1); c.end();
    const r = e.beginRenderPass({ colorAttachments: [{ view: t.createView(), loadOp: "clear", storeOp: "store", clearValue: [0, 0, 0, 1] }] });
    r.setPipeline(draw); r.setVertexBuffer(0, verts); r.setVertexBuffer(1, ib, skip * 16, instances * 16); r.setIndexBuffer(idx, "uint32");
    if (indirect) r.drawIndexedIndirect(args, 0); else r.drawIndexed(6, instances);
    r.end();
    e.copyTextureToBuffer({ texture: t }, { buffer: rb, bytesPerRow: W * 4 }, [W, W]);
    d.queue.submit([e.finish()]);
    await rb.mapAsync(GPUMapMode.READ); const px = new Uint8Array(rb.getMappedRange().slice(0)); rb.unmap(); return px;
  };
  const t0 = performance.now(); const direct = await shot(false); const t1 = performance.now(); const indirect = await shot(true); const t2 = performance.now();
  let diff = 0, lit = 0; for (let i = 0; i < direct.length; i += 4) { if (direct[i + 2]) lit++; if (direct[i] !== indirect[i] || direct[i + 1] !== indirect[i + 1] || direct[i + 2] !== indirect[i + 2]) diff++; }
  d.destroy();
  return { instances, skip, litPixels: lit, differingPixels: diff, directMs: Math.round(t1 - t0), indirectMs: Math.round(t2 - t1), errs };
};
</script></body></html>`;
const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const b = await Safari.launch({ width: 640, height: 400 });
let failed = false;
try {
  const p = await b.open(`http://127.0.0.1:${server.port}/`);
  await p.until("typeof probe === 'function' ? 1 : 0");
  for (const [n, skip] of [[16, 0], [16, 20000], [16, 40000]]) {
    const r = await p.eval<{ differingPixels: number; errs: string[] }>(`probe(${n}, ${skip})`);
    console.log(JSON.stringify(r));
    failed ||= r.differingPixels > 0 || r.errs.length > 0;
  }
} finally {
  await b.close();
  server.stop();
}
process.exit(failed ? 1 : 0);
