// Bundles a game for a host (`og build`, `og run`, `og dev` call it):
//
//   native  the game's own code in one module, `@origingame/engine` left
//           external and pointed at this package's files, so the game and the
//           native host share one runtime instance (and a re-imported bundle
//           re-registers into the same engine on hot reload);
//   web     the engine Worker: `startWorker(game)`, the runtime and the game in
//           one module, as the play page's shell starts it;
//   script  any other browser module (the shell's play.js, the audio worklet), as is.
//   web-live  `web` for `og dev --web`: the engine modules the game imports are
//           also published on `globalThis.__og_engine__` (by specifier), so a
//   web-code  bundle of the game's own code alone (built on each edit) imports
//           them from there: the edited game re-registers into the running
//           engine and the probe's `reload` swaps it in (src/node/dev-web.ts).
//
// Each bundle gets its source map beside it, `<out>.map`, its `sources`
// named so that a stack maps back to the game's TypeScript: by default
// relative to the bundle's own directory, with the `//# sourceMappingURL`
// comment, which bun honours when it loads the bundle (a native run's
// `og run`, `og dev`, `og test` and playtest stacks name `src/main.ts` lines);
// `maps.root` names them relative to a project root instead (the engine's
// own files as `@origingame/engine/<path>`), and `maps.link` false leaves the
// comment out (a publish build, whose maps are not served).
//
//   bun src/node/build.ts --entry <src/main.ts> --out <file.js> [--target native|web] [--minify] [--json]

import { appendFileSync, existsSync, mkdirSync, readdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createRequire, isBuiltin } from "node:module";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { placeAssertions } from "./assert-sites.ts";
import { addonPath } from "./addon.ts";
import { projectRoot, readProject, readStringTableIndex } from "./project.ts";

type Compiled = { ok: true; code: string; map: string; warnings: string[] } | { ok: false; reason: string };
let jsxAddon: { og_node_jsx(path: string, source: string): string } | undefined;

function base64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let bin = "";
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin);
}

/** A TSX/JSX module through nsh-jsx (the native addon's og_node_jsx). */
export function compileJsx(path: string, source: string): Compiled {
  jsxAddon ??= createRequire(import.meta.url)(addonPath()) as typeof jsxAddon;
  return JSON.parse(jsxAddon!.og_node_jsx(path, source)) as Compiled;
}

/** The bundler's import graph (esbuild's metafile shape): each input by its path relative to the working directory. */
interface Metafile {
  inputs: Record<string, { imports: Array<{ path: string; external?: boolean }> }>;
}

declare const Bun: {
  build(o: {
    entrypoints: string[];
    outdir?: string;
    target: string;
    format: string;
    minify?: boolean;
    throw?: boolean;
    sourcemap?: "none" | "inline" | "linked" | "external";
    naming?: string;
    plugins?: unknown[];
    metafile?: boolean;
    define?: Record<string, string>;
  }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ path: string; kind: string; size: number; text(): Promise<string> }>; metafile?: Metafile }>;
  write(path: string, data: string): Promise<number>;
  resolveSync(specifier: string, parent: string): string;
  JSONC: { parse(text: string): unknown };
  Transpiler: new (o: { loader: string }) => { scan(text: string): { exports: string[] } };
};

/** The trace file OG_BUILD_TRACE names (the gate's build trace, crate::commands::bundle): opt-in evidence, never a failure. */
function bundleTrace(event: string, fields: Record<string, unknown> = {}): void {
  const file = process.env.OG_BUILD_TRACE;
  if (!file) return;
  try {
    appendFileSync(file, JSON.stringify({ at: Date.now(), pid: process.pid, event, ...fields }) + "\n");
  } catch {
    // Diagnostics cannot make a build fail.
  }
}

const here = dirname(fileURLToPath(import.meta.url));
export const ENGINE_ROOT = resolve(here, "..", "..");

