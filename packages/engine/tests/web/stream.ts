// Streaming groups on the web, in headless Chrome: the three-group path of
// tests/asset-stream.test.ts built with `og build --publish`'s web half and
// played over WebGPU. The camera flies the path by itself (the staged
// main.ts).
//
// Ways this could fail: the boot fetches a group's pack or a grouped
// output's own file; a pack is fetched before its group is wanted, more
// than once, or never; a streamed Basis texture is not transcoded, so the
// hut stays grey (its proxy) or the group fails; the service worker
// precaches a pack, or does not keep one the engine fetched; offline, a
// group the player has been to does not load from the cache.
//
//   bun tests/asset-stream.test.ts   (stages and cooks the project first)
//   bun tests/web/stream.ts [--out <dir>] [--chrome <path>]
//
// Writes <out>/stream-result.json and PNGs (default
// target/evidence/asset-stream/web). Exits 1 when a check fails.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { DEFAULT_PUBLISH, settingsOf, webBuild } from "../../src/node/web-build.ts";
import { decodePng, pixel } from "../support/png.ts";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { serve } from "./serve.ts";

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..", "..", "..");
const project = join(repo, "target", "evidence", "asset-stream", "project");
const out = resolve(opt("--out") ?? join(repo, "target", "evidence", "asset-stream", "web"));
mkdirSync(out, { recursive: true });
if (!existsSync(join(project, "cooked", ".cook.ogt"))) throw new Error(`${project} is not staged: run \`bun test tests/asset-stream.test.ts\` in packages/engine first`);

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 400)}` : ""}`);
}
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

const dist = join(out, "dist");
const b = await webBuild(project, join(project, "main.ts"), dist, true, { ...settingsOf(project), publish: { ...DEFAULT_PUBLISH, gate: "none" } });
if (!b.ok) throw new Error(b.reason);
const packs = ["cave", "meadow", "tower"].map((g) => `/game/cooked/stream/${g}.ogb`);
const grouped = /\/cooked\/(models\/hut-[abc]\.glb\.(ogb|tex0\.ogb)|textures\/sign\.png\.ogb|audio\/blip\.wav\.ogb)$/;

/** Requests as the server saw them, with the last tick the page reported before each. */
const requests: Array<{ path: string; tick: number }> = [];
let tick = 0;
const server = serve({ dist, isolate: true, onRequest: (path) => requests.push({ path, tick }) });
const browser = await Browser.launch({ chrome: chromePath(args) });
const log: string[] = [];
const results: Record<string, unknown> = {};

/** The centre pixel's colour, as tests/asset-stream.test.ts reads it. */
function hue(png: Uint8Array): string {
  const img = decodePng(png);
  const [r, g, bl] = pixel(img, Math.floor(img.width / 2), Math.floor(img.height / 2));
  if (r > g + 60 && r > bl + 60) return "red";
  if (g > r + 60 && g > bl + 60) return "green";
  if (bl > r + 60 && bl > g + 60) return "blue";
  return "grey";
}

/** Waits for the page's tick to reach `t` (the camera at z = 20 - t / 2), then screenshots. */
async function at(page: Page, t: number, name: string): Promise<{ tick: number; hue: string; shot: string }> {
  for (;;) {
    tick = await page.eval<number>("__runtime_probe__.tick()");
    if (tick >= t) break;
    await wait(20);
  }
  const png = await page.screenshot();
  writeFileSync(join(out, `${name}.png`), png);
  return { tick, hue: hue(png), shot: `${name}.png` };
}

/** The path flown: the start, each zone with its group resident, and the gaps between. */
async function fly(page: Page, label: string): Promise<Record<string, { tick: number; hue: string; shot: string }>> {
  await page.until("window.__runtime_probe__", 60_000);
  return {
    start: await at(page, 2, `${label}-00-start`),
    meadow: await at(page, 120, `${label}-01-meadow`),
    cave: await at(page, 340, `${label}-02-cave`),
    tower: await at(page, 500, `${label}-03-tower`),
  };
}

