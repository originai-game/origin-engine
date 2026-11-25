// The engine wasm's split modules (docs/web-export.md §Modules). A publish
// wasm is cut by Binaryen's wasm-split (scripts/wasm-split.ts) into the
// primary, which boots, and one module per optional capability (physics,
// nav, story, motion, destruction, video). The primary imports a placeholder
// for each function a module holds (namespace `placeholder.<module>`, field
// the function's slot in the primary's placeholder table, exported as
// `%table_<n>` beside the function table `%table`); a
// module, once instantiated against the primary's exports, writes its
// functions into those slots. A placeholder called before its module arrived
// loads the module there and then (the tick waits for it) and calls the
// function it stood for, so a module that loads late changes no result: the
// split moves code, never what it does.
//
// The placeholder protocol itself is src/web/split-protocol.js, shared with
// the lanes pool Workers' installer (scripts/og-pool-split.js).

/** How a host fetches and instantiates one split module. */
export interface ModuleHost {
  /** Instantiates module `name` with `imports`: the prefetch after the first frame, and every load on a host without loadSync. */
  load(name: string, imports: WebAssembly.Imports): Promise<WebAssembly.Instance>;
  /**
   * The same, synchronously: an engine call into a module that has not
   * arrived yet, which the tick waits for. A host that cannot load
   * synchronously (WeChat) leaves it out; such a call then throws, so that
   * host loads the game's modules before its first tick (loadEngineModules).
   */
  loadSync?(name: string, imports: WebAssembly.Imports): WebAssembly.Instance;
  /**
   * Where the modules are, for the lanes pool's Workers (scripts/og-pool-split.js): each
   * instantiates the primary again and loads a module into its own instance when a lane first
   * calls into it. A host without it cannot serve a split lanes primary's pool.
   */
  readonly pool?: PoolSplit;
}

/** A module host's `base` and `files`, as the pool Workers fetch from them. */
export interface PoolSplit {
  readonly base: string;
  readonly files: Readonly<Record<string, string>>;
}

/**
 * What wasm-bindgen-rayon's Worker helper hands each pool Worker it starts (scripts/rayon-worker.sh
 * adds it to the Worker's init message): the engine's module host's {@link PoolSplit}, or null.
 */
declare global {
  var __ogPoolSplit: PoolSplit | null | undefined;
}

import { moduleUrl, placeholderNamespace, placeholderTable, withPlaceholderNamespaces } from "./split-protocol.js";

/** The loaded engine's split modules: one per process, as the binding is. */
interface Split {
  host: ModuleHost | null;
  exports: WebAssembly.Exports | null;
  /** A module's load: done, or in flight. */
  loads: Map<string, true | Promise<void>>;
}
let split: Split | null = null;

function imports(): WebAssembly.Imports {
  if (!split?.exports) throw new Error("the engine wasm is not instantiated yet");
  return { primary: split.exports as Record<string, WebAssembly.ImportValue> };
}

/** Module `name` in place now, loaded synchronously when it is not. */
function loadNow(name: string): void {
  const s = split!;
  if (s.loads.get(name) === true) return;
  if (!s.host?.loadSync) {
    throw new Error(`engine module ${name} is not loaded: this host loads modules only asynchronously, so it loads the game's modules (og-build.json modules) before its first tick`);
  }
  s.host.loadSync(name, imports());
  s.loads.set(name, true);
}

/** The split table of the engine's loaded primary, or null for an unsplit wasm. */
function table(): WebAssembly.Table | null {
  return split?.exports ? placeholderTable(split.exports) : null;
}

/** The split table, refusing to run without it (a placeholder call in an unsplit wasm). */
function splitTable(): WebAssembly.Table {
  const t = table();
  if (!t) throw new Error("the engine wasm has no placeholder table");
  return t;
}

/**
 * Runs `init` (the glue's initialisation) with `WebAssembly.instantiate` and
 * `instantiateStreaming`, whatever they are at the time (a mini-game host's
 * shim included), passing the placeholder namespaces in `imports` and
 * keeping the instance's exports. An unsplit wasm asks for none of them.
 */
