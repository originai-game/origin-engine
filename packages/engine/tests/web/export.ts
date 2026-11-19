// The web export end to end in headless Chrome: both archetypes built with
// `og build --publish`'s web half, served with the build's own `_headers`,
// loaded on WebGPU.
//
// Per archetype and backend it records the first-frame time (the shell's
// `og:first-frame` mark, ms since navigation), the tier the engine reports,
// and a frame after a real tap on the play gate. On the runner over WebGPU
// it also: streams the wasm and cooked/ slowly so the boot can be seen, and
// screenshots the splash (genesis) and a progress frame with its real value
// (the page's main thread asks for no adapter, which would hold genesis);
// checks og-build.json against the files served; checks the web manifest and
// its icons; registers the service worker and reloads offline (the game must
// start from the cache); and lays the page out at phone width (390×844, DPR
// 3: the canvas fills the viewport and is drawn at the DPR cap of 2).
// Lanes: served with the build's COOP/COEP the page is cross-origin isolated
// and boots the lanes variant (`__runtime_probe__.lanes.on`, Workers beside
// the engine's), offline too from the service worker's precache; served
// without them it runs the single-threaded engine and says why; served
// isolated under a strict Content-Security-Policy (no inline script or
// style, no `blob:` Worker) the shell is styled, its gate works and the
// lanes pool starts from its own file; under one that refuses the pool's
// script it falls back to the single-threaded engine within the pool's
// deadline and says the lanes variant did not start. A 3D build of cooked
// Basis KTX2 textures (enough for the boot's transcoding pool) boots under
// the strict policy with every texture transcoded: the transcoder is
// imported, never evaluated from text, so no 'unsafe-eval'.
// Both archetypes are 2D games, so their lanes pages run the 2D lanes module.
//
// With --safari (webdriver.ts) the same checks run in this Mac's Safari:
// its document-start scripts come from the test server (serve.ts
// `preloads`), offline is the server stopped (WebDriver cannot take the
// network away), and the phone page is Safari's window at 390×844 at the
// display's own DPR, with a mouse (no touch emulation on a Mac).
//
//   bun tests/web/export.ts [--out <dir>] [--chrome <path> | --safari [<safaridriver>]]
//
// Writes <out>/export-result.json and PNGs (default dist/test-export/out).
// Exits 1 when a check fails.

import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, sizesOf, webBuild, type BuildManifest } from "../../src/node/web-build.ts";
import { Browser, chromePath, type Page as CdpPage } from "./cdp.ts";
import { Safari, safariDriverPath, type SafariPage } from "./webdriver.ts";
import { serve } from "./serve.ts";
import { LANES_POOL_DEADLINE_MS } from "../../src/web/host.ts";
import { cookProject } from "../support/og-bin.ts";

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
const safari = args.includes("--safari") ? safariDriverPath(args) : undefined;
const chrome = safari ? "" : chromePath(args);
type Page = CdpPage | SafariPage;

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

// ── the builds ─────────────────────────────────────────────────────────
const builds: Record<string, { dist: string; manifest: BuildManifest; total: { bytes: number; compressed: number }; files: Array<{ path: string; bytes: number; compressed: number }> }> = {};
for (const game of ["runner", "match3"]) {
  const project = join(repo, "tests", "fixtures", game === "runner" ? "browser-host" : "sprite-2d");
  cookProject(project, repo);
  const dist = join(work, game);
  const b = await webBuild(project, join(project, "src", "main.ts"), dist, true, settingsOf(project));
  if (!b.ok) throw new Error(b.reason);
  builds[game] = { dist, manifest: b.manifest!, ...(await sizesOf(dist, true)) };
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
  // Both archetypes are 2D: the page boots the 2D lanes module, the one
  // beside the 2D publish module (owed row 71, docs/web-export.md §Variants).
  const engineDir = join(pkg, "dist", m.variant === "2d" ? "web-publish-2d" : "web-publish");
  // A publish build serves the engine by content-hashed names (m.names).
  const same = (f: string) => {
    const [x, y] = [new Uint8Array(readFileSync(join(dist, m.names[f] ?? f))), new Uint8Array(readFileSync(join(engineDir, f)))];
    return x.length === y.length && x.every((v, i) => v === y[i]);
  };
  check(m.game.profile === "2d" && m.variant === "2d" && m.lanes !== null && same("origin-engine-lanes_bg.wasm") && same("origin-engine-lanes.js"), `${game}: a 2d game ships the 2D lanes module (dist/web-publish-2d)`, { profile: m.game.profile, variant: m.variant, lanes: m.lanes });
}