/**
 * The file `@origingame/engine[/sub]` names: this package's `exports`, as
 * the bundler resolves the package from inside it (its own name), so
 * package.json is the one map from specifier to file.
 */
export function engineFile(specifier: string): string | null {
  try {
    return Bun.resolveSync(specifier, ENGINE_ROOT);
  } catch {
    return null;
  }
}

export interface Bundled {
  readonly ok: boolean;
  readonly out: string;
  readonly bytes: number;
  readonly ms: number;
  /** The source map written beside the bundle (`<out>.map`). */
  readonly map?: string;
  readonly reason?: string;
  /** Every resolved compiler input was in the caller's validated snapshot. */
  readonly cacheable?: boolean;
}

/** How a bundle's source map is written (`bundleGame`). */
export interface MapOptions {
  /** Name the sources relative to this directory (a project root) instead of the bundle's own. */
  readonly root?: string;
  /** Write `//# sourceMappingURL=<name>.map` into the bundle (default true). */
  readonly link?: boolean;
  /** Where the map goes (default `<out>.map`). */
  readonly file?: string;
}

/**
 * A map's `sources` (bun names them relative to the working directory)
 * renamed: relative to `root` when given (the game's files under it, the
 * engine's own as `@origingame/engine/<path>`, any other relative to it),
 * else relative to `dir`, the bundle's
 * directory, which is where a loader resolves them from. A virtual source
 * (`og:worker`) keeps its name.
 */
export function renameSources(sources: readonly (string | null)[], dir: string, root?: string): (string | null)[] {
  const slash = (p: string) => p.split(sep).join("/");
  return sources.map((s) => {
    if (s === null || /^[a-z][\w+.-]*:/i.test(s)) return s;
    const abs = resolve(process.cwd(), s);
    if (root === undefined) return slash(relative(dir, abs));
    const own = relative(root, abs);
    if (!own.startsWith("..") && !isAbsolute(own)) return slash(own);
    const engine = relative(ENGINE_ROOT, abs);
    if (!engine.startsWith("..") && !isAbsolute(engine)) return `@origingame/engine/${slash(engine)}`;
    return slash(relative(root, abs));
  });
}

/**
 * Gives the bundle's game (`game`, its entry's default export) the og.assert
 * sites its files listed (src/node/assert-sites.ts) under this module's URL,
 * and drops them from the list: each imported copy of a bundle is its own
 * module with its own sites, so a host that loads many games (a resident
 * session, og ui check's cells) reports each engine's own and keeps none.
 */
const ADOPT_SITES = `{
  const listed = globalThis.__og_assert_sites;
  const sites = listed?.[import.meta.url] ?? [];
  if (listed) delete listed[import.meta.url];
  if (game !== null && typeof game === "object") (globalThis.__og_assert_game_sites ??= new WeakMap()).set(game, sites);
}
`;

/**
 * The files the game's entry reaches, as the bundler resolved its imports
 * (the bundle's metafile, `<out>.graph.json`): the game's own modules and
 * every engine module and string table it imports, by absolute path, with
 * the engine package's root, from each of `roots`: the entry, and the
 * built-in FlowScreen a project's `flow` line mounts. What else the bundle
 * adds beside the entry (the development inspector) is not the game's. The cook reads it to know
 * which built-in UI, string tables and raycast BVHs the code needs
 * (og-cook's `CodeGraph`).
 */
export function reached(meta: Metafile, roots: readonly string[]): { engine: string; files: string[] } {
  const abs = (p: string) => (/^[a-z][\w+.-]*:/i.test(p) ? p : resolve(process.cwd(), p));
  const inputs = new Map(Object.entries(meta.inputs).map(([k, v]) => [abs(k), v]));
  const seen = new Set<string>();
  const stack = [...roots];
  while (stack.length > 0) {
    const file = stack.pop()!;
    if (seen.has(file)) continue;
    seen.add(file);
    for (const i of inputs.get(file)?.imports ?? []) stack.push(abs(i.path));
  }
  return { engine: ENGINE_ROOT, files: [...seen].filter((f) => isAbsolute(f)).sort() };
}

