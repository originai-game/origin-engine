// Scratch: WebGPU adapter info, features, limits and WGSL language features
// from Chrome on the phone (adb reverse + CDP on 9322) and desktop Chrome.
import { writeFileSync } from "node:fs";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";

const out = process.argv[2]!;
const PORT = 8790;
const server = Bun.serve({ port: PORT, fetch: () => new Response("<!doctype html><title>adapter</title>", { headers: { "content-type": "text/html" } }) });

const PROBE = `(async () => {
  const a = await navigator.gpu?.requestAdapter({ powerPreference: "high-performance" });
  if (!a) return { error: "no adapter", gpu: !!navigator.gpu, secure: isSecureContext };
  const info = {}; for (const k in a.info) if (typeof a.info[k] !== "function") info[k] = a.info[k];
  const limits = {}; for (const k in a.limits) limits[k] = a.limits[k];
  const d = await a.requestDevice();
  const fallback = await navigator.gpu.requestAdapter({ forceFallbackAdapter: true });
  return {
    userAgent: navigator.userAgent, dpr: devicePixelRatio, screen: [screen.width, screen.height],
    hardwareConcurrency: navigator.hardwareConcurrency, deviceMemory: navigator.deviceMemory ?? null,
    preferredCanvasFormat: navigator.gpu.getPreferredCanvasFormat(),
    info, isFallbackAdapter: a.info.isFallbackAdapter ?? a.isFallbackAdapter ?? null,
    features: [...a.features].sort(), limits,
    wgslLanguageFeatures: [...(navigator.gpu.wgslLanguageFeatures ?? [])].sort(),
    deviceDefaultLimits: Object.fromEntries(Object.keys(limits).map((k) => [k, d.limits[k]])),
    fallbackAdapter: !!fallback,
  };
})()`;

async function cdpEval(ws: WebSocket, expr: string, session?: string): Promise<unknown> {
  return send(ws, "Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true }, session).then((r: any) => {
    if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails));
    return r.result.value;
  });
}
let next = 1;
const waiting = new Map<number, (v: any) => void>();
function send(ws: WebSocket, method: string, params: object = {}, sessionId?: string): Promise<any> {
  const id = next++;
  return new Promise((res) => { waiting.set(id, res); ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) })); });
}

const result: Record<string, unknown> = { at: new Date().toISOString() };

// Phone.
Bun.spawnSync(["adb", "reverse", `tcp:${PORT}`, `tcp:${PORT}`]);
const v = (await (await fetch("http://127.0.0.1:9322/json/version")).json()) as { webSocketDebuggerUrl: string; Browser: string };
const ws = new WebSocket(v.webSocketDebuggerUrl.replace(/^ws:\/\/[^/]+/, "ws://127.0.0.1:9322"));
ws.addEventListener("message", (e) => { const m = JSON.parse(String(e.data)); if (m.id) { waiting.get(m.id)?.(m.result ?? { exceptionDetails: m.error }); waiting.delete(m.id); } });
await new Promise((r) => ws.addEventListener("open", r));
const { targetId } = await send(ws, "Target.createTarget", { url: `http://localhost:${PORT}/` });
const { sessionId } = await send(ws, "Target.attachToTarget", { targetId, flatten: true });
await Bun.sleep(1500);
result["android"] = { browser: v.Browser, ...(await cdpEval(ws, PROBE, sessionId) as object) };
await send(ws, "Target.closeTarget", { targetId });
ws.close();

// Desktop Chrome, launched as the harness launches it.
const b = await Browser.launch({});
const p = await b.open(`http://127.0.0.1:${PORT}/`);
await Bun.sleep(1000);
result["desktop"] = await p.eval(PROBE);
await b.close();
server.stop();
writeFileSync(out, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify({ android: (result.android as any).info, desktop: (result.desktop as any).info }, null, 1));
