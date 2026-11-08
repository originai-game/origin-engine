// The publish build's service worker (shell/sw.ts) updating from one build
// to the next: only what changed is downloaded; every file whose content the
// old build's cache holds (precached, or a pack the player visited) is
// carried over by its hash (docs/web-export.md §Incremental update). Run on
// a CacheStorage and fetch of this file's own, so it needs no browser.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { preloadBundle } from "../src/web/worker.ts";
import { downloadsOf, serviceWorkerLists, streamGroupsOf } from "../src/node/web-build.ts";

declare const Bun: { build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; outputs: Array<{ text(): Promise<string> }> }> };

test("publish keeps a shared file needed outside a streaming pack", () => {
  const output = (id: string, file: string, group: boolean) => `  output "${id}" {\n    file "${file}"\n${group ? '    group "later"\n' : ""}  }\n`;
  for (const reversed of [false, true]) {
    const outputs = [output("a", "shared.ogb", true), output("b", "shared.ogb", false), output("c", "only-packed.ogb", true)];
    if (reversed) outputs.reverse();
    expect(streamGroupsOf(outputs.join("")).grouped).toEqual(["only-packed.ogb"]);
  }
});

test("bundle preload fetches a shared file once but retains every texture id, and leaves font-pack faces for later", async () => {
  const fetchBefore = globalThis.fetch;
  const fetched: string[] = [];
  const stored: string[] = [];
  try {
    globalThis.fetch = (async (input: string | URL | Request) => {
      fetched.push(String(input));
      return new Response("shared bytes");
    }) as typeof fetch;
    const result = await preloadBundle({
      og_web_bundle_files: () => JSON.stringify([
        { id: "z.png", file: "a.png.ogb", type: "og.Texture" },
        { id: "a.png", file: "a.png.ogb", type: "og.Texture" },
        { id: "stream.png", file: "a.png.ogb", type: "og.Texture", group: "later" },
        { id: "a.png#mips", file: "a.png.mips.ogb", type: "og.Texture" },
        { id: "fonts/japanese/extension.json", file: "fonts/japanese/extension.json.ogb", type: "og.FontPack" },
        { id: "fonts/japanese/NotoSansJP-Regular.ttf", file: "fonts/japanese/NotoSansJP-Regular.ttf.face.ogb", type: "og.Font" },
      ]),
      og_web_put_file: (path) => { stored.push(path); },
      og_web_fail_file: (path, reason) => { throw new Error(`${path}: ${reason}`); },
    }, "https://play.example", () => {});
    // The pack's index boots (it is a few hundred bytes); its face is fetched when the locale's chain needs it.
    expect(fetched).toEqual(["https://play.example/cooked/.cook.ogt", "https://play.example/cooked/a.png.ogb", "https://play.example/cooked/fonts/japanese/extension.json.ogb"]);
    expect(stored).toEqual(fetched);
    expect(result.files).toBe(2);
    expect(result.failed).toBe(0);
    expect(result.streamed).toBe(3);
    expect(result.textures.map((t) => t.id)).toEqual(["z.png", "a.png"]);
    expect(result.textures[0]!.bytes).toBe(result.textures[1]!.bytes);
  } finally {
    globalThis.fetch = fetchBefore;
  }
});

test("a font pack's faces are cached when fetched, never precached nor counted in the boot", () => {
  const face = "cooked/fonts/japanese/NotoSansJP-Regular.ttf.face.ogb";
  const index = "cooked/fonts/japanese/extension.json.ogb";
  const ui = "cooked/fonts/Display.ttf.ogb";
  const lists = serviceWorkerLists(["cooked/.cook.ogt", ui, index, face], new Set());
  expect(lists.lazy).toEqual([face]);
  expect(lists.files).toEqual(["./", "cooked/.cook.ogt", ui, index]);
  const row = (path: string, bytes: number) => ({ path, bytes, compressed: bytes, codec: "brotli-11" as const });
  const d = downloadsOf([row(ui, 40_000), row(index, 300), row(face, 900_000)]);
  expect(d.boot.single.cooked.bytes).toBe(40_300);
  expect(d.later.fonts).toEqual({ files: 1, bytes: 900_000, compressed: 900_000 });
});

const SCOPE = "https://play.example/g/r/";
const swSource = (async () => {
  const r = await Bun.build({ entrypoints: [join(dirname(fileURLToPath(import.meta.url)), "..", "shell", "sw.ts")], target: "browser", format: "esm" });
  if (!r.success) throw new Error("sw.ts did not bundle");
  return r.outputs[0]!.text();
})();

/** A CacheStorage whose caches hold response bodies by URL. */
function cacheStorage() {
  const stores = new Map<string, Map<string, { body: string; type: string }>>();
  const cache = (m: Map<string, { body: string; type: string }>) => ({
    match: async (url: string) => {
      const hit = m.get(url);
      return hit ? new Response(hit.body, { headers: { "Content-Type": hit.type } }) : undefined;
    },
    put: async (url: string, r: Response) => void m.set(url, { body: await r.text(), type: r.headers.get("Content-Type") ?? "" }),
    addAll: async (urls: string[]) => {
      const got = await Promise.all(urls.map(async (u) => [u, await fetchOf(u)] as const));
      for (const [u, r] of got) {
        if (!r.ok) throw new Error(`${u}: ${r.status}`);
        m.set(u, { body: await r.text(), type: "" });
      }
    },
  });
  let fetchOf: (url: string) => Promise<Response> = async () => new Response("", { status: 404 });
  return {
    stores,
    setFetch: (f: typeof fetchOf) => (fetchOf = f),
    api: {
      keys: async () => [...stores.keys()],
      open: async (name: string) => {
        if (!stores.has(name)) stores.set(name, new Map());
        return cache(stores.get(name)!);
      },
      delete: async (name: string) => stores.delete(name),
    },
  };
}

