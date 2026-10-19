// The Player wire, protocol 2 (origingame/docs/player-protocol.md), from a
// stand-in Player in headless Chrome: a page on the play origin frames the
// runner's publish build at the same origin, as play.<domain>/g/:slug frames a
// release, and speaks protocol 2 to it. Asserts: the game says `hello` with
// its substrate, reports `loading` values that only grow and are byte
// fractions, says `ready` after its first frame; it waits for `start` (its
// own gate stays down under a Player), plays on `start`, holds the clock on
// `pause` and plays again on `resume`; nothing is posted to another origin.
// og.platform through the Player: the runner's first tick asks the store for
// its best score; the shell holds that call until the Player's hello, then
// sends `store.get { key: "best" }` as a protocol 2 `call` (the keyed store
// the Player's hello lists), and the answer's value becomes the game's best
// (its `state()` facts).
//
// Then the same build as a top-level page on its play origin (the HTTP
// transport, origingame/docs/player-protocol.md §The top-level engine page):
// served at /play/<game>/<release>/ with the platform's `window.__OG_GAME`
// added to its entry and a stand-in play origin at /__player/v2/…, opened
// with a Portal hand-off code in the fragment. Asserts: the code leaves the
// address bar and reaches the play origin only in the hello's body; the
// account credential stays out of storage and rides the store.get call; the
// shell draws its own gate, the cover backdrop and Back; nothing is reported
// before the gate's tap, then `start` and `ready`; Back navigates to the
// game's page; screenshots at desktop and phone widths.
//
// Last, the same build as a top-level Stage build with the Stage overlay in
// its own document (origingame agent-server/src/stage/creator.ts
// `stageInPageSnippet`): served under a Stage ticket with
// `window.__OG_STAGE_HOST__` naming its own origin, no `__OG_GAME`, and a
// stand-in overlay script (deferred, before `</head>`, as the proxy injects
// the bundle) that answers Player protocol 2 in the same window as the
// platform's Stage overlay does (sdk/src/stage-page.ts: started, the scratch
// store). Asserts, at desktop and phone sizes: the shell's hello reaches the
// overlay, the overlay starts play (no gate, no tap), the overlay's stored
// best is the game's best, nothing of og.platform's is in localStorage, no
// service worker is installed on the Stage origin, and the og-probe wire
// still answers the overlay's `state` from the same window. With an overlay
// that never answers, the shell repeats its hello, draws its own gate and
// still keeps og.platform out of localStorage.
//
//   bun tests/web/player.ts [--out <result.json>] [--chrome <path>] [--evidence <dir>]

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { readHeaders, serve, TYPES } from "./serve.ts";

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "archetypes", "runner");
const work = join(pkg, "dist", "test-player");
const outFile = resolve(opt("--out") ?? join(work, "player-result.json"));
const b = await webBuild(project, join(project, "src", "main.ts"), join(work, "publish"), true, settingsOf(project));
if (!b.ok) throw new Error(b.reason);

const player = `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0">
<iframe id="g" src="/game/index.html" style="width:800px;height:450px;border:0"></iframe>
<script>
const g = document.getElementById("g");
window.got = [];
addEventListener("message", (e) => {
  if (e.source !== g.contentWindow || e.origin !== location.origin) return;
  const m = e.data;
  if (!m || m.og !== "player" || m.protocol !== 2) return;
  got.push({ ...m, at: Math.round(performance.now()) });
  if (m.kind === "hello") g.contentWindow.postMessage({ og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", capabilities: ["pause", "save.read", "save.set", "store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark"], started: false, paused: false }, location.origin);
  if (m.kind === "call") {
    const data = m.method === "store.get" ? { key: m.args.key, value: m.args.key === "best" ? 4321 : null, revision: m.args.key === "best" ? 3 : 0 } : {};
    g.contentWindow.postMessage({ og: "player", protocol: 2, kind: "result", id: m.id, ok: true, data }, location.origin);
  }
});
window.send = (kind) => g.contentWindow.postMessage({ og: "player", protocol: 2, kind }, location.origin);
</script></body></html>`;

