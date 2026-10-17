// Bundles a game for a host (`og build`, `og run`, `og dev` call it):
//
//   native  the game's own code in one module, `@origingame/engine` left
//           external and pointed at this package's files, so the game and the
//           native host share one runtime instance (and a re-imported bundle
//           re-registers into the same engine on hot reload);
//   web     the engine Worker: `startWorker(game)`, the runtime and the game in
//           one module, as the play page's shell starts it.
//
//   bun src/node/build.ts --entry <src/main.ts> --out <file.js> [--target native|web] [--json]

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  build(o: {
    entrypoints: string[];
    outdir?: string;
    target: string;
    format: string;
    minify?: boolean;
    naming?: string;
    plugins?: unknown[];
  }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ path: string; size: number; text(): Promise<string> }> }>;
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
  readonly reason?: string;
}

/** Bundle `entry` (whose default export is `og.game(...)`) into `out`. */
export async function bundleGame(entry: string, out: string, target: "native" | "web" = "native", minify = false): Promise<Bundled> {
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
    target: target === "web" ? "browser" : "bun",
    format: "esm",
    minify,
    plugins: [engine],
  });
  if (!r.success || r.outputs.length === 0) {
    const reason = r.logs.map((l) => String((l as { message?: string }).message ?? l)).join("\n") || "bun build failed";
    return { ok: false, out, bytes: 0, ms: performance.now() - t0, reason };
  }
  const text = await r.outputs[0]!.text();
  await Bun.write(out, text);
  return { ok: true, out, bytes: new TextEncoder().encode(text).length, ms: +(performance.now() - t0).toFixed(1) };
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
