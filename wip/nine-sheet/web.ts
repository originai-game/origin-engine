// bun web.ts <engine package dir> <out dir> [--chrome path]: the nine-slice sheet in Chrome WebGPU at DPR 1 and 2.
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
const pkg = resolve(process.argv[2]!);
const out = resolve(process.argv[3]!);
const { settingsOf, webBuild } = await import(join(pkg, "src/node/web-build.ts"));
const { Browser, chromePath } = await import(join(pkg, "tests/web/cdp.ts"));
const { serve } = await import(join(pkg, "tests/web/serve.ts"));
const { cookProject } = await import(join(pkg, "tests/support/og-bin.ts"));
const here = dirname(fileURLToPath(import.meta.url));
const project = join(here, "project");
mkdirSync(out, { recursive: true });
cookProject(project, join(pkg, "..", ".."));
const dist = join(out, "dist");
const built = await webBuild(project, join(project, "src", "main.tsx"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><style>html,body{margin:0;background:#000}canvas{display:block;width:1280px;height:720px}</style></head><body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/n.html": html } });
const browser = await Browser.launch({ chrome: chromePath(process.argv.slice(2)), width: 1280, height: 720 });
const log: string[] = [];
try {
  for (const pack of (process.env.PACKS ?? "cartoon,fantasy,scifi").split(",")) for (const dpr of (process.env.DPRS ?? "1,2").split(",").map(Number)) {
    const t = await browser.open("about:blank", log);
    await t.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: dpr, mobile: false });
    await t.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/n.html?dpr=${dpr}&profile=2d` });
    await t.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 120_000);
    const failed = await t.eval("globalThis.__og_shell_error__");
    if (failed) throw new Error(String(failed));
    await t.eval(`__og_shell__.call("probe", "commands", ${JSON.stringify(`0.02:pack:${pack};0.3:sheet`)})`);
    await t.eval(`(async () => { for (let i = 0; i < 400; i++) await __og_shell__.call("frame", 1000 / 60); await new Promise((r) => setTimeout(r, 300)); })()`);
    const W = 1280 * dpr, H = 720 * dpr;
    const png = await t.eval(`(async () => { let s = await __og_shell__.probe("capture", { width: ${W}, height: ${H} }); for (let i = 0; !s && i < 600; i++) { await new Promise((r) => setTimeout(r, 16)); s = await __og_shell__.probe("capture", { width: ${W}, height: ${H} }); } return s && s.png; })()`);
    const dump = await t.eval(`(async () => JSON.stringify(await __og_shell__.probe("dump", { what: "ui" })))()`);
    writeFileSync(join(out, `dump-${pack}-${dpr}.json`), String(dump));
    if (!png) throw new Error(`no capture for ${pack} ${dpr}`);
    writeFileSync(join(out, `web-${pack}-${dpr}.png`), Uint8Array.from(atob(png as string), (c) => c.charCodeAt(0)));
    console.log(`web-${pack}-${dpr}.png`, (await t.eval("JSON.stringify(globalThis.__og_log__.filter(l => /warn|error|crash|image|texture/i.test(l) && !/device|renderer again/i.test(l)).slice(0,20))")));
    await t.close?.();
  }
} finally {
  await browser.close();
  server.stop?.();
}
