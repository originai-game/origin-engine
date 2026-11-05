// Probe: do dependent dispatches in one WebGPU compute pass see each other's
// storage writes in Safari? cull.wgsl's chain trimmed: mark -> block sums ->
// scan of blocks -> prefix, all in one pass (as cull_gpu.rs runs it), versus
// one pass per dispatch. Prints mismatches of the final prefix.
//
//   sh tests/evidence/compat/safari/run.sh probe tests/evidence/compat/safari/probes/pass-order.ts
import { Safari } from "../../../../../packages/engine/tests/web/webdriver.ts";
declare const Bun: any;
const page = `<!doctype html><html><body><script>
window.probe = async (n, onePass, reps) => {
  const a = await navigator.gpu.requestAdapter(); const d = await a.requestDevice();
  const errs = []; d.onuncapturederror = (e) => errs.push(e.error.message);
  const WG = 256, blocks = Math.ceil(n / WG);
  const code = \`
@group(0) @binding(0) var<storage, read_write> scan: array<u32>;
const N: u32 = \${n}u; const B: u32 = \${blocks}u;
var<workgroup> s: array<u32, 256>;
fn hs(li: u32) { for (var k = 1u; k < 256u; k <<= 1u) { var v = s[li]; if (li >= k) { v += s[li - k]; } workgroupBarrier(); s[li] = v; workgroupBarrier(); } }
@compute @workgroup_size(256) fn mark(@builtin(global_invocation_id) g: vec3<u32>, @builtin(local_invocation_index) li: u32, @builtin(workgroup_id) w: vec3<u32>) {
  var v = 0u; if (g.x < N) { v = (g.x * 2654435761u >> 7u) & 1u; scan[g.x] = v; }
  s[li] = v; workgroupBarrier();
  for (var k = 128u; k > 0u; k >>= 1u) { if (li < k) { s[li] += s[li + k]; } workgroupBarrier(); }
  if (li == 0u) { scan[N + 1u + w.x] = s[0]; } }
@compute @workgroup_size(256) fn blocks(@builtin(local_invocation_index) li: u32) {
  var carry = 0u;
  for (var base = 0u; base < B; base += 256u) { let i = base + li; var v = 0u; if (i < B) { v = scan[N + 1u + i]; }
    s[li] = v; workgroupBarrier(); hs(li); if (i < B) { scan[N + 1u + i] = carry + s[li] - v; } carry += s[255]; workgroupBarrier(); } }
@compute @workgroup_size(256) fn prefix(@builtin(global_invocation_id) g: vec3<u32>, @builtin(local_invocation_index) li: u32, @builtin(workgroup_id) w: vec3<u32>) {
  var v = 0u; if (g.x < N) { v = scan[g.x]; } s[li] = v; workgroupBarrier(); hs(li);
  let before = scan[N + 1u + w.x]; if (g.x < N) { scan[g.x] = before + s[li] - v; } }\`;
  const m = d.createShaderModule({ code });
  const info = await m.getCompilationInfo(); for (const x of info.messages) errs.push(x.type + ": " + x.message);
  const p = (e) => d.createComputePipeline({ layout: "auto", compute: { module: m, entryPoint: e } });
  const pm = p("mark"), pb = p("blocks"), pp = p("prefix");
  const size = (n + 1 + blocks) * 4;
  const buf = d.createBuffer({ size, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC });
  const rb = d.createBuffer({ size: n * 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST });
  const bg = (pl) => d.createBindGroup({ layout: pl.getBindGroupLayout(0), entries: [{ binding: 0, resource: { buffer: buf } }] });
  const gm = bg(pm), gb = bg(pb), gp = bg(pp);
  let bad = 0, worst = 0;
  for (let r = 0; r < reps; r++) {
    const e = d.createCommandEncoder();
    const steps = [[pm, gm, blocks], [pb, gb, 1], [pp, gp, blocks]];
    if (onePass) { const c = e.beginComputePass(); for (const [pl, g, x] of steps) { c.setPipeline(pl); c.setBindGroup(0, g); c.dispatchWorkgroups(x); } c.end(); }
    else for (const [pl, g, x] of steps) { const c = e.beginComputePass(); c.setPipeline(pl); c.setBindGroup(0, g); c.dispatchWorkgroups(x); c.end(); }
    e.copyBufferToBuffer(buf, 0, rb, 0, n * 4); d.queue.submit([e.finish()]);
    await rb.mapAsync(GPUMapMode.READ); const out = new Uint32Array(rb.getMappedRange().slice(0)); rb.unmap();
    let acc = 0, miss = 0;
    for (let i = 0; i < n; i++) { if (out[i] !== acc) miss++; acc += ((Math.imul(i, 2654435761) >>> 0) >>> 7) & 1; }
    if (miss) bad++; worst = Math.max(worst, miss);
  }
  d.destroy();
  return { n, onePass, reps, badRuns: bad, worstMismatches: worst, errs };
};
</script></body></html>`;
const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const b = await Safari.launch({ width: 640, height: 400 });
try {
  const p = await b.open(`http://127.0.0.1:${server.port}/`);
  await p.until("typeof probe === 'function' ? 1 : 0");
  for (const n of [20000, 100000]) for (const one of [true, false]) console.log(JSON.stringify(await p.eval(`probe(${n}, ${one}, 20)`)));
} finally {
  await b.close();
  server.stop();
}
