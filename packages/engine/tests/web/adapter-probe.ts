// What a cold Chrome's first requestAdapter answers, for the final pass's
// web-compat section (docs/ci.md §The final L4 pass): each run launches Chrome
// fresh and asks a worker for a high-performance compatibility adapter twice.
// A first `ok:false` then `ok:true` is the cold-launch null the engine's
// adapter retry covers; two `ok:false` are a host with no adapter, which no
// retry covers. The wgpu fork thread's probe.
//
//   bun tests/web/adapter-probe.ts <chrome> [runs]
//
// Prints one line a run (`run <i> compat=<OG_L4_COMPAT> early [...]`) and
// exits 0: the answers are facts for the pass, not a verdict.
import { Browser } from "./cdp.ts";

declare const Bun: { serve(o: { port: number; hostname: string; fetch(r: Request): Response }): { port: number; stop(): void } };

const chrome = process.argv[2];
if (!chrome) throw new Error("adapter-probe.ts: <chrome> is required");
const runs = Number(process.argv[3] ?? 6);
const worker = `
onmessage = async (e) => {
  const out = [];
  for (let i = 0; i < e.data; i++) {
    const t0 = performance.now();
    let a = null, err = null;
    try { a = await navigator.gpu.requestAdapter({ powerPreference: "high-performance", featureLevel: "compatibility" }); } catch (x) { err = String(x); }
    out.push({ ms: Math.round(performance.now() - t0), ok: !!a, core: a ? a.features.has("core-features-and-limits") : null, vendor: a?.info?.vendor, arch: a?.info?.architecture, err });
  }
  postMessage(out);
};`;
const page = `<script>
window.early = new Promise((res) => { const w = new Worker(URL.createObjectURL(new Blob([${JSON.stringify(worker)}]))); w.onmessage = (e) => res(JSON.stringify(e.data)); w.postMessage(2); });
</script>`;
const server = Bun.serve({ port: 0, hostname: "127.0.0.1", fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
for (let i = 0; i < runs; i++) {
  const b = await Browser.launch({ chrome });
  try {
    const p = await b.open(`http://127.0.0.1:${server.port}/`);
    await p.until("globalThis.early", 20_000);
    console.log(`run ${i} compat=${process.env.OG_L4_COMPAT ?? 0} early ${await p.eval<string>("early")}`);
  } catch (e) {
    console.log(`run ${i} error ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    await b.close();
  }
}
server.stop();
