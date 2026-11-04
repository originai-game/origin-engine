// The PowerVR refusal of the visibility buffer's resolve, outside the engine.
//
//   bun tests/evidence/compat/android/powervr-resolve/repro.ts [--android [serial]] [entry[@OVERRIDE=n]...]
//
// meshlet-resolve.wgsl is the module the engine made for the "meshlet
// resolve" pipeline (captured from its Worker on the Pixel 10, Chrome 156,
// at 012a073a3), meshlet-resolve.pipeline.json its pipeline's descriptor;
// variants.wgsl and record-variants.wgsl add entry points that change one
// thing each. Every named entry is made with createRenderPipelineAsync on a
// bare device (layout auto, the descriptor's rgba16float target and
// depth32float, EFFECTS and ANALYTIC true unless @OVERRIDE=n says
// otherwise) and printed with the device's answer. Without --android the
// page runs in this machine's Chrome (cdp.ts: --chrome or $CHROME), the
// control. Default entries: the ones README.md quotes.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Browser, chromePath } from "../../../../../packages/engine/tests/web/cdp.ts";

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const android = args.includes("--android");
const named = args.filter((a, i) => !a.startsWith("--") && !(args[i - 1] === "--android" || args[i - 1] === "--chrome"));
const entries = named.length
  ? named
  : ["fs_meshlet_resolve", "fs_meshlet_resolve@ANALYTIC=0", "v_constlit", "v_synth", "v_record_unused", "v_rec_noanalytic", "v_rec_onlydx", "v_rec_nouv"];
const code = ["meshlet-resolve.wgsl", "variants.wgsl", "record-variants.wgsl"].map((f) => readFileSync(join(here, f), "utf8")).join("\n");

const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>repro</title>", { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(args, { chrome: android ? "" : chromePath(args), reverse: [server.port] });
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/`, []);
  await page.until("document.readyState === 'complete'", 15000);
  const answers = await page.eval<Record<string, string>>(`(async () => {
    const a = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
    const d = await a.requestDevice({ requiredLimits: { maxStorageBuffersPerShaderStage: 8 } });
    const m = d.createShaderModule({ code: ${JSON.stringify(code)} });
    const errors = (await m.getCompilationInfo()).messages.filter((x) => x.type === "error");
    if (errors.length) return { module: errors.map((x) => x.lineNum + ": " + x.message).join("; ") };
    const out = { adapter: a.info.vendor + " " + a.info.architecture };
    for (const spec of ${JSON.stringify(entries)}) {
      const [fs, ...kv] = spec.split("@");
      const constants = { EFFECTS: 1, ANALYTIC: 1 };
      for (const p of kv) { const [k, v] = p.split("="); constants[k] = Number(v); }
      try {
        await d.createRenderPipelineAsync({
          layout: "auto",
          vertex: { module: m, entryPoint: "vs_meshlet_resolve" },
          fragment: { module: m, entryPoint: fs, constants, targets: [{ format: "rgba16float" }] },
          depthStencil: { format: "depth32float", depthCompare: "less", depthWriteEnabled: true },
        });
        out[spec] = "ok";
      } catch (e) { out[spec] = "refused (" + (e.reason || "") + "): " + String(e.message).split("\\n")[0]; }
    }
    return out;
  })()`);
  for (const [k, v] of Object.entries(answers)) console.log(`${k.padEnd(32)} ${v}`);
} finally {
  await browser.close();
  server.stop();
}
