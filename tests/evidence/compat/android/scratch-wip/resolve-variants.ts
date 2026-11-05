// Scratch: compile the captured "meshlet resolve" module on the phone with
// variant fragment entry points appended, to find what PowerVR's driver rejects.
import { readFileSync, writeFileSync } from "node:fs";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";

const dir = process.argv[2]!;
let base = readFileSync(`${dir}/resolve-7.wgsl`, "utf8");
for (const kv of (process.env.REPLACE ?? "").split("|||").filter(Boolean)) { const [a, b] = kv.split("==>"); if (!base.includes(a!)) throw new Error("no " + a); base = base.split(a!).join(b!); }
const extra = readFileSync(process.argv[3]!, "utf8");
const entries = (process.argv[4] ?? "fs_meshlet_resolve").split(",");
const code = base + "\n" + extra;
const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>v</title>", { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const page = await browser.open(`http://127.0.0.1:${server.port}/`, []);
await page.until("document.readyState === 'complete'", 15000);
const r = await page.eval(`(async () => {
  const a = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
  const d = await a.requestDevice({ requiredFeatures: [...a.features].filter((f) => ["timestamp-query", "shader-f16", "rg11b10ufloat-renderable", "float32-blendable", "texture-compression-astc"].includes(f)), requiredLimits: { maxStorageBuffersPerShaderStage: 8 } });
  const code = ${JSON.stringify(code)};
  d.pushErrorScope("validation");
  const m = d.createShaderModule({ code });
  const info = await m.getCompilationInfo();
  const ve = await d.popErrorScope();
  const out = { compile: info.messages.filter((x) => x.type === "error").slice(0, 5).map((x) => x.lineNum + ": " + x.message), moduleError: ve && ve.message, entries: {} };
  for (const spec of ${JSON.stringify(entries)}) {
    const [fs, ...kv] = spec.split("@"); const constants = { EFFECTS: 1, ANALYTIC: 1 }; for (const p of kv) { const [k, v] = p.split("="); constants[k] = Number(v); }
    try {
      await d.createRenderPipelineAsync({
        label: fs, layout: "auto",
        vertex: { module: m, entryPoint: "vs_meshlet_resolve" },
        fragment: { module: m, entryPoint: fs, constants, targets: [{ format: "rgba16float" }] },
        depthStencil: { format: "depth32float", depthCompare: "less", depthWriteEnabled: true },
      });
      out.entries[spec] = "ok";
    } catch (e) { out.entries[spec] = (e.reason || "") + ": " + String(e.message).split("\\n")[0]; }
  }
  return out;
})()`);
console.log(JSON.stringify(r, null, 1));
await browser.close();
server.stop();


