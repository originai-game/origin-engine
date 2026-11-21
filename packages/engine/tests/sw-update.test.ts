// The publish build's service worker (shell/sw.ts) updating from one build
// to the next: only what changed is downloaded; every file whose content the
// old build's cache holds (precached, or a pack the player visited) is
// carried over by its hash (docs/web-export.md §Incremental update). Run on
// a CacheStorage and fetch of this file's own, so it needs no browser.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { installFetcher, installTranscoder, preloadBundle } from "../src/web/boot.ts";
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
        // A streamed video's index and chunks: its players fetch them.
        { id: "clips/intro.ogt#stream", file: "clips/intro.ogt.stream.ogb", type: "og.VideoStream" },
        { id: "clips/intro.ogt#stream.0", file: "clips/intro.ogt.stream.0.ogb", type: "og.VideoChunk" },
        { id: "clips/intro.ogt#streamhevc.0", file: "clips/intro.ogt.streamhevc.0.ogb", type: "og.VideoChunk" },
      ]),
      og_web_bundle_integrity: () => "{}",
      og_web_put_file: (path) => { stored.push(path); },
      og_web_fail_file: (path, reason) => { throw new Error(`${path}: ${reason}`); },
    }, "https://play.example", () => {});
    expect(result.integrity).toBeNull();
    // The pack's index boots (it is a few hundred bytes); its face is fetched when the locale's chain needs it.
    expect(fetched).toEqual(["https://play.example/cooked/.cook.ogt", "https://play.example/cooked/a.png.ogb", "https://play.example/cooked/fonts/japanese/extension.json.ogb"]);
    expect(stored).toEqual(fetched);
    expect(result.files).toBe(2);
    expect(result.failed).toBe(0);
    expect(result.streamed).toBe(6);
  } finally {
    globalThis.fetch = fetchBefore;
  }
});

// A package on an asset base (docs/web-export.md §Asset base): the manifest
// is fetched against the page's integrity for it, and every file the boot
// or the engine fetches later against the manifest's own, so the browser
// refuses bytes that differ. A refusal is the bytes, not the network: one
// fetch, never retried, and the file fails naming its integrity.
test("a package from an asset base is fetched file by file against its integrity", async () => {
  const fetchBefore = globalThis.fetch;
  const asked: Array<[string, string | undefined]> = [];
  const failed: string[] = [];
  const base = "https://cdn.example/g/1";
  const sri = { "a.png.ogb": "sha384-AAA", "b.png.ogb": "sha384-BBB", "stream/vale.ogb": "sha384-VVV" };
  try {
    globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
      asked.push([String(input), init?.integrity]);
      if (String(input).endsWith("b.png.ogb")) throw new TypeError("Failed to fetch");
      return new Response("bytes");
    }) as typeof fetch;
    const glue = {
      og_web_bundle_files: () => JSON.stringify([
        { id: "a.png", file: "a.png.ogb", type: "og.Texture" },
        { id: "b.png", file: "b.png.ogb", type: "og.Texture" },
        { id: "v.png", file: "a.png.ogb", type: "og.Texture", group: "vale" },
      ]),
      og_web_bundle_integrity: () => JSON.stringify(sri),
      og_web_put_file: () => {},
      og_web_fail_file: (path: string, reason: string) => void failed.push(`${path}: ${reason}`),
    };
    const result = await preloadBundle(glue, base, () => {}, undefined, "sha384-MMM");
    expect(asked).toEqual([
      [`${base}/cooked/.cook.ogt`, "sha384-MMM"],
      [`${base}/cooked/a.png.ogb`, "sha384-AAA"],
      [`${base}/cooked/b.png.ogb`, "sha384-BBB"],
    ]);
    expect(failed).toHaveLength(1);
    expect(failed[0]).toContain(`${base}/cooked/b.png.ogb`);
    expect(failed[0]).toContain("integrity");
    expect(result.integrity).toEqual(sri);
    // A pack the engine asks for later: against its integrity too.
    let fetcher: ((path: string) => void) | undefined;
    installFetcher({ ...glue, og_web_set_fetcher: (f: (path: string) => void) => void (fetcher = f) } as never, result.integrity);
    asked.length = 0;
    fetcher!(`${base}/cooked/stream/vale.ogb`);
    await new Promise((ok) => setTimeout(ok, 0));
    expect(asked).toEqual([[`${base}/cooked/stream/vale.ogb`, "sha384-VVV"]]);
  } finally {
    globalThis.fetch = fetchBefore;
  }
});