// A 3D game of cooked Basis textures (cook-assets' stone, ETC1S colour and UASTC normal): 24
// colour copies, so the boot transcodes them on its pool of Workers (boot.ts POOL_MIN_JOBS).
// Kept out of the size table: brotli at quality 11 over the 3D engine takes minutes.
const TEXTURES = 25;
const texturesDist = join(work, "textures");
{
  const project = join(work, "textures-project");
  rmSync(project, { recursive: true, force: true });
  mkdirSync(join(project, "assets", "textures"), { recursive: true });
  mkdirSync(join(project, "src"), { recursive: true });
  const stone = join(repo, "tests", "fixtures", "cook-assets", "assets", "textures");
  const ids = ["textures/stone_n.png"];
  for (const f of ["stone_n.png", "stone_n.png.ogt"]) cpSync(join(stone, f), join(project, "assets", "textures", f));
  for (let i = 0; i < TEXTURES - 1; i++) {
    cpSync(join(stone, "stone.png"), join(project, "assets", "textures", `stone${i}.png`));
    ids.push(`textures/stone${i}.png`);
  }
  writeFileSync(join(project, "project.ogt"), 'type project\nversion 1\n---\nname "Basis textures"\nid "basis-textures"\nengine "0.9.0"\nentry "src/main.ts"\nprofile "3d"\n');
  // The game loads them all at its start; its facts report the GPU bytes the renderer's
  // textures hold. A texture whose transcode is missing would be a 1×1 placeholder.
  writeFileSync(
    join(project, "src", "main.ts"),
    [
      `import { og } from "@origingame/engine";`,
      `const IDS = ${JSON.stringify(ids)};`,
      `export default og.game({`,
      `  start(world) { for (const id of IDS) { const t = og.load.texture(world, id); if (!t.ok) throw new Error(t.reason); } },`,
      `  facts(world) { const d = og.render.display(world); return { textures: d && d.gpu ? d.gpu.textures : -1 }; },`,
      `});`,
      ``,
    ].join("\n"),
  );
  cookProject(project, repo);
  const b = await webBuild(project, join(project, "src", "main.ts"), texturesDist, true, settingsOf(project));
  if (!b.ok) throw new Error(b.reason);
}

// The ticks behind the gate: the first (it builds the first screen), then one per round of
// og.platform answers the Worker applies before the tap (src/web/settle.ts). The runner asks the
// store for its best on its first tick, so a second tick applies the answer; match3 asks nothing.
const BEHIND_GATE: Record<string, number> = { runner: 2, match3: 1 };

// What the test sees of Chrome's install prompt, from the document's start: whether Chrome
// fired beforeinstallprompt, and the prompt() calls the page made on it.
const INSTALL_PROBE = "window.__ogTest = { prompted: false, prompts: 0 }; addEventListener('beforeinstallprompt', () => { __ogTest.prompted = true; }); if (window.BeforeInstallPromptEvent) { const p = BeforeInstallPromptEvent.prototype.prompt; BeforeInstallPromptEvent.prototype.prompt = function () { __ogTest.prompts++; const r = p.call(this); r.catch(() => {}); return r; }; }";
// The page's own requestAdapter calls (an init script reaches the page, not its Workers): Chrome's
// first one blocks the main thread while WebGPU starts, seconds on a busy GPU process, which held
// the splash past genesis (src/web/webgpu.ts). The engine Worker asks instead.
const ADAPTERS = "window.__ogAdapters = 0; if (window.GPU) { const r = GPU.prototype.requestAdapter; GPU.prototype.requestAdapter = function (...a) { __ogAdapters++; return r.apply(this, a); }; }";
const HINT = "fetch('./manifest.webmanifest').then((r) => r.json()).then((m) => ({ ...__ogTest, offer: `Install ${m.short_name} to play offline`, hidden: document.getElementById('og-install').hidden, text: document.querySelector('#og-install span').textContent, button: document.querySelector('#og-install button').hidden ? null : document.querySelector('#og-install button').textContent }))";
type Hint = { prompted: boolean; prompts: number; hidden: boolean; text: string; offer: string; button: string | null };

