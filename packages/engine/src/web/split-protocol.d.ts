// The split-wasm placeholder protocol (src/web/split-protocol.js), as the
// engine's TypeScript reads it. The module is plain JavaScript: the lanes
// pool Workers' installer (scripts/og-pool-split.js) imports it untyped.
// The regex and the namespace are a wire protocol with the split wasm
// (scripts/wasm-split.ts); they are pinned, not edited, in split.ts's
// tests.

/** The primary's exports of the placeholders' tables (wasm-split's name, with scripts/wasm-split.ts's export prefix `%`). */
export declare const PLACEHOLDERS: RegExp;
/** The namespace a split primary imports a module's placeholders under. */
export declare const NAMESPACE: "placeholder.";

/** The placeholder table of a split primary's exports, or null for an unsplit wasm. */
export declare function placeholderTable(exports: WebAssembly.Exports): WebAssembly.Table | null;

/**
 * The placeholder namespace of module `name`: each field (a slot) loads the
 * module (`load`) and calls the slot's function in `table`. The functions
 * made are kept, so the same slot answers the same function.
 */
export declare function placeholderNamespace(
  name: string,
  load: (name: string) => void,
  table: () => WebAssembly.Table,
): Record<string, (...args: unknown[]) => unknown>;

/**
 * `given` (an instantiate call's imports, or undefined) with the placeholder
 * namespaces a split primary asks for added; everything else as given.
 */
export declare function withPlaceholderNamespaces(
  given: WebAssembly.Imports | undefined,
  load: (name: string) => void,
  table: () => WebAssembly.Table,
): WebAssembly.Imports;

/** Where module `name`'s file is: `files[name]`, else `modules/<name>.wasm`, under `base`. */
export declare function moduleUrl(name: string, files: Readonly<Record<string, string>>, base: string | URL): URL;
