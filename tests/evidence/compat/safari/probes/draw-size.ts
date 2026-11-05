// Probe: does Safari draw every vertex of a large instanced indexed draw?
// spheres-20k's torn draws are 1,152 indices x 19,028 and x 16,717
// instances, both past 2^24 vertex invocations. Sweeps instances x indices
// across 2^24, direct (drawIndexed) and indirect (drawIndexedIndirect with
// args written by a compute pass, as cull_gpu.rs), each against the picture
// worked out on the CPU: instance i is a one-pixel quad at cell i mod 128^2,
// coloured by i, so the last instance drawn in a cell is the one seen and a
// draw that stops early or reads the wrong instance shows. The mesh is the
// quad's six indices repeated (indexCount / 6 copies), so the index count
// changes the vertex work and not the picture.
//
// The size sweep found no threshold in Safari 27.2 (draw-size.out.txt); the
// last rows put the scene's draws at its args offsets and instance regions.
//
//   sh tests/evidence/compat/safari/run.sh probe tests/evidence/compat/safari/probes/draw-size.ts
import { Safari } from "../../../../../packages/engine/tests/web/webdriver.ts";
declare const Bun: any;
const page = `<!doctype html><html><body><script>
let d;
window.probe = async (indexCount, instances, stride, argWord = 0, skip = 0, via = "write", work = 0) => {
  d ??= await (await navigator.gpu.requestAdapter()).requestDevice();
  const errs = []; d.onuncapturederror = (e) => errs.push(e.error.message);
  const W = 128;
  const code = \`
struct O { @builtin(position) p: vec4<f32>, @location(0) @interpolate(flat) i: u32 };
@vertex fn vs(@location(0) v: vec2<f32>, @location(1) i: u32) -> O {
  let cell = i % (\${W}u * \${W}u);
  let px = vec2<f32>(f32(cell % \${W}u), f32(cell / \${W}u)) + v;
  var o: O; o.p = vec4<f32>(px.x / \${W}.0 * 2.0 - 1.0, 1.0 - px.y / \${W}.0 * 2.0, 0.5, 1.0); o.i = i; return o; }
@fragment fn fs(o: O) -> @location(0) vec4<f32> {
  return vec4<f32>(f32(o.i & 255u), f32((o.i >> 8u) & 255u), f32((o.i >> 16u) & 255u), 255.0) / 255.0; }
@group(0) @binding(0) var<storage, read_write> args: array<atomic<u32>>;
@compute @workgroup_size(1) fn write() {
  let a = \${argWord}u;
  atomicStore(&args[a], \${indexCount}u); atomicStore(&args[a + 1u], \${instances}u);
  atomicStore(&args[a + 2u], 0u); atomicStore(&args[a + 3u], 0u); atomicStore(&args[a + 4u], 0u); }
@group(0) @binding(0) var<storage, read> src: array<u32>;
@group(0) @binding(1) var<storage, read_write> dst: array<u32>;
@compute @workgroup_size(256) fn copy(@builtin(global_invocation_id) g: vec3<u32>) {
  if (g.x >= arrayLength(&src)) { return; }
  // Busy work (\${work} rounds) so the write lands late, as a big cull's scatter does.
  var v = src[g.x]; var h = v;
  for (var k = 0u; k < \${work}u; k++) { h = h * 1664525u + 1013904223u; }
  dst[g.x] = v + select(0u, 1u, h == 0xdeadbeefu && v == 0x7fffffffu); }\`;
  const m = d.createShaderModule({ code });
  for (const x of (await m.getCompilationInfo()).messages) errs.push(x.type + ": " + x.message);
  const verts = d.createBuffer({ size: 32, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST });
  d.queue.writeBuffer(verts, 0, new Float32Array([0, 0, 1, 0, 1, 1, 0, 1]));
  const ix = new Uint32Array(indexCount); for (let k = 0; k < indexCount; k++) ix[k] = [0, 1, 2, 0, 2, 3][k % 6];
  const idx = d.createBuffer({ size: ix.byteLength, usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST });
  d.queue.writeBuffer(idx, 0, ix);
  // \`skip\` records of junk before the drawn ones (cull_gpu.rs's regions).
  const words = stride / 4, inst = new Uint32Array((skip + instances) * words);
  for (let i = 0; i < skip; i++) inst[i * words] = 0xffffff;
  for (let i = 0; i < instances; i++) inst[(skip + i) * words] = i;
  // via "write": queue.writeBuffer; "compute": a compute pass copies the
  // records in, in the same encoder before the draw (as the cull's scatter
  // writes cull_gpu.rs's instances), cleared to junk first.
  const ib = d.createBuffer({ size: inst.byteLength, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
  let src, cg, cp;
  if (via !== "write") {
    src = d.createBuffer({ size: inst.byteLength, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
    d.queue.writeBuffer(src, 0, inst);
    cp = d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: "copy" } });
    cg = d.createBindGroup({ layout: cp.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: src } }, { binding: 1, resource: { buffer: ib } }] });
  } else d.queue.writeBuffer(ib, 0, inst);
  const args = d.createBuffer({ size: Math.max(32, (argWord + 5) * 4), usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.STORAGE });
  const draw = d.createRenderPipeline({ layout: "auto",
    vertex: { module: m, entryPoint: "vs", buffers: [
      { arrayStride: 8, attributes: [{ shaderLocation: 0, offset: 0, format: "float32x2" }] },
      { arrayStride: stride, stepMode: "instance", attributes: [{ shaderLocation: 1, offset: 0, format: "uint32" }] }] },
    fragment: { module: m, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] } });
  const write = d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: "write" } });
  const wg = d.createBindGroup({ layout: write.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: args } }] });
  const shot = async (indirect) => {
    const t = d.createTexture({ size: [W, W], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
    const rb = d.createBuffer({ size: W * W * 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
    const e = d.createCommandEncoder();
    // via "compute2": the compute in its own command buffer, submitted with the draw's (cull_gpu.rs).
    if (via !== "write") d.queue.writeBuffer(ib, 0, new Uint32Array(inst.length).fill(0x00ffffff));
    const ce = via === "compute2" ? d.createCommandEncoder() : e;
    const c = ce.beginComputePass(); c.setPipeline(write); c.setBindGroup(0, wg); c.dispatchWorkgroups(1);
    if (via !== "write") { c.setPipeline(cp); c.setBindGroup(0, cg); c.dispatchWorkgroups(Math.ceil(inst.length / 256)); }
    c.end();
    const r = e.beginRenderPass({ colorAttachments: [{ view: t.createView(), loadOp: "clear", storeOp: "store", clearValue: [0, 0, 0, 0] }] });
    r.setPipeline(draw); r.setVertexBuffer(0, verts); r.setVertexBuffer(1, ib, skip * stride, instances * stride); r.setIndexBuffer(idx, "uint32");
    if (indirect) r.drawIndexedIndirect(args, argWord * 4); else r.drawIndexed(indexCount, instances);
    r.end();
    e.copyTextureToBuffer({ texture: t }, { buffer: rb, bytesPerRow: W * 4 }, [W, W]);
    const t0 = performance.now();
    d.queue.submit(ce === e ? [e.finish()] : [ce.finish(), e.finish()]);
    await rb.mapAsync(GPUMapMode.READ); const px = new Uint32Array(rb.getMappedRange().slice(0)); rb.unmap();
    t.destroy(); rb.destroy();
    // The expected picture: the last instance in each cell, or clear.
    let wrong = 0, firstWrong = -1, seenMax = -1;
    for (let cell = 0; cell < W * W; cell++) {
      const last = cell < instances ? cell + Math.floor((instances - 1 - cell) / (W * W)) * W * W : -1;
      const want = last < 0 ? 0 : (0xff000000 | last) >>> 0;
      const got = px[cell] >>> 0;
      if (got >>> 24) seenMax = Math.max(seenMax, got & 0xffffff);
      if (got !== want) { wrong++; if (firstWrong < 0) firstWrong = cell; }
    }
    return { wrongCells: wrong, firstWrong, lastInstanceSeen: seenMax, ms: Math.round(performance.now() - t0) };
  };
  const direct = await shot(false), indirect = await shot(true);
  ib.destroy(); idx.destroy(); args.destroy(); src?.destroy();
  return { indexCount, instances, stride, argOffset: argWord * 4, skip, via, work, vertices: indexCount * instances, past24: indexCount * instances > 16777216, direct, indirect, errs };
};
</script></body></html>`;
const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const b = await Safari.launch({ width: 640, height: 400 });
// [indexCount, instances, stride]: 1,152 indices across 2^24 / 1,152 = 14,563.6
// instances, the scene's two counts at its 64-byte stride; then the index
// count varied at a product just under and just over 2^24.
// Then the scene's exact draws (runs/cull/spheres-20k-args.json): their args
// at 20-byte offsets in one buffer, their instances a region into it.
const SWEEP: Array<[number, number, number, number?, number?, string?, number?]> = [
  [1152, 14000, 64], [1152, 14563, 64], [1152, 14564, 64], [1152, 15000, 64], [1152, 16717, 64], [1152, 19028, 64],
  [1152, 14564, 16],
  [96, 174762, 16], [96, 174763, 16],
  [6, 2796202, 16], [6, 2796203, 16],
  [12288, 1365, 16], [12288, 1366, 16],
  [1152, 19028, 64, 0, 20001], [1152, 16717, 64, 30, 120006], [1152, 16717, 64, 30, 0], [1152, 16, 64, 30, 0], [1152, 19028, 64, 0, 0],
  // The instances written by a compute pass in the same encoder.
  [6, 16, 16, 0, 0, "compute"], [1152, 1000, 64, 0, 0, "compute"], [1152, 19028, 64, 0, 0, "compute"], [1152, 16717, 64, 30, 120006, "compute"],
  // The same with busy work, in one command buffer and then in two.
  [1152, 1000, 64, 0, 0, "compute", 2000], [1152, 1000, 64, 0, 0, "compute2", 0], [1152, 1000, 64, 0, 0, "compute2", 2000],
  [1152, 19028, 64, 0, 20001, "compute", 2000], [1152, 19028, 64, 0, 20001, "compute2", 0], [1152, 19028, 64, 0, 20001, "compute2", 2000],
];
let failed = false;
try {
  const p = await b.open(`http://127.0.0.1:${server.port}/`);
  await p.until("typeof probe === 'function' ? 1 : 0");
  for (const [n, k, s, a = 0, skip = 0, via = "write", work = 0] of SWEEP) {
    const r = await p.eval<{ direct: { wrongCells: number }; indirect: { wrongCells: number }; errs: string[] }>(`probe(${n}, ${k}, ${s}, ${a}, ${skip}, ${JSON.stringify(via)}, ${work})`);
    console.log(JSON.stringify(r));
    failed ||= r.direct.wrongCells > 0 || r.indirect.wrongCells > 0 || r.errs.length > 0;
  }
} finally {
  await b.close();
  server.stop();
}
process.exit(failed ? 1 : 0);
