// The web export end to end in headless Chrome: both archetypes built with
// `og build --publish`'s web half, served with the build's own `_headers`,
// loaded on WebGPU and on the WebGL2 fallback.
//
// Per archetype and backend it records the first-frame time (the shell's
// `og:first-frame` mark, ms since navigation), the tier the engine reports,
// and a frame after a real tap on the play gate. On the runner over WebGPU
// it also: streams the wasm and cooked/ slowly so the boot can be seen, and
// screenshots the splash (genesis) and a progress frame with its real value;
// checks og-build.json against the files served; checks the web manifest and
// its icons; registers the service worker and reloads offline (the game must
// start from the cache); and lays the page out at phone width (390×844, DPR
// 3: the canvas fills the viewport and is drawn at the DPR cap of 2).
//
//   bun tests/web/export.ts [--out <dir>] [--chrome <path>]
//
// Writes <out>/export-result.json and PNGs (default dist/test-export/out).
// Exits 1 when a check fails.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild, type BuildManifest } from "../../src/node/web-build.ts";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { serve } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
  CryptoHasher: new (a: string) => { update(d: Uint8Array): unknown; digest(e: "hex"): string };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const work = join(pkg, "dist", "test-export");
const out = resolve(opt("--out") ?? join(work, "out"));
mkdirSync(out, { recursive: true });
const chrome = chromePath(args);

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(ok && detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 400)}` : ""}`);
}
const save = (name: string, png: Uint8Array) => {
  writeFileSync(join(out, name), png);
  return name;
};
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

// ── the builds ─────────────────────────────────────────────────────────
const builds: Record<string, { dist: string; manifest: BuildManifest; total: { bytes: number; compressed: number }; files: Array<{ path: string; bytes: number; compressed: number }> }> = {};
for (const game of ["runner", "match3"]) {
  const project = join(repo, "templates", "archetypes", game);
  if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
    const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
    if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
  }
  const dist = join(work, game);
  const b = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
  if (!b.ok) throw new Error(b.reason);
  builds[game] = { dist, manifest: b.manifest!, total: b.total, files: b.files };
  // og-build.json names every file with its bytes and sha256.
  const m = b.manifest!;
  const bad = m.files.filter((f) => {
    const bytes = new Uint8Array(readFileSync(join(dist, f.path)));
    const h = new Bun.CryptoHasher("sha256");
    h.update(bytes);
    return bytes.length !== f.bytes || h.digest("hex") !== f.sha256;
  });
  check(bad.length === 0 && m.files.length > 10, `${game}: og-build.json lists every file with its bytes and sha256`, bad.map((f) => f.path));
  check(m.build === "publish" && m.probe.build === "publish" && m.probe.members.join(",") === "hello,tick,state,pick,entity,input,capture", `${game}: a publish build with the 7 player probe members`, m.probe);
}

const results: Record<string, unknown> = {};
const browser = await Browser.launch({ chrome });

async function load(page: Page): Promise<{ firstFrame: number; ready: number; tier: string }> {
  await page.until("window.__runtime_probe__", 60_000);
  const firstFrame = await page.until<number>("performance.getEntriesByName('og:first-frame')[0]?.startTime", 30_000);
  const ready = await page.eval<number>("performance.getEntriesByName('og:ready')[0]?.startTime ?? 0");
  const hello = await page.eval<{ tier: string }>("__runtime_probe__.hello()");
  return { firstFrame: Math.round(firstFrame), ready: Math.round(ready), tier: hello.tier };
}

