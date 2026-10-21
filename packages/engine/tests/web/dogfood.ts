// The web dogfood round (wave 12): a game published as its author would
// (`og build --publish`, the og binary, the project's own profile), served
// the way the platform opens a release (the top-level page on its play
// origin with `window.__OG_GAME` in its entry and a stand-in play origin at
// /__player/v2/…, the Player protocol 2 HTTP transport, docs/web-export.md
// §The top-level page; the build's own `_headers`, so the page is
// cross-origin isolated and the service worker registers) and played in
// headless Chrome on one tier (WebGPU, or the WebGL2 fallback with
// --webgl, the shell's `?webgl=1`), through real input on the canvas, the
// accessibility mirror (to find what to press) and `__runtime_probe__`.
//
// Per game and tier it records: the published files (raw and brotli -q 11),
// the time from navigation to the first frame (`og:first-frame`) and to the
// title (the first time the title's node is in the accessibility mirror),
// the lanes (`__runtime_probe__.lanes`, the Workers the page runs), the
// played frames over 600 ticks of play (`__og_timings__`: display interval
// and engine work per frame; `state().timings`: step and render, sampled),
// memory at the end (`performance.measureUserAgentSpecificMemory`, and the
// browser's processes' resident memory), console warnings and errors (the
// page, its Workers, the browser's log), the platform calls the stand-in
// play origin answered (store, leaderboard), a save of the settings with
// the language switched and a reload that must come back in that language
// from the stub's store, the a11y tree on the title, and quit (the game's
// own quit to the title, then Back, which must report `end`).
//
// The game's own playtest frames are a second run (--frames): the frames
// playtest's script (its commands without screenshots, `quit` and dumps)
// through tests/web/reference.ts on the dev export, one tick and one render
// per frame as `og run` does natively, each screenshot compared with the
// game's metal reference under og-tests' rule, and 600 more frames timed
// (`--time 600`). A publish build has no command channel (og-build §6), so
// the scripted sequence cannot run on the published page itself
// (examples/WEB-FINDINGS.md #6: the page is driven by real clicks on the
// controls the accessibility mirror locates instead).
//
//   bun tests/web/dogfood.ts --project examples/hollowbell [--webgl] [--frames] [--no-play]
//       [--no-build] [--no-isolation] [--throttle <Mbit/s>] [--boot] [--label <name>]
//       [--out <dir>] [--chrome <path>]
//
// --no-build reuses the last publish build; --no-isolation serves it without
// COOP/COEP (the single-threaded engine); --throttle sends every file
// through one shared link of that throughput (see below); --boot loads the
// page to its gate only (the boot: first frame, title, what was fetched
// before the first frame, the language behind the gate, the engine's
// memory) and stops, for before/after measurements; --label names the run's
// directory's suffix (a before and an after side by side). Results go to
// <out>/<id>/<tier>[-single][-<n>mbit][-boot][-<label>]/.
//
// Every response the server sends is logged with its time from navigation
// and its brotli size (`requests`), and `boot` sums what the first load
// fetched before its first frame (`og:first-frame`): the boot download a
// player waits for, by kind (engine, shell, basis, cooked, music chunks).
//
// Writes <out>/<id>/<tier>/dogfood.json and screenshots (default
// dist/dogfood). Never fails on a finding: it records them.

import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { brotliCompressSync, constants } from "node:zlib";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { readHeaders, TYPES } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";
import { decodePng, type Image } from "../support/png.ts";
import { bundleGame } from "../../src/node/build.ts";
import type { PlaytestDesc } from "../../src/playtest.ts";