export async function instantiateSplit<T>(host: ModuleHost | null, init: () => Promise<T>): Promise<T> {
  split = { host, exports: null, loads: new Map() };
  globalThis.__ogPoolSplit = host?.pool ?? null;
  const wa = WebAssembly as unknown as Record<string, ((...args: unknown[]) => unknown) | undefined>;
  const saved = { instantiate: wa.instantiate, instantiateStreaming: wa.instantiateStreaming };
  const keep = (r: unknown) => {
    const instance = r instanceof WebAssembly.Instance ? r : (r as { instance: WebAssembly.Instance }).instance;
    split!.exports = instance.exports;
    return r;
  };
  for (const k of ["instantiate", "instantiateStreaming"] as const) {
    const original = saved[k];
    if (!original) continue;
    wa[k] = (source: unknown, given?: unknown) =>
      Promise.resolve(original.call(WebAssembly, source, withPlaceholderNamespaces(given as WebAssembly.Imports | undefined, loadNow, splitTable))).then(keep);
  }
  try {
    return await init();
  } finally {
    wa.instantiate = saved.instantiate;
    wa.instantiateStreaming = saved.instantiateStreaming;
  }
}

/** Loads the named split modules now; a loaded one, and every name for an unsplit wasm, is a no-op. */
export async function loadEngineModules(names: readonly string[]): Promise<void> {
  const s = split;
  if (!s?.exports || !placeholderTable(s.exports)) return;
  await Promise.all(
    names.map((name) => {
      const at = s.loads.get(name);
      if (at === true) return;
      if (at) return at;
      if (!s.host) throw new Error(`engine module ${name}: no module host to load it with`);
      const p = s.host.load(name, imports()).then(() => {
        s.loads.set(name, true);
      });
      s.loads.set(name, p);
      return p;
    }),
  );
}

/** The split modules loaded so far, by name (the probe's and a test's view). */
export function loadedEngineModules(): string[] {
  return split ? [...split.loads].filter(([, v]) => v === true).map(([k]) => k).sort() : [];
}

/**
 * Where a build's split modules are (og-build.json `modules`,
 * docs/web-export.md §Modules): `base` (the engine version's directory on the
 * CDN, or `modules/` beside the page for a build that copies them), each
 * module's file under it, and the ones the game references, which the boot
 * fetches beside the primary.
 */
export interface EngineModules {
  readonly base: string;
  readonly files: Readonly<Record<string, string>>;
  readonly referenced: readonly string[];
  /** Optional sha256 hex digest per module, verified with Subresource Integrity on fetch. */
  readonly hashes?: Readonly<Record<string, string>>;
}

/** Converts a 64-char sha256 hex digest to an SRI `sha256-<base64>` string. */
function sriOf(hex: string | undefined): string | undefined {
  if (!hex || !/^[0-9a-f]{64}$/i.test(hex)) return undefined;
  let bin = "";
  for (let i = 0; i < 64; i += 2) bin += String.fromCharCode(parseInt(hex.slice(i, i + 2), 16));
  return `sha256-${btoa(bin)}`;
}

/**
 * The browser's module host: `files[name]` (else `modules/<name>.wasm`)
 * under `base`, fetched in CORS mode (with SRI when `hashes` gives its
 * sha256) and compiled as it streams; synchronously a blocking request,
 * which a Worker may make. The modules in `start` are fetched at once, so
 * they download beside the primary.
 */
export function fetchModuleHost(
  base: string | URL,
  files: Readonly<Record<string, string>> = {},
  start: readonly string[] = [],
  hashes: Readonly<Record<string, string>> = {},
): ModuleHost {
  const url = (name: string) => moduleUrl(name, files, base).href;
  const get = async (name: string) => {
    const integrity = sriOf(hashes[name]);
    const r = await fetch(url(name), { mode: "cors", credentials: "omit", ...(integrity ? { integrity } : {}) });
    if (!r.ok) throw new Error(`cannot fetch engine module ${name} from ${url(name)} (HTTP ${r.status})`);
    return r;
  };
  const started = new Map(start.map((name) => [name, get(name)] as const));
  for (const p of started.values()) p.catch(() => {});
  return {
    pool: { base: String(base), files },
    async load(name, imports) {
      const r = await (started.get(name) ?? get(name));
      started.delete(name);
      return (await WebAssembly.instantiateStreaming(r, imports)).instance;
    },
    loadSync(name, imports) {
      const x = new XMLHttpRequest();
      x.open("GET", url(name), false);
      x.responseType = "arraybuffer";
      x.send();
      if (x.status !== 200) throw new Error(`cannot fetch engine module ${name} from ${url(name)} (HTTP ${x.status})`);
      return new WebAssembly.Instance(new WebAssembly.Module(x.response as ArrayBuffer), imports);
    },
  };
}
