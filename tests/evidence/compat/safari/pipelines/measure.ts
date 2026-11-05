// Scratch: fx-showcase's real gated page (dev export, its gate and clock) in Safari or Chrome, one
// pipelines mode a run. Prints one JSON line: first frame, the warm-up's end behind the gate, the page's
// rAF gaps, and after the tap each station's worst tick stall and time to 90 ticks.
//   bun measure.ts --browser safari|chrome --mode auto|sync|async|prewarm --tap early|late
import { join, resolve } from "node:path";
import { existsSync } from "node:fs";
const repo = resolve(import.meta.dir, "../../../../..");
const web = join(repo, "packages/engine/tests/web");
const { webBuild, settingsOf } = await import(join(repo, "packages/engine/src/node/web-build.ts"));
const { Browser, chromePath } = await import(join(web, "cdp.ts"));
const { Safari } = await import(join(web, "webdriver.ts"));
const { serve } = await import(join(web, "serve.ts"));
const { cookProject } = await import(join(repo, "packages/engine/tests/support/og-bin.ts"));

const arg = (n: string, d: string) => {
  const i = process.argv.indexOf(n);
  return i >= 0 ? process.argv[i + 1]! : d;
};
const browserName = arg("--browser", "safari");
const mode = arg("--mode", "auto");
const tap = arg("--tap", "early");
const project = join(repo, "templates/fx-showcase");
const dist = join(repo, ".amp/in/sp", `dist-${mode}`);
if (!existsSync(join(dist, "og-build.json")) || process.argv.includes("--build")) {
  cookProject(project, repo);
  const s = settingsOf(project);
  const b = await webBuild(project, join(project, "src", "main.ts"), dist, false, { ...s, publish: { ...s.publish, pipelines: mode, gate: "tap" } });
  if (!b.ok) throw new Error(b.reason);
}
if (process.argv.includes("--build-only")) process.exit(0);
const server = serve({ dist, isolate: true });
const W = 1280, H = 720;
const browser = browserName === "safari" ? await Safari.launch({ width: W, height: H }) : await Browser.from(process.argv, { chrome: chromePath(process.argv), width: W, height: H, reverse: [] });
const log: string[] = [];
const out: Record<string, unknown> = { browser: browserName, mode, tap };
try {
  const page = await browser.open("about:blank", log);
  await page.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
  // The page's own frames (a compositor frame each), from as early as WebDriver gets in.
  await page.until("document.readyState !== 'loading'", 30_000);
  await page.eval(`(() => { globalThis.__gaps = []; let last = performance.now(); const f = (t) => { __gaps.push([Math.round(t), Math.round(t - last)]); last = t; requestAnimationFrame(f); }; requestAnimationFrame(f); return 1; })()`);
  const ff = await page.until<number>("performance.getEntriesByName('og:first-frame')[0]?.startTime", 120_000);
  out["firstFrame"] = Math.round(ff);
  out["atFirstFrame"] = await page.eval("__runtime_probe__.boot?.pipelines ?? null");
  // The warm-up's end: the Worker reports the counts again (a new boot object), or play stops it.
  await page.eval("(globalThis.__b0 = __runtime_probe__.boot, 1)");
  const warmed = () => page.eval<number | null>("__runtime_probe__.boot !== __b0 ? performance.now() : null");
  if (tap === "late") {
    let end: number | null = null;
    for (let i = 0; i < 1200 && end === null; i++) {
      await new Promise((r) => setTimeout(r, 100));
      end = await warmed();
    }
    out["warmEndAfterFF"] = end === null ? "none in 120 s" : Math.round(end - ff);
    out["warmed"] = await page.eval("__runtime_probe__.boot?.pipelines ?? null");
  } else {
    await page.until(`performance.now() > ${ff + 1000}`, 30_000);
  }
  const gapsBehind = await page.eval<Array<[number, number]>>(`__gaps.filter(([t]) => t > ${ff})`);
  out["behindGate"] = { worstRafGap: Math.max(0, ...gapsBehind.map((g) => g[1])), frames: gapsBehind.length };
  const k0 = await page.eval<number>("__runtime_probe__.tick()");
  const tapAt = await page.eval<number>("performance.now()");
  await page.tap(W / 2, H / 2);
  const run = await page.eval<Record<string, unknown>>(`(async () => {
    const P = __runtime_probe__;
    const wait = (ms) => new Promise((r) => setTimeout(r, ms));
    const t0 = performance.now();
    let k = await P.tick();
    while (k === ${k0} && performance.now() - t0 < 30000) { await wait(5); k = await P.tick(); }
    const started = Math.round(performance.now() - ${tapAt});
    const stations = [];
    for (const s of ["post", "lens", "fx", "2d", "stylised"]) {
      if (s !== "post") await P.commands("0.02:station:" + s);
      const a = performance.now(), g0 = __gaps.length;
      const k1 = await P.tick();
      let last = k1, lastAt = a, worst = 0;
      for (;;) {
        await wait(5);
        const kk = await P.tick(), now = performance.now();
        if (kk !== last) { worst = Math.max(worst, now - lastAt); last = kk; lastAt = now; }
        if (kk - k1 >= 90 || now - a > 60000) { worst = Math.max(worst, now - lastAt); break; }
      }
      stations.push({ s, worstTickStall: Math.round(worst), ms90: Math.round(performance.now() - a), worstRafGap: Math.max(0, ...__gaps.slice(g0).map((g) => g[1])) });
    }
    return { started, stations, total: Math.round(performance.now() - t0), pipelines: P.boot?.pipelines ?? null, error: P.error ?? null };
  })()`);
  Object.assign(out, run);
} catch (e) {
  out["error"] = String(e);
  out["log"] = log.slice(-15);
} finally {
  await browser.close();
  server.stop();
}
console.log(JSON.stringify(out));
process.exit(0);
