// The exported game's service worker (sw.js, publish builds): every file of
// the build's first group is precached under a cache named by the build's
// content hash, so the game plays offline after its first load and a new
// build replaces the old one atomically. A streaming group's pack is not
// precached: it is cached in the same cache when the engine first fetches it,
// so a place the player has been plays offline too. `og build` prepends
// `const OG_SW = { version, files, lazy, single, lanes }` (the hash, the
// precached relative paths, the packs and long sounds' chunks, and the two
// engine variants' files)
// to this file's bundle.
//
// The engine comes in two variants (docs/web-export.md §Lanes) and a page
// boots one by the shell's rule (src/web/lanes.ts `laneChoice`: the build
// ships lanes, the page is cross-origin isolated, more than two cores). The
// worker precaches the variant the same rule picks in its own scope (it is
// isolated when sw.js got the page's COEP, which `_headers` sends to every
// file; a worker sees the page's core count) and caches the other on first
// use, like a pack; a wrong guess costs one download, never a failed boot
// (the engine Worker falls back to the single-threaded variant). Once the
// engine is ready, the page names the variant it booted (`og-cache`), and
// the worker caches whichever of that variant's files it lacks, so the
// variant the page runs plays offline even when the guess was wrong or the
// lanes pool failed and the page fell back.
//
// - message `{ type: "og-cache", files }`: files of one variant (absolute
//   URLs); each names its whole variant (the lanes variant includes
//   snippets/), and only the build's own variant files are cached.
// - install: fetch every precached file into `og-<scope>-<version>`; a failure keeps the old build.
//   CacheStorage is per origin, so the name carries a hash of the registration's scope:
//   builds served side by side on one origin keep their own caches.
// - activate: delete this scope's other caches (`og-<scope>-*`, and an older build's
//   unscoped `og-<version>` only when it holds this scope's page), take the clients.
// - fetch: the page (a navigation, index.html) from the network first, so
//   whatever the host adds to it (the Stage proxy's told origin, the
//   platform's `window.__OG_GAME`, the Player's modes) is current, and the
//   cached index.html offline; every other precached GET from the cache; a
//   pack from the cache, or from the network into the cache; anything else
//   from the network. Responses keep their own headers, so COOP/COEP survive
//   offline.
//
// A navigation is answered from the browser's own navigation request
// (navigation preload, enabled on activation), never a copy the worker
// fetches: a worker's `fetch(event.request)` of a navigation leaves as
// `Sec-Fetch-Dest: empty`, so a host that answers a document differently
// from a plain read (the platform serves a release's derived entry, with
// `window.__OG_GAME`, only to a document or frame load) would hand the
// revisiting page its stored bytes instead. Only where the browser has no
// navigation preload does the worker fetch the page itself.

declare const OG_SW: {
  readonly version: string;
  readonly files: readonly string[];
  readonly lazy?: readonly string[];
  readonly single?: readonly string[];
  readonly lanes?: readonly string[];
};

import { laneChoice } from "../src/web/lanes.ts";

interface SwScope {
  readonly registration: { readonly scope: string; readonly navigationPreload?: { enable(): Promise<void> } };
  readonly clients: { claim(): Promise<void> };
  skipWaiting(): Promise<void>;
  addEventListener(type: "install" | "activate", fn: (e: { waitUntil(p: Promise<unknown>): void }) => void): void;
  addEventListener(type: "fetch", fn: (e: { request: Request; preloadResponse?: Promise<Response | undefined>; respondWith(r: Promise<Response>): void }) => void): void;
  addEventListener(type: "message", fn: (e: { data: unknown; waitUntil(p: Promise<unknown>): void }) => void): void;
}
const scope = globalThis as unknown as SwScope;