try {
  for (const game of ["runner", "match3"]) {
    for (const backend of ["webgpu", "webgl2"] as const) {
      const label = `${game} ${backend}`;
      const server = serve({ dist: builds[game]!.dist, isolate: true });
      const log: string[] = [];
      const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html${backend === "webgl2" ? "?webgl=1" : ""}`, log);
      try {
        const t = await load(page);
        check(t.tier === backend, `${label}: the engine reports tier ${backend}`, t);
        check(t.firstFrame > 0 && t.firstFrame < 10_000, `${label}: first frame at ${t.firstFrame} ms after navigation`, t);
        const gate = await page.eval<boolean>("!document.getElementById('og-gate').hidden");
        check(gate, `${label}: the play gate is up after the first frame`);
        const gateShot = save(`${game}-${backend}-gate.png`, await page.screenshot());
        const before = await page.eval<number>("__runtime_probe__.tick()");
        check(before === 1, `${label}: behind the gate the game took its first tick (its first screen, UI included)`, before);
        await page.tap(640, 360);
        await wait(1500);
        const after = await page.eval<number>("__runtime_probe__.tick()");
        check(after > before + 30, `${label}: a tap on the gate starts the clock`, { before, after });
        const isolated = await page.eval<boolean>("crossOriginIsolated");
        const playShot = save(`${game}-${backend}-play.png`, await page.screenshot());
        const errors = log.filter((l) => /^(error|exception)/.test(l));
        check(errors.length === 0, `${label}: no console errors`, errors.slice(0, 5));
        results[label] = { ...t, isolated, ticksAfterTap: after - before, gate: gateShot, play: playShot };
      } catch (e) {
        check(false, `${label}: loads`, { error: String(e), log: log.slice(-20) });
      } finally {
        await page.send("Page.close").catch(() => {});
        server.stop();
      }
    }
  }

  // ── the runner over WebGPU: boot frames, manifest, offline, phone width ──
  {
    const dist = builds["runner"]!.dist;
    const server = serve({ dist, isolate: true, throttle: 1_200_000 });
    const log: string[] = [];
    const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log);
    const boot: Record<string, unknown> = {};
    try {
      await page.until("document.querySelector('.og-boot-mark')", 10_000);
      const motion0 = await page.eval<string>("document.querySelector('.og-boot-mark').dataset.motion");
      boot.splash = save("runner-boot-splash.png", await page.screenshot());
      check(motion0 === "genesis", "boot: the splash plays genesis", motion0);
      await wait(450);
      boot.genesis = save("runner-boot-genesis.png", await page.screenshot());
      const p = await page.until<{ motion: string; label: string; offset: string; now: string }>(
        "(() => { const m = document.querySelector('.og-boot-mark'); const v = Number(document.getElementById('og-boot').getAttribute('aria-valuenow')); return m.dataset.motion === 'progress' && v >= 20 && v <= 80 ? { motion: m.dataset.motion, label: document.getElementById('og-boot-label').textContent, offset: m.querySelector('[data-part=wipe]').style.strokeDashoffset, now: String(v) } : null })()",
        60_000,
      );
      boot.progress = save("runner-boot-progress.png", await page.screenshot());
      check(Number(p.offset) === 100 - Number(p.now) && p.label.includes(`${p.now}%`), "boot: progress shows the real value (wipe = 100 − percent)", p);
      boot.progressValue = p;
      await load(page);
      const madeWith = await page.eval<{ hidden: boolean; text: string }>("({ hidden: document.getElementById('og-made-with').hidden, text: document.getElementById('og-made-with').textContent })");
      check(!madeWith.hidden && madeWith.text.includes("Made with Origin Engine"), "the shell shows Made with Origin Engine", madeWith);
      const meta = await page.eval<string>("document.querySelector('meta[name=og-engine]').content");
      check(meta === builds["runner"]!.manifest.engine.version, "index.html carries <meta name=og-engine>", meta);
      // The PWA: manifest, icons, service worker.
      const wm = await page.eval<{ display: string; icons: Array<{ src: string; sizes: string }>; name: string }>("fetch('./manifest.webmanifest').then((r) => r.json())");
      check(wm.display === "standalone" && wm.icons.some((i) => i.sizes === "512x512"), "the web manifest is standalone with a 512 px icon", wm);
      const icons = await page.eval<number[]>(`Promise.all(${JSON.stringify(wm.icons.map((i) => i.src))}.map((s) => fetch(s).then((r) => r.status)))`);
      check(icons.every((s) => s === 200), "the manifest's icons are served", icons);
      const sw = await page.until<string>("navigator.serviceWorker.ready.then((r) => r.active && r.active.state)", 30_000);
      await page.until<boolean>("caches.keys().then((k) => k.some((n) => n.startsWith('og-')))", 30_000);
      const cached = await page.eval<number>("caches.keys().then((k) => caches.open(k.find((n) => n.startsWith('og-')))).then((c) => c.keys()).then((k) => k.length)");
      check(!!sw && cached > 10, "the service worker precached the build", { sw, cached });
      await page.send("Network.enable");
      await page.send("Network.emulateNetworkConditions", { offline: true, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
      await page.send("Page.reload", { ignoreCache: false });
      await wait(500);
      const offline = await load(page);
      check(offline.firstFrame > 0, "offline: the game starts from the service worker's cache", offline);
      boot.offline = { ...offline, shot: save("runner-offline.png", await page.screenshot()) };
      await page.send("Network.emulateNetworkConditions", { offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
    } catch (e) {
      check(false, "runner boot, PWA and offline", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
    results["runner boot"] = boot;
  }
  {
    const server = serve({ dist: builds["match3"]!.dist, isolate: false });
    const log: string[] = [];
    const page = await browser.open("about:blank", log);
    try {
      await page.send("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
      await page.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      await load(page);
      const size = await page.eval<{ cssW: number; cssH: number; w: number; h: number; iso: boolean; button: string }>(
        "({ cssW: document.getElementById('og-canvas').getBoundingClientRect().width, cssH: document.getElementById('og-canvas').getBoundingClientRect().height, w: document.getElementById('og-canvas').width, h: document.getElementById('og-canvas').height, iso: crossOriginIsolated, button: document.querySelector('#og-gate button').textContent })",
      );
      check(size.cssW === 390 && size.cssH === 844, "phone: the canvas fills 390×844", size);
      check(size.w === 780 && size.h === 1688, "phone: drawn at the DPR cap of 2 (780×1688), not 3", size);
      check(!size.iso && size.button === "Tap to play", "phone: without isolation, a touch gate", size);
      await wait(500);
      const hint = await page.eval<boolean>("document.getElementById('og-install').hidden");
      check(hint, "phone: no install hint where the criteria do not hold (Chrome made no install prompt; not iOS Safari)", hint);
      const gate = save("match3-phone-gate.png", await page.screenshot());
      await page.tap(195, 422);
      await wait(1500);
      const play = save("match3-phone-play.png", await page.screenshot());
      results["match3 phone"] = { ...size, gate, play };
    } catch (e) {
      check(false, "match3 at phone width", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
  }
} finally {
  await browser.close();
}

const sizes = Object.fromEntries(Object.entries(builds).map(([g, b]) => [g, { total: b.total, files: b.files.map((f) => ({ path: f.path, bytes: f.bytes, brotli: f.compressed })) }]));
writeFileSync(join(out, "export-result.json"), JSON.stringify({ chrome, failures, checks, results, sizes }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} web export: ${checks.length} checks, ${failures} failed; ${join(out, "export-result.json")}`);
process.exit(failures === 0 ? 0 : 1);
