// `og build`'s web half: the play page of a game, ready to serve from one
// directory (with COOP/COEP for the input and request rings).
//
//   bun src/node/web-build.ts --project <dir> --entry <src/main.ts> --out <dist> [--publish] [--json]
//
// Writes into <dist>:
//   game.js                    the engine Worker: startWorker(game), the runtime, the game (minified with --publish)
//   play.js, index.html        the page: the shell on a full-viewport canvas, ink #14161b while booting
//   origin-engine.js, origin-engine_bg.wasm   the web host (dist/web, or dist/web-publish with --publish)
//   basis/                     Binomial's Basis transcoder (vendor/basis) and its licence
//   cooked/                    the project's cooked bundle (og cook writes it)
// and prints a size table (bytes and brotli -q 11, or gzip -9 without the
// brotli CLI). With --json the table is one JSON line:
//   { ok, out, publish, files: [{ path, bytes, compressed, codec }], total: { bytes, compressed }, probe: { build, members } }
// `probe` is read from the copied web host itself (its hello under bun), so a
// publish build proves it serves only the player-facing members.

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame, ENGINE_ROOT } from "./build.ts";

declare module "node:fs" {
  export function cpSync(from: string, to: string, options?: { recursive?: boolean }): void;
  export function readdirSync(path: string): string[];
}
declare module "node:path" {
  export function relative(from: string, to: string): string;
}

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string; minify?: boolean }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { stdin?: Uint8Array; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  gzipSync(data: Uint8Array, o?: { level?: number }): Uint8Array;
  which(cmd: string): string | null;
  version: string;
};

const here = dirname(fileURLToPath(import.meta.url));

export interface SizeRow {
  readonly path: string;
  readonly bytes: number;
  readonly compressed: number;
  readonly codec: "brotli-11" | "gzip-9";
}

export interface WebBuild {
  readonly ok: boolean;
  readonly out: string;
  readonly publish: boolean;
  readonly files: SizeRow[];
  readonly total: { bytes: number; compressed: number };
  readonly probe: { build: string; members: string[] } | null;
  readonly reason?: string;
}

