// The published page as the platform opens a release, for the browser
// harnesses that play the bytes `og build --publish` wrote (dogfood.ts,
// published.ts): the build itself (`publish`, with each file's bytes and
// brotli -q 11), a stand-in play origin that serves it (the top-level page
// at /play/<game>/<release>/ with `window.__OG_GAME` in its entry, the
// build's own `_headers`, Player protocol 2 over HTTP at /__player/v2/…
// with a store and a leaderboard: docs/web-export.md §The top-level page),
// and real input found through the accessibility mirror (an element by its
// og-ui path or name, pressed with the mouse where it is drawn; keys as
// CDP key events).

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { brotliCompressSync, constants } from "node:zlib";
import type { Page } from "./cdp.ts";
import { readHeaders, TYPES } from "./serve.ts";

declare module "node:fs" {
  export function readdirSync(path: string): string[];
}
declare const Bun: {
  serve(o: { port: number; hostname: string; fetch(r: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(p: string): Blob;
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined>; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const td = new TextDecoder();

/** A file of the build: its path under the build, its bytes and its brotli -q 11 size. */
export interface PublishedFile {
  path: string;
  bytes: number;
  brotli: number;
}

/** Every file under `dir`, sorted, with its bytes and brotli -q 11 size. */
export function publishedFiles(dir: string): PublishedFile[] {
  const list: PublishedFile[] = [];
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

/** `og build <project> --publish --out <dist>`, as the author runs it (the og binary, this package as the engine). */
export function publish(o: { og: string; project: string; dist: string; repo: string; pkg: string }): { exit: number; ms: number; said: string } {
  const t0 = performance.now();
  const r = Bun.spawnSync([o.og, "build", o.project, "--publish", "--out", o.dist], { cwd: o.repo, env: { ...process.env, OG_ENGINE_PACKAGE: o.pkg }, stdout: "pipe", stderr: "pipe" });
  return { exit: r.exitCode, ms: Math.round(performance.now() - t0), said: td.decode(r.stdout) + td.decode(r.stderr) };
}

/** A call the stand-in play origin answered. */
export interface PlayCall {
  at: number;
  method: string;
  args: Record<string, unknown>;
  answer: unknown;
}

export interface PlayOrigin {
  readonly port: number;
  /** The release's page path: /play/<game>/<release>/. */
  readonly root: string;
  readonly game: { id: string; releaseId: string };
  /** Every Player call (store, leaderboard, …), in order. */
  readonly calls: PlayCall[];
  /** Every report message (start, ready, sample, end), with whether it came with keepalive. */
  readonly reports: Array<{ kind: string; at: number; keepalive?: boolean }>;
  /** Paths asked for that the build does not have. */
  readonly missing: string[];
  /** The store the Player's store.* calls read and write. */
  readonly store: Map<string, { value: unknown; revision: number }>;
  /** The page's URL, with an optional query (`?webgl=1`). */
  url(query?: string): string;
  stop(): void;
}

/**
 * The stand-in play origin for the build in `dist`, game `id`: the page at
 * /play/<game>/<release>/ (`window.__OG_GAME` injected into a document
 * load of index.html, as the platform's derived entry does), the build's
 * `_headers` (and COOP/COEP/CORP unless `isolated` is false: then the
 * single-threaded engine), Player protocol 2 at /__player/v2/<game>/<release>/
 * (hello, call, report) with a store, a leaderboard ("ascent") and the
 * Portal's game page for Back. `throttle` (Mbit/s) sends every file
 * through one shared link of that throughput, charged at its brotli size
 * (`files`), 40 ms before each response.
 */
export function playOrigin(o: { dist: string; id: string; isolated?: boolean; throttle?: number; files?: readonly PublishedFile[] }): PlayOrigin {
  const { dist, id } = o;
  const throttle = o.throttle ?? 0;
  const isolated = o.isolated ?? true;
  const game = { id: `${id.replace(/[^a-z0-9]/g, "")}01`.slice(0, 24), releaseId: "release_dogfood01" };
  const root = `/play/${game.id}/${game.releaseId}/`;
  const rules = readHeaders(dist);
  const calls: PlayCall[] = [];
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
  /** The shared link of `throttle`: each chunk waits for its turn on it. */
  let linkFree = 0;
  const link = async (n: number) => {
    const now = performance.now();
    const start = Math.max(now, linkFree);
    linkFree = start + (n * 8 * 1000) / (throttle * 1e6);
    await wait(linkFree - now);
  };
  const isolation: Record<string, string> = isolated ? { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "cross-origin" } : {};
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    async fetch(req) {
      const url = new URL(req.url);
      if (url.pathname.startsWith(`/__player/v2/${game.id}/${game.releaseId}/`)) {
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
      if (!url.pathname.startsWith(root)) {
        missing.push(url.pathname);
        return new Response("not found", { status: 404 });
      }
      let rel = url.pathname.slice(root.length - 1);
      if (rel.endsWith("/")) rel += "index.html";
      const path = join(dist, rel);
      if (!path.startsWith(dist) || !existsSync(path)) {
        missing.push(url.pathname);
        return new Response("not found", { status: 404 });
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
        const html = readFileSync(path, "utf8").replace(/<head[^>]*>/, (m) => `${m}<script id="__og-game-config">window.__OG_GAME=${JSON.stringify({ id: game.id, version: 1, releaseId: game.releaseId, protocol: 2 })}</script>`);
        return new Response(html, { headers });
      }
      if (!throttle) return new Response(Bun.file(path), { headers });
      const bytes = new Uint8Array(readFileSync(path));
      headers["content-length"] = String(bytes.length);
      await wait(40);
      // Charged at its brotli size: a host sends it compressed (this server does not).
      const listed = o.files?.find((x) => `/${x.path}` === rel);
      const ratio = listed && listed.bytes > 0 ? listed.brotli / listed.bytes : 1;
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
  return {
    port: server.port,
    root,
    game,
    calls,
    reports,
    missing,
    store,
    url: (query = "") => `http://127.0.0.1:${server.port}${root}${query}`,
    stop: () => server.stop(),
  };
}

// ── the accessibility mirror: what to press, and where it is drawn ────────

/** An init script: `__dog.seen[path]` (and `name:<name>`) is when each node of the mirror first appeared, ms. */
export const OBSERVE_MIRROR = `(() => {
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

/**
 * An expression: the `nth` mirrored element whose og-ui path is `what`,
 * ends in `/<what>`, or whose name is `what` (`name~<text>`: whose name
 * contains it), as `{ x, y, w, h, path }` at its centre in CSS pixels, or null.
 */
export const findInMirror = (what: string, nth = 0) => `(() => {
  const w = ${JSON.stringify(what)};
  const part = w.startsWith("name~") ? w.slice(5) : null;
  const all = [...document.querySelectorAll("[data-og-path]")].filter((el) => {
    const p = el.getAttribute("data-og-path");
    const n = (el.getAttribute("aria-label") || (el.firstChild && el.firstChild.nodeType === 3 ? el.firstChild.textContent : "") || "").trim();
    return part !== null ? n.includes(part) : p === w || p.endsWith("/" + w) || n === w;
  });
  const el = all[${nth < 0 ? `all.length + ${nth}` : nth}];
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return { x: r.left + r.width / 2, y: r.top + r.height / 2, w: r.width, h: r.height, path: el.getAttribute("data-og-path") };
})()`;

/** An expression: every mirrored element's path, role, name, states and focusability. */
export const MIRROR_TREE = `[...document.querySelectorAll("[data-og-path]")].map((el) => ({ path: el.getAttribute("data-og-path"), role: el.getAttribute("role"), name: (el.getAttribute("aria-label") || (el.firstChild && el.firstChild.nodeType === 3 ? el.firstChild.textContent : "") || ""), states: [el.getAttribute("aria-selected") === "true" ? "selected" : null, el.getAttribute("aria-checked") === "true" ? "checked" : null, el.getAttribute("aria-disabled") === "true" ? "disabled" : null, document.activeElement === el ? "focused" : null].filter(Boolean), focusable: el.tabIndex >= 0 }))`;

// ── real input ────────────────────────────────────────────────────────

/** Windows virtual key codes of the keys the harnesses press (CDP wants one for a real key event). */
export const VK: Record<string, number> = {
  Enter: 13, Escape: 27, Space: 32, Tab: 9, Backspace: 8, Delete: 46, ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40,
  ShiftLeft: 16, ShiftRight: 16, ControlLeft: 17, ControlRight: 17, AltLeft: 18, AltRight: 18, MetaLeft: 91, MetaRight: 92,
  ...Object.fromEntries([..."ABCDEFGHIJKLMNOPQRSTUVWXYZ"].map((c) => [`Key${c}`, c.charCodeAt(0)])),
  ...Object.fromEntries([..."0123456789"].map((c) => [`Digit${c}`, c.charCodeAt(0)])),
};

/** A key going down or up by its KeyboardEvent.code, as a real key event. */
export async function keyEvent(page: Page, type: "keyDown" | "keyUp", code: string, o: { modifiers?: number; text?: string } = {}): Promise<void> {
  const key = o.text ?? (code.startsWith("Key") ? code.slice(3).toLowerCase() : code.startsWith("Digit") ? code.slice(5) : code === "Space" ? " " : code);
  await page.send("Input.dispatchKeyEvent", {
    type,
    code,
    key,
    windowsVirtualKeyCode: VK[code] ?? 0,
    ...(o.modifiers ? { modifiers: o.modifiers } : {}),
    ...(type === "keyDown" && o.text ? { text: o.text, unmodifiedText: o.text } : {}),
  });
}

/** A real mouse press where `at` is: moved there, pressed, released, with a human's pauses. */
export async function pressAt(page: Page, at: { x: number; y: number }): Promise<void> {
  await page.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: at.x, y: at.y });
  await wait(50);
  await page.send("Input.dispatchMouseEvent", { type: "mousePressed", x: at.x, y: at.y, button: "left", clickCount: 1 });
  await wait(70);
  await page.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: at.x, y: at.y, button: "left", clickCount: 1 });
}