const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
const check = (ok: boolean, what: string, detail?: unknown) => {
  checks.push({ check: what, ok, ...(ok ? {} : { detail }) });
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${ok ? "" : `: ${String(JSON.stringify(detail)).slice(0, 400)}`}`);
};
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const server = serve({ dist: join(work, "publish"), isolate: true, extra: { "/g/runner": player } });
const browser = await Browser.launch({ chrome: chromePath(args) });
let got: Array<{ kind: string; value?: number; substrate?: string; at: number }> = [];
try {
  const page = await browser.open(`http://127.0.0.1:${server.port}/g/runner`);
  await page.until("got.some((m) => m.kind === 'ready')", 60_000);
  await wait(800);
  got = await page.eval("got");
  const kinds = got.map((m) => m.kind);
  check(kinds[0] === "hello" && got[0]!.substrate === "origin-engine", "hello first, with substrate origin-engine", got[0]);
  const loading = got.filter((m) => m.kind === "loading").map((m) => m.value!);
  check(loading.length >= 3 && loading.every((v, i) => i === 0 || v > loading[i - 1]!) && loading.every((v) => v >= 0 && v <= 1), "loading values grow and stay in [0, 1]", loading);
  check(kinds.indexOf("ready") > kinds.lastIndexOf("loading"), "ready after the last loading", kinds);
  check(!kinds.includes("fail") && !kinds.includes("crash"), "no fail, no crash", kinds);
  const gate = await page.eval<boolean>("g.contentWindow.document.getElementById('og-gate').hidden");
  check(gate === true, "under a Player the shell draws no gate of its own", gate);
  const t0 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  await wait(400);
  const t1 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t0 === t1, "before start the clock holds", { t0, t1 });
  await page.eval("send('start')");
  await wait(700);
  const t2 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t2 > t1 + 20, "start plays", { t1, t2 });
  await page.eval("send('pause')");
  await wait(200);
  const t3 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  await wait(500);
  const t4 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t4 === t3, "pause holds the clock", { t3, t4 });
  await page.eval("send('resume')");
  await wait(600);
  const t5 = await page.eval<number>("g.contentWindow.__runtime_probe__.tick()");
  check(t5 > t4 + 20, "resume plays again", { t4, t5 });
  const calls = (await page.eval<Array<{ kind: string; method?: string; id?: string; args?: { key?: string } }>>("got")).filter((m) => m.kind === "call");
  check(calls.some((c) => c.method === "store.get" && c.args?.key === "best" && typeof c.id === "string"), "og.platform's store.get reached the Player as its store.get call", calls);
  check(!calls.some((c) => c.method?.startsWith("save.")), "with store.* in the hello, the save is never read", calls);
  const state = await page.eval<{ facts?: { best?: number } }>("g.contentWindow.__runtime_probe__.state()");
  const best = state.facts?.best;
  check(best === 4321, "the store's best is the game's best score", state);
  const sample = (await page.eval<typeof got>("got")).find((m) => m.kind === "sample") as { timings?: Record<string, number>; renderer?: string } | undefined;
  check(!!sample?.timings?.["firstFrame"] && sample.renderer === "webgpu", "a sample with the first-frame time and the renderer", sample);
} finally {
  await browser.close();
  server.stop();
}
// ── the top-level page over HTTP ──────────────────────────────────────
const GAME = { id: "runner01", releaseId: "release_page01" };
const ROOT = `/play/${GAME.id}/${GAME.releaseId}/`;
const CODE = "h".repeat(43);
const evidence = opt("--evidence");
type Req = { path: string; url: string; bearer: string | null; body: Record<string, unknown> | null };
const requests: Req[] = [];
const served: string[] = [];
const dist = join(work, "publish");
const rules = readHeaders(dist);
declare const Bun: { serve(o: { port: number; hostname: string; fetch(r: Request): Response | Promise<Response> }): { port: number; stop(): void }; file(p: string): Blob };
const json = (status: number, body: unknown) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", "cache-control": "no-store" } });
const origin = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const url = new URL(req.url);
    served.push(url.pathname + url.search);
    if (url.pathname.startsWith(`/__player/v2/${GAME.id}/${GAME.releaseId}/`)) {
      const what = url.pathname.split("/").at(-1)!;
      const body = (await req.json().catch(() => null)) as Record<string, unknown> | null;
      requests.push({ path: what, url: url.pathname + url.search, bearer: req.headers.get("authorization"), body });
      if (what === "hello") {
        return json(200, { og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", started: false, paused: false,
          capabilities: ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark"],
          ...(body?.["handoff"] === CODE ? { player: { token: "ogp_account_token", guest: false } } : {}),
          page: { title: "Runner", cover: `${url.origin}/cover.png`, backdrop: `${url.origin}/cover.png`, back: `${url.origin}/portal/games/runner`, portal: "https://portal.invalid" } });
      }
      if (what === "report") return json(202, { accepted: (body?.["messages"] as unknown[] | undefined)?.length ?? 0, rejected: 0 });
      const a = (body?.["args"] ?? {}) as Record<string, unknown>;
      const data = body?.["method"] === "store.get" ? { key: a["key"], value: a["key"] === "best" ? 777 : null, revision: a["key"] === "best" ? 2 : 0 } : {};
      return json(200, { og: "player", protocol: 2, kind: "result", id: body?.["id"], ok: true, data });
    }
    const isolation = { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "cross-origin" };
    if (url.pathname === "/portal/games/runner") return new Response("<!doctype html><title>Runner on the Portal</title><h1>game page</h1>", { headers: { "content-type": TYPES[".html"]! } });
    if (url.pathname === "/cover.png") return new Response(Bun.file(join(pkg, "shell", "brand", "icon-512.png")), { headers: { "content-type": "image/png", ...isolation } });
    if (!url.pathname.startsWith(ROOT)) return new Response("not found", { status: 404 });
    let rel = url.pathname.slice(ROOT.length - 1);
    if (rel.endsWith("/")) rel += "index.html";
    const path = join(dist, rel);
    if (!path.startsWith(dist) || !existsSync(path)) return new Response("not found", { status: 404 });
    const headers: Record<string, string> = { "cache-control": "no-store", ...isolation };
    for (const r of rules) if (r.pattern === rel || (r.pattern.startsWith("/*.") && rel.endsWith(r.pattern.slice(2)))) Object.assign(headers, Object.fromEntries(Object.entries(r.headers).map(([k, v]) => [k.toLowerCase(), v])));
    headers["content-type"] = TYPES[rel.slice(rel.lastIndexOf("."))] ?? "application/octet-stream";
    // Like the platform, the derived entry only to a document or frame load (a plain read gets the stored bytes).
    const opening = ["document", "iframe", "frame"].includes(req.headers.get("sec-fetch-dest") ?? "");
    if (rel === "/index.html" && opening) {
      // What the platform's publication adds to the entry (origingame server/src/play/entry.ts).
      const html = readFileSync(path, "utf8").replace(/<head[^>]*>/, (m) => `${m}<script id="__og-game-config">window.__OG_GAME=${JSON.stringify({ id: GAME.id, version: 1, releaseId: GAME.releaseId, protocol: 2 })}</script>`);
      return new Response(html, { headers });
    }
    // The engine binary arrives late, so the boot splash (and its backdrop) can be seen.
    if (rel.endsWith(".wasm")) await wait(2500);
    return new Response(Bun.file(path), { headers });
  },
});
const top: Record<string, unknown> = {};
const shots: string[] = [];
const save = (name: string, png: Uint8Array) => {
  if (!evidence) return;
  mkdirSync(evidence, { recursive: true });
  writeFileSync(join(evidence, name), png);
  shots.push(join(evidence, name));
};
for (const view of [{ name: "desktop", width: 1280, height: 720, mobile: false }, { name: "phone", width: 390, height: 844, mobile: true }]) {
  requests.length = 0;
  const browser2 = await Browser.launch({ chrome: chromePath(args), width: view.width, height: view.height });
  try {
    const page = await browser2.open("about:blank");
    if (view.mobile) {
      await page.send("Emulation.setDeviceMetricsOverride", { width: view.width, height: view.height, deviceScaleFactor: 3, mobile: true });
      await page.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    }
    await page.send("Page.navigate", { url: `http://127.0.0.1:${origin.port}${ROOT}?level=2#og-h=${CODE}&lvl=3` });
    await page.until("document.querySelector('#og-backdrop[data-shown]') && !document.getElementById('og-boot').hidden", 20_000).catch(() => null);
    await wait(400);
    const booting = await page.eval<{ boot: boolean; backdrop: boolean }>("({ boot: !document.getElementById('og-boot').hidden, backdrop: !!document.querySelector('#og-backdrop[data-shown]') })");
    check(booting.boot && booting.backdrop, `${view.name}: the boot splash stands on the cover's backdrop`, booting);
    save(`top-level-${view.name}-boot.png`, await page.screenshot());
    await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000);
    await page.until("document.querySelector('#og-backdrop[data-shown]')", 10_000).catch(() => null);
    const v = view.name;
    const address = await page.eval<string>("location.pathname + location.search + location.hash");
    check(address === `${ROOT}?level=2#lvl=3`, `${v}: the hand-off code left the address bar, the game's fragment stayed`, address);
    const hello = requests.find((r) => r.path === "hello");
    check(!!hello && hello.body?.["handoff"] === CODE && hello.bearer === null && hello.body?.["kind"] === "hello" && hello.body?.["substrate"] === "origin-engine", `${v}: the hello carried the code in its body and no bearer`, hello);
    check(!served.some((p) => p.includes(CODE)), `${v}: no URL the browser fetched names the code`, served.filter((p) => p.includes(CODE)));
    const storeGet = () => requests.find((r) => r.path === "call" && r.body?.["method"] === "store.get");
    for (let i = 0; i < 50 && !storeGet(); i++) await wait(100);
    const call = storeGet();
    check(call?.bearer === "Bearer ogp_account_token" && (call?.body?.["args"] as { key?: string })?.key === "best", `${v}: og.platform's store.get went to /__player/v2/…/call with the redeemed credential`, call ?? requests);
    const stored = await page.eval<Record<string, string | null>>(`({ token: localStorage.getItem("og_token_${GAME.id}"), portal: localStorage.getItem("og_portal_${GAME.id}"), any: Object.keys(localStorage).filter((k) => (localStorage.getItem(k) ?? "").includes("ogp_account_token")) })`);
    check(stored["token"] === null && stored["portal"] === "1" && (stored["any"] as unknown as string[]).length === 0, `${v}: the account credential is in no storage, the Player's marker is set`, stored);
    check(!requests.some((r) => r.path === "report"), `${v}: nothing is reported before the gate`, requests.map((r) => r.path));
    const chrome = await page.eval<{ back: boolean; backdrop: string | null; meta: string | null; iso: boolean }>(
      "({ back: !document.getElementById('og-back').hidden, backdrop: document.querySelector('#og-backdrop img')?.getAttribute('src') ?? null, meta: document.querySelector('meta[name=og-player]')?.content ?? null, iso: crossOriginIsolated })");
    check(chrome.back && !!chrome.backdrop?.endsWith("/cover.png") && chrome.meta === "frame page", `${v}: the shell draws Back and the cover's backdrop; the page says og-player frame page`, chrome);
    save(`top-level-${v}-gate.png`, await page.screenshot());
    if (view.mobile) await page.tap(view.width / 2, view.height / 2);
    else await page.click(view.width / 2, view.height / 2);
    await page.until("document.getElementById('og-back').hasAttribute('data-playing')", 10_000);
    const t1 = await page.eval<number>("__runtime_probe__.tick()");
    await wait(1500);
    const t2 = await page.eval<number>("__runtime_probe__.tick()");
    check(t2 > t1 + 20, `${v}: the gate's tap starts play`, { t1, t2 });
    const best = (await page.eval<{ facts?: { best?: number } }>("__runtime_probe__.state()")).facts?.best;
    check(best === 777, `${v}: the play origin's stored best is the game's best`, best);
    for (let i = 0; i < 40 && !requests.some((r) => r.path === "report"); i++) await wait(100);
    const reports = requests.filter((r) => r.path === "report").flatMap((r) => (r.body?.["messages"] as Array<Record<string, unknown>>) ?? []);
    const kinds = reports.map((m) => m["kind"]);
    check(kinds[0] === "start" && kinds.includes("ready") && reports.every((m) => /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(String(m["id"]))), `${v}: start is reported first, then ready, with UUID v7 ids`, kinds);
    save(`top-level-${v}-playing.png`, await page.screenshot());
    // The release's service worker is in charge of the next open.
    const worker = await Promise.race([page.eval<boolean>("navigator.serviceWorker.ready.then(() => true)"), wait(10_000).then(() => false)]);
    check(worker === true, `${v}: the release's service worker is active`, worker);
    await page.eval("document.getElementById('og-back').click()");
    await page.until("location.pathname === '/portal/games/runner'", 10_000).catch(() => null);
    const after = await page.eval<string>("location.pathname");
    check(after === "/portal/games/runner", `${v}: Back with no Portal referrer goes to the game's page`, after);
    // A second open in the same browser, now under the release's own service worker: the
    // navigation must still reach the play origin as a document (the derived entry, with
    // window.__OG_GAME), so the page speaks the transport again.
    await page.send("Page.close").catch(() => {});
    requests.length = 0;
    const again = await browser2.open("about:blank");
    if (view.mobile) {
      await again.send("Emulation.setDeviceMetricsOverride", { width: view.width, height: view.height, deviceScaleFactor: 3, mobile: true });
      await again.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    }
    const CODE2 = "k".repeat(43);
    await again.send("Page.navigate", { url: `http://127.0.0.1:${origin.port}${ROOT}?level=2#og-h=${CODE2}` });
    await again.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => null);
    const revisit = await again.eval<{ controlled: boolean; game: boolean; address: string }>("({ controlled: !!navigator.serviceWorker.controller, game: !!window.__OG_GAME, address: location.pathname + location.search + location.hash })");
    check(revisit.controlled && revisit.game, `${v}: the second open is under the service worker and still gets the platform's entry`, revisit);
    for (let i = 0; i < 50 && !requests.some((r) => r.path === "hello"); i++) await wait(100);
    const hello2 = requests.find((r) => r.path === "hello");
    check(hello2?.body?.["handoff"] === CODE2 && revisit.address === `${ROOT}?level=2`, `${v}: the second open redeems its own code and takes it out of the address`, { hello: hello2?.body, address: revisit.address });
    save(`top-level-${v}-revisit-gate.png`, await again.screenshot());
    top[v] = { address, best, reports: kinds, chrome, revisit };
  } finally {
    await browser2.close();
  }
}
origin.stop();