/** FNV-1a, 32 bits, as 8 hex digits: a short stable name for the scope, not a secret. */
function scopeHash(text: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 0x01000193);
  return (h >>> 0).toString(16).padStart(8, "0");
}
const SCOPE = `og-${scopeHash(scope.registration.scope)}-`;
const CACHE = SCOPE + OG_SW.version;
const base = new URL(scope.registration.scope);
const variants = { single: OG_SW.single ?? [], lanes: OG_SW.lanes ?? [] };
const cores = (globalThis as { navigator?: { hardwareConcurrency?: number } }).navigator?.hardwareConcurrency ?? 0;
const choice = laneChoice(variants.lanes.length > 0, (globalThis as { crossOriginIsolated?: boolean }).crossOriginIsolated === true, cores);
const [mine, other] = "threads" in choice ? [variants.lanes, variants.single] : [variants.single, variants.lanes];
const urls = new Set([...OG_SW.files, ...mine].map((f) => new URL(f, base).href));
const lazy = new Set([...(OG_SW.lazy ?? []), ...other].map((f) => new URL(f, base).href));
const index = new URL("index.html", base).href;

scope.addEventListener("install", (e) => {
  e.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll([...urls]))
      .then(() => scope.skipWaiting()),
  );
});

// The variant the page booted, cached whole: each of its files names its variant.
const variantOf = new Map<string, readonly string[]>();
for (const files of [variants.single, variants.lanes]) {
  const hrefs = files.map((f) => new URL(f, base).href);
  for (const h of hrefs) variantOf.set(h, hrefs);
}
scope.addEventListener("message", (e) => {
  const m = e.data as { type?: unknown; files?: unknown } | null;
  if (!m || m.type !== "og-cache" || !Array.isArray(m.files)) return;
  const wanted = new Set<string>();
  for (const f of m.files) {
    if (typeof f !== "string") continue;
    for (const h of variantOf.get(f) ?? []) wanted.add(h);
  }
  if (wanted.size === 0) return;
  e.waitUntil(
    caches
      .open(CACHE)
      .then(async (cache) => {
        const missing: string[] = [];
        for (const h of wanted) if (!(await cache.match(h))) missing.push(h);
        if (missing.length > 0) await cache.addAll(missing);
      })
      .catch(() => {}),
  );
});

scope.addEventListener("activate", (e) => {
  e.waitUntil(
    Promise.all([
      // The browser makes navigations itself (as documents), in parallel with this worker's start.
      scope.registration.navigationPreload?.enable().catch(() => {}),
      caches.keys().then((keys) =>
        Promise.all(
          keys.map(async (k) => {
            if (k === CACHE) return;
            const mine = k.startsWith(SCOPE) || (/^og-[0-9a-f]{16}$/.test(k) && (await (await caches.open(k)).match(base.href)) !== undefined);
            if (mine) await caches.delete(k);
          }),
        ),
      ),
    ]).then(() => scope.clients.claim()),
  );
});

scope.addEventListener("fetch", (e) => {
  if (e.request.method !== "GET") return;
  const url = new URL(e.request.url);
  url.search = "";
  url.hash = "";
  const key = url.href === base.href ? index : url.href;
  const page = e.request.mode === "navigate" || key === index;
  if (lazy.has(key)) {
    e.respondWith(
      caches.open(CACHE).then(async (cache) => {
        const hit = await cache.match(key);
        if (hit) return hit;
        const r = await fetch(e.request);
        if (r.ok) await cache.put(key, r.clone());
        return r;
      }),
    );
    return;
  }
  if (!urls.has(key) && !page) return;
  e.respondWith(
    caches.open(CACHE).then(async (cache) => {
      // The page from the network first (a host may add to it), the cache offline.
      if (page) {
        try {
          const preloaded = e.request.mode === "navigate" ? await e.preloadResponse : undefined;
          return preloaded ?? (await fetch(e.request));
        } catch (err) {
          const hit = await cache.match(index);
          if (hit) return hit;
          throw err;
        }
      }
      return (await cache.match(key)) ?? fetch(e.request);
    }),
  );
});