// A platform page's Content-Security-Policy: no inline script or style, no `blob:` Worker
// (docs/web-export.md §Hosting); `connect-src 'self'` where the platform names its content base.
const STRICT_CSP = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
// The page's own CSP violations, from the document's start (a Worker's are seen by what it starts).
const VIOLATIONS = "window.__ogViolations = []; addEventListener('securitypolicyviolation', (e) => __ogViolations.push(`${e.effectiveDirective} ${e.blockedURI}`));";
const results: Record<string, unknown> = {};
const browser = safari ? await Safari.launch({ driver: safari }) : await Browser.launch({ chrome });
const browserName = safari ? `Safari ${(browser as Safari).capabilities["browserVersion"]}` : chrome;

type Lanes = { on: boolean; threads: number; why: string };
async function load(page: Page): Promise<{ firstFrame: number; ready: number; tier: string; lanes: Lanes }> {
  await page.until("window.__runtime_probe__", 60_000);
  const firstFrame = await page.until<number>("performance.getEntriesByName('og:first-frame')[0]?.startTime", 30_000);
  const ready = await page.eval<number>("performance.getEntriesByName('og:ready')[0]?.startTime ?? 0");
  const hello = await page.eval<{ tier: string }>("__runtime_probe__.hello()");
  const lanes = await page.eval<Lanes>("__runtime_probe__.lanes");
  return { firstFrame: Math.round(firstFrame), ready: Math.round(ready), tier: hello.tier, lanes };
}

