// The split-wasm placeholder protocol (docs/web-export.md §Modules), one
// module both readers share: the engine Worker's loader (src/web/split.ts)
// and the lanes pool Workers' installer (scripts/og-pool-split.js, copied
// beside wasm-bindgen-rayon's Worker helper by scripts/rayon-worker.sh,
// which imports this file beside itself). A split primary imports a
// placeholder for each function a module holds (namespace
// `placeholder.<module>`, field the function's slot in the primary's
// placeholder table, exported as `%table_<n>` beside the function table
// `%table`); a module, once instantiated against the primary's exports,
// writes its functions into those slots. A placeholder called before its
// module arrived loads the module there and then (`load`) and calls the
// function it stood for in `table`, so a module that loads late changes no
// result: the split moves code, never what it does.

/** The primary's exports of the placeholders' tables (wasm-split's name, with scripts/wasm-split.ts's export prefix `%`). */
export const PLACEHOLDERS = /^%table_\d+$/;
/** The namespace a split primary imports a module's placeholders under. */
export const NAMESPACE = "placeholder.";

/** The placeholder table of a split primary's exports, or null for an unsplit wasm. */
export function placeholderTable(exports) {
  for (const [k, v] of Object.entries(exports)) if (PLACEHOLDERS.test(k) && v instanceof WebAssembly.Table) return v;
  return null;
}

/**
 * The placeholder namespace of module `name`: each field (a slot) loads the
 * module (`load`) and calls the slot's function in `table`. The functions
 * made are kept, so the same slot answers the same function.
 */
export function placeholderNamespace(name, load, table) {
  const fns = new Map();
  return new Proxy(
    {},
    {
      get(_t, slot) {
        if (typeof slot !== "string") return undefined;
        let f = fns.get(slot);
        if (!f) {
          const index = Number(slot);
          f = (...args) => {
            load(name);
            return table().get(index)(...args);
          };
          fns.set(slot, f);
        }
        return f;
      },
    },
  );
}

/**
 * `given` (an instantiate call's imports, or undefined) with the placeholder
 * namespaces a split primary asks for added; everything else as given.
 */
export function withPlaceholderNamespaces(given, load, table) {
  const base = given ?? {};
  const spaces = new Map();
  return new Proxy(base, {
    get(t, key) {
      if (typeof key === "string" && !(key in t) && key.startsWith(NAMESPACE)) {
        let ns = spaces.get(key);
        if (!ns) spaces.set(key, (ns = placeholderNamespace(key.slice(NAMESPACE.length), load, table)));
        return ns;
      }
      return Reflect.get(t, key);
    },
  });
}

/** Where module `name`'s file is: `files[name]`, else `modules/<name>.wasm`, under `base`. */
export function moduleUrl(name, files, base) {
  return new URL(files[name] ?? `modules/${name}.wasm`, base);
}
