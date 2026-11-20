// The lanes pool Worker's side of a split engine (docs/web-export.md
// §Modules, src/web/split.ts). scripts/rayon-worker.sh copies this file
// beside wasm-bindgen-rayon's Worker helper and has the helper call
// installPoolSplit before the pool Worker instantiates the primary.
//
// Each pool Worker instantiates the primary again on the shared memory, so
// it needs the placeholder namespaces (`placeholder.<module>`) the primary
// imports, and a module a lane calls into is put into this instance's own
// table: a split module carries no data, only its functions, so
// instantiating it once per instance writes nothing in the shared memory.
// A placeholder loads its module there and then (a blocking request, which
// a Worker may make, served from the HTTP cache once the engine Worker
// fetched it) and calls through, as on the engine Worker: a module that
// arrives late changes no result.

const NAMESPACE = "placeholder.";

/** Module `name`'s bytes at `url`: a blocking request, or a file read where the host has no XMLHttpRequest (Bun, Node: the web host run outside a browser). */
function bytes(name, url) {
  const fs = url.protocol === "file:" ? globalThis.process?.getBuiltinModule?.("node:fs") : undefined;
  if (fs) return fs.readFileSync(url);
  const x = new XMLHttpRequest();
  x.open("GET", url.href, false);
  x.responseType = "arraybuffer";
  x.send();
  if (x.status !== 200) throw new Error(`a lanes Worker cannot fetch engine module ${name} from ${url.href} (HTTP ${x.status})`);
  return x.response;
}
const PLACEHOLDERS = /^%table_\d+$/;

/**
 * `split`: what the engine Worker's module host loads from (split.ts
 * `poolSplit`): `base`, an absolute URL, and `files`, each module's file
 * under it (else `modules/<name>.wasm`). Absent (an unsplit primary): nothing.
 */
export function installPoolSplit(split) {
  if (!split) return;
  let exports = null;
  const loaded = new Set();
  const table = () => {
    for (const [k, v] of Object.entries(exports)) if (PLACEHOLDERS.test(k) && v instanceof WebAssembly.Table) return v;
    throw new Error("the engine wasm has no placeholder table");
  };
  const load = (name) => {
    if (loaded.has(name)) return;
    const url = new URL(split.files[name] ?? `modules/${name}.wasm`, split.base);
    new WebAssembly.Instance(new WebAssembly.Module(bytes(name, url)), { primary: exports });
    loaded.add(name);
  };
  const namespace = (name) =>
    new Proxy(
      {},
      {
        get(_t, slot) {
          if (typeof slot !== "string") return undefined;
          const index = Number(slot);
          return (...args) => {
            load(name);
            return table().get(index)(...args);
          };
        },
      },
    );
  const withPlaceholders = (given) => {
    const spaces = new Map();
    return new Proxy(given ?? {}, {
      get(t, key) {
        if (typeof key === "string" && !(key in t) && key.startsWith(NAMESPACE)) {
          if (!spaces.has(key)) spaces.set(key, namespace(key.slice(NAMESPACE.length)));
          return spaces.get(key);
        }
        return Reflect.get(t, key);
      },
    });
  };
  const keep = (r) => {
    exports = (r instanceof WebAssembly.Instance ? r : r.instance).exports;
    return r;
  };
  for (const k of ["instantiate", "instantiateStreaming"]) {
    const original = WebAssembly[k];
    if (original) WebAssembly[k] = (source, imports) => Promise.resolve(original.call(WebAssembly, source, withPlaceholders(imports))).then(keep);
  }
}
