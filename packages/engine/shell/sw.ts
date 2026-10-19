// The exported game's service worker (sw.js, publish builds): every file of
// the build's first group is precached under a cache named by the build's
// content hash, so the game plays offline after its first load and a new
// build replaces the old one atomically. A streaming group's pack is not
// precached: it is cached in the same cache when the engine first fetches it,
// so a place the player has been plays offline too. `og build` prepends
// `const OG_SW = { version, files, lazy }` (the hash, the precached relative
// paths and the packs) to this file's bundle.
//
// - install: fetch every precached file into `og-<version>`; a failure keeps the old build.
// - activate: delete the other `og-*` caches of this scope, take the clients.
// - fetch: the page (a navigation, index.html) from the network first, so
//   whatever the host adds to it (the Stage proxy's told origin, the Player's
//   modes) is current, and the cached index.html offline; every other
//   precached GET from the cache; a pack from the cache, or from the network
//   into the cache; anything else from the network. Responses
//   keep their own headers, so COOP/COEP survive offline.

declare const OG_SW: { readonly version: string; readonly files: readonly string[]; readonly lazy?: readonly string[] };

export {};

interface SwScope {
  readonly registration: { readonly scope: string };
  readonly clients: { claim(): Promise<void> };
  skipWaiting(): Promise<void>;
  addEventListener(type: "install" | "activate", fn: (e: { waitUntil(p: Promise<unknown>): void }) => void): void;
  addEventListener(type: "fetch", fn: (e: { request: Request; respondWith(r: Promise<Response>): void }) => void): void;
}
const scope = globalThis as unknown as SwScope;

const CACHE = `og-${OG_SW.version}`;
const base = new URL(scope.registration.scope);
const urls = new Set(OG_SW.files.map((f) => new URL(f, base).href));
const lazy = new Set((OG_SW.lazy ?? []).map((f) => new URL(f, base).href));
const index = new URL("index.html", base).href;

scope.addEventListener("install", (e) => {
  e.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll([...urls]))
      .then(() => scope.skipWaiting()),
  );
});

scope.addEventListener("activate", (e) => {
  e.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k.startsWith("og-") && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => scope.clients.claim()),
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
          return await fetch(e.request);
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