/** Bundle `entry` (whose default export is `og.game(...)`) into `out`, its source map into `<out>.map`. */
export async function bundleGame(entry: string, out: string, target: "native" | "web" | "script" | "web-live" | "web-code" = "native", minify = false, maps: MapOptions = {}, cacheInputs?: ReadonlySet<string>): Promise<Bundled> {
  const t0 = performance.now();
  bundleTrace("bundle-start", { entry, target });
  let bundlerMs = 0;
  let jsxMs = 0;
  let jsxCount = 0;
  const absEntry = resolve(entry);
  let cacheable = cacheInputs !== undefined && target === "native";
  for (const file of cacheInputs ?? []) {
    if (!/^(?:ts|js)config(?:\.[^.]+)*\.json$/.test(basename(file))) continue;
    try {
      const config = Bun.JSONC.parse(readFileSync(file, "utf8"));
      // Extended configs can read outside the captured tree, including npm
      // packages. Compilation still works; only preparation reuse is refused.
      if (typeof config !== "object" || config === null || "extends" in config) cacheable = false;
    } catch { cacheable = false; }
  }
  const gameRoot = projectRoot(absEntry) ?? dirname(absEntry);
  const checkedProject = readProject(gameRoot);
  const flowScreen = checkedProject?.startup.flow ? join(ENGINE_ROOT, "src/ui/screens/flow-screen.tsx") : null;
  const localeTables = checkedProject ? readStringTableIndex(gameRoot) : [];
  const localeIds = new Map(localeTables.map(t => [resolve(gameRoot, t.path), t.id]));
  const engine = {
    name: "origin-engine",
    setup(build: {
      onResolve(o: { filter: RegExp; namespace?: string }, f: (a: { path: string; resolveDir: string; importer?: string; namespace?: string }) => unknown): void;
      onLoad(o: { filter: RegExp; namespace?: string }, f: (a: { path: string }) => unknown): void;
    }) {
      build.onResolve({ filter: /^@origingame\/engine(\/.*)?$/ }, (a) => {
        const file = engineFile(a.path);
        if (!file) throw new Error(`${a.path} is not an export of @origingame/engine`);
        // The game's own imports (not the engine's, nor the og:worker entry's) go
        // through the shared namespaces (`web-live` publishes them, `web-code` reads them).
        // (A sheet project lives inside the engine package, so this keys on the game's project, not the engine's root.)
        const importer = a.importer ?? "";
        const fromGame = a.namespace !== "og" && importer.startsWith(gameRoot + sep) && !importer.includes(`${sep}node_modules${sep}`);
        if (fromGame && target === "web-live") return { path: a.path, namespace: "og-pub" };
        if (fromGame && target === "web-code") return { path: a.path, namespace: "og-ext" };
        // Engine TSX (the components) is bundled even natively, so nsh-jsx
        // compiles it; the renderer it imports stays external and shared.
        return { path: file, external: target === "native" && !/\.[jt]sx$/.test(file) };
      });
      build.onLoad({ filter: /.*/, namespace: "og-pub" }, (a) => {
        const path = engineFile(a.path)!;
        const file = JSON.stringify(path);
        // The built-in components are TSX: scanning them as TS is a parse error.
        const hasDefault = new Bun.Transpiler({ loader: path.endsWith(".tsx") ? "tsx" : "ts" }).scan(readFileSync(path, "utf8")).exports.includes("default");
        return {
          contents: `import * as m from ${file};\n(globalThis.__og_engine__ ??= {})[${JSON.stringify(a.path)}] = m;\nexport * from ${file};\n${hasDefault ? `export { default } from ${file};\n` : ""}`,
          loader: "ts",
        };
      });
      build.onLoad({ filter: /.*/, namespace: "og-ext" }, async (a) => {
        // The names the module exports, read by importing it here (bun runs the engine's modules natively).
        const names = Object.keys(await import(engineFile(a.path)!));
        const spec = JSON.stringify(a.path);
        const own = names.filter((n) => n !== "default" && /^[A-Za-z_$][\w$]*$/.test(n));
        return {
          contents: `const m = globalThis.__og_engine__?.[${spec}];\nif (!m) throw new Error("og dev: the running page has no " + ${spec} + " (a cold restart loads it)");\nexport const { ${own.join(", ")} } = m;\n${names.includes("default") ? "export default m.default;\n" : ""}`,
          loader: "js",
        };
      });
      build.onResolve({ filter: /^og:(worker|game)$/ }, (a) => ({ path: a.path.slice(3), namespace: "og" }));
      // A component's string table resolves here, ahead of the cache's
      // catch-all: an import that catch-all resolves (returning nothing) is
      // dropped from the bundle as unused, and its strings() call with it.
      build.onResolve({ filter: /\.(?:ogt|ts)(?:\?asset)?$/ }, (a) => {
        const file = resolve(a.resolveDir || process.cwd(), a.path.replace(/\?asset$/, ""));
        if (!a.path.endsWith(".ogt") && !a.path.endsWith(".strings.ts") && !a.path.endsWith(".particles.ts?asset") && !localeIds.has(file)) return undefined;
        if (cacheInputs && !(existsSync(file) && cacheInputs.has(realpathSync(file)))) cacheable = false;
        return { path: file };
      });
      if (cacheInputs) {
        build.onResolve({ filter: /.*/ }, (a) => {
          if (isBuiltin(a.path) || a.path === "bun" || a.path.startsWith("bun:")) return undefined;
          try {
            const file = realpathSync(Bun.resolveSync(a.path, a.resolveDir || process.cwd()));
            if (!cacheInputs.has(file)) cacheable = false;
          } catch { cacheable = false; }
          return undefined;
        });
        build.onLoad({ filter: /.*/, namespace: "file" }, (a) => {
          if (!cacheInputs.has(realpathSync(a.path))) cacheable = false;
          return undefined;
        });
      }
      // og.assert's places (src/node/assert-sites.ts), in the game's own files only.
      const project = projectRoot(absEntry) ?? dirname(absEntry);
      // A component's strings (`import "./x.en.strings.ts"`) are the cook's: the
      // bundle records only the table's id, which the UI renderer sends the
      // host to register. An engine table's id is `@ui/<path under src/ui>`
      // (the cook bundles them for a game using the built-in components), a
      // game's its canonical content ID. A file.particles declaration (an
      // engine component's built-in effect) has nothing to register: its
      // import's default is its id, which og.load.emitter loads.
      const engineUi = join(ENGINE_ROOT, "src", "ui") + sep;
      const assets = join(project, "assets") + sep;
      build.onLoad({ filter: /\.(?:ogt|ts)$/, namespace: "file" }, (a) => {
        const engineTable = a.path.startsWith(engineUi) && a.path.endsWith(".strings.ts");
        const engineEmitter = a.path.startsWith(engineUi) && a.path.endsWith(".particles.ts");
        if (!a.path.endsWith(".ogt") && !engineTable && !engineEmitter && !localeIds.has(a.path)) return undefined;
        const id = a.path.startsWith(engineUi)
          ? `@ui/${a.path.slice(engineUi.length)}`
          : localeIds.get(a.path) ?? relative(a.path.startsWith(assets) ? assets : project, a.path);
        const name = JSON.stringify(id.split(sep).join("/"));
        if (engineEmitter) return { contents: `export default ${name};\n`, loader: "js" };
        if (a.path.endsWith(".ogt")) {
          throw new Error(`${a.path}: OGT author imports are removed; import a checked TS declaration under content/`);
        }
        // An engine table's translations (`x.zh-CN.strings.ts` beside `x.en.strings.ts`)
        // come with it: bundled with it, and registered as translations of it.
        const translations = engineTable && a.path.endsWith(".en.strings.ts")
          ? readdirSync(dirname(a.path))
              .filter((f) => f.startsWith(basename(a.path, ".en.strings.ts") + ".") && f.endsWith(".strings.ts") && f !== basename(a.path) && /^[a-z]{2,3}(-[A-Za-z0-9]+)*$/.test(f.slice(basename(a.path, ".en.strings.ts").length + 1, -11)))
              .sort()
              .map((f) => `import ${JSON.stringify(join(dirname(a.path), f))};\n`)
              .join("")
          : "";
        return {
          contents: `import { strings } from "@origingame/engine/ui/rt";\nstrings(${name});\n${translations}export default ${name};\n`,
          loader: "js",
        };
      });
      // TSX and JSX (the game's and the engine's UI components) compile with
      // nsh-jsx to the UI renderer's calls (crates/og-ui-next/CONVENTIONS.md
      // §JSX compile), after the og.assert sites are placed.
      build.onLoad({ filter: /\.[cm]?[jt]sx?$/, namespace: "file" }, (a) => {
        if (a.path.includes(`${sep}node_modules${sep}`)) return undefined;
        const own = a.path.startsWith(project + sep) && !a.path.startsWith(ENGINE_ROOT + sep);
        const jsx = /x$/.test(a.path);
        if (!own && !jsx) return undefined;
        let text = readFileSync(a.path, "utf8");
        const asserts = own && text.includes("og.assert.");
        if (asserts) text = placeAssertions(text, relative(project, a.path).split(sep).join("/"));
        if (jsx) {
          const tJsx = performance.now();
          const out = compileJsx(a.path, text);
          jsxMs += performance.now() - tJsx;
          jsxCount++;
          if (!out.ok) throw new Error(out.reason);
          for (const warning of out.warnings) process.stderr.write(`${relative(process.cwd(), a.path)}: ${warning}\n`);
          const map = base64(out.map);
          return { contents: `${out.code}\n//# sourceMappingURL=data:application/json;base64,${map}\n`, loader: "js" };
        }
        if (!asserts) return undefined;
        const loader = /\.[cm]?ts$/.test(a.path) ? "ts" : "js";
        return { contents: text, loader };
      });
      // In publish web builds, strip docs/metadata/unused call fields from the
      // embedded bindings table, keeping only what the runtime actually reads
      // (runtime/host.ts, web/host.ts, runtime/probe-params.ts, runtime/declare.ts).
      const bindingsPath = join(ENGINE_ROOT, "types", "bindings.json");
      if (target === "web" && minify) {
        build.onLoad({ filter: /[\\/]types[\\/]bindings\.json$/, namespace: "file" }, (a) => {
          if (resolve(a.path) !== bindingsPath) return undefined;
          const raw = JSON.parse(readFileSync(a.path, "utf8")) as {
            columns: unknown[];
            calls: Array<{
              symbol?: string;
              module?: string;
              kind?: string;
              name?: string;
              returns?: string;
              publish?: boolean;
              params?: Array<{ name: string; type: string; optional?: boolean }>;
            }>;
            components: Array<{ name: string; schema: string; fields: Array<{ name: string; column?: string; type?: string; default?: unknown; list?: number }> }>;
            resources: Array<{ name: string; schema: string; fields: Array<{ name: string; column?: string; type?: string; default?: unknown; list?: number }> }>;
            events: Array<{ name: string; schema: string }>;
          };
          const slimRecord = (r: { name: string; schema: string; fields: Array<{ name: string; column?: string; type?: string; default?: unknown; list?: number }> }) => ({
            name: r.name,
            schema: r.schema,
            fields: r.fields.map((f) => ({
              name: f.name,
              ...(f.column !== undefined ? { column: f.column } : f.type !== undefined ? { type: f.type } : {}),
              ...(f.default !== undefined ? { default: f.default } : {}),
              ...(f.list !== undefined ? { list: f.list } : {}),
            })),
          });
          const calls: unknown[] = [];
          for (const c of raw.calls) {
            if (c.module === "probe" && c.kind === "method") {
              calls.push({
                ...(c.symbol !== undefined ? { symbol: c.symbol } : {}),
                module: c.module,
                kind: c.kind,
                name: c.name,
                returns: c.returns,
                publish: c.publish,
                params: (c.params ?? []).map((p) => ({ name: p.name, type: p.type, optional: Boolean(p.optional) })),
              });
            } else if (c.publish === false) {
              calls.push({ symbol: c.symbol, module: c.module, returns: c.returns, publish: false });
            } else if (typeof c.symbol === "string" && typeof c.returns === "string" && c.returns.startsWith("Result<")) {
              calls.push({ symbol: c.symbol, returns: "Result<", publish: true });
            }
          }
          const slim = {
            columns: raw.columns,
            calls,
            components: raw.components.map(slimRecord),
            resources: raw.resources.map(slimRecord),
            events: raw.events.map((e) => ({ name: e.name, schema: e.schema })),
          };
          return { contents: JSON.stringify(slim), loader: "json" };
        });
      }
      // The bundle's entry: the game's own (og:game, native) or the worker that
      // starts it (og:worker, web), each giving its og.game() token the og.assert
      // sites of this bundle's files (ADOPT_SITES).
      // A development bundle carries the inspector's TSX panel; a publish
      // build (minified) has no inspector.
      const inspector = (minify ? "" : `import ${JSON.stringify(join(ENGINE_ROOT, "src/ui/inspector.tsx"))};\n`)
        // project.ogt's `flow`: the engine mounts the built-in FlowScreen, which only such a bundle carries.
        + (flowScreen
          ? `import { FlowScreen as __ogFlowScreen } from ${JSON.stringify(flowScreen)};\nimport { flowScreen as __ogFlowSlot } from "@origingame/engine/ui/rt";\n__ogFlowSlot.component = __ogFlowScreen;\n`
          : "");
      build.onLoad({ filter: /.*/, namespace: "og" }, (a) => ({
        contents:
          a.path === "worker"
            ? `import { startWorker } from "@origingame/engine/web/worker";\n${inspector}import * as entry from ${JSON.stringify(absEntry)};\nconst game = entry.default;\n${ADOPT_SITES}startWorker(game);\n`
            : `export * from ${JSON.stringify(absEntry)};\n${inspector}import * as entry from ${JSON.stringify(absEntry)};\nconst game = entry.default;\n${ADOPT_SITES}export default game;\n`,
        loader: "ts",
      }));
    },
  };
  const tBun = performance.now();
  const r = await Bun.build({
    entrypoints: [target === "web" || target === "web-live" ? "og:worker" : target === "native" || target === "web-code" ? "og:game" : absEntry],
    target: target === "native" ? "bun" : "browser",
    format: "esm",
    minify,
    throw: false,
    sourcemap: "external",
    plugins: [engine],
    metafile: true,
    ...(target === "web" || target === "script" ? { define: { __OG_PUBLISH__: minify ? "true" : "false" } } : {}),
  });
  bundlerMs = +(performance.now() - tBun).toFixed(1);
  bundleTrace("bundle-bundler", { ms: bundlerMs, jsxMs: +jsxMs.toFixed(1), jsxCount, minify });
  if (!r.success || r.outputs.length === 0) {
    const reason = r.logs.map((l) => String((l as { message?: string }).message ?? l)).join("\n") || "bun build failed";
    bundleTrace("bundle-end", { ok: false, ms: +(performance.now() - t0).toFixed(1), bundlerMs, jsxMs: +jsxMs.toFixed(1), jsxCount });
    return { ok: false, out, bytes: 0, ms: performance.now() - t0, reason };
  }
  const code = r.outputs.find((o) => o.kind !== "sourcemap") ?? r.outputs[0]!;
  const sourcemap = r.outputs.find((o) => o.kind === "sourcemap");
  const mapFile = maps.file ?? `${out}.map`;
  let text = await code.text();
  if (sourcemap) {
    const raw = JSON.parse(await sourcemap.text()) as { sources: (string | null)[]; sourcesContent?: (string | null)[]; file?: string; mappings: string };
    // The bundle's module function starts at line 1, column 0 (bun's `// @bun`
    // banner), which bun maps to nothing, so a CPU profile's frame for the
    // module's top-level code named the bundle. Line 1 points at the first
    // source's first line; the segment is all zeros, so every later segment's
    // deltas stay as they were.
    if (raw.mappings.startsWith(";") && raw.sources.length > 0) raw.mappings = `AAAA${raw.mappings}`;
    // A virtual module's text (og:worker's, which imports the entry by its absolute path) is not the game's: left out.
    if (raw.sourcesContent) raw.sourcesContent = raw.sourcesContent.map((c, i) => (/^og:/.test(raw.sources[i] ?? "") ? null : c));
    raw.sources = cacheInputs
      ? raw.sources.map((source) => source === null || /^[a-z][\w+.-]*:/.test(source) ? source : resolve(process.cwd(), source))
      : renameSources(raw.sources, dirname(resolve(out)), maps.root === undefined ? undefined : resolve(maps.root));
    raw.file = basename(out);
    mkdirSync(dirname(mapFile), { recursive: true });
    writeFileSync(mapFile, JSON.stringify(raw));
    if (maps.link !== false) text = `${text.replace(/\n*$/, "\n")}//# sourceMappingURL=${basename(mapFile)}\n`;
  } else rmSync(mapFile, { force: true });
  await Bun.write(out, text);
  if (r.metafile) writeFileSync(`${out}.graph.json`, JSON.stringify(reached(r.metafile, flowScreen ? [absEntry, flowScreen] : [absEntry])));
  const ms = +(performance.now() - t0).toFixed(1);
  bundleTrace("bundle-end", { ok: true, ms, bytes: new TextEncoder().encode(text).length, bundlerMs, jsxMs: +jsxMs.toFixed(1), jsxCount, cacheable });
  return { ok: true, out, bytes: new TextEncoder().encode(text).length, ms, ...(sourcemap ? { map: mapFile } : {}), ...(cacheInputs ? { cacheable } : {}) };
}

