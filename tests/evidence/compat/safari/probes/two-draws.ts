// Probe: two indexed-indirect draws of two meshes in one render pass, their
// args in one buffer five words apart (cull_gpu.rs: spheres-20k's spheres and
// its ground plane, the ground's removal making Safari's frame right).
// Mesh A has `verts` vertices (its indices reach all of them), mesh B four.
// Each instance is a one-pixel cell coloured by its id, so the picture is
// known on the CPU; indirect is compared with it and with drawIndexed.
//
//   sh tests/evidence/compat/safari/run.sh probe tests/evidence/compat/safari/probes/two-draws.ts
import { Safari } from "../../../../../packages/engine/tests/web/webdriver.ts";
declare const Bun: any;
const page = `<!doctype html><html><body><script>
let d;
window.probe = async (o) => {
  d ??= await (await navigator.gpu.requestAdapter()).requestDevice();
  const errs = []; d.onuncapturederror = (e) => errs.push(e.error.message);
  const W = 128, CELLS = W * W;
  const code = \`
struct O { @builtin(position) p: vec4<f32>, @location(0) @interpolate(flat) i: u32 };
@vertex fn vs(@location(0) v: vec2<f32>, @location(1) i: u32) -> O {
  let cell = i % \${CELLS}u;
  let px = vec2<f32>(f32(cell % \${W}u), f32(cell / \${W}u)) + v;
  var o: O; o.p = vec4<f32>(px.x / \${W}.0 * 2.0 - 1.0, 1.0 - px.y / \${W}.0 * 2.0, 0.5, 1.0); o.i = i; return o; }
@fragment fn fs(o: O) -> @location(0) vec4<f32> {
  return vec4<f32>(f32(o.i & 255u), f32((o.i >> 8u) & 255u), f32((o.i >> 16u) & 255u), 255.0) / 255.0; }
@group(0) @binding(0) var<storage, read_write> args: array<atomic<u32>>;
@compute @workgroup_size(1) fn write() {
  atomicStore(&args[0], \${o.aIndices}u); atomicStore(&args[1], \${o.aInstances}u);
  atomicStore(&args[2], 0u); atomicStore(&args[3], 0u); atomicStore(&args[4], 0u);
  atomicStore(&args[5], 6u); atomicStore(&args[6], 1u);
  atomicStore(&args[7], 0u); atomicStore(&args[8], 0u); atomicStore(&args[9], 0u); }\`;
  const m = d.createShaderModule({ code });
  for (const x of (await m.getCompilationInfo()).messages) errs.push(x.type + ": " + x.message);
  const corners = [0, 0, 1, 0, 1, 1, 0, 1];
  // Mesh A: verts vertices, each a quad corner; its indices walk them all (k*4 .. k*4+3 per quad).
  const av = new Float32Array(o.verts * 2); for (let k = 0; k < o.verts; k++) av.set(corners.slice((k % 4) * 2, (k % 4) * 2 + 2), k * 2);
  const ai = new Uint32Array(o.aIndices); const quads = o.verts / 4;
  for (let k = 0; k < o.aIndices; k++) ai[k] = (Math.floor(k / 6) % quads) * 4 + [0, 1, 2, 0, 2, 3][k % 6];
  const bv = new Float32Array(corners), bi = new Uint32Array([0, 1, 2, 0, 2, 3]);
  const buf = (data, usage) => { const b = d.createBuffer({ size: Math.max(16, data.byteLength), usage: usage | GPUBufferUsage.COPY_DST }); d.queue.writeBuffer(b, 0, data); return b; };
  const A = { v: buf(av, GPUBufferUsage.VERTEX), i: buf(ai, GPUBufferUsage.INDEX), n: o.aIndices };
  const B = { v: buf(bv, GPUBufferUsage.VERTEX), i: buf(bi, GPUBufferUsage.INDEX), n: 6 };
  // Instances: A's ids 0..aInstances, B's one instance the id \${CELLS - 1} (the last cell), stride 64,
  // in one buffer: A's region, then B's record.
  const words = 16, inst = new Uint32Array((o.aInstances + 1) * words);
  for (let i = 0; i < o.aInstances; i++) inst[i * words] = i;
  const bId = CELLS - 1 + CELLS * 64; inst[o.aInstances * words] = bId;
  const ib = buf(inst, GPUBufferUsage.VERTEX);
  const args = d.createBuffer({ size: 64, usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.STORAGE });
  const draw = d.createRenderPipeline({ layout: "auto",
    vertex: { module: m, entryPoint: "vs", buffers: [
      { arrayStride: 8, attributes: [{ shaderLocation: 0, offset: 0, format: "float32x2" }] },
      { arrayStride: 64, stepMode: "instance", attributes: [{ shaderLocation: 1, offset: 0, format: "uint32" }] }] },
    fragment: { module: m, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] } });
  const write = d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: "write" } });
  const wg = d.createBindGroup({ layout: write.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: args } }] });
  const shot = async (indirect) => {
    const t = d.createTexture({ size: [W, W], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
    const rb = d.createBuffer({ size: W * W * 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
    const e = d.createCommandEncoder();
    const c = e.beginComputePass(); c.setPipeline(write); c.setBindGroup(0, wg); c.dispatchWorkgroups(1); c.end();
    const r = e.beginRenderPass({ colorAttachments: [{ view: t.createView(), loadOp: "clear", storeOp: "store", clearValue: [0, 0, 0, 0] }] });
    r.setPipeline(draw);
    const one = (mesh, first, count, argAt) => {
      r.setVertexBuffer(0, mesh.v); r.setVertexBuffer(1, ib, first * 64, count * 64); r.setIndexBuffer(mesh.i, "uint32");
      if (indirect) r.drawIndexedIndirect(args, argAt); else r.drawIndexed(mesh.n, count);
    };
    const drawA = () => one(A, 0, o.aInstances, 0), drawB = () => one(B, o.aInstances, 1, 20);
    if (o.order === "AB") { drawA(); if (o.withB) drawB(); } else { if (o.withB) drawB(); drawA(); }
    r.end();
    e.copyTextureToBuffer({ texture: t }, { buffer: rb, bytesPerRow: W * 4 }, [W, W]);
    d.queue.submit([e.finish()]);
    await rb.mapAsync(GPUMapMode.READ); const px = new Uint32Array(rb.getMappedRange().slice(0)); rb.unmap();
    t.destroy(); rb.destroy();
    let wrong = 0;
    for (let cell = 0; cell < CELLS; cell++) {
      let want = 0;
      const lastA = cell < o.aInstances ? cell + Math.floor((o.aInstances - 1 - cell) / CELLS) * CELLS : -1;
      const fromA = lastA < 0 ? 0 : (0xff000000 | lastA) >>> 0;
      const fromB = o.withB && cell === CELLS - 1 ? (0xff000000 | (bId & 0xffffff)) >>> 0 : 0;
      want = o.order === "AB" ? fromB || fromA : fromA || fromB;
      if ((px[cell] >>> 0) !== want) wrong++;
    }
    return wrong;
  };
  const direct = await shot(false), indirect = await shot(true);
  for (const b of [A.v, A.i, B.v, B.i, ib, args]) b.destroy();
  return { ...o, directWrong: direct, indirectWrong: indirect, errs };
};
</script></body></html>`;
const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const b = await Safari.launch({ width: 640, height: 400 });
const CASES = [
  { verts: 4, aIndices: 6, aInstances: 1000, withB: false, order: "AB" },
  { verts: 4, aIndices: 6, aInstances: 1000, withB: true, order: "AB" },
  { verts: 196, aIndices: 1152, aInstances: 100, withB: true, order: "AB" },
  { verts: 196, aIndices: 1152, aInstances: 1000, withB: false, order: "AB" },
  { verts: 196, aIndices: 1152, aInstances: 1000, withB: true, order: "AB" },
  { verts: 196, aIndices: 1152, aInstances: 1000, withB: true, order: "BA" },
  { verts: 196, aIndices: 1152, aInstances: 20000, withB: true, order: "AB" },
  { verts: 196, aIndices: 1152, aInstances: 20000, withB: true, order: "BA" },
];
let failed = false;
try {
  const p = await b.open(`http://127.0.0.1:${server.port}/`);
  await p.until("typeof probe === 'function' ? 1 : 0");
  for (const c of CASES) {
    const r = await p.eval<{ directWrong: number; indirectWrong: number; errs: string[] }>(`probe(${JSON.stringify(c)})`);
    console.log(JSON.stringify(r));
    failed ||= r.directWrong > 0 || r.indirectWrong > 0 || r.errs.length > 0;
  }
} finally {
  await b.close();
  server.stop();
}
process.exit(failed ? 1 : 0);
