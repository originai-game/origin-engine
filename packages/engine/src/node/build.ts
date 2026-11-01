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

import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  build(o: {
    entrypoints: string[];
    outdir?: string;
    target: string;
    format: string;
    minify?: boolean;
    sourcemap?: "none" | "inline" | "linked" | "external";
    naming?: string;
    plugins?: unknown[];
  }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ path: string; kind: string; size: number; text(): Promise<string> }> }>;
  write(path: string, data: string): Promise<number>;
};

const here = dirname(fileURLToPath(import.meta.url));
export const ENGINE_ROOT = resolve(here, "..", "..");

type Exports = Record<string, string | { default?: string; types?: string }>;
const exports = (JSON.parse(readFileSync(join(ENGINE_ROOT, "package.json"), "utf8")) as { exports: Exports }).exports;

/** The file `@origingame/engine[/sub]` names in this package's exports. */
export function engineFile(specifier: string): string | null {
  const sub = specifier.slice("@origingame/engine".length);
  const target = exports[sub === "" ? "." : `.${sub}`];
  const file = typeof target === "string" ? target : target?.default;
  return file ? join(ENGINE_ROOT, file) : null;
}

export interface Bundled {
  readonly ok: boolean;
  readonly out: string;
  readonly bytes: number;
  readonly ms: number;
  /** The source map written beside the bundle (`<out>.map`). */
  readonly map?: string;
  readonly reason?: string;
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

/** Bundle `entry` (whose default export is `og.game(...)`) into `out`, its source map into `<out>.map`. */
export async function bundleGame(entry: string, out: string, target: "native" | "web" | "script" = "native", minify = false, maps: MapOptions = {}): Promise<Bundled> {
  const t0 = performance.now();
  const absEntry = resolve(entry);
  const engine = {
    name: "origin-engine",
    setup(build: {
      onResolve(o: { filter: RegExp; namespace?: string }, f: (a: { path: string }) => unknown): void;
      onLoad(o: { filter: RegExp; namespace: string }, f: (a: { path: string }) => unknown): void;
    }) {
      build.onResolve({ filter: /^@origingame\/engine(\/.*)?$/ }, (a) => {
        const file = engineFile(a.path);
        if (!file) throw new Error(`${a.path} is not an export of @origingame/engine`);
        return { path: file, external: target === "native" };
      });
      build.onResolve({ filter: /^og:worker$/ }, () => ({ path: "worker", namespace: "og" }));
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
    raw.sources = renameSources(raw.sources, dirname(resolve(out)), maps.root === undefined ? undefined : resolve(maps.root));
    raw.file = basename(out);
    mkdirSync(dirname(mapFile), { recursive: true });
    writeFileSync(mapFile, JSON.stringify(raw));
    if (maps.link !== false) text = `${text.replace(/\n*$/, "\n")}//# sourceMappingURL=${basename(mapFile)}\n`;
  } else rmSync(mapFile, { force: true });
  await Bun.write(out, text);
  return { ok: true, out, bytes: new TextEncoder().encode(text).length, ms: +(performance.now() - t0).toFixed(1), ...(sourcemap ? { map: mapFile } : {}) };
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
  const r = await bundleGame(entry, out, target, argv.includes("--minify"));
  if (argv.includes("--json")) process.stdout.write(JSON.stringify(r) + "\n");
  else if (r.ok) process.stdout.write(`bundled ${entry} -> ${out} (${r.bytes} bytes, ${r.ms} ms)\n`);
  if (!r.ok) {
    process.stderr.write(`og build: ${entry}: ${r.reason}\n`);
    process.exit(1);
  }
}