try {
  for (const game of ["runner", "match3"]) {
    for (const backend of ["webgpu"] as const) {
      const label = `${game} ${backend}`;
      const server = serve({ dist: builds[game]!.dist, isolate: true });
      const log: string[] = [];
      const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log);
      try {
        const t = await load(page);
        check(t.tier === backend, `${label}: the engine reports tier ${backend}`, t);
        check(t.firstFrame > 0 && t.firstFrame < 10_000, `${label}: first frame at ${t.firstFrame} ms after navigation`, t);
        const gate = await page.eval<boolean>("!document.getElementById('og-gate').hidden");
        check(gate, `${label}: the play gate is up after the first frame`);
        const gateShot = save(`${game}-${backend}-gate.png`, await page.screenshot());
        // Sampled once the clock holds: the settling ticks may still land after the first frame.
        const before = await page.held<number>("__runtime_probe__.tick()", `${label}'s tick behind the gate`);
        check(before === BEHIND_GATE[game], `${label}: behind the gate the game took its first tick (its first screen, UI included) and ${BEHIND_GATE[game]! - 1} to apply the store's answers, then held`, before);
        await page.tap(640, 360);
        const after = await page.until<number>(`__runtime_probe__.tick().then((t) => t > ${before + 30} && t)`, 30_000).catch((e: Error) => e.message);
        check(typeof after === "number", `${label}: a tap on the gate starts the clock`, { before, after });
        const isolated = await page.eval<boolean>("crossOriginIsolated");
        check(isolated && t.lanes.on && t.lanes.threads > 1, `${label}: isolated, the engine runs the lanes variant on ${t.lanes.threads} Workers`, t.lanes);
        const playShot = save(`${game}-${backend}-play.png`, await page.screenshot());
        const errors = log.filter((l) => /^(error|exception)/.test(l));
        check(errors.length === 0, `${label}: no console errors`, errors.slice(0, 5));
        results[label] = { ...t, isolated, ticksAfterTap: Number(after) - before, gate: gateShot, play: playShot };
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
    const page = await browser.open("about:blank", log);
    const boot: Record<string, unknown> = {};
    try {
      await page.send("Page.addScriptToEvaluateOnNewDocument", { source: INSTALL_PROBE });
      await page.send("Page.addScriptToEvaluateOnNewDocument", { source: ADAPTERS });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      await page.until("document.querySelector('.og-boot-mark')", 10_000);
      const motion0 = await page.eval<string>("document.querySelector('.og-boot-mark').dataset.motion");
      boot.splash = save("runner-boot-splash.png", await page.screenshot());
      check(motion0 === "genesis", "boot: the splash plays genesis", motion0);
      // genesis part-way: its animations 450 ms in (or already handed to progress).
      await page.until("document.querySelector('.og-boot-mark').dataset.motion !== 'genesis' || document.querySelector('.og-boot-mark').getAnimations({ subtree: true }).some((a) => a.currentTime >= 450)", 30_000);
      boot.genesis = save("runner-boot-genesis.png", await page.screenshot());
      const p = await page.until<{ motion: string; label: string; offset: string; now: string }>(
        "(() => { const m = document.querySelector('.og-boot-mark'); const v = Number(document.getElementById('og-boot').getAttribute('aria-valuenow')); return m.dataset.motion === 'progress' && v >= 20 && v <= 80 ? { motion: m.dataset.motion, label: document.getElementById('og-boot-label').textContent, offset: m.querySelector('[data-part=wipe]').style.strokeDashoffset, now: String(v) } : null })()",
        60_000,
      );
      boot.progress = save("runner-boot-progress.png", await page.screenshot());
      check(Number(p.offset) === 100 - Number(p.now) && p.label.includes(`${p.now}%`), "boot: progress shows the real value (wipe = 100 − percent)", p);
      boot.progressValue = p;
      await load(page);
      const adapters = await page.eval<number>("__ogAdapters");
      check(adapters === 0, "boot: the page's main thread asks for no WebGPU adapter (the engine Worker does)", adapters);
      const madeWith = await page.eval<{ hidden: boolean; text: string }>("({ hidden: document.getElementById('og-made-with').hidden, text: document.getElementById('og-made-with').textContent })");
      check(!madeWith.hidden && madeWith.text.includes("Made with Origin Engine"), "the shell shows Made with Origin Engine", madeWith);
      const meta = await page.eval<string>("document.querySelector('meta[name=og-engine]').content");
      check(meta === builds["runner"]!.manifest.engine.version, "index.html carries <meta name=og-engine>", meta);
      // The PWA: manifest, icons, service worker.
      const wm = await page.eval<{ display: string; icons: Array<{ src: string; sizes: string }>; name: string }>("fetch('./manifest.webmanifest').then((r) => r.json())");
      check(wm.display === "standalone" && wm.icons.some((i) => i.sizes === "512x512"), "the web manifest is standalone with a 512 px icon", wm);
      const icons = await page.eval<number[]>(`Promise.all(${JSON.stringify(wm.icons.map((i) => i.src))}.map((s) => fetch(s).then((r) => r.status)))`);
      check(icons.every((s) => s === 200), "the manifest's icons are served", icons);
      // Desktop Chrome prompts (it did before the shell booted): the hint offers Install, and its
      // button calls the prompt the page kept. Where Chrome makes no prompt, no hint.
      const prompted = await page.eval<boolean>("__ogTest.prompted");
      const hint = await page.until<Hint>(`${HINT}.then((h) => h.hidden !== h.prompted && h)`, 30_000).catch(() => page.eval<Hint>(HINT));
      check(prompted ? !hint.hidden && hint.text === hint.offer && hint.button === "Install" : hint.hidden, `desktop: ${prompted ? "Chrome prompted: the install hint names the game and offers Install" : "Chrome made no install prompt: no hint"}`, hint);
      if (prompted && !hint.hidden) {
        boot.installHint = save("runner-install-hint.png", await page.screenshot());
        const at = await page.eval<{ x: number; y: number }>("(() => { const r = document.querySelector('#og-install button').getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()");
        await page.click(at.x, at.y);
        const after = await page.until<Hint>(`${HINT}.then((h) => h.prompts > 0 && h)`, 30_000).catch(() => page.eval<Hint>(HINT));
        check(after.prompts === 1 && after.hidden, "desktop: the hint's Install button shows Chrome's install prompt and hides the hint", after);
      }
      const sw = await page.until<string>("navigator.serviceWorker.ready.then((r) => r.active && r.active.state)", 30_000);
      await page.until<boolean>("caches.keys().then((k) => k.some((n) => n.startsWith('og-')))", 30_000);
      const cached = await page.eval<number>("caches.keys().then((k) => caches.open(k.find((n) => n.startsWith('og-')))).then((c) => c.keys()).then((k) => k.length)");
      check(!!sw && cached > 10, "the service worker precached the build", { sw, cached });
      // Offline: Chrome takes the network away; Safari's WebDriver cannot, so its server stops.
      if (safari) server.stop(true);
      else {
        await page.send("Network.enable");
        await page.send("Network.emulateNetworkConditions", { offline: true, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
      }
      // The reloaded document is the one without the mark.
      await page.eval("window.__ogBeforeReload = true");
      await page.send("Page.reload", { ignoreCache: false });
      await page.until("document.readyState !== 'loading' && !window.__ogBeforeReload", 30_000);
      const offline = await load(page);
      check(offline.firstFrame > 0, "offline: the game starts from the service worker's cache", offline);
      check(offline.lanes.on, "offline: the lanes variant boots from the service worker's precache", offline.lanes);
      boot.offline = { ...offline, shot: save("runner-offline.png", await page.screenshot()) };
      if (!safari) await page.send("Network.emulateNetworkConditions", { offline: false, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
    } catch (e) {
      check(false, "runner boot, PWA and offline", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
    results["runner boot"] = boot;
  }
  // ── a strict CSP (a platform page's): no inline script or style, no blob: Workers ──
  {
    const server = serve({ dist: builds["runner"]!.dist, isolate: true, headers: { "content-security-policy": STRICT_CSP } });
    const log: string[] = [];
    const page = await browser.open("about:blank", log);
    try {
      await page.send("Page.addScriptToEvaluateOnNewDocument", { source: VIOLATIONS });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      const t = await load(page);
      const isolated = await page.eval<boolean>("crossOriginIsolated");
      check(isolated && t.lanes.on && t.lanes.threads > 1, `strict csp: isolated, the lanes pool starts from its own file on ${t.lanes.threads} Workers`, t.lanes);
      // The shell's CSS applied: the gate is a layer over the viewport, not laid out below the page.
      const gate = await page.eval<{ hidden: boolean; position: string; top: number; bottom: number; height: number }>(
        "(() => { const g = document.getElementById('og-gate'); const r = g.getBoundingClientRect(); return { hidden: g.hidden, position: getComputedStyle(g).position, top: r.top, bottom: r.bottom, height: innerHeight }; })()",
      );
      // Unstyled, it is `static` and laid out below the page (R6: y=975 in a 540 px viewport).
      check(!gate.hidden && gate.position !== "static" && gate.top >= 0 && gate.bottom <= gate.height, "strict csp: the shell's CSS applies and the play gate covers the viewport", gate);
      const before = await page.held<number>("__runtime_probe__.tick()", "strict csp's tick behind the gate");
      await page.tap(640, 360);
      const after = await page.until<number>(`__runtime_probe__.tick().then((t) => t > ${before + 30} && t)`, 30_000).catch((e: Error) => e.message);
      check(typeof after === "number", "strict csp: a tap on the gate starts the clock", { before, after });
      const refused = await page.eval<string[]>("__ogViolations");
      const csp = log.filter((l) => /Content Security Policy|Refused to/.test(l));
      check(refused.length === 0 && csp.length === 0, "strict csp: the page breaks no directive", { refused, csp: csp.slice(0, 5) });
      results["runner strict csp"] = { ...t, isolated, shot: save("runner-strict-csp.png", await page.screenshot()) };
    } catch (e) {
      check(false, "runner under a strict csp", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
  }
  // ── Basis textures under the strict CSP: the transcoder loads without 'unsafe-eval' ──
  {
    const server = serve({ dist: texturesDist, isolate: true, headers: { "content-security-policy": STRICT_CSP } });
    const log: string[] = [];
    const page = await browser.open("about:blank", log);
    try {
      await page.send("Page.addScriptToEvaluateOnNewDocument", { source: VIOLATIONS });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      // The boot transcodes the bundle's textures before the game starts: a transcoder that
      // cannot load fails the boot ("The game could not start"), so the first frame is the test.
      const t = await load(page);
      check(t.firstFrame > 0, `strict csp: a 3D page of ${TEXTURES} Basis textures boots to its first frame`, t);
      // Each 512×512 texture with its mips takes at least 4/3 of a byte a pixel in any format it
      // is transcoded to; the renderer's own textures without them take about 5.8 MB.
      const want = Math.round((TEXTURES * 512 * 512 * 4) / 3);
      const textures = await page.until<number>(`__runtime_probe__.state().then((s) => s.facts && s.facts.textures >= ${want} && s.facts.textures)`, 30_000).catch(() => page.eval<{ facts?: { textures?: number } }>("__runtime_probe__.state()").then((s) => s.facts?.textures ?? -1));
      check(textures >= want, `strict csp: the GPU holds the ${TEXTURES} textures transcoded (${textures} ≥ ${want} bytes)`, { textures, log: log.filter((l) => /basis|transcod|eval/i.test(l)).slice(0, 5) });
      const refused = await page.eval<string[]>("__ogViolations");
      const csp = log.filter((l) => /Content Security Policy|Refused to|unsafe-eval/.test(l));
      check(refused.length === 0 && csp.length === 0, "strict csp: Basis textures break no directive", { refused, csp: csp.slice(0, 5) });
      results["textures strict csp"] = { ...t, textures, shot: save("textures-strict-csp.png", await page.screenshot()) };
    } catch (e) {
      check(false, "Basis textures under a strict csp", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
  }
  // ── lanes refused by the host's CSP: the single-threaded engine, in time ──
  {
    // Only the engine Worker's own script may start: the pool's helper is refused.
    const csp = `worker-src 127.0.0.1:*/game/${builds["runner"]!.manifest.worker}`;
    const server = serve({ dist: builds["runner"]!.dist, isolate: true, headers: { "content-security-policy": csp } });
    const log: string[] = [];
    const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log);
    try {
      const t = await load(page);
      const isolated = await page.eval<boolean>("crossOriginIsolated");
      check(isolated && !t.lanes.on && t.lanes.threads === 1 && /the lanes variant did not start/.test(t.lanes.why), "csp refusing the pool: isolated, the pool's Workers are refused and the single-threaded engine says the lanes variant did not start", t.lanes);
      // The deadline bounds the wait for the pool; the rest of the boot gets as long again.
      check(t.ready > 0 && t.ready < 2 * LANES_POOL_DEADLINE_MS, `csp refusing the pool: ready at ${t.ready} ms, within the pool's deadline (${LANES_POOL_DEADLINE_MS} ms) and a boot`, t);
      check(t.firstFrame > 0, "csp refusing the pool: the single-threaded engine draws its first frame", t);
      results["runner csp"] = { ...t, isolated, shot: save("runner-csp.png", await page.screenshot()) };
    } catch (e) {
      check(false, "runner with the pool refused", { error: String(e), log: log.slice(-20) });
    } finally {
      await page.send("Page.close").catch(() => {});
      server.stop();
    }
  }
  {
    const server = serve({ dist: builds["match3"]!.dist, isolate: false });
    const log: string[] = [];
    const page = await browser.open("about:blank", log);
    try {
      await page.send("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
      await page.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
      await page.send("Page.addScriptToEvaluateOnNewDocument", { source: INSTALL_PROBE });
      await page.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/game/index.html` });
      const phone = await load(page);
      check(!phone.lanes.on && /not cross-origin isolated/.test(phone.lanes.why), "phone: without isolation, the single-threaded engine says why", phone.lanes);
      const size = await page.eval<{ cssW: number; cssH: number; w: number; h: number; iso: boolean; button: string; dpr: number; coarse: boolean }>(
        "({ cssW: document.getElementById('og-canvas').getBoundingClientRect().width, cssH: document.getElementById('og-canvas').getBoundingClientRect().height, w: document.getElementById('og-canvas').width, h: document.getElementById('og-canvas').height, iso: crossOriginIsolated, button: document.querySelector('#og-gate button').textContent, dpr: devicePixelRatio, coarse: matchMedia('(pointer: coarse)').matches })",
      );
      check(size.cssW === 390 && size.cssH === 844, "phone: the canvas fills 390×844", size);
      // Chrome emulates DPR 3; Safari keeps the display's.
      const scale = Math.min(size.dpr, 2);
      check(size.w === 390 * scale && size.h === 844 * scale, `phone: drawn at DPR ${size.dpr} capped at 2 (${390 * scale}×${844 * scale})`, size);
      check(!size.iso && size.button === (size.coarse ? "Tap to play" : "Play"), `phone: without isolation, a ${size.coarse ? "touch" : "pointer"} gate`, size);
      // A phone is no different (docs/web-export.md §The shell): where Chrome prompts, the hint
      // offers Install over the gate; where it does not, no hint (and this is not iOS Safari).
      const install = await page.until<Hint>(`${HINT}.then((h) => h.hidden !== h.prompted && h)`, 30_000).catch(() => page.eval<Hint>(HINT));
      check(install.prompted ? !install.hidden && install.text === install.offer && install.button === "Install" : install.hidden, `phone: ${install.prompted ? "Chrome prompted: the install hint names the game and offers Install" : "Chrome made no install prompt: no hint"}`, install);
      const gate = save("match3-phone-gate.png", await page.screenshot());
      const before = await page.eval<number>("__runtime_probe__.tick()");
      await page.tap(195, 422);
      await page.until(`__runtime_probe__.tick().then((t) => t > ${before + 30})`, 30_000);
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
writeFileSync(join(out, "export-result.json"), JSON.stringify({ browser: browserName, failures, checks, results, sizes }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} web export: ${checks.length} checks, ${failures} failed; ${join(out, "export-result.json")}`);
process.exit(failures === 0 ? 0 : 1);