/** Runs one build's sw.js (its OG_SW) against `caches`, through install and activate; answers the URLs it fetched. */
async function install(og: Record<string, unknown>, server: Record<string, string>, storage: ReturnType<typeof cacheStorage>): Promise<string[]> {
  const fetched: string[] = [];
  const fetchFn = async (input: string | Request) => {
    const url = typeof input === "string" ? input : input.url;
    fetched.push(url.slice(SCOPE.length) || "./");
    const body = server[url.slice(SCOPE.length) || "./"];
    return body === undefined ? new Response("", { status: 404 }) : new Response(body);
  };
  storage.setFetch(fetchFn);
  const handlers: Record<string, (e: { waitUntil(p: Promise<unknown>): void }) => void> = {};
  const scope = {
    registration: { scope: SCOPE, navigationPreload: { enable: async () => {} } },
    clients: { claim: async () => {} },
    skipWaiting: async () => {},
    navigator: { hardwareConcurrency: 4 },
    crossOriginIsolated: false,
    addEventListener: (type: string, fn: (e: { waitUntil(p: Promise<unknown>): void }) => void) => (handlers[type] = fn),
  };
  const code = `const OG_SW=${JSON.stringify(og)};\n${await swSource}`;
  new Function("globalThis", "caches", "fetch", code)(scope, storage.api, fetchFn);
  for (const type of ["install", "activate"]) {
    let done: Promise<unknown> = Promise.resolve();
    handlers[type]!({ waitUntil: (p) => (done = p) });
    await done;
  }
  return fetched.sort();
}

test("a new build's service worker downloads only what changed, carrying the rest and visited packs over from the old cache", async () => {
  const storage = cacheStorage();
  const one = {
    "./": "page 1",
    "index.html": "page 1",
    "game.aaaaaaaaaaaa.js": "game 1",
    "origin-engine_bg.111111111111.wasm": "engine",
    "cooked/.cook.ogt": "manifest 1",
    "cooked/hero.png.ogb": "hero",
    "cooked/unused.png.ogb": "unused candidate",
    "cooked/stream/vale.ogb": "vale",
  };
  const first = await install(
    {
      version: "1111111111111111",
      files: ["./", "index.html", "game.aaaaaaaaaaaa.js", "cooked/.cook.ogt", "cooked/hero.png.ogb", "cooked/unused.png.ogb"],
      single: ["origin-engine_bg.111111111111.wasm"],
      lanes: [],
      lazy: ["cooked/stream/vale.ogb"],
      hashes: { "index.html": "i1", "game.aaaaaaaaaaaa.js": "g1", "origin-engine_bg.111111111111.wasm": "e1", "cooked/.cook.ogt": "m1", "cooked/hero.png.ogb": "h1", "cooked/unused.png.ogb": "u1", "cooked/stream/vale.ogb": "v1" },
    },
    one,
    storage,
  );
  expect(first).toEqual(["./", "cooked/.cook.ogt", "cooked/hero.png.ogb", "cooked/unused.png.ogb", "game.aaaaaaaaaaaa.js", "index.html", "origin-engine_bg.111111111111.wasm"]);
  // The player visits the vale: the engine fetches its pack, the worker caches it.
  const old = [...storage.stores.keys()][0]!;
  storage.stores.get(old)!.set(`${SCOPE}cooked/stream/vale.ogb`, { body: "vale", type: "" });

  // Build 2 changes the game code and the cook manifest; the engine, the hero texture and the pack are the same bytes.
  const two = { ...one, "./": "page 2", "index.html": "page 2", "game.bbbbbbbbbbbb.js": "game 2", "cooked/.cook.ogt": "manifest 2" };
  const second = await install(
    {
      version: "2222222222222222",
      files: ["./", "index.html", "game.bbbbbbbbbbbb.js", "cooked/.cook.ogt", "cooked/hero.png.ogb"],
      single: ["origin-engine_bg.111111111111.wasm"],
      lanes: [],
      lazy: ["cooked/stream/vale.ogb"],
      hashes: { "index.html": "i2", "game.bbbbbbbbbbbb.js": "g2", "origin-engine_bg.111111111111.wasm": "e1", "cooked/.cook.ogt": "m2", "cooked/hero.png.ogb": "h1", "cooked/stream/vale.ogb": "v1" },
    },
    two,
    storage,
  );
  // Only what changed (and the page, which is always the network's).
  expect(second).toEqual(["./", "cooked/.cook.ogt", "game.bbbbbbbbbbbb.js", "index.html"]);
  // The old build's cache is gone after activation; the new one holds everything, the visited pack included.
  expect(storage.stores.size).toBe(1);
  const now = storage.stores.get([...storage.stores.keys()][0]!)!;
  expect(now.has(`${SCOPE}cooked/unused.png.ogb`)).toBeFalse();
  expect(now.get(`${SCOPE}origin-engine_bg.111111111111.wasm`)?.body).toBe("engine");
  expect(now.get(`${SCOPE}cooked/hero.png.ogb`)?.body).toBe("hero");
  expect(now.get(`${SCOPE}cooked/stream/vale.ogb`)?.body).toBe("vale");
  expect(now.get(`${SCOPE}game.bbbbbbbbbbbb.js`)?.body).toBe("game 2");
  expect(now.get(`${SCOPE}cooked/.cook.ogt`)?.body).toBe("manifest 2");
});
