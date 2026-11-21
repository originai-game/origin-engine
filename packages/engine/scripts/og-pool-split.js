// The lanes pool Worker's side of a split engine (docs/web-export.md
// §Modules, src/web/split.ts). scripts/rayon-worker.sh copies this file
// beside wasm-bindgen-rayon's Worker helper — with the placeholder protocol
// it shares, ./split-protocol.js — and has the helper call installPoolSplit
// before the pool Worker instantiates the primary.
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

import { moduleUrl, placeholderTable, withPlaceholderNamespaces } from "./split-protocol.js";

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
    const t = exports && placeholderTable(exports);
    if (!t) throw new Error("the engine wasm has no placeholder table");
    return t;
  };
  const load = (name) => {
    if (loaded.has(name)) return;
    new WebAssembly.Instance(new WebAssembly.Module(bytes(name, moduleUrl(name, split.files, split.base))), { primary: exports });
    loaded.add(name);
  };
  const keep = (r) => {
    exports = (r instanceof WebAssembly.Instance ? r : r.instance).exports;
    return r;
  };
  for (const k of ["instantiate", "instantiateStreaming"]) {
    const original = WebAssembly[k];
    if (original) WebAssembly[k] = (source, imports) => Promise.resolve(original.call(WebAssembly, source, withPlaceholderNamespaces(imports, load, table))).then(keep);
  }
}