try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/game/index.html`, log);
  const online = await fly(page, "online");
  results.online = online;
  check(online.start.hue === "grey", "the first frame draws the proxy before any group is wanted", online.start);
  check(online.meadow.hue === "red" && online.cave.hue === "green" && online.tower.hue === "blue", "each zone draws its group's own texture (transcoded from the pack) once it is resident", online);
  const boot = requests.filter((r) => r.tick < 50);
  check(!boot.some((r) => r.path.includes("/cooked/stream/") || grouped.test(r.path)), "the boot fetches no pack and no grouped output's own file", boot.map((r) => r.path));
  check(!requests.some((r) => grouped.test(r.path)), "no grouped output's own file is ever fetched", requests.filter((r) => grouped.test(r.path)));
  const fetched = packs.map((p) => requests.filter((r) => r.path === p));
  check(fetched.every((f) => f.length === 1), "each pack is fetched once, when its group loads", fetched);
  // meadow is wanted from z = -10 (tick 60), cave from z = -140 (tick 320), tower from z = -210 (tick 460).
  const [cave, meadow, tower] = fetched.map((f) => f[0]?.tick ?? -1);
  check(meadow! >= 50 && cave! >= 300 && tower! >= 440, "no pack is fetched before its trigger wants its group", { meadow, cave, tower });
  // The service worker: the first group precached, the packs cached as they were fetched.
  await page.until<string>("navigator.serviceWorker.ready.then((r) => r.active && r.active.state)", 30_000);
  const cached = await page.eval<string[]>("caches.keys().then((k) => caches.open(k.find((n) => n.startsWith('og-')))).then((c) => c.keys()).then((k) => k.map((r) => new URL(r.url).pathname))");
  results.cached = cached;
  check(packs.every((p) => cached.includes(p)), "the service worker keeps every pack the engine fetched", cached.filter((c) => c.includes("/stream/")));
  check(!cached.some((c) => grouped.test(c)), "the service worker holds no grouped output's own file", cached.filter((c) => grouped.test(c)));
  const precached = (JSON.parse(/^const OG_SW=(.*);$/m.exec(await (await fetch(`http://127.0.0.1:${server.port}/game/sw.js`)).text())![1]!) as { files: string[] }).files;
  check(!precached.some((f) => f.includes("stream/")), "sw.js precaches no pack", precached.filter((f) => f.includes("stream/")));
  // Offline: the server is gone (the page's emulated offline does not cover
  // the service worker's own fetches), and the reload plays the path again,
  // every group from the cache.
  server.stop();
  await page.send("Network.enable");
  await page.send("Network.emulateNetworkConditions", { offline: true, latency: 0, downloadThroughput: -1, uploadThroughput: -1 });
  const before = requests.length;
  tick = 0;
  await page.send("Page.reload", { ignoreCache: false });
  await wait(500);
  const offline = await fly(page, "offline");
  results.offline = offline;
  check(offline.meadow.hue === "red" && offline.cave.hue === "green" && offline.tower.hue === "blue", "offline, every group the player has been to loads from the service worker's cache", offline);
  check(requests.length === before, "offline, nothing reached the server", requests.slice(before));
  const errors = log.filter((l) => /^(error|exception)/.test(l));
  check(errors.length === 0, "no console errors", errors.slice(0, 5));
  await page.send("Page.close").catch(() => {});
} catch (e) {
  check(false, "the stream path plays in Chrome", { error: String(e), log: log.slice(-20) });
} finally {
  await browser.close();
  server.stop();
}

results.requests = requests.filter((r) => r.path.includes("/cooked/"));
results.build = { stream: b.manifest!.stream, total: b.total };
writeFileSync(join(out, "stream-result.json"), JSON.stringify({ failures, checks, results }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} web streaming groups: ${checks.length} checks, ${failures} failed; ${join(out, "stream-result.json")}`);
process.exit(failures === 0 ? 0 : 1);