if (import.meta.main) {
  const argv = process.argv.slice(2);
  const opt = (name: string) => {
    const i = argv.indexOf(name);
    return i >= 0 ? argv[i + 1] : undefined;
  };
  const entry = opt("--entry");
  const out = opt("--out");
  const target = (opt("--target") ?? "native") as "native" | "web";
  if (!entry || !out || (target !== "native" && target !== "web")) {
    process.stderr.write("usage: bun src/node/build.ts --entry <src/main.ts> --out <file.js> [--target native|web] [--minify] [--json]\n");
    process.exit(2);
  }
  const planFile = opt("--cache-plan");
  let cacheInputs: Set<string> | undefined;
  if (planFile) {
    const plan: unknown = JSON.parse(readFileSync(planFile, "utf8"));
    if (typeof plan !== "object" || plan === null || !("files" in plan) || !Array.isArray(plan.files) || !plan.files.every((file): file is string => typeof file === "string"))
      throw new Error("invalid native preparation plan");
    cacheInputs = new Set(plan.files);
  }
  const r = await bundleGame(entry, out, target, argv.includes("--minify"), {}, cacheInputs);
  if (planFile && r.ok && r.cacheable) writeFileSync(join(dirname(planFile), "eligible"), "1");
  if (argv.includes("--json")) process.stdout.write(JSON.stringify(r) + "\n");
  else if (r.ok) process.stdout.write(`bundled ${entry} -> ${out} (${r.bytes} bytes, ${r.ms} ms)\n`);
  if (!r.ok) {
    process.stderr.write(`og build: ${entry}: ${r.reason}\n`);
    process.exit(1);
  }
}