/** The page; `profile` is project.ogt's physics profile, read by play.ts. */
const pageHtml = (profile: string) => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="og-profile" content="${profile}">
<title>Origin Engine</title>
<style>html,body{margin:0;height:100%;background:#14161b;overscroll-behavior:none}canvas{display:block;width:100vw;height:100dvh;touch-action:none}</style>
</head>
<body><canvas></canvas><script type="module" src="./play.js"></script></body>
</html>
`;

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir).sort()) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...walk(path));
    else out.push(path);
  }
  return out;
}

function compressedSize(bytes: Uint8Array): { size: number; codec: SizeRow["codec"] } {
  if (Bun.which("brotli")) {
    const r = Bun.spawnSync(["brotli", "-q", "11", "-c"], { stdin: bytes, stdout: "pipe", stderr: "pipe" });
    if (r.exitCode === 0) return { size: r.stdout.length, codec: "brotli-11" };
  }
  return { size: Bun.gzipSync(bytes, { level: 9 }).length, codec: "gzip-9" };
}

/** The probe build and members the copied web host serves: its own hello, under bun. */
async function probeOf(glue: string, wasm: string): Promise<{ build: string; members: string[] } | null> {
  const script = `
    const glue = await import(${JSON.stringify(pathToFileURL(glue).href)});
    const { loadWebHost } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "web", "host.ts")).href)});
    const { og } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "index.ts")).href)});
    const { createEngine } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "runtime.ts")).href)});
    const { readFileSync } = await import("node:fs");
    await loadWebHost(glue, readFileSync(${JSON.stringify(wasm)}));
    const e = createEngine(og.game({}), {});
    if (!e.ok) throw new Error(e.reason);
    const hello = JSON.parse(e.value.call(JSON.stringify({ jsonrpc: "2.0", id: 1, method: "hello", params: {} }))).result;
    console.log(JSON.stringify({ build: hello.build, members: hello.members }));
  `;
  const r = Bun.spawnSync(["bun", "-e", script], { stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) return null;
  const line = new TextDecoder().decode(r.stdout).trim().split("\n").pop() ?? "";
  try {
    return JSON.parse(line) as { build: string; members: string[] };
  } catch {
    return null;
  }
}

export async function webBuild(project: string, entry: string, out: string, publish: boolean): Promise<WebBuild> {
  const fail = (reason: string): WebBuild => ({ ok: false, out, publish, files: [], total: { bytes: 0, compressed: 0 }, probe: null, reason });
  const cooked = join(project, "cooked");
  if (!existsSync(join(cooked, ".cook.ogt"))) return fail(`${cooked} has no cooked bundle; run \`og cook ${project}\` first`);
  const host = join(ENGINE_ROOT, "dist", publish ? "web-publish" : "web");
  for (const f of ["origin-engine.js", "origin-engine_bg.wasm"]) {
    if (!existsSync(join(host, f))) return fail(`${join(host, f)} is missing: build the web host with \`bun run build\` in packages/engine (scripts/build.sh)`);
  }
  const vendor = join(ENGINE_ROOT, "vendor", "basis");
  if (!existsSync(join(vendor, "basis_transcoder.wasm"))) return fail(`${vendor} is missing the Basis transcoder`);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });

  const game = await bundleGame(entry, join(out, "game.js"), "web", publish);
  if (!game.ok) return fail(`bundling ${entry}: ${game.reason}`);
  const page = await Bun.build({ entrypoints: [join(here, "..", "web", "play.ts")], target: "browser", format: "esm", minify: publish });
  if (!page.success) return fail(`bundling the play page: ${page.logs.map(String).join("\n")}`);
  writeFileSync(join(out, "play.js"), await page.outputs[0]!.text());
  const manifest = existsSync(join(project, "project.ogt")) ? readFileSync(join(project, "project.ogt"), "utf8") : "";
  const profile = /^\s*profile\s+"(2d|3d)"/m.exec(manifest)?.[1] ?? "3d";
  writeFileSync(join(out, "index.html"), pageHtml(profile));
  for (const f of ["origin-engine.js", "origin-engine_bg.wasm"]) cpSync(join(host, f), join(out, f));
  cpSync(vendor, join(out, "basis"), { recursive: true });
  cpSync(cooked, join(out, "cooked"), { recursive: true });

  const files = walk(out).map((path): SizeRow => {
    const bytes = new Uint8Array(readFileSync(path));
    const c = compressedSize(bytes);
    return { path: relative(out, path), bytes: bytes.length, compressed: c.size, codec: c.codec };
  });
  const total = files.reduce((t, f) => ({ bytes: t.bytes + f.bytes, compressed: t.compressed + f.compressed }), { bytes: 0, compressed: 0 });
  const probe = await probeOf(join(out, "origin-engine.js"), join(out, "origin-engine_bg.wasm"));
  return { ok: true, out, publish, files, total, probe };
}

function table(b: WebBuild): string {
  const codec = b.files[0]?.codec ?? "brotli-11";
  const rows = [["file", "bytes", codec], ...b.files.map((f) => [f.path, String(f.bytes), String(f.compressed)]), ["total", String(b.total.bytes), String(b.total.compressed)]];
  const w = [0, 1, 2].map((i) => Math.max(...rows.map((r) => r[i]!.length)));
  return rows.map((r) => `${r[0]!.padEnd(w[0]!)}  ${r[1]!.padStart(w[1]!)}  ${r[2]!.padStart(w[2]!)}`).join("\n") + "\n";
}

if (import.meta.main) {
  const argv = process.argv.slice(2);
  const opt = (name: string) => {
    const i = argv.indexOf(name);
    return i >= 0 ? argv[i + 1] : undefined;
  };
  const project = opt("--project");
  const entry = opt("--entry");
  const out = opt("--out");
  if (!project || !entry || !out) {
    process.stderr.write("usage: bun src/node/web-build.ts --project <dir> --entry <src/main.ts> --out <dist> [--publish] [--json]\n");
    process.exit(2);
  }
  const b = await webBuild(resolve(project!), resolve(entry!), resolve(out!), argv.includes("--publish"));
  if (!b.ok) {
    if (argv.includes("--json")) process.stdout.write(JSON.stringify(b) + "\n");
    process.stderr.write(`og build: ${b.reason}\n`);
    process.exit(1);
  }
  if (argv.includes("--json")) process.stdout.write(JSON.stringify(b) + "\n");
  else {
    process.stdout.write(table(b));
    if (b.probe) process.stdout.write(`probe: ${b.probe.build} build, ${b.probe.members.length} members: ${b.probe.members.join(", ")}\n`);
  }
}
