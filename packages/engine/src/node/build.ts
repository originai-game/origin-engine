// Bundles a game for a host (`og build`, `og run`, `og dev` call it):
//
//   native  the game's own code in one module, `@origingame/engine` left
//           external and pointed at this package's files, so the game and the
//           native host share one runtime instance (and a re-imported bundle
//           re-registers into the same engine on hot reload);
//   web     the engine Worker: `startWorker(game)`, the runtime and the game in
//           one module, as the play page's shell starts it;
//   script  any other browser module (the shell's play.js, the audio worklet), as is.
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

import { existsSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { isBuiltin } from "node:module";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { placeAssertions } from "./assert-sites.ts";

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
  }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ path: string; kind: string; size: number; text(): Promise<string> }> }>;
  write(path: string, data: string): Promise<number>;
  resolveSync(specifier: string, parent: string): string;
  JSONC: { parse(text: string): unknown };
};

const here = dirname(fileURLToPath(import.meta.url));
export const ENGINE_ROOT = resolve(here, "..", "..");

type Exports = Record<string, string | { default?: string; types?: string }>;
const exports = (JSON.parse(readFileSync(join(ENGINE_ROOT, "package.json"), "utf8")) as { exports: Exports }).exports;

/** The file `@origingame/engine[/sub]` names in this package's exports. */
export function engineFile(specifier: string): string | null {
  const sub = specifier.slice("@origingame/engine".length);
  const key = sub === "" ? "." : `.${sub}`;
  const target = exports[key];
  const file = typeof target === "string" ? target : target?.default;
  if (file) return join(ENGINE_ROOT, file);
  // A pattern export (`./extensions/*`): its `*` stands for the rest of the path.
  for (const [pattern, to] of Object.entries(exports)) {
    const star = pattern.indexOf("*");
    const dest = typeof to === "string" ? to : to.default;
    if (star < 0 || !dest || !key.startsWith(pattern.slice(0, star)) || !key.endsWith(pattern.slice(star + 1))) continue;
    const rest = key.slice(star, key.length - (pattern.length - star - 1));
    if (rest.split("/").includes("..")) return null;
    return join(ENGINE_ROOT, dest.replace("*", rest));
  }
  return null;
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

/** The project an entry belongs to: the nearest directory above it with a project.ogt, else the entry's own. */
function projectRoot(entry: string): string {
  for (let dir = dirname(entry); ; dir = dirname(dir)) {
    if (existsSync(join(dir, "project.ogt"))) return dir;
    if (dirname(dir) === dir) return dirname(entry);
  }
}

/** Bundle `entry` (whose default export is `og.game(...)`) into `out`, its source map into `<out>.map`. */
export async function bundleGame(entry: string, out: string, target: "native" | "web" | "script" = "native", minify = false, maps: MapOptions = {}, cacheInputs?: ReadonlySet<string>): Promise<Bundled> {
  const t0 = performance.now();
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
  const engine = {
    name: "origin-engine",
    setup(build: {
      onResolve(o: { filter: RegExp; namespace?: string }, f: (a: { path: string; resolveDir: string }) => unknown): void;
      onLoad(o: { filter: RegExp; namespace?: string }, f: (a: { path: string }) => unknown): void;
    }) {
      build.onResolve({ filter: /^@origingame\/engine(\/.*)?$/ }, (a) => {
        const file = engineFile(a.path);
        if (!file) throw new Error(`${a.path} is not an export of @origingame/engine`);
        return { path: file, external: target === "native" };
      });
      build.onResolve({ filter: /^og:worker$/ }, () => ({ path: "worker", namespace: "og" }));
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
      const project = projectRoot(absEntry);
      build.onLoad({ filter: /\.[cm]?[jt]sx?$/, namespace: "file" }, (a) => {
        if (!a.path.startsWith(project + sep) || a.path.includes(`${sep}node_modules${sep}`) || a.path.startsWith(ENGINE_ROOT + sep)) return undefined;
        const text = readFileSync(a.path, "utf8");
        if (!text.includes("og.assert.")) return undefined;
        const loader = /\.[cm]?tsx$/.test(a.path) ? "tsx" : /\.[cm]?ts$/.test(a.path) ? "ts" : /x$/.test(a.path) ? "jsx" : "js";
        return { contents: placeAssertions(text, relative(project, a.path).split(sep).join("/")), loader };
      });
      build.onLoad({ filter: /.*/, namespace: "og" }, () => ({
        contents: `import { startWorker } from "@origingame/engine/web/worker";\nimport game from ${JSON.stringify(absEntry)};\nstartWorker(game);\n`,
        loader: "ts",
      }));
    },
  };
  const r = await Bun.build({
    entrypoints: [target === "web" ? "og:worker" : absEntry],
    target: target === "native" ? "bun" : "browser",
    format: "esm",
    minify,
    throw: false,
    sourcemap: "external",
    plugins: [engine],
  });
  if (!r.success || r.outputs.length === 0) {
    const reason = r.logs.map((l) => String((l as { message?: string }).message ?? l)).join("\n") || "bun build failed";
    return { ok: false, out, bytes: 0, ms: performance.now() - t0, reason };
  }
  const code = r.outputs.find((o) => o.kind !== "sourcemap") ?? r.outputs[0]!;
  const sourcemap = r.outputs.find((o) => o.kind === "sourcemap");
  const mapFile = maps.file ?? `${out}.map`;
  let text = await code.text();
  if (sourcemap) {
    const raw = JSON.parse(await sourcemap.text()) as { sources: (string | null)[]; sourcesContent?: (string | null)[]; file?: string };
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
  return { ok: true, out, bytes: new TextEncoder().encode(text).length, ms: +(performance.now() - t0).toFixed(1), ...(sourcemap ? { map: mapFile } : {}), ...(cacheInputs ? { cacheable } : {}) };
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