// ── the Stage overlay in the engine's own document ────────────────────
const STAGE_ROOT = "/__og/stage/tkt01/";
const stageServed: Array<{ path: string; worker: boolean }> = [];
// The stand-in overlay: one listener on the page's own window for Player protocol 2 (skipping
// the Player messages it posts itself) and a client of the og-probe wire.
const overlayScript = `(() => {
  window.stageGot = [];
  // ?silent: an overlay that hears but never answers (the shell must fall back to memory, not storage).
  const silent = new URLSearchParams(location.search).has("silent");
  const caps = ["save.get", "save.read", "save.set", "store.get", "store.set", "store.list"];
  const store = new Map([["best", { value: 555, revision: 4 }]]);
  let clock = 1800000000000;
  const post = (m) => window.postMessage({ og: "player", protocol: 2, ...m }, location.origin);
  addEventListener("message", (e) => {
    if (e.source !== window || e.origin !== location.origin) return;
    const m = e.data;
    if (!m || m.og !== "player" || m.protocol !== 2) return;
    if (["result", "start", "gesture", "pause", "resume"].includes(m.kind) || (m.kind === "hello" && Array.isArray(m.capabilities))) return;
    stageGot.push({ ...m, at: Math.round(performance.now()) });
    if (silent) return;
    if (m.kind === "hello") post({ kind: "hello", locale: "en", region: "global", capabilities: caps, started: true, paused: false });
    if (m.kind !== "call") return;
    const a = m.args || {};
    const s = store.get(a.key);
    let r;
    if (m.method === "store.get") r = { ok: true, data: { key: a.key, value: s ? s.value : null, revision: s ? s.revision : 0 } };
    else if (m.method === "store.set") { const revision = (s ? s.revision : 0) + 1; store.set(a.key, { value: a.value, revision }); r = { ok: true, data: { key: a.key, revision } }; }
    else if (m.method === "store.list") r = { ok: true, data: { keys: [...store].map(([key, v]) => ({ key, revision: v.revision })) } };
    else r = { ok: false, error: { code: "unavailable", message: "the Stage answers no " + m.method, retryable: false } };
    post({ kind: "result", id: m.id, ...r, time: (clock += 1000) });
  });
  const pending = new Map(); let n = 0;
  addEventListener("message", (e) => {
    if (e.source !== window || e.origin !== location.origin) return;
    const m = e.data;
    if (!m || m.og !== "probe" || m.v !== 1 || m.kind === "call" || m.kind === "hello") return;
    const p = pending.get(m.id); if (p) { pending.delete(m.id); p(m); }
  });
  window.wire = (method, params = {}) => new Promise((resolve) => { const id = "st" + (++n); pending.set(id, resolve);
    window.postMessage({ og: "probe", v: 1, kind: "call", id, method, params }, location.origin);
    setTimeout(() => { if (pending.delete(id)) resolve({ silent: true }); }, 8000); });
})();`;
const stage = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const url = new URL(req.url);
    const worker = req.headers.get("service-worker") === "script";
    stageServed.push({ path: url.pathname, worker });
    // The shared Stage origin installs no service worker (agent-server isServiceWorkerScript).
    if (worker) return new Response("no service worker on the Stage", { status: 404 });
    if (url.pathname === `${STAGE_ROOT}__og/creator.js`) return new Response(overlayScript, { headers: { "content-type": "text/javascript", "cache-control": "no-store", "cross-origin-resource-policy": "same-origin" } });
    if (!url.pathname.startsWith(STAGE_ROOT)) return new Response("not found", { status: 404 });
    let rel = url.pathname.slice(STAGE_ROOT.length - 1);
    if (rel.endsWith("/")) rel += "index.html";
    const path = join(dist, rel);
    if (!path.startsWith(dist) || !existsSync(path)) return new Response("not found", { status: 404 });
    const headers: Record<string, string> = { "cache-control": "no-store", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "same-origin" };
    for (const r of rules) if (r.pattern === rel || (r.pattern.startsWith("/*.") && rel.endsWith(r.pattern.slice(2)))) Object.assign(headers, Object.fromEntries(Object.entries(r.headers).map(([k, v]) => [k.toLowerCase(), v])));
    headers["content-type"] = TYPES[rel.slice(rel.lastIndexOf("."))] ?? "application/octet-stream";
    if (rel === "/index.html") {
      // What the Stage proxy injects before </head>: the Stage origin, then the overlay's bundle.
      const told = `<script>window.__OG_STAGE_HOST__=${JSON.stringify(url.origin)};</script><script src="${STAGE_ROOT}__og/creator.js" defer></script>`;
      return new Response(readFileSync(path, "utf8").replace(/<\/head\s*>/i, (m) => `${told}${m}`), { headers });
    }
    return new Response(Bun.file(path), { headers });
  },
});
const inStage: Record<string, unknown> = {};
for (const view of [{ name: "desktop", width: 1280, height: 720, mobile: false }, { name: "phone", width: 390, height: 844, mobile: true }]) {
  const v = `stage ${view.name}`;
  const browser3 = await Browser.launch({ chrome: chromePath(args), width: view.width, height: view.height });
  try {
    const page = await browser3.open("about:blank");
    if (view.mobile) {
      await page.send("Emulation.setDeviceMetricsOverride", { width: view.width, height: view.height, deviceScaleFactor: 3, mobile: true });
      await page.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    }
    await page.send("Page.navigate", { url: `http://127.0.0.1:${stage.port}${STAGE_ROOT}` });
    await page.until("window.stageGot && stageGot.some((m) => m.kind === 'ready') && typeof __runtime_probe__ === 'object'", 60_000);
    await wait(800);
    const heard = await page.eval<Array<{ kind: string; substrate?: string; method?: string; args?: { key?: string } }>>("stageGot");
    const kinds = heard.map((m) => m.kind);
    check(heard[0]?.kind === "hello" && heard[0].substrate === "origin-engine", `${v}: the shell's hello reaches the overlay in its own document`, heard[0]);
    check(kinds.includes("ready") && !kinds.includes("fail") && !kinds.includes("crash"), `${v}: ready reaches the overlay, no fail, no crash`, kinds);
    const told = await page.eval<{ host: string; origin: string; game: boolean; framed: boolean }>("({ host: window.__OG_STAGE_HOST__, origin: location.origin, game: '__OG_GAME' in window, framed: window.parent !== window })");
    check(told.host === told.origin && !told.game && !told.framed, `${v}: a top-level page, told its own origin, with no __OG_GAME`, told);
    const gate = await page.eval<boolean>("document.getElementById('og-gate').hidden");
    check(gate === true, `${v}: the overlay owns play start: the shell draws no gate`, gate);
    const t1 = await page.eval<number>("__runtime_probe__.tick()");
    await wait(1000);
    const t2 = await page.eval<number>("__runtime_probe__.tick()");
    check(t2 > t1 + 20, `${v}: play runs without a tap (the overlay's hello said started)`, { t1, t2 });
    const calls = heard.filter((m) => m.kind === "call");
    check(calls.some((c) => c.method === "store.get" && c.args?.key === "best"), `${v}: og.platform's store.get went to the overlay as a protocol 2 call`, calls);
    const best = (await page.eval<{ facts?: { best?: number } }>("__runtime_probe__.state()")).facts?.best;
    check(best === 555, `${v}: the overlay's stored best is the game's best`, best);
    const keys = await page.eval<string[]>("Object.keys(localStorage)");
    check(!keys.some((k) => k.startsWith("og.platform")), `${v}: nothing of og.platform's is in the Stage origin's localStorage`, keys);
    const workers = await page.eval<number>("navigator.serviceWorker.getRegistrations().then((r) => r.length)");
    check(workers === 0 && !stageServed.some((r) => r.worker || r.path.endsWith("/sw.js")), `${v}: no service worker is installed or fetched on the Stage origin`, { workers, fetched: stageServed.filter((r) => r.worker || r.path.endsWith("/sw.js")) });
    const st = await page.eval<{ kind?: string; result?: { tick?: number } }>("wire('state')");
    check(st.kind === "result" && typeof st.result?.tick === "number", `${v}: the og-probe wire still answers the overlay's state from the same window`, st);
    save(`stage-in-page-${view.name}-playing.png`, await page.screenshot());
    inStage[view.name] = { heard: kinds, best, t1, t2, keys };
  } catch (e) {
    check(false, `${v}: ran to the end`, String(e instanceof Error ? e.stack : e));
  } finally {
    await browser3.close();
  }
}
{
  // An overlay that never answers: the shell repeats its hello, then plays alone on this load's
  // memory with its own gate, and still writes nothing of og.platform's to the Stage origin.
  const v = "stage silent";
  const browser4 = await Browser.launch({ chrome: chromePath(args), width: 1280, height: 720 });
  try {
    const page = await browser4.open(`http://127.0.0.1:${stage.port}${STAGE_ROOT}?silent=1`);
    await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000);
    // The first tick's store.get (behind the gate) is answered once the 1.5 s hello wait ends.
    await wait(2000);
    const hellos = await page.eval<number>("stageGot.filter((m) => m.kind === 'hello').length");
    check(hellos >= 2 && hellos <= 6, `${v}: the shell repeats its hello (at most six) while no overlay answers`, hellos);
    const keys = await page.eval<string[]>("Object.keys(localStorage)");
    check(!keys.some((k) => k.startsWith("og.platform")), `${v}: og.platform falls back to the page's memory, never the Stage origin's localStorage`, keys);
    save("stage-in-page-silent-gate.png", await page.screenshot());
    inStage["silent"] = { hellos, keys };
  } catch (e) {
    check(false, `${v}: ran to the end`, String(e instanceof Error ? e.stack : e));
  } finally {
    await browser4.close();
  }
}
stage.stop();
if (evidence) writeFileSync(join(evidence, "stage-in-page-receipt.json"), JSON.stringify({ checks: checks.filter((c) => c.check.startsWith("stage ")), stage: inStage, shots: shots.filter((s) => s.includes("stage-in-page")) }, null, 1) + "\n");
if (evidence) writeFileSync(join(evidence, "top-level-receipt.json"), JSON.stringify({ checks: checks.filter((c) => c.check.startsWith("desktop") || c.check.startsWith("phone")), top, shots: shots.filter((s) => s.includes("top-level")) }, null, 1) + "\n");

const failures = checks.filter((c) => !c.ok).length;
mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, JSON.stringify({ failures, checks, messages: got }, null, 1) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} Player protocol 2: ${checks.length} checks, ${failures} failed; ${outFile}`);
process.exit(failures === 0 ? 0 : 1);
