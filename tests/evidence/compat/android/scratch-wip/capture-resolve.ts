// Scratch: capture the "meshlet resolve" pipeline's WGSL and descriptor from the
// engine Worker on the phone (game.js served with a hook prepended), and
// whether each createRenderPipeline call on the phone succeeded.
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../packages/engine/tests/web/serve.ts";

const src = "C:/Users/win/src/origin-engine-android/packages/engine/dist/test-reference/fx-showcase";
const out = process.argv[2]!;
const dist = join(out, "dist");
rmSync(dist, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
cpSync(src, dist, { recursive: true });

const HOOK = `(() => {
  const code = new WeakMap();
  const P = GPUDevice.prototype;
  const csm = P.createShaderModule;
  P.createShaderModule = function (d) { const m = csm.call(this, d); code.set(m, d.code); return m; };
  const post = (o) => fetch("/game/__capture", { method: "POST", body: JSON.stringify(o) }).catch(() => {});
  const st = (s) => s && { entryPoint: s.entryPoint, constants: s.constants, targets: s.targets, code: code.get(s.module) };
  const keep = (dev, d, how) => {
    if (!String(d.label || "").includes("meshlet resolve")) return;
    return async () => {
      const v = null, i = null;
      post({ how, label: d.label, vertex: st(d.vertex), fragment: st(d.fragment), primitive: d.primitive, depthStencil: d.depthStencil, multisample: d.multisample, validation: v && v.message, internal: i && i.message });
    };
  };
  post({ how: "hook up", where: String(self.location) }); let n = 0; self.__n = () => n;
  const crp = P.createRenderPipeline;
  P.createRenderPipeline = function (d) { if (++n % 20 === 1 || /meshlet/.test(d.label)) post({ how: "seen", label: d.label, n }); const done = keep(this, d, "sync"); const p = crp.call(this, d); done && done(); return p; };
  const crpa = P.createRenderPipelineAsync;
  P.createRenderPipelineAsync = function (d) {
    const r = crpa.call(this, d); if (++n % 20 === 1 || /meshlet/.test(d.label)) post({ how: "seen async", label: d.label, n });
    if (String(d.label || "").includes("meshlet resolve")) r.then(() => post({ how: "async ok", label: d.label, vertex: st(d.vertex), fragment: st(d.fragment), depthStencil: d.depthStencil, multisample: d.multisample }), (e) => post({ how: "async rejected", error: String(e && e.message || e), label: d.label, vertex: st(d.vertex), fragment: st(d.fragment), primitive: d.primitive, depthStencil: d.depthStencil, multisample: d.multisample }));
    return r;
  };
})();
`;
writeFileSync(join(dist, "game.js"), HOOK + readFileSync(join(dist, "game.js"), "utf8"));

const captured: Array<Record<string, any>> = [];
const html = `<!doctype html><html><head><meta charset="utf-8"><title>reference</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:1280px;height:720px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/reference.html": html } });
// The capture sink beside serve.ts's server.
const sink = Bun.serve({ port: 0, async fetch(req) { captured.push(await req.json()); return new Response("ok"); } });
void sink;
// serve.ts has no POST route of ours: point the hook at its own origin through a tiny proxy instead.
const proxy = Bun.serve({
  port: 0,
  async fetch(req) {
    const u = new URL(req.url);
    if (u.pathname === "/game/__capture") { captured.push(await req.json()); return new Response("ok"); }
    const r = await fetch(`http://127.0.0.1:${server.port}${u.pathname}${u.search}`);
    return new Response(r.body, { status: r.status, headers: r.headers });
  },
});
const browser = await Browser.from(["--android"], { reverse: [proxy.port] });
const plog: string[] = [];
const page = await browser.open("about:blank", plog);
await page.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
await page.send("Page.navigate", { url: `http://127.0.0.1:${proxy.port}/reference.html?profile=3d` });
await page.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
await page.eval(`__og_shell__.call("probe", "commands", "0.5:station:post")`);
await page.eval(`(async () => { for (let i = 0; i < 40; i++) await __og_shell__.call("frame", 1000 / 60); })()`);
await Bun.sleep(2000);
console.log(`captured ${captured.length}`);
captured.forEach((c, i) => {
  writeFileSync(join(out, `resolve-${i}.wgsl`), c.fragment?.code ?? "");
  writeFileSync(join(out, `resolve-${i}.json`), JSON.stringify({ ...c, vertex: { ...c.vertex, code: undefined }, fragment: { ...c.fragment, code: undefined } }, null, 1));
  if (c.how.startsWith("seen") || c.how === "hook up") { console.log(i, JSON.stringify(c)); return; }
  console.log(i, c.how, c.label, c.vertex?.entryPoint, c.fragment?.entryPoint, JSON.stringify(c.fragment?.constants), JSON.stringify(c.fragment?.targets), JSON.stringify(c.depthStencil), JSON.stringify(c.multisample), "same:", c.vertex?.code === c.fragment?.code, (c.fragment?.code ?? "").length, c.validation ?? "", c.internal ?? "", c.error ?? "");
});
console.log(plog.slice(-10).join("\n"));
await browser.close();
proxy.stop();
sink.stop();
server.stop();