declare const Bun: {
  serve(o: { port: number; hostname: string; fetch(r: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(p: string): Blob;
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined>; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const projectRel = opt("--project") ?? "examples/hollowbell";
const project = join(repo, projectRel);
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const chrome = chromePath(args);
const id = projectRel.split("/").pop()!;
const work = resolve(opt("--out") ?? join(pkg, "dist", "dogfood"));
// --throttle <Mbit/s>: the server sends every file of the build through one shared link of that
// throughput, 40 ms before each response's first byte (DevTools' network emulation does not reach
// the engine Worker's fetches, which are most of the bytes).
const throttle = Number(opt("--throttle") ?? 0);
const bootOnly = args.includes("--boot");
const label = opt("--label");
const out = join(work, id, `${tier}${args.includes("--no-isolation") ? "-single" : ""}${throttle ? `-${throttle}mbit` : ""}${bootOnly ? "-boot" : ""}${label ? `-${label}` : ""}`);
mkdirSync(out, { recursive: true });
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const td = new TextDecoder();

/** What the run found, in the order found (WEB-FINDINGS reads these). */
const notes: string[] = [];
const note = (s: string) => {
  notes.push(s);
  console.log(`note: ${s}`);
};
const result: Record<string, unknown> = { project: projectRel, tier, notes };
const save = () => writeFileSync(join(out, "dogfood.json"), JSON.stringify(result, null, 1) + "\n");

// ── the game's page plan: what to press, found by the names in its accessibility tree ──
/** A step on the published page: a press found in the mirror, a key, a wait, or a wait for a node. */
type Step =
  | { click: string; nth?: number }
  | { key: string; hold?: number }
  | { wait: number }
  | { until: string; timeout?: number };
interface Plan {
  /** A node (path suffix or name) that says the title is up. */
  readonly title: string;
  /** From the title into play. */
  readonly start: Step[];
  /** Keys held in turn during the 600 timed ticks. */
  readonly play: string[];
  /** From play back to the title (the game's own quit). */
  readonly quit: Step[];
  /** From the title: open the settings, switch the language, save. */
  readonly language: Step[];
  /** A node (path suffix or name) that must be on the title after the reload, in the other language. */
  readonly translated: string;
  /** From the title: a screen that reads the leaderboard, and back. */
  readonly board?: Step[];
}
/** Each game's plan, from its README's controls and its ui.ts ids (the mirror's paths end in them). */
const PLANS: Record<string, Plan> = {
  "lantern-vale": {
    title: "play",
    start: [{ click: "play" }, { wait: 1500 }],
    play: ["KeyW", "KeyD", "KeyS", "KeyA"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    language: [{ click: "settings" }, { wait: 700 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "出发",
  },
  hollowbell: {
    title: "new",
    start: [{ click: "new" }, { wait: 700 }, { click: "Begin" }, { wait: 1500 }],
    play: ["ArrowRight", "Space", "ArrowRight", "KeyX"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    language: [{ click: "settings" }, { wait: 700 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "新游戏",
  },
  "kestrel-stair": {
    title: "new",
    start: [{ click: "new" }, { wait: 1200 }, { key: "Enter" }, { wait: 400 }, { click: "Skip" }, { wait: 500 }, { click: "Skip" }, { wait: 1500 }],
    play: ["KeyW", "KeyD", "KeyW", "KeyA"],
    quit: [{ key: "Escape" }, { wait: 600 }, { click: "quit" }],
    board: [{ click: "records" }, { wait: 1500 }, { key: "Escape" }, { wait: 800 }],
    language: [{ click: "settings" }, { wait: 900 }, { click: "Language" }, { wait: 500 }, { click: "日本語" }, { wait: 400 }, { click: "Done" }],
    translated: "新しい旅",
  },
  "ui-kit": {
    title: "UI kit",
    start: [{ click: "platformer" }, { wait: 1000 }],
    play: ["ArrowRight", "Space", "ArrowLeft", "Space"],
    // Escape over the platformer HUD opens its pause menu; its Quit returns to the gallery.
    quit: [{ key: "Escape" }, { wait: 800 }, { click: "quit" }],
    // The kit's leaderboard is demo rows (no og.platform call), so no board step.
    language: [{ click: "settings" }, { wait: 900 }, { click: "Language" }, { wait: 500 }, { click: "简体中文" }, { wait: 400 }, { click: "Done" }],
    translated: "简体中文",
  },
};

// ── 1. publish, as the author does ────────────────────────────────────
const og = ogBin();
const dist = join(work, id, "publish");
function files(dir: string): Array<{ path: string; bytes: number; brotli: number }> {
  const list: Array<{ path: string; bytes: number; brotli: number }> = [];
  const walk = (d: string) => {
    for (const name of readdirSync(d).sort()) {
      const p = join(d, name);
      if (statSync(p).isDirectory()) walk(p);
      else {
        const bytes = readFileSync(p);
        list.push({ path: relative(dir, p), bytes: bytes.length, brotli: brotliCompressSync(bytes, { params: { [constants.BROTLI_PARAM_QUALITY]: 11, [constants.BROTLI_PARAM_SIZE_HINT]: bytes.length } }).length });
      }
    }
  };
  walk(dir);
  return list;
}
if (!args.includes("--no-build") || !existsSync(join(dist, "og-build.json"))) {
  const t0 = performance.now();
  const r = Bun.spawnSync([og, "build", project, "--publish", "--out", dist], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg }, stdout: "pipe", stderr: "pipe" });
  const said = td.decode(r.stdout) + td.decode(r.stderr);
  writeFileSync(join(work, id, "og-build.txt"), said);
  result["build"] = { exit: r.exitCode, ms: Math.round(performance.now() - t0), warnings: said.split("\n").filter((l) => /warn|warning|missing|single-threaded/i.test(l)) };
  console.log(said);
  if (r.exitCode !== 0) {
    note(`og build --publish failed (exit ${r.exitCode})`);
    save();
    process.exit(1);
  }
}
const manifest = JSON.parse(readFileSync(join(dist, "og-build.json"), "utf8")) as Record<string, unknown>;
const listed = files(dist);
const sum = (f: (x: { path: string; bytes: number; brotli: number }) => boolean) => listed.filter(f).reduce((a, x) => ({ bytes: a.bytes + x.bytes, brotli: a.brotli + x.brotli }), { bytes: 0, brotli: 0 });
result["package"] = {
  variant: manifest["variant"],
  lanes: manifest["lanes"] ?? null,
  probe: manifest["probe"],
  crossOriginIsolation: manifest["crossOriginIsolation"],
  stream: manifest["stream"] ? (manifest["stream"] as { groups: unknown[] }).groups.length : 0,
  total: sum(() => true),
  engine: sum((x) => /^origin-engine(-lanes)?(_bg\.wasm|\.js)$/.test(x.path)),
  single: sum((x) => /^origin-engine(_bg\.wasm|\.js)$/.test(x.path)),
  lanesModule: sum((x) => /^origin-engine-lanes(_bg\.wasm|\.js)$/.test(x.path) || x.path.startsWith("snippets/")),
  cooked: sum((x) => x.path.startsWith("cooked/")),
  shell: sum((x) => /^(index\.html|play\.js|game\.js|audio-worklet\.js|sw\.js)$/.test(x.path)),
  basis: sum((x) => x.path.startsWith("basis/")),
  modules: listed.filter((x) => !x.path.startsWith("cooked/") || x.bytes > 200_000),
  cookedFiles: listed.filter((x) => x.path.startsWith("cooked/")).length,
};
save();

// ── 2. the stand-in play origin ───────────────────────────────────────
const GAME = { id: `${id.replace(/[^a-z0-9]/g, "")}01`.slice(0, 24), releaseId: "release_dogfood01" };
const ROOT = `/play/${GAME.id}/${GAME.releaseId}/`;
const rules = readHeaders(dist);
type Call = { at: number; method: string; args: Record<string, unknown>; answer: unknown };
const calls: Call[] = [];
const reports: Array<{ kind: string; at: number; keepalive?: boolean }> = [];
const missing: string[] = [];
const store = new Map<string, { value: unknown; revision: number }>();
const boards = new Map<string, Array<{ name: string; score: number; me?: boolean }>>([
  ["ascent", [{ name: "Wren", score: 212.4 }, { name: "Ilse", score: 245.0 }, { name: "Oswin", score: 301.9 }]],
]);
const t0 = performance.now();
const json = (status: number, body: unknown) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", "cache-control": "no-store" } });
function answer(method: string, a: Record<string, unknown>): { ok: true; data: unknown } | { ok: false; error: Record<string, unknown> } {
  switch (method) {
    case "store.get": {
      const s = store.get(String(a["key"]));
      return { ok: true, data: { key: a["key"], value: s?.value ?? null, revision: s?.revision ?? 0 } };
    }
    case "store.set": {
      const key = String(a["key"]);
      const s = store.get(key);
      if (typeof a["revision"] === "number" && a["revision"] !== (s?.revision ?? 0)) return { ok: false, error: { code: "conflict", message: "stale", retryable: false, revision: s?.revision ?? 0 } };
      const revision = (s?.revision ?? 0) + 1;
      store.set(key, { value: a["value"] ?? null, revision });
      return { ok: true, data: { key, revision } };
    }
    case "store.list":
      return { ok: true, data: { keys: [...store].map(([key, v]) => ({ key, revision: v.revision })) } };
    case "leaderboard.submit": {
      const board = String(a["board"] ?? "default");
      const list = boards.get(board) ?? [];
      list.push({ name: "You", score: Number(a["score"]), me: true });
      const asc = a["order"] === "asc";
      list.sort((x, y) => (asc ? x.score - y.score : y.score - x.score));
      boards.set(board, list);
      const rank = list.findIndex((e) => e.me) + 1;
      return { ok: true, data: { ok: true, board, rank, best: Number(a["score"]) } };
    }
    case "leaderboard.top":
    case "leaderboard.around": {
      const board = String(a["board"] ?? "default");
      const list = (boards.get(board) ?? []).map((e, i) => ({ rank: i + 1, name: e.name, score: e.score, me: !!e.me }));
      const me = list.find((e) => e.me) ?? null;
      return { ok: true, data: { board, sort: "asc", [method === "leaderboard.top" ? "top" : "entries"]: list.slice(0, Number(a["n"] ?? 10)), me } };
    }
    case "point.award":
      return { ok: true, data: { granted: 0, balance: null } };
    case "ai.mark":
      return { ok: true, data: { marked: true } };
    default:
      return { ok: false, error: { code: "unavailable", message: `the stand-in answers no ${method}`, retryable: false } };
  }
}
// --no-isolation: a host that sends the files without the build's COOP/COEP (the single-threaded engine).
const isolated = !args.includes("--no-isolation");
/** The shared link of --throttle: each chunk waits for its turn on it. */
let linkFree = 0;
const link = async (n: number) => {
  const now = performance.now();
  const start = Math.max(now, linkFree);
  linkFree = start + (n * 8 * 1000) / (throttle * 1e6);
  await wait(linkFree - now);
};
/** Every file the server sent: which page load (1 the first, 2 after the reload), ms from that load's navigation, its sizes. */
const requests: Array<{ load: number; at: number; path: string; bytes: number; brotli: number; sw?: boolean }> = [];
let loads = 0;
let navAt = performance.now();
/**
 * What a load fetched before `firstFrame` ms (from its navigation), by kind, brotli bytes: each
 * file once (a host that lets the browser cache would send the pool Workers' glue once), the
 * service worker's precache apart (`sw`: it starts once the engine is ready, beside the boot).
 */
function bootOf(load: number, firstFrame: number | null) {
  const kind = (p: string) =>
    /^origin-engine(-lanes)?(_bg\.wasm|\.js)$|^snippets\//.test(p) ? "engine"
    : p.startsWith("basis/") ? "basis"
    : /\.chunk\d+\.ogb$/.test(p) ? "chunks"
    : p.startsWith("cooked/stream/") ? "packs"
    : p.startsWith("cooked/") ? "cooked"
    : "shell";
  const once = (rs: typeof requests) => [...new Map(rs.map((r) => [r.path, r])).values()];
  const page = requests.filter((r) => r.load === load && !r.sw);
  const before = once(page.filter((r) => firstFrame === null || r.at <= firstFrame));
  const sw = once(requests.filter((r) => r.load === load && r.sw));
  const by: Record<string, { files: number; brotli: number }> = {};
  for (const r of before) {
    const k = kind(r.path);
    by[k] ??= { files: 0, brotli: 0 };
    by[k].files++;
    by[k].brotli += r.brotli;
  }
  const after = once(page.filter((r) => firstFrame !== null && r.at > firstFrame && !before.some((b) => b.path === r.path)));
  return {
    firstFrame,
    requests: page.filter((r) => firstFrame === null || r.at <= firstFrame).length,
    sw: { files: sw.length, brotli: sw.reduce((n, r) => n + r.brotli, 0), beforeFirstFrame: sw.filter((r) => firstFrame !== null && r.at <= firstFrame).length },
    files: before.length,
    brotli: before.reduce((n, r) => n + r.brotli, 0),
    by,
    after: { files: after.length, brotli: after.reduce((n, r) => n + r.brotli, 0), chunks: after.filter((r) => kind(r.path) === "chunks").length, packs: after.filter((r) => kind(r.path) === "packs").length },
  };
}
const isolation: Record<string, string> = isolated ? { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "cross-origin" } : {};
const server = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  async fetch(req) {
    const url = new URL(req.url);
    if (url.pathname.startsWith(`/__player/v2/${GAME.id}/${GAME.releaseId}/`)) {
      const what = url.pathname.split("/").at(-1)!;
      const body = (await req.json().catch(() => null)) as Record<string, unknown> | null;
      const at = Math.round(performance.now() - t0);
      if (what === "hello")
        return json(200, { og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", started: false, paused: false,
          capabilities: ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark"],
          player: { token: "ogp_guest_dogfood", guest: true },
          page: { title: id, back: `${url.origin}/portal/games/${id}`, portal: "https://portal.invalid" } });
      if (what === "report") {
        for (const m of (body?.["messages"] as Array<Record<string, unknown>>) ?? []) reports.push({ kind: String(m["kind"]), at, keepalive: req.keepalive });
        return json(202, { accepted: ((body?.["messages"] as unknown[]) ?? []).length, rejected: 0 });
      }
      const method = String(body?.["method"]);
      const a = (body?.["args"] ?? {}) as Record<string, unknown>;
      const r = answer(method, a);
      calls.push({ at, method, args: method === "store.set" ? { ...a, value: JSON.stringify(a["value"]).slice(0, 160) } : a, answer: r.ok ? JSON.stringify(r.data).slice(0, 200) : r.error });
      return json(200, { og: "player", protocol: 2, kind: "result", id: body?.["id"], ...r, time: Date.now() });
    }
    if (url.pathname === `/portal/games/${id}`) return new Response("<!doctype html><title>game page</title><h1>game page</h1>", { headers: { "content-type": TYPES[".html"]! } });
    if (!url.pathname.startsWith(ROOT)) {
      missing.push(url.pathname);
      return new Response("not found", { status: 404 });
    }
    let rel = url.pathname.slice(ROOT.length - 1);
    if (rel.endsWith("/")) rel += "index.html";
    const path = join(dist, rel);
    if (!path.startsWith(dist) || !existsSync(path)) {
      missing.push(url.pathname);
      return new Response("not found", { status: 404 });
    }
    {
      const f = listed.find((x) => `/${x.path}` === rel);
      // The service worker's own fetches (its precache) name sw.js as their referrer.
      const sw = /\/sw\.js$/.test(req.headers.get("referer") ?? "");
      requests.push({ load: loads, at: Math.round(performance.now() - navAt), path: rel.slice(1), bytes: f?.bytes ?? 0, brotli: f?.brotli ?? 0, ...(sw ? { sw } : {}) });
    }
    const headers: Record<string, string> = { "cache-control": "no-store", ...isolation };
    for (const r of rules) if (r.pattern === rel || (r.pattern.endsWith("*") && rel.startsWith(r.pattern.slice(0, -1))) || (r.pattern.startsWith("/*.") && rel.endsWith(r.pattern.slice(2)))) Object.assign(headers, Object.fromEntries(Object.entries(r.headers).map(([k, v]) => [k.toLowerCase(), v])));
    if (!isolated) {
      delete headers["cross-origin-opener-policy"];
      delete headers["cross-origin-embedder-policy"];
    }
    headers["content-type"] = TYPES[rel.slice(rel.lastIndexOf("."))] ?? "application/octet-stream";
    // The platform's derived entry, only to a document load (server/src/play/entry.ts).
    const opening = ["document", "iframe", "frame"].includes(req.headers.get("sec-fetch-dest") ?? "");
    if (rel === "/index.html" && opening) {
      const html = readFileSync(path, "utf8").replace(/<head[^>]*>/, (m) => `${m}<script id="__og-game-config">window.__OG_GAME=${JSON.stringify({ id: GAME.id, version: 1, releaseId: GAME.releaseId, protocol: 2 })}</script>`);
      return new Response(html, { headers });
    }
    if (!throttle) return new Response(Bun.file(path), { headers });
    const bytes = new Uint8Array(readFileSync(path));
    headers["content-length"] = String(bytes.length);
    await wait(40);
    // Charged at its brotli size: a host sends it compressed (this server does not).
    const listedFile = listed.find((x) => `/${x.path}` === rel);
    const ratio = listedFile && listedFile.bytes > 0 ? listedFile.brotli / listedFile.bytes : 1;
    let at = 0;
    return new Response(new ReadableStream<Uint8Array>({
      async pull(c) {
        if (at >= bytes.length) return c.close();
        const n = Math.min(16 * 1024, bytes.length - at);
        await link(n * ratio);
        c.enqueue(bytes.subarray(at, at + n));
        at += n;
      },
    }), { headers });
  },
});

// ── 3. play it ────────────────────────────────────────────────────────
const plan = PLANS[id];
const OBSERVE = `(() => {
  window.__dog = { seen: {}, first: 0 };
  const look = () => {
    for (const el of document.querySelectorAll("[data-og-path]")) {
      const p = el.getAttribute("data-og-path");
      if (!(p in __dog.seen)) __dog.seen[p] = Math.round(performance.now());
      const n = el.getAttribute("aria-label") || el.textContent || "";
      if (n && !(("name:" + n) in __dog.seen)) __dog.seen["name:" + n] = Math.round(performance.now());
    }
  };
  new MutationObserver(look).observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ["aria-label"] });
})();`;
/** The mirror's element for `what`: a path, a path's last part, or a name. */
const FIND = (what: string, nth = 0) => `(() => {
  const w = ${JSON.stringify(what)};
  const all = [...document.querySelectorAll("[data-og-path]")].filter((el) => {
    const p = el.getAttribute("data-og-path");
    const n = (el.getAttribute("aria-label") || (el.firstChild && el.firstChild.nodeType === 3 ? el.firstChild.textContent : "") || "").trim();
    return p === w || p.endsWith("/" + w) || n === w;
  });
  const el = all[${nth}];
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return { x: r.left + r.width / 2, y: r.top + r.height / 2, w: r.width, h: r.height, path: el.getAttribute("data-og-path") };
})()`;
const TREE = `[...document.querySelectorAll("[data-og-path]")].map((el) => ({ path: el.getAttribute("data-og-path"), role: el.getAttribute("role"), name: (el.getAttribute("aria-label") || (el.firstChild && el.firstChild.nodeType === 3 ? el.firstChild.textContent : "") || ""), states: [el.getAttribute("aria-selected") === "true" ? "selected" : null, el.getAttribute("aria-checked") === "true" ? "checked" : null, el.getAttribute("aria-disabled") === "true" ? "disabled" : null, document.activeElement === el ? "focused" : null].filter(Boolean), focusable: el.tabIndex >= 0 }))`;

async function run(page: Page, steps: readonly Step[], label: string): Promise<boolean> {
  for (const s of steps) {
    if ("wait" in s) await wait(s.wait);
    else if ("key" in s) {
      await page.send("Input.dispatchKeyEvent", { type: "keyDown", code: s.key, key: s.key.replace(/^Key/, "").toLowerCase(), windowsVirtualKeyCode: VK[s.key] ?? 0 });
      await wait(s.hold ?? 60);
      await page.send("Input.dispatchKeyEvent", { type: "keyUp", code: s.key, key: s.key.replace(/^Key/, "").toLowerCase(), windowsVirtualKeyCode: VK[s.key] ?? 0 });
      await wait(120);
    } else if ("until" in s) {
      const ok = await page.until(`${FIND(s.until)}`, s.timeout ?? 8000).then(() => true, () => false);
      if (!ok) {
        note(`${label}: '${s.until}' never appeared in the accessibility mirror`);
        return false;
      }
    } else {
      const at = await page.until<{ x: number; y: number; path: string } | null>(FIND(s.click, s.nth ?? 0), 8000).catch(() => null);
      if (!at) {
        note(`${label}: nothing named '${s.click}' to press (the mirror held: ${(await page.eval<Array<{ path: string; name: string }>>(TREE)).map((n) => `${n.path}=${n.name}`).join(", ").slice(0, 600)})`);
        return false;
      }
      await page.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: at.x, y: at.y });
      await wait(50);
      await page.send("Input.dispatchMouseEvent", { type: "mousePressed", x: at.x, y: at.y, button: "left", clickCount: 1 });
      await wait(70);
      await page.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: at.x, y: at.y, button: "left", clickCount: 1 });
      await wait(350);
    }
  }
  return true;
}
const VK: Record<string, number> = { Enter: 13, Escape: 27, Space: 32, Tab: 9, ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40, KeyW: 87, KeyA: 65, KeyS: 83, KeyD: 68, KeyE: 69, KeyJ: 74, KeyX: 88, KeyZ: 90, KeyF: 70, ShiftLeft: 16 };

const pct = (xs: number[], q: number) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? +s[Math.min(s.length - 1, Math.floor(q * s.length))]!.toFixed(3) : null;
};
const stats = (xs: number[]) => ({ n: xs.length, mean: xs.length ? +(xs.reduce((a, b) => a + b, 0) / xs.length).toFixed(3) : null, p50: pct(xs, 0.5), p95: pct(xs, 0.95), max: pct(xs, 1) });
/** og-tests' rule (crates/og-tests/src/compare.rs), as tests/web/reference.ts measures it. */
function compare(a: Image, b: Image): { outliers: number; rmse: number } {
  if (a.width !== b.width || a.height !== b.height) return { outliers: 100, rmse: 1 };
  const n = a.width * a.height;
  let outliers = 0;
  let sq = 0;
  for (let p = 0; p < n; p++) {
    let max = 0;
    for (let k = 0; k < 3; k++) {
      const d = Math.abs(a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!);
      max = Math.max(max, d);
      sq += (d / 255) ** 2;
    }
    if (max > 32) outliers++;
  }
  return { outliers: +((outliers / n) * 100).toFixed(3), rmse: +Math.sqrt(sq / (n * 3)).toFixed(5) };
}

if (!args.includes("--no-play")) {
  const browser = await Browser.launch({ chrome, width: 1280, height: 720, flags: ["--enable-blink-features=ForceEagerMeasureMemory"] });
  const log: string[] = [];
  const workerLog: string[] = [];
  const browserLog: string[] = [];
  const workers = new Map<string, { type: string; url: string }>();
  // The Workers' consoles (the engine Worker, the lanes pool, the service worker).
  browser.on((method, params, session) => {
    if (method === "Target.attachedToTarget") {
      const info = params["targetInfo"] as { type: string; url: string; targetId: string };
      const child = params["sessionId"] as string;
      if (info.type === "worker" || info.type === "service_worker" || info.type === "shared_worker") {
        workers.set(child, { type: info.type, url: info.url });
        void browser.send("Runtime.enable", {}, child).catch(() => {});
        void browser.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: false, flatten: true }, child).catch(() => {});
        void browser.send("Runtime.runIfWaitingForDebugger", {}, child).catch(() => {});
      }
    } else if (session && workers.has(session) && (method === "Runtime.consoleAPICalled" || method === "Runtime.exceptionThrown")) {
      const w = workers.get(session)!;
      if (method === "Runtime.exceptionThrown") workerLog.push(`${w.type} exception: ${(params["exceptionDetails"] as { exception?: { description?: string }; text?: string }).exception?.description ?? (params["exceptionDetails"] as { text?: string }).text}`);
      else {
        const a = (params["args"] as Array<{ value?: unknown; description?: string }>) ?? [];
        workerLog.push(`${w.type} ${params["type"]}: ${a.map((x) => (x.value !== undefined ? String(x.value) : (x.description ?? ""))).join(" ")}`);
      }
    } else if (method === "Log.entryAdded") {
      const e = params["entry"] as { level: string; source: string; text: string; url?: string };
      browserLog.push(`${e.level} ${e.source}: ${e.text}${e.url ? ` (${e.url})` : ""}`);
    }
  });
  const shot = async (page: Page, name: string) => writeFileSync(join(out, `${name}.png`), await page.screenshot());
  /** The engine's own capture at 1280×720 against the game's metal reference of the same screen. */
  const versus = async (page: Page, name: string, reference: string | null) => {
    let c = await page.eval<{ png: string; tick: number; width: number; height: number } | null>("__runtime_probe__.capture(1280, 720)").catch((e) => ({ error: String(e) }) as never);
    for (let i = 0; !c?.png && i < 60; i++) {
      await wait(50);
      c = await page.eval<{ png: string; tick: number; width: number; height: number } | null>("__runtime_probe__.capture(1280, 720)").catch(() => null);
    }
    if (!c?.png) return { error: `capture answered ${JSON.stringify(c)?.slice(0, 200)}` };
    const bytes = Uint8Array.from(atob(c.png), (ch) => ch.charCodeAt(0));
    writeFileSync(join(out, `capture-${name}.png`), bytes);
    const row: Record<string, unknown> = { tick: c.tick, size: `${c.width}x${c.height}` };
    if (reference && existsSync(reference)) Object.assign(row, { against: relative(repo, reference) }, compare(decodePng(bytes), decodePng(new Uint8Array(readFileSync(reference)))));
    return row;
  };
  const metal = (f: string) => {
    for (const d of [join(project, "references", "metal"), join(repo, "crates", "og-ui", "references", "metal")]) if (existsSync(join(d, f))) return join(d, f);
    return null;
  };
  const play: Record<string, unknown> = {};
  result["play"] = play;
  try {
    const page = await browser.open("about:blank", log);
    await page.send("Log.enable");
    await page.send("Page.addScriptToEvaluateOnNewDocument", { source: OBSERVE });
    await page.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
    const url = `http://127.0.0.1:${server.port}${ROOT}${webgl ? "?webgl=1" : ""}`;
    loads = 1;
    navAt = performance.now();
    await page.send("Page.navigate", { url });
    await page.until("typeof __runtime_probe__ === 'object' || !document.getElementById('og-error')?.hidden", 120_000);
    const failed = await page.eval<string | null>("document.getElementById('og-error') && !document.getElementById('og-error').hidden ? document.getElementById('og-error').innerText.slice(0, 3000) : null");
    if (failed) {
      note(`the page did not start: ${failed.split("\n").slice(0, 3).join(" / ")}`);
      play["error"] = failed;
      await shot(page, "error");
      throw new Error("no start");
    }
    await page.until("performance.getEntriesByName('og:first-frame').length > 0", 60_000).catch(() => note("no og:first-frame mark within 60 s"));
    await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => note("the play gate never showed"));
    const titleNode = plan?.title ?? "";
    if (plan) await page.until(`(() => { const s = __dog.seen; return Object.keys(s).some((k) => k === ${JSON.stringify(titleNode)} || k.endsWith("/" + ${JSON.stringify(titleNode)}) || k === "name:" + ${JSON.stringify(titleNode)}); })()`, 30_000).catch(() => note(`the title's '${titleNode}' never reached the mirror`));
    const marks = await page.eval<Record<string, number>>(`(() => { const m = {}; for (const e of performance.getEntriesByType("mark")) if (e.name.startsWith("og:")) m[e.name] = Math.round(e.startTime); const nav = performance.getEntriesByType("navigation")[0]; m.domContentLoaded = Math.round(nav?.domContentLoadedEventEnd ?? 0); m.load = Math.round(nav?.loadEventEnd ?? 0); return m; })()`);
    const seen = await page.eval<Record<string, number>>("__dog.seen");
    const titleAt = Object.entries(seen).filter(([k]) => k === titleNode || k.endsWith(`/${titleNode}`) || k === `name:${titleNode}`).map(([, v]) => v).sort((a, b) => a - b)[0] ?? null;
    play["timings"] = { ...marks, title: titleAt, firstMirror: Math.min(...Object.values(seen)) };
    play["hello"] = await page.eval("__runtime_probe__.hello()");
    play["lanes"] = await page.eval("__runtime_probe__.lanes");
    play["isolated"] = await page.eval("crossOriginIsolated");
    play["cores"] = await page.eval("navigator.hardwareConcurrency");
    const targets = (await browser.send<{ targetInfos: Array<{ type: string; url: string }> }>("Target.getTargets")).targetInfos;
    const ours = targets.filter((t) => !t.url.startsWith("chrome-extension:"));
    play["workers"] = { dedicated: ours.filter((t) => t.type === "worker").length, service: ours.filter((t) => t.type === "service_worker").length, urls: targets.filter((t) => t.type.endsWith("worker") && !t.url.startsWith("chrome-extension:")).map((t) => t.url.replace(/^.*\/play\/[^/]+\/[^/]+\//, "./").replace(/^blob:.*/, "blob:")) };
    play["boot"] = { ...bootOf(1, marks["og:first-frame"] ?? null), phases: await page.eval("__runtime_probe__.boot ?? null").catch(() => null) };
    play["memoryProbe"] = await page.eval("__runtime_probe__.memory?.() ?? null").catch((e) => ({ error: String(e) }));
    await shot(page, "gate");
    play["titleCapture"] = await versus(page, "title-behind-gate", metal(id === "ui-kit" ? "kit-desktop-gallery.png" : "title.png"));
    if (bootOnly) {
      // The boot only: is the title behind the gate in the store's language (a run after a full one, whose store is gone: none is saved here)?
      await wait(1500);
      play["gateTree"] = (await page.eval<Array<{ path: string; role: string; name: string }>>(TREE)).map((n) => `${n.role}:${n.name}`).slice(0, 30);
      throw new Error("boot only");
    }
    const tree = await page.eval<Array<{ path: string; role: string; name: string; focusable: boolean }>>(TREE);
    play["a11yTitle"] = { nodes: tree.length, roles: Object.fromEntries([...new Set(tree.map((n) => n.role))].map((r) => [r, tree.filter((n) => n.role === r).length])), unnamed: tree.filter((n) => !n.name && n.role !== "group" && n.role !== "generic").map((n) => `${n.role} ${n.path}`), names: tree.map((n) => `${n.role}:${n.name}`).slice(0, 60) };
    const ax = await page.send<{ nodes: Array<{ role?: { value: string }; name?: { value: string }; ignored: boolean }> }>("Accessibility.getFullAXTree").catch(() => ({ nodes: [] }));
    play["chromeAx"] = { nodes: ax.nodes.filter((n) => !n.ignored).length, buttons: ax.nodes.filter((n) => !n.ignored && n.role?.value === "button").map((n) => n.name?.value ?? "") };
    writeFileSync(join(out, "a11y-title.json"), JSON.stringify(tree, null, 1));
    // The gate: a real click.
    await page.click(640, 360);
    await page.until("document.getElementById('og-back')?.hasAttribute('data-playing') || document.getElementById('og-gate')?.hidden", 10_000).catch(() => note("the gate's click did not start play"));
    await wait(1500);
    await shot(page, "title");
    play["titleCaptureStarted"] = await versus(page, "title", metal(id === "ui-kit" ? "kit-desktop-gallery.png" : "title.png"));
    if (!plan) {
      note(`no plan for ${id}: dumped the title's tree only`);
    } else {
      // Start a run.
      const started = await run(page, plan.start, "start");
      await wait(1500);
      await shot(page, "play");
      play["started"] = started;
      play["playCapture"] = await versus(page, "play", null);
      // 600 ticks of play, timed: the frames' work and display interval, and step/render sampled.
      await page.eval("__og_timings__(true)");
      const tick0 = await page.eval<number>("__runtime_probe__.tick()");
      const samples: Array<{ step: number; render: number }> = [];
      const w0 = performance.now();
      let k = 0;
      for (;;) {
        const tick = await page.eval<number>("__runtime_probe__.tick()");
        if (tick - tick0 >= 600 || performance.now() - w0 > 60_000) break;
        const key = plan.play[Math.floor(k / 8) % Math.max(1, plan.play.length)];
        if (key && k % 8 === 0) void page.send("Input.dispatchKeyEvent", { type: "keyDown", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
        if (key && k % 8 === 7) void page.send("Input.dispatchKeyEvent", { type: "keyUp", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
        const s = await page.eval<{ timings: { step: number; render: number } }>("__runtime_probe__.state()");
        samples.push(s.timings);
        k++;
        await wait(100);
      }
      for (const key of plan.play) await page.send("Input.dispatchKeyEvent", { type: "keyUp", code: key, key, windowsVirtualKeyCode: VK[key] ?? 0 });
      const wall = performance.now() - w0;
      const tick1 = await page.eval<number>("__runtime_probe__.tick()");
      const t = await page.eval<{ frames: number; intervals: number[]; work: number[] }>("__og_timings__()");
      play["steady"] = {
        ticks: tick1 - tick0,
        wallMs: Math.round(wall),
        ticksPerSecond: +(((tick1 - tick0) / wall) * 1000).toFixed(1),
        frames: t.frames,
        interval: stats(t.intervals),
        work: stats(t.work),
        step: stats(samples.map((s) => s.step)),
        render: stats(samples.map((s) => s.render)),
      };
      play["lanesReport"] = await page.eval("__runtime_probe__.lanes").catch(() => null);
      await shot(page, "played");
      play["facts"] = await page.eval<{ facts: unknown }>("__runtime_probe__.state()").then((s) => s.facts).catch(() => null);
      // The game's own quit, back to the title.
      play["quitToTitle"] = await run(page, plan.quit, "quit");
      await wait(1200);
      await shot(page, "after-quit");
      // The leaderboard, where the game reads one.
      if (plan.board) {
        const before = calls.length;
        play["board"] = await run(page, plan.board, "board");
        await wait(800);
        play["boardCalls"] = calls.slice(before).filter((c) => c.method.startsWith("leaderboard."));
        if ((play["boardCalls"] as unknown[]).length === 0) note("the game's leaderboard screen made no leaderboard call through the Player");
      }
      // A save: the settings with the language switched, kept by the stand-in's store.
      const before = calls.length;
      play["language"] = await run(page, plan.language, "language");
      await wait(1500);
      await shot(page, "language-saved");
      const sets = calls.slice(before).filter((c) => c.method === "store.set");
      play["saveCalls"] = sets;
      if (sets.length === 0) note("switching the language and saving the settings sent no store.set to the Player");
      play["storeAfterSave"] = Object.fromEntries([...store].map(([k2, v]) => [k2, { revision: v.revision, bytes: JSON.stringify(v.value).length }]));
      // Reload: the release's service worker now controls the page; the settings come back from the store.
      const reloadAt = calls.length;
      loads = 2;
      navAt = performance.now();
      await page.send("Page.reload", { ignoreCache: false });
      await page.until("typeof __runtime_probe__ === 'object'", 60_000);
      await page.until("document.querySelector('#og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => {});
      // Behind the gate (the first screen a returning player sees), then after the gate: the
      // Worker applies the store's answers behind the gate (src/web/settle.ts; WEB-FINDINGS #1).
      await wait(1500);
      const behind = await page.eval<Array<{ path: string; role: string; name: string; states: string[] }>>(TREE);
      const translatedBehindGate = (await page.eval(FIND(plan.translated))) !== null;
      await shot(page, "reloaded-gate");
      await page.click(640, 360);
      const translated = await page.until(FIND(plan.translated), 5_000).then(() => true, () => false);
      await wait(1000);
      await shot(page, "reloaded-played");
      play["reload"] = {
        behindGate: behind.map((n) => `${n.role}:${n.name}${n.states.length ? ` [${n.states.join(",")}]` : ""}`).slice(0, 30),
        translatedBehindGate,
        controlled: await page.eval<boolean>("!!navigator.serviceWorker.controller"),
        firstFrame: await page.eval<number | null>("performance.getEntriesByName('og:first-frame')[0]?.startTime ?? null").then((v) => (v === null ? null : Math.round(v))),
        storeReads: calls.slice(reloadAt).filter((c) => c.method === "store.get").map((c) => c.args["key"]),
        translated,
      };
      if (!translatedBehindGate) note(`after the reload the title behind the gate is not in the saved language ('${plan.translated}' missing)${translated ? ", it is once the gate is passed" : ""}`);
      if (!translated) note(`after the reload and the gate the title is not in the saved language ('${plan.translated}' missing)`);
      const tree2 = await page.eval<Array<{ path: string; role: string; name: string }>>(TREE);
      play["a11yReloaded"] = tree2.map((n) => `${n.role}:${n.name}`).slice(0, 40);
      // Memory at the end of the run.
      // measureUserAgentSpecificMemory never resolves headless, and the probe says nothing of the
      // wasm memory (WEB-FINDINGS #9): the processes' resident memory stands in for it.
      play["memoryApi"] = await page.eval<string>("typeof performance.measureUserAgentSpecificMemory");
      let heap = 0;
      for (const [session] of workers) {
        const h = await browser.send<{ usedSize: number }>("Runtime.getHeapUsage", {}, session).catch(() => null);
        heap += h?.usedSize ?? 0;
      }
      const pageHeap = await page.send<{ usedSize: number }>("Runtime.getHeapUsage").catch(() => null);
      play["jsHeap"] = { page: pageHeap?.usedSize ?? null, workers: heap };
      play["memory"] = await page
        .eval<{ bytes: number; breakdown: Array<{ bytes: number; types: string[]; attribution: Array<{ scope: string; url: string }> }> }>(
          "Promise.race([performance.measureUserAgentSpecificMemory?.() ?? Promise.resolve(null), new Promise((r) => setTimeout(() => r(null), 45000))])",
        )
        .then((m) => (m ? { bytes: m.bytes, byScope: m.breakdown.reduce((acc: Record<string, number>, b) => { const s = b.attribution[0]?.scope ?? "shared"; acc[s] = (acc[s] ?? 0) + b.bytes; return acc; }, {}) } : null))
        .catch((e) => ({ error: String(e) }));
      const info = await browser.send<{ processInfo: Array<{ type: string; id: number }> }>("SystemInfo.getProcessInfo").catch(() => ({ processInfo: [] }));
      const rss: Record<string, number> = {};
      for (const p of info.processInfo) {
        const r = Bun.spawnSync(["ps", "-o", "rss=", "-p", String(p.id)], { stdout: "pipe", stderr: "pipe" });
        const kb = Number(td.decode(r.stdout).trim());
        if (kb > 0) rss[p.type] = (rss[p.type] ?? 0) + kb * 1024;
      }
      play["processRss"] = rss;
      // Quit: Back to the game's page, which reports `end` (keepalive).
      const reportsBefore = reports.length;
      const back = await page.eval<boolean>("(() => { const b = document.getElementById('og-back'); if (!b || b.hidden) return false; b.click(); return true; })()");
      await page.until(`location.pathname === '/portal/games/${id}'`, 10_000).catch(() => {});
      await wait(800);
      play["quit"] = { back, at: await page.eval<string>("location.pathname"), reports: reports.slice(reportsBefore).map((r) => r.kind) };
      if (!reports.slice(reportsBefore).some((r) => r.kind === "end")) note("Back reached the game's page but no `end` report reached the play origin");
    }
  } catch (e) {
    if (!(bootOnly && e instanceof Error && e.message === "boot only")) play["error"] = play["error"] ?? String(e instanceof Error ? e.stack : e);
    if (play["error"]) console.log(`error: ${play["error"]}`);
  } finally {
    play["calls"] = calls;
    play["requests"] = requests;
    play["reports"] = [...new Set(reports.map((r) => r.kind))];
    play["notFound"] = [...new Set(missing)];
    play["console"] = {
      page: log.filter((l) => /^(warn|warning|error|exception|assert)/.test(l)),
      workers: workerLog.filter((l) => / (warn|warning|error|assert)|exception/.test(l)),
      browser: browserLog.filter((l) => !/^verbose|^info/.test(l)),
      pageAll: log.length,
      workersAll: workerLog.length,
    };
    writeFileSync(join(out, "console.txt"), [...log.map((l) => `page ${l}`), ...workerLog, ...browserLog.map((l) => `browser ${l}`)].join("\n") + "\n");
    await browser.close();
    server.stop();
    save();
  }
}

// ── 4. the playtest's frames on the dev export (tests/web/reference.ts) ──
if (args.includes("--frames")) {
  const playtest = existsSync(join(project, "tests", "frames.og.ts")) ? join(project, "tests", "frames.og.ts") : join(project, "tests", "frames-desktop.og.ts");
  const bundled = join(work, id, ".frames.playtest.js");
  const b = await bundleGame(playtest, bundled, "native");
  if (!b.ok) throw new Error(`${playtest} does not bundle: ${b.reason}`);
  const desc = ((await import(pathToFileURL(bundled).href)) as { default: PlaytestDesc }).default;
  const script: string[] = [];
  const shots: string[] = [];
  const prefix = id === "ui-kit" ? "kit-desktop-" : "";
  for (const step of desc.commands.split(";")) {
    const [at, action, ...rest] = step.split(":");
    if (action === "screenshot") shots.push(`${prefix}${rest[0]!.replace(/\.png$/, "")}@${Math.round(Number(at) * 60)}`);
    else if (action !== "quit" && !action!.startsWith("dump_")) script.push(step);
  }
  // The metal references only (ui-kit's live in crates/og-ui/references, beside its browser tiers').
  const refs = id === "ui-kit" ? join(work, "ui-kit-metal-refs") : join(project, "references");
  if (id === "ui-kit" && !existsSync(join(refs, "metal"))) {
    mkdirSync(refs, { recursive: true });
    Bun.spawnSync(["ln", "-s", join(repo, "crates", "og-ui", "references", "metal"), join(refs, "metal")]);
  }
  const framesOut = join(out, "frames");
  const cmd = ["bun", join(here, "reference.ts"), ...(webgl ? ["--webgl"] : []), "--chrome", chrome, "--project", projectRel, "--references", relative(repo, refs), "--size", desc.size ?? "1280x720", "--script", script.join(";"), "--shots", shots.join(","), "--out", framesOut, "--time", "600"];
  console.log(`---- frames: ${shots.length} shots, ${script.length} commands`);
  const t1 = performance.now();
  const r = Bun.spawnSync(cmd, { cwd: pkg, stdout: "pipe", stderr: "pipe" });
  writeFileSync(join(out, "frames.txt"), td.decode(r.stdout) + td.decode(r.stderr));
  console.log(td.decode(r.stdout).split("\n").filter((l) => /^(ok|FAIL|PASS|frame time)/.test(l)).join("\n"));
  const ref = existsSync(join(framesOut, `reference-${tier}.json`)) ? JSON.parse(readFileSync(join(framesOut, `reference-${tier}.json`), "utf8")) : null;
  result["frames"] = { exit: r.exitCode, ms: Math.round(performance.now() - t1), playtest: relative(repo, playtest), shots: ref?.stations ?? null, frameMs: ref?.frameMs ?? null, renderer: ref?.renderer ?? null, textureFormat: ref?.textureFormat ?? null, error: ref?.error?.split("\n").slice(0, 8).join("\n") ?? null, engineLog: (ref?.engine ?? []).filter((l: string) => /warn|error/i.test(l)).slice(-20) };
  save();
}
{
  // One line per measure, for a before/after table.
  const p = (result["play"] ?? {}) as Record<string, any>;
  const b = p["boot"] as ReturnType<typeof bootOf> | undefined;
  if (b) console.log(`phases: ${JSON.stringify((b as { phases?: unknown }).phases ?? null)}`);
  if (b) console.log(`boot: first frame ${b.firstFrame} ms, ready ${p["timings"]?.["og:ready"]} ms, title ${p["timings"]?.["title"]} ms; fetched before it ${b.files} files (${b.requests} requests), ${b.brotli} B brotli (${Object.entries(b.by).map(([k, v]) => `${k} ${v.files}/${v.brotli}`).join(", ")}); after it ${b.after.files} files ${b.after.brotli} B (${b.after.chunks} chunks, ${b.after.packs} packs); the service worker's precache ${b.sw.files} files ${b.sw.brotli} B (${b.sw.beforeFirstFrame} before the first frame)`);
  if (p["memoryProbe"]) console.log(`memory: ${JSON.stringify(p["memoryProbe"])}`);
  if (p["reload"]) console.log(`reload: translated behind the gate ${p["reload"].translatedBehindGate}, after the tap ${p["reload"].translated}`);
}
console.log(`wrote ${join(out, "dogfood.json")}`);
process.exit(0);