test("every Basis texture the engine asks for is transcoded by other Workers, and by this one when they break", async () => {
  const g = globalThis as { fetch: typeof fetch; Worker?: unknown; location?: unknown };
  const before = { fetch: g.fetch, Worker: g.Worker, location: g.location };
  const made: FakeWorker[] = [];
  let broken = false;
  let asks = 0;
  // A transcoding Worker that answers each job a turn later with levels naming it, or never loads.
  class FakeWorker {
    onmessage: ((e: { data: unknown }) => void) | null = null;
    onerror: (() => void) | null = null;
    fails = broken;
    stopped = false;
    readonly url: string;
    constructor(url: string) {
      this.url = url;
      made.push(this);
    }
    postMessage(job: { type: string; id: number; ktx2: Uint8Array; target: string }): void {
      asks++;
      // One that was stopped answers nothing, as a real one.
      if (this.stopped) return;
      setTimeout(() => {
        if (this.fails) return this.onerror?.();
        // A copy of the engine's Worker also posts its console's lines: not an answer.
        this.onmessage?.({ data: { type: "log", lines: ["a line"] } });
        this.onmessage?.({ data: { type: "og-basis", id: job.id, format: job.target, width: 4, height: 4, srgb: true, levels: Uint8Array.of(0xa5, job.ktx2[0]!) } });
      }, 5);
    }
    terminate(): void {
      this.stopped = true;
    }
  }
  try {
    g.Worker = FakeWorker;
    g.location = { href: "https://game.test/worker.js" };
    const events: string[] = [];
    let transcode: ((key: string, ktx2: Uint8Array, target: string) => void) | undefined;
    let settle = (): void => {};
    const glue = {
      og_web_set_transcoder: (f: typeof transcode) => void (transcode = f),
      og_web_put_texture: () => "",
      og_web_put_texture_encoded: (key: string, format: string, _w: number, _h: number, _srgb: boolean, levels: Uint8Array) => (events.push(`texture ${key} ${format} ${[...levels]}`), settle(), ""),
      og_web_fail_texture: (key: string, reason: string) => (events.push(`failed ${key}: ${reason}`), settle()),
    };
    installTranscoder(glue as never, undefined);
    // The engine's texture hook asks for jobs; resolves once `n` answers came back.
    const asked = (jobs: Array<[string, number]>, n = jobs.length) =>
      new Promise<void>((ok) => {
        const start = events.length;
        settle = () => void (events.length - start >= n && ok());
        for (const [key, byte] of jobs) transcode!(key, Uint8Array.of(byte), "bc7");
      });

    await asked([["1:a", 1], ["2:b", 2], ["3:c", 3]]);
    expect(made.length).toBeGreaterThan(0);
    expect(made.every((w) => w.url === "https://game.test/worker.js")).toBe(true);
    // Every texture is the Workers' answer, under the key the engine gave it.
    expect(events.sort()).toEqual(["texture 1:a bc7 165,1", "texture 2:b bc7 165,2", "texture 3:c bc7 165,3"]);

    // A Worker that breaks hands its job to this Worker's own transcoder, which refuses these bytes
    // (they are no KTX2): the texture fails with the transcoder's reason, and nothing waits for ever.
    events.length = 0;
    broken = true;
    for (const w of made) w.fails = true;
    await asked([["4:d", 4]]);
    expect(events).toHaveLength(1);
    expect(events[0]).toContain("failed 4:d");
    expect(events[0]).toContain("KTX2");
    // And no Worker is started or asked for the textures after it.
    const [workers, jobs] = [made.length, asks];
    events.length = 0;
    await asked([["5:e", 5]]);
    expect(events[0]).toContain("KTX2");
    expect([made.length, asks]).toEqual([workers, jobs]);
  } finally {
    Object.assign(g, before);
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

test("a streamed video's units and every rendition are cached when fetched, never precached nor counted in the boot", () => {
  const meta = "cooked/clips/intro.ogt.ogb";
  const poster = "cooked/clips/intro.ogt.poster.ogb";
  const streams = ["stream", "stream.0", "stream.12", "stream720", "stream720.0", "streamhevc", "streamhevc.3", "streamh264", "streamh264.0"].map((s) => `cooked/clips/intro.ogt.${s}.ogb`);
  const lists = serviceWorkerLists(["cooked/.cook.ogt", meta, poster, ...streams], new Set());
  expect(lists.lazy).toEqual([...streams].sort());
  expect(lists.files).toEqual(["./", "cooked/.cook.ogt", meta, poster]);
  const row = (path: string, bytes: number) => ({ path, bytes, compressed: bytes, codec: "brotli-11" as const });
  const d = downloadsOf([row(meta, 100), row(poster, 2_000), ...streams.map((s) => row(s, 1_000_000))]);
  expect(d.boot.single.cooked.bytes).toBe(2_100);
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
    // A file on another origin (an asset base) by its whole URL.
    const key = url.startsWith(SCOPE) ? url.slice(SCOPE.length) || "./" : url;
    fetched.push(key);
    const body = server[key];
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

// A build on an asset base keeps its page, engine and code next to the
// page and its cooked/ on the base: the worker precaches and caches each
// cooked file under its URL there (the URL the engine fetches), and a
// later build carries an unchanged one over by its content.
test("a build on an asset base caches its cooked files under their URLs on that base", async () => {
  const storage = cacheStorage();
  const cdn = "https://cdn.example/g/1/";
  const og = {
    version: "3333333333333333",
    files: ["./", "index.html", "cooked/.cook.ogt", "cooked/hero.png.ogb"],
    single: ["origin-engine_bg.111111111111.wasm"],
    lanes: [],
    lazy: ["cooked/stream/vale.ogb"],
    hashes: { "index.html": "i3", "origin-engine_bg.111111111111.wasm": "e1", "cooked/.cook.ogt": "m3", "cooked/hero.png.ogb": "h1", "cooked/stream/vale.ogb": "v1" },
    assets: "https://cdn.example/g/1",
  };
  const server = { "./": "page", "index.html": "page", "origin-engine_bg.111111111111.wasm": "engine", [`${cdn}cooked/.cook.ogt`]: "manifest", [`${cdn}cooked/hero.png.ogb`]: "hero" };
  expect(await install(og, server, storage)).toEqual(["./", "https://cdn.example/g/1/cooked/.cook.ogt", "https://cdn.example/g/1/cooked/hero.png.ogb", "index.html", "origin-engine_bg.111111111111.wasm"]);
  const cache = storage.stores.get([...storage.stores.keys()][0]!)!;
  expect(cache.get(`${cdn}cooked/hero.png.ogb`)?.body).toBe("hero");
  expect(cache.has(`${SCOPE}cooked/hero.png.ogb`)).toBeFalse();
  // The index names the base's URL, so the next build carries the hero over.
  expect(JSON.parse(cache.get(`${SCOPE}og-sw-index.json`)!.body)[`${cdn}cooked/hero.png.ogb`]).toBe("h1");
});
