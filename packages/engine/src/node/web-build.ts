// `og build`'s web half: the exported game, a self-contained directory any
// static host serves (docs/web-export.md).
//
//   bun src/node/web-build.ts --project <dir> --entry <src/main.ts> --out <dist>
//       [--publish] [--json] [--settings <json>]
//
// `--settings` is project.ogt as og read and checked it (name, id, engine,
// profile and the `publish` block); without it the defaults apply. Writes
// into <dist>:
//
//   index.html                 the player shell (shell/page.ts): boot splash, gate, error card, badge;
//                              `<meta name="og-engine" content="<version>">`
//   play.js                    the shell, bundled (minified with --publish)
//   game.js                    the engine Worker: startWorker(game), the runtime, the game
//   audio-worklet.js           the AudioWorklet the shell loads on the first gesture
//   origin-engine.js, origin-engine_bg.wasm   the web host: dist/web (every member, both physics
//                              profiles), or with --publish the variant of the game's profile:
//                              dist/web-publish (3D) or dist/web-publish-2d (2D), each with only
//                              its profile's physics (docs/web-export.md §Variants)
//   origin-engine-lanes.js, origin-engine-lanes_bg.wasm, snippets/
//                              (--publish, isolation not "off") the lanes variant of the same
//                              profile, from the same directory: wasm atomics, rayon on a pool of
//                              Workers when the page is cross-origin isolated
//   basis/                     Binomial's Basis transcoder and its licence
//   cooked/                    the project's cooked bundle: the first group (everything no streaming
//                              group holds, the proxies included) and each streaming group's pack,
//                              cooked/stream/<name>.ogb; a grouped output's own file is left out
//   icons/                     the project's icon, or the brand tile at 192 and 512 px
//   og-build.json              the build manifest (the Player's contract; shape in docs/web-export.md)
//   _headers                   COOP/COEP/CORP and caching for static hosts (Netlify/Cloudflare Pages format)
//   origingame.runtime.json    the platform's isolation declaration, when the project wants isolation
//   manifest.webmanifest, sw.js   (--publish) the PWA manifest and the offline service worker
//
// The bytes depend only on the inputs: no dates, no absolute paths, files in
// sorted order. Prints a size table (bytes and brotli -q 11, or gzip -9
// without the brotli CLI), then what a player downloads (downloadsOf: each
// page's boot, the lanes and the single-threaded page, and what comes
// later); with --json one JSON line:
//   { ok, out, publish, files: [{ path, bytes, compressed, codec }], total, downloads, probe: { build, members }, manifest }

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, realpathSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame, ENGINE_ROOT } from "./build.ts";
import { bootMark, engineMark, motionSubset, shellTokens, SHELL_MOTIONS, BRAND_DIR } from "../../shell/brand.ts";

declare module "node:fs" {
  export function cpSync(from: string, to: string, options?: { recursive?: boolean }): void;
  export function readdirSync(path: string): string[];
  export function realpathSync(path: string): string;
}
declare module "node:path" {
  export function relative(from: string, to: string): string;
  export function isAbsolute(path: string): boolean;
  export const sep: string;
}

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string; minify?: boolean }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { stdin?: Uint8Array; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  gzipSync(data: Uint8Array, o?: { level?: number }): Uint8Array;
  which(cmd: string): string | null;
  CryptoHasher: new (algorithm: string) => { update(data: Uint8Array | string): unknown; digest(encoding: "hex"): string };
  version: string;
};

const here = dirname(fileURLToPath(import.meta.url));
const SHELL_DIR = join(ENGINE_ROOT, "shell");
export const ENGINE_VERSION = (JSON.parse(readFileSync(join(ENGINE_ROOT, "package.json"), "utf8")) as { version: string }).version;

/** The manifest's file name (the platform's Player reads it). */
export const BUILD_MANIFEST = "og-build.json";
/**
 * The Player transports the shell speaks: `frame` (protocol 2 over postMessage to a
 * framing Player) and `page` (protocol 2 over same-origin HTTP when the page is the
 * top-level document). Written as `<meta name="og-player">` and og-build.json `player`.
 */
export const PLAYER_TRANSPORTS = ["frame", "page"] as const;

export interface SizeRow {
  readonly path: string;
  readonly bytes: number;
  readonly compressed: number;
  readonly codec: "brotli-11" | "gzip-9";
}

/** project.ogt's `publish` block as og checked it (crates/og-cli/src/project.rs). */
export interface PublishSettings {
  readonly probe: "player" | "all";
  readonly isolation: "required" | "preferred" | "off";
  readonly orientation: "any" | "landscape" | "portrait";
  readonly fit: "letterbox" | "fill";
  /** `"16:9"` and the like; absent: the screen's own. */
  readonly aspect: string | null;
  readonly dprCap: number;
  readonly fullscreen: "gesture" | "off";
  readonly gate: "tap" | "none";
  readonly madeWith: boolean;
  /** A PNG relative to the project, square, at least 192 px. */
  readonly icon: string | null;
  /** The installed app's short name; absent: the name. */
  readonly shortName: string | null;
}

export interface ProjectSettings {
  readonly name: string;
  readonly id: string;
  /** The engine version project.ogt pins. */
  readonly engine: string;
  readonly profile: "2d" | "3d";
  readonly publish: PublishSettings;
}

export const DEFAULT_PUBLISH: PublishSettings = {
  probe: "player",
  isolation: "preferred",
  orientation: "any",
  fit: "fill",
  aspect: null,
  dprCap: 2,
  fullscreen: "off",
  gate: "tap",
  madeWith: true,
  icon: null,
  shortName: null,
};

/** The settings of a project read without og (tests): name, id, engine and profile; publish defaults. */
export function settingsOf(project: string): ProjectSettings {
  const text = existsSync(join(project, "project.ogt")) ? readFileSync(join(project, "project.ogt"), "utf8") : "";
  const word = (key: string) => new RegExp(`^\\s*${key}\\s+"([^"]*)"`, "m").exec(text)?.[1];
  return {
    name: word("name") ?? "Origin Engine game",
    id: word("id") ?? "game",
    engine: word("engine") ?? ENGINE_VERSION,
    profile: word("profile") === "2d" ? "2d" : "3d",
    publish: DEFAULT_PUBLISH,
  };
}

export interface BuildFile {
  readonly path: string;
  readonly bytes: number;
  readonly sha256: string;
}

/** og-build.json (docs/web-export.md §The manifest). */
export interface BuildManifest {
  readonly format: 1;
  readonly engine: { readonly name: "origin-engine"; readonly version: string; readonly pinned: string };
  readonly build: "publish" | "dev";
  readonly game: { readonly name: string; readonly id: string; readonly profile: "2d" | "3d" };
  readonly entry: "index.html";
  readonly worker: "game.js";
  readonly wasm: "origin-engine_bg.wasm";
  /**
   * Which engine module `wasm` is (docs/web-export.md §Variants): `full`
   * (a dev build: every member, both physics profiles), or the publish
   * variant of the game's profile, `3d` or `2d`, which has only that
   * profile's physics. The lanes variant, when there is one, is the same
   * profile's (its physics and the publish members, with wasm atomics).
   */
  readonly variant: "full" | "3d" | "2d";
  /**
   * The lanes variant (docs/web-export.md §Lanes): the same engine built with
   * wasm atomics, which the shell boots instead when the page is cross-origin
   * isolated. Null when the build has none (a dev build, isolation "off").
   */
  readonly lanes: { readonly glue: "origin-engine-lanes.js"; readonly wasm: "origin-engine-lanes_bg.wasm" } | null;
  readonly probe: { readonly protocol: 5; readonly wire: 1; readonly build: string; readonly members: readonly string[] };
  readonly crossOriginIsolation: "required" | "preferred" | "off";
  readonly display: {
    readonly fit: "letterbox" | "fill";
    readonly aspect: string | null;
    readonly orientation: "any" | "landscape" | "portrait";
    readonly dprCap: number;
    readonly fullscreen: "gesture" | "off";
    readonly gate: "tap" | "none";
    readonly madeWith: boolean;
  };
  readonly icons: ReadonlyArray<{ readonly src: string; readonly sizes: string; readonly type: "image/png"; readonly source: "project" | "brand" }>;
  readonly pwa: { readonly manifest: string; readonly serviceWorker: string } | null;
  /** The Player protocol the shell speaks and over which transports (origingame docs/player-protocol.md §The top-level engine page). */
  readonly player: { readonly protocol: 2; readonly transports: readonly ("frame" | "page")[] };
  /** sha256 over the files' paths and hashes: the build's identity (the service worker's cache name). */
  readonly version: string;
  readonly files: readonly BuildFile[];
  readonly total: { readonly files: number; readonly bytes: number };
  /**
   * The bundle split by streaming group: the first group is what the boot
   * fetches and the service worker precaches; each other group is one pack
   * the engine fetches when the group loads (and the service worker caches
   * then). Null for a bundle without streaming groups.
   */
  readonly stream: {
    readonly first: { readonly files: number; readonly bytes: number };
    readonly groups: ReadonlyArray<StreamGroupFile>;
  } | null;
}

/** One streaming group's pack in og-build.json. */
export interface StreamGroupFile {
  readonly name: string;
  /** Its pack, a path of `files`. */
  readonly file: string;
  readonly bytes: number;
  readonly sha256: string;
  readonly priority: number;
  /** What its assets take once loaded. */
  readonly memory: number;
  readonly assets: readonly string[];
}

/** A cooked manifest's streaming groups and the files of their outputs (written by og cook; crates/og-cook/src/runtime.rs). */
function streamGroupsOf(manifest: string): { groups: Array<{ name: string; file: string; priority: number; memory: number; assets: string[] }>; grouped: string[] } {
  const groups: Array<{ name: string; file: string; priority: number; memory: number; assets: string[] }> = [];
  const grouped: string[] = [];
  let output: { file?: string; group?: boolean } | null = null;
  let group: (typeof groups)[number] | null = null;
  const unquote = (v: string) => JSON.parse(v) as string;
  for (const line of manifest.split("\n")) {
    let m: RegExpExecArray | null;
    if ((m = /^stream ("(?:[^"\\]|\\.)*") \{$/.exec(line))) {
      group = { name: unquote(m[1]!), file: "", priority: 0, memory: 0, assets: [] };
      groups.push(group);
    } else if (/^\S/.test(line)) group = null;
    if (group) {
      if ((m = /^  file ("(?:[^"\\]|\\.)*")$/.exec(line))) group.file = unquote(m[1]!);
      else if ((m = /^  priority (-?\d+)$/.exec(line))) group.priority = Number(m[1]);
      else if ((m = /^  memory (\d+)$/.exec(line))) group.memory = Number(m[1]);
      else if ((m = /^  asset ("(?:[^"\\]|\\.)*")$/.exec(line))) group.assets.push(unquote(m[1]!));
      continue;
    }
    if (/^  output /.test(line)) output = {};
    else if (output && /^  \}$/.test(line)) {
      if (output.group && output.file) grouped.push(output.file);
      output = null;
    } else if (output && (m = /^    file ("(?:[^"\\]|\\.)*")$/.exec(line))) output.file = unquote(m[1]!);
    else if (output && /^    group "/.test(line)) output.group = true;
  }
  return { groups, grouped };
}

/** Files, raw bytes and compressed bytes (the size table's codec). */
export interface DownloadSum {
  readonly files: number;
  readonly bytes: number;
  readonly compressed: number;
}

/**
 * What a player downloads, as the size gate counts it (each file's brotli
 * -q 11 size, gzip -9 without the brotli CLI): the boot of each page the
 * build serves, everything fetched before its first frame, and what comes
 * later. A page takes one engine module: the lanes module when the page is
 * cross-origin isolated and the build ships it, else the single-threaded
 * one (docs/web-export.md §Lanes). The WebGPU and WebGL2 pages fetch the
 * same files. `total` (every file of the directory) is none of these: it
 * counts both engine modules, the licences, the icons and the manifest.
 */
export interface Downloads {
  /** The boot of the single-threaded page, and of the lanes page when the build ships lanes. */
  readonly boot: { readonly single: DownloadBoot; readonly lanes: DownloadBoot | null };
  readonly later: {
    /** Each streaming group's pack, fetched when its group loads. */
    readonly packs: DownloadSum;
    /** The chunks of the sounds cooked in chunks after their first, fetched ahead of their playheads. */
    readonly chunks: DownloadSum;
    /** The AudioWorklet, fetched on the first gesture. */
    readonly worklet: DownloadSum;
    /** The service worker, the web manifest and the icons (a publish build's PWA, after the first frame). */
    readonly pwa: DownloadSum;
  };
}

/** One page's boot: its total and its parts. */
export interface DownloadBoot extends DownloadSum {
  readonly engine: DownloadSum;
  /** The cooked first group (everything no streaming group holds; a chunked sound's first chunk is its own output). */
  readonly cooked: DownloadSum;
  /** index.html, play.js, game.js. */
  readonly shell: DownloadSum;
  /** The Basis transcoder (fetched when the first group has Basis textures). */
  readonly basis: DownloadSum;
}

/** What each page of a build downloads (Downloads), from its size table. */
export function downloadsOf(files: readonly SizeRow[]): Downloads {
  const rows = files.map((f) => ({ ...f, path: f.path.split("\\").join("/") }));
  const sum = (test: (path: string) => boolean): DownloadSum =>
    rows.filter((f) => test(f.path)).reduce((a, f) => ({ files: a.files + 1, bytes: a.bytes + f.bytes, compressed: a.compressed + f.compressed }), { files: 0, bytes: 0, compressed: 0 });
  const add = (...parts: DownloadSum[]): DownloadSum => parts.reduce((a, b) => ({ files: a.files + b.files, bytes: a.bytes + b.bytes, compressed: a.compressed + b.compressed }), { files: 0, bytes: 0, compressed: 0 });
  const isPack = (p: string) => /^cooked\/stream\/[^/]+\.ogb$/.test(p);
  const cooked = sum((p) => p.startsWith("cooked/") && !isPack(p) && !SOUND_CHUNK.test(p));
  const shell = sum((p) => /^(index\.html|play\.js|game\.js)$/.test(p));
  const basis = sum((p) => /^basis\/basis_transcoder\.(js|wasm)$/.test(p));
  const page = (engine: DownloadSum): DownloadBoot => ({ ...add(engine, cooked, shell, basis), engine, cooked, shell, basis });
  const lanes = sum((p) => LANES_ENGINE.test(p));
  return {
    boot: { single: page(sum((p) => SINGLE_ENGINE.test(p))), lanes: lanes.files > 0 ? page(lanes) : null },
    later: {
      packs: sum(isPack),
      chunks: sum((p) => SOUND_CHUNK.test(p)),
      worklet: sum((p) => p === "audio-worklet.js"),
      pwa: sum((p) => p === "sw.js" || p === "manifest.webmanifest" || p.startsWith("icons/")),
    },
  };
}

/** The download lines `og build` prints under its size table. */
export function downloadLines(d: Downloads, codec: SizeRow["codec"]): string {
  const kb = (n: number) => `${(n / 1000).toFixed(1)} kB`;
  const boot = (name: string, b: DownloadBoot) =>
    `  ${name.padEnd(30)} ${String(b.files).padStart(4)} files  ${String(b.compressed).padStart(10)} B  (engine ${kb(b.engine.compressed)}, cooked ${kb(b.cooked.compressed)}, shell ${kb(b.shell.compressed)}, basis ${kb(b.basis.compressed)})`;
  const lines = [`download (${codec}, what a player fetches; a WebGPU and a WebGL2 page fetch the same files):`];
  if (d.boot.lanes) lines.push(boot("boot, lanes page (isolated)", d.boot.lanes));
  lines.push(boot("boot, single-threaded page", d.boot.single));
  const later = [
    d.later.packs.files ? `${d.later.packs.files} streaming packs ${d.later.packs.compressed} B (as their groups load)` : "",
    d.later.chunks.files ? `${d.later.chunks.files} sound chunks ${d.later.chunks.compressed} B (ahead of their playheads)` : "",
    d.later.worklet.files ? `the audio worklet ${d.later.worklet.compressed} B (first gesture)` : "",
    d.later.pwa.files ? `the service worker and icons ${d.later.pwa.compressed} B` : "",
  ].filter(Boolean);
  if (later.length) lines.push(`  later: ${later.join("; ")}`);
  return lines.join("\n") + "\n";
}

export interface WebBuild {
  readonly ok: boolean;
  readonly out: string;
  readonly publish: boolean;
  readonly files: SizeRow[];
  readonly total: { bytes: number; compressed: number };
  /** What each page downloads (boot and later); `total` is every file of the directory. */
  readonly downloads?: Downloads;
  readonly probe: { build: string; members: string[] } | null;
  readonly manifest?: BuildManifest;
  readonly reason?: string;
}

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir).sort()) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...walk(path));
    else out.push(path);
  }
  return out;
}

function sha256(data: Uint8Array | string): string {
  const h = new Bun.CryptoHasher("sha256");
  h.update(data);
  return h.digest("hex");
}

// Facts that depend only on a file's bytes and are slow to recompute (brotli
// -q 11 of the 8 MB wasm is about 12 s; the host's hello needs a bun process
// that loads it) are kept by sha256 in dist/.build-cache.json across builds,
// so a test that builds four times or a `og build` after an edit to game code
// pays for the engine once. The cache keeps its newest 2000 entries.
const CACHE_FILE = join(here, "..", "..", "dist", ".build-cache.json");
let cache: Record<string, unknown> | null = null;
function cacheGet(key: string): unknown {
  if (!cache) {
    try {
      cache = JSON.parse(readFileSync(CACHE_FILE, "utf8")) as Record<string, unknown>;
    } catch {
      cache = {};
    }
  }
  return cache[key];
}
function cachePut(key: string, value: unknown): void {
  cache ??= {};
  delete cache[key];
  cache[key] = value;
  const keys = Object.keys(cache);
  for (const k of keys.slice(0, Math.max(0, keys.length - 2000))) delete cache[k];
  try {
    mkdirSync(dirname(CACHE_FILE), { recursive: true });
    writeFileSync(CACHE_FILE, JSON.stringify(cache));
  } catch {
    // A read-only checkout still builds; it only recomputes.
  }
}

function compressedSize(bytes: Uint8Array): { size: number; codec: SizeRow["codec"] } {
  const brotli = Boolean(Bun.which("brotli"));
  const key = `size:${brotli ? "brotli-11" : "gzip-9"}:${sha256(bytes)}`;
  const hit = cacheGet(key) as { size: number; codec: SizeRow["codec"] } | undefined;
  if (hit) return hit;
  let out: { size: number; codec: SizeRow["codec"] } | null = null;
  if (brotli) {
    const r = Bun.spawnSync(["brotli", "-q", "11", "-c"], { stdin: bytes, stdout: "pipe", stderr: "pipe" });
    if (r.exitCode === 0) out = { size: r.stdout.length, codec: "brotli-11" };
  }
  out ??= { size: Bun.gzipSync(bytes, { level: 9 }).length, codec: "gzip-9" };
  cachePut(key, out);
  return out;
}

/**
 * The probe build and members the copied web host serves: its own hello,
 * under bun, from an engine of the game's profile (a publish variant has
 * only that profile's physics).
 */
async function probeOf(glue: string, wasm: string, profile: "2d" | "3d"): Promise<{ build: string; members: string[] } | null> {
  const key = `probe:${profile}:${sha256(new Uint8Array(readFileSync(glue)))}:${sha256(new Uint8Array(readFileSync(wasm)))}`;
  const hit = cacheGet(key) as { build: string; members: string[] } | undefined;
  if (hit) return hit;
  const probe = await probeOfUncached(glue, wasm, profile);
  if (probe) cachePut(key, probe);
  return probe;
}

async function probeOfUncached(glue: string, wasm: string, profile: "2d" | "3d"): Promise<{ build: string; members: string[] } | null> {
  const script = `
    const glue = await import(${JSON.stringify(pathToFileURL(glue).href)});
    const { loadWebHost } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "web", "host.ts")).href)});
    const { og } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "index.ts")).href)});
    const { createEngine } = await import(${JSON.stringify(pathToFileURL(join(here, "..", "runtime.ts")).href)});
    const { readFileSync } = await import("node:fs");
    await loadWebHost(glue, readFileSync(${JSON.stringify(wasm)}));
    const e = createEngine(og.game({}), { profile: ${JSON.stringify(profile)} });
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

/** A PNG's width and height (its IHDR), or null when the bytes are not a PNG. */
export function pngSize(bytes: Uint8Array): { width: number; height: number } | null {
  const sig = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (bytes.length < 24 || sig.some((b, i) => bytes[i] !== b)) return null;
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return { width: v.getUint32(16), height: v.getUint32(20) };
}

/** `"16:9"` as a ratio, or null. */
export function aspectRatio(aspect: string | null): number | null {
  const m = aspect ? /^(\d+(?:\.\d+)?):(\d+(?:\.\d+)?)$/.exec(aspect) : null;
  if (!m) return null;
  const r = Number(m[1]) / Number(m[2]);
  return Number.isFinite(r) && r > 0 ? r : null;
}

const escapeHtml = (s: string) => s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);

/** index.html: the shell's markup with its CSS, marks and configuration inlined. */
export function pageHtml(o: { settings: ProjectSettings; publish: boolean; config: Record<string, unknown>; icon: string; pwa: boolean }): string {
  const css = [shellTokens(), motionSubset(SHELL_MOTIONS), readFileSync(join(SHELL_DIR, "shell.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "").replace(/\n\s*/g, "\n").trim()].join("\n");
  // `<` cannot appear in the JSON's strings unescaped, so the script element cannot be closed early.
  const config = JSON.stringify(o.config).replace(/</g, "\\u003c");
  const title = escapeHtml(o.settings.name);
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover, interactive-widget=resizes-content">
<meta name="og-engine" content="${escapeHtml(ENGINE_VERSION)}">
<meta name="og-build" content="${o.publish ? "publish" : "dev"}">
<meta name="og-profile" content="${o.settings.profile}">
<meta name="og-player" content="${PLAYER_TRANSPORTS.join(" ")}">
<meta name="theme-color" content="#14161b">
<meta name="color-scheme" content="dark">
<meta name="mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
<meta name="apple-mobile-web-app-title" content="${escapeHtml(o.settings.publish.shortName ?? o.settings.name)}">
<title>${title}</title>
<link rel="icon" href="./${o.icon}">
<link rel="apple-touch-icon" href="./${o.icon}">
${o.pwa ? '<link rel="manifest" href="./manifest.webmanifest">\n' : ""}<link rel="modulepreload" href="./play.js">
<style>
${css}
</style>
</head>
<body>
<div id="og-stage"><canvas id="og-canvas" tabindex="0" aria-label="${title}"></canvas></div>
<div id="og-boot" class="og-layer" role="progressbar" aria-label="Loading" aria-valuemin="0" aria-valuemax="100">
<div id="og-backdrop" aria-hidden="true"></div>
${bootMark()}
<div class="og-boot-label" id="og-boot-label" aria-live="polite"></div>
</div>
<div id="og-gate" class="og-layer" hidden><button type="button">Play</button></div>
<div id="og-error" class="og-layer" role="alert" hidden>
<div class="og-card">
<h1></h1>
<p></p>
<details><summary>Log</summary><pre id="og-error-log"></pre></details>
<details hidden><summary>World</summary><pre id="og-error-dump"></pre></details>
<div class="og-actions"><button type="button" data-action="reload">Reload</button><button type="button" data-action="copy">Copy report</button></div>
</div>
</div>
<div id="og-install" role="dialog" aria-label="Install" hidden><span></span><button type="button"></button><button type="button"></button></div>
<button id="og-back" type="button" aria-label="Back" hidden><svg viewBox="0 0 24 24" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="m15 6-6 6 6 6"/></svg><span>Back</span></button>
<a id="og-made-with" href="https://origingame.ai" target="_blank" rel="noopener"${o.settings.publish.madeWith ? "" : " hidden"}>${engineMark()}<span>Made with Origin Engine</span></a>
<script type="application/json" id="og-config">${config}</script>
<script type="module" src="./play.js"></script>
</body>
</html>
`;
}

/** The `_headers` file: COOP/COEP when the project wants isolation, CORP and caching always. */
export function headersFile(isolation: PublishSettings["isolation"]): string {
  const lines = ["/*", "  Cross-Origin-Resource-Policy: cross-origin", "  X-Content-Type-Options: nosniff"];
  if (isolation !== "off") lines.push("  Cross-Origin-Opener-Policy: same-origin", "  Cross-Origin-Embedder-Policy: require-corp");
  lines.push(
    "/index.html",
    "  Cache-Control: no-cache",
    "/",
    "  Cache-Control: no-cache",
    "/sw.js",
    "  Cache-Control: no-cache",
    "/og-build.json",
    "  Cache-Control: no-cache",
    "/*.wasm",
    "  Content-Type: application/wasm",
    "/manifest.webmanifest",
    "  Content-Type: application/manifest+json",
  );
  return lines.join("\n") + "\n";
}

async function bundleScript(entry: string, minify: boolean): Promise<string | { error: string }> {
  const r = await Bun.build({ entrypoints: [entry], target: "browser", format: "esm", minify });
  if (!r.success) return { error: r.logs.map(String).join("\n") };
  return r.outputs[0]!.text();
}

/** Files that never go into the service worker's precache (not fetched by the page). */
const NOT_PRECACHED = /(^|\/)(LICENSE|NOTICE|README\.md|_headers|origingame\.runtime\.json|og-build\.json|sw\.js)$/;
/**
 * The lanes variant's files beside each profile's publish module
 * (dist/web-publish for 3D, dist/web-publish-2d for 2D; scripts/build.sh);
 * snippets/ holds wasm-bindgen-rayon's Worker module.
 */
const LANES_FILES = ["origin-engine-lanes.js", "origin-engine-lanes_bg.wasm", "snippets"] as const;
/** Each engine variant's own files: the service worker precaches the one its page boots. */
const SINGLE_ENGINE = /^origin-engine(\.js|_bg\.wasm)$/;
const LANES_ENGINE = /^(origin-engine-lanes(\.js|_bg\.wasm)|snippets\/.*)$/;
/** A chunk of a sound cooked in chunks: fetched as its playhead nears it, cached then. */
const SOUND_CHUNK = /^cooked\/.*\.chunk\d+\.ogb$/;

/**
 * Why `out` may not be wiped for a build, or null. A build replaces its
 * output directory whole, so it refuses the project itself, any ancestor of
 * it, a directory holding a project.ogt, and any non-empty directory that a
 * previous build did not write (no og-build.json in it).
 */
function outRefusal(project: string, out: string): string | null {
  const real = (path: string) => (existsSync(path) ? realpathSync(path) : resolve(path));
  const root = real(project);
  const target = real(out);
  const up = relative(target, root);
  if (up === "" || !(up === ".." || up.startsWith(`..${sep}`) || isAbsolute(up))) return `${out} is the project or contains it; give an output directory inside or beside the project (default dist)`;
  if (!existsSync(target)) return null;
  if (!statSync(target).isDirectory()) return `${out} is a file, not a directory`;
  if (existsSync(join(target, "project.ogt"))) return `${out} holds a project.ogt; give an output directory of its own (default dist)`;
  if (readdirSync(target).length > 0 && !existsSync(join(target, BUILD_MANIFEST))) return `${out} is not empty and holds no ${BUILD_MANIFEST} from an earlier build, so og build will not replace it; empty or remove it, or give another output directory`;
  return null;
}

export async function webBuild(project: string, entry: string, out: string, publish: boolean, given?: ProjectSettings): Promise<WebBuild> {
  // Once the output directory is replaced, a failure removes what was half written,
  // so the next build is not refused for a directory without og-build.json.
  let replaced = false;
  const fail = (reason: string): WebBuild => {
    if (replaced) rmSync(out, { recursive: true, force: true });
    return { ok: false, out, publish, files: [], total: { bytes: 0, compressed: 0 }, probe: null, reason };
  };
  const refusal = outRefusal(project, out);
  if (refusal) return fail(refusal);
  const settings = given ?? settingsOf(project);
  const p = settings.publish;
  const cooked = join(project, "cooked");
  if (!existsSync(join(cooked, ".cook.ogt"))) return fail(`${cooked} has no cooked bundle; run \`og cook ${project}\` first`);
  // The engine module: a dev build takes the full one; a publish build the
  // variant of the game's profile, which a player downloads alone, and the
  // lanes variant of that profile from the same directory (one lanes module
  // per profile since owed row 71: a 2D game's lanes page never downloads
  // rapier3d, nor a 3D game's rapier2d).
  const variant: BuildManifest["variant"] = publish ? settings.profile : "full";
  const engineDir = join(ENGINE_ROOT, "dist", variant === "2d" ? "web-publish-2d" : publish ? "web-publish" : "web");
  for (const f of ["origin-engine.js", "origin-engine_bg.wasm"]) {
    if (!existsSync(join(engineDir, f))) return fail(`${join(engineDir, f)} is missing: build the web host with \`bun run build\` in packages/engine (scripts/build.sh)${variant === "2d" ? "; a 2d game ships the 2D publish variant, dist/web-publish-2d" : ""}`);
  }
  const vendor = join(ENGINE_ROOT, "vendor", "basis");
  if (!existsSync(join(vendor, "basis_transcoder.wasm"))) return fail(`${vendor} is missing the Basis transcoder`);
  let projectIcon: { bytes: Uint8Array; size: number } | null = null;
  if (p.icon) {
    const file = resolve(project, p.icon);
    if (!existsSync(file)) return fail(`project.ogt publish.icon: ${p.icon} does not exist; give a square PNG of at least 192 px, or remove the line for the brand tile`);
    const bytes = new Uint8Array(readFileSync(file));
    const size = pngSize(bytes);
    if (!size) return fail(`project.ogt publish.icon: ${p.icon} is not a PNG; give a square PNG of at least 192 px`);
    if (size.width !== size.height || size.width < 192) return fail(`project.ogt publish.icon: ${p.icon} is ${size.width}x${size.height}; give a square PNG of at least 192 px (512 px is best)`);
    projectIcon = { bytes, size: size.width };
  }
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  replaced = true;

  const game = await bundleGame(entry, join(out, "game.js"), "web", publish);
  if (!game.ok) return fail(`bundling ${entry}: ${game.reason}`);
  for (const [file, src] of [["play.js", join(here, "..", "web", "play.ts")], ["audio-worklet.js", join(here, "..", "web", "audio-worklet.ts")]] as const) {
    const text = await bundleScript(src, publish);
    if (typeof text !== "string") return fail(`bundling ${file}: ${text.error}`);
    writeFileSync(join(out, file), text);
  }
  for (const f of ["origin-engine.js", "origin-engine_bg.wasm"]) cpSync(join(engineDir, f), join(out, f));
  // The lanes variant needs cross-origin isolation, which only a build that declares it can get.
  const missingLanes = publish && p.isolation !== "off" ? LANES_FILES.filter((f) => !existsSync(join(engineDir, f))) : [];
  const lanes = publish && p.isolation !== "off" && missingLanes.length === 0;
  if (missingLanes.length > 0) {
    process.stderr.write(
      `og build: warning: ${join(engineDir, missingLanes[0]!)} is missing, so this build ships single-threaded (no ${variant === "2d" ? "2D" : "3D"} lanes variant); \`bun run build\` in packages/engine builds it (scripts/build.sh, docs/web-export.md §Lanes)\n`,
    );
  }
  if (lanes) {
    for (const f of LANES_FILES) cpSync(join(engineDir, f), join(out, f), { recursive: true });
  }
  cpSync(vendor, join(out, "basis"), { recursive: true });
  cpSync(cooked, join(out, "cooked"), { recursive: true });
  // The cook's own timings (og cook --report) are not the game's.
  rmSync(join(out, "cooked", ".cook-times.ogt"), { force: true });
  // Streaming groups: a grouped output is read from its group's pack, so its own file is not shipped.
  const split = streamGroupsOf(readFileSync(join(cooked, ".cook.ogt"), "utf8"));
  for (const f of split.grouped) rmSync(join(out, "cooked", f), { force: true });
  const packs = new Set(split.groups.map((g) => `cooked/${g.file}`));

  // Icons: the project's, or the brand tile (shell/brand, from the platform's packages/brand).
  mkdirSync(join(out, "icons"), { recursive: true });
  const icons: Array<BuildManifest["icons"][number]> = [];
  if (projectIcon) {
    writeFileSync(join(out, "icons", "icon.png"), projectIcon.bytes);
    icons.push({ src: "icons/icon.png", sizes: `${projectIcon.size}x${projectIcon.size}`, type: "image/png", source: "project" });
  } else {
    for (const n of [192, 512]) {
      cpSync(join(BRAND_DIR, `icon-${n}.png`), join(out, "icons", `icon-${n}.png`));
      icons.push({ src: `icons/icon-${n}.png`, sizes: `${n}x${n}`, type: "image/png", source: "brand" });
    }
  }
  const icon = icons[icons.length - 1]!.src;

  // What the boot fetches, so its progress is the bytes received.
  const wasmBytes = statSync(join(out, "origin-engine_bg.wasm")).size;
  const firstGroup = walk(join(out, "cooked")).filter((f) => !packs.has(relative(out, f).split("\\").join("/")));
  const cookedBytes = firstGroup.filter((f) => !f.endsWith(".cook.ogt")).reduce((n, f) => n + statSync(f).size, 0);
  const config = {
    engine: ENGINE_VERSION,
    build: publish ? "publish" : "dev",
    name: settings.name,
    profile: settings.profile,
    // project.ogt's text: the engine takes its tick, seed, scenes and scene
    // for what GameDesc leaves unset (CreateOptions.project).
    ...(existsSync(join(project, "project.ogt")) ? { project: readFileSync(join(project, "project.ogt"), "utf8") } : {}),
    isolation: p.isolation,
    fit: p.fit,
    aspect: aspectRatio(p.aspect),
    orientation: p.orientation,
    dprCap: p.dprCap,
    fullscreen: p.fullscreen,
    gate: p.gate,
    madeWith: p.madeWith,
    serviceWorker: publish,
    expect: { wasm: wasmBytes, cooked: cookedBytes, lanes: lanes ? statSync(join(out, "origin-engine-lanes_bg.wasm")).size : 0 },
  };
  writeFileSync(join(out, "index.html"), pageHtml({ settings, publish, config, icon, pwa: publish }));
  writeFileSync(join(out, "_headers"), headersFile(p.isolation));
  if (p.isolation !== "off") writeFileSync(join(out, "origingame.runtime.json"), JSON.stringify({ version: 1, crossOriginIsolated: true }) + "\n");
  if (publish) {
    const webmanifest = {
      id: "./",
      name: settings.name,
      short_name: p.shortName ?? settings.name,
      start_url: "./",
      scope: "./",
      display: "standalone",
      orientation: p.orientation,
      background_color: "#14161b",
      theme_color: "#14161b",
      icons: icons.map((i) => ({ src: i.src, sizes: i.sizes, type: i.type, purpose: "any" })),
    };
    writeFileSync(join(out, "manifest.webmanifest"), JSON.stringify(webmanifest, null, 2) + "\n");
  }

  const fileRows = (): BuildFile[] =>
    walk(out)
      .map((path) => {
        const bytes = new Uint8Array(readFileSync(path));
        return { path: relative(out, path).split("\\").join("/"), bytes: bytes.length, sha256: sha256(bytes) };
      })
      .filter((f) => f.path !== BUILD_MANIFEST)
      .sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  const identity = (rows: BuildFile[]) => sha256(rows.filter((f) => f.path !== "sw.js").map((f) => `${f.path}\0${f.sha256}\n`).join(""));
  let version = identity(fileRows());
  if (publish) {
    const sw = await bundleScript(join(SHELL_DIR, "sw.ts"), true);
    if (typeof sw !== "string") return fail(`bundling sw.js: ${sw.error}`);
    // The first group is precached; a streaming group's pack is cached when the engine fetches it.
    // So is a long sound's chunk (cooked/<sound>.chunk<n>.ogb).
    const precache = fileRows().map((f) => f.path).filter((f) => !NOT_PRECACHED.test(f) && !packs.has(f) && !SOUND_CHUNK.test(f));
    const chunks = fileRows().map((f) => f.path).filter((f) => SOUND_CHUNK.test(f));
    const common = precache.filter((f) => !SINGLE_ENGINE.test(f) && !LANES_ENGINE.test(f));
    const variants = { single: precache.filter((f) => SINGLE_ENGINE.test(f)), lanes: precache.filter((f) => LANES_ENGINE.test(f)) };
    writeFileSync(join(out, "sw.js"), `const OG_SW=${JSON.stringify({ version: version.slice(0, 16), files: ["./", ...common], lazy: [...packs, ...chunks].sort(), ...variants })};\n${sw}`);
    version = identity(fileRows());
  }

  const probe = await probeOf(join(out, "origin-engine.js"), join(out, "origin-engine_bg.wasm"), settings.profile);
  const rows = fileRows();
  const manifest: BuildManifest = {
    format: 1,
    engine: { name: "origin-engine", version: ENGINE_VERSION, pinned: settings.engine },
    build: publish ? "publish" : "dev",
    game: { name: settings.name, id: settings.id, profile: settings.profile },
    entry: "index.html",
    worker: "game.js",
    wasm: "origin-engine_bg.wasm",
    variant,
    lanes: lanes ? { glue: "origin-engine-lanes.js", wasm: "origin-engine-lanes_bg.wasm" } : null,
    probe: { protocol: 5, wire: 1, build: probe?.build ?? (publish ? "publish" : "release"), members: probe?.members ?? [] },
    crossOriginIsolation: p.isolation,
    display: { fit: p.fit, aspect: p.aspect, orientation: p.orientation, dprCap: p.dprCap, fullscreen: p.fullscreen, gate: p.gate, madeWith: p.madeWith },
    icons,
    pwa: publish ? { manifest: "manifest.webmanifest", serviceWorker: "sw.js" } : null,
    player: { protocol: 2, transports: PLAYER_TRANSPORTS },
    version,
    files: rows,
    total: { files: rows.length, bytes: rows.reduce((n, f) => n + f.bytes, 0) },
    stream:
      split.groups.length === 0
        ? null
        : {
            first: { files: firstGroup.length, bytes: firstGroup.reduce((n, f) => n + statSync(f).size, 0) },
            groups: split.groups.map((g) => {
              const row = rows.find((f) => f.path === `cooked/${g.file}`);
              return { name: g.name, file: `cooked/${g.file}`, bytes: row?.bytes ?? 0, sha256: row?.sha256 ?? "", priority: g.priority, memory: g.memory, assets: g.assets };
            }),
          },
  };
  writeFileSync(join(out, BUILD_MANIFEST), JSON.stringify(manifest, null, 2) + "\n");

  const files = walk(out).map((path): SizeRow => {
    const bytes = new Uint8Array(readFileSync(path));
    const c = compressedSize(bytes);
    return { path: relative(out, path), bytes: bytes.length, compressed: c.size, codec: c.codec };
  });
  const total = files.reduce((t, f) => ({ bytes: t.bytes + f.bytes, compressed: t.compressed + f.compressed }), { bytes: 0, compressed: 0 });
  return { ok: true, out, publish, files, total, downloads: downloadsOf(files), probe, manifest };
}

function table(b: WebBuild): string {
  const codec = b.files[0]?.codec ?? "brotli-11";
  const rows = [["file", "bytes", codec], ...b.files.map((f) => [f.path, String(f.bytes), String(f.compressed)]), ["total (every file; not a download)", String(b.total.bytes), String(b.total.compressed)]];
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
    process.stderr.write("usage: bun src/node/web-build.ts --project <dir> --entry <src/main.ts> --out <dist> [--publish] [--json] [--settings <json>]\n");
    process.exit(2);
  }
  const given = opt("--settings");
  let settings: ProjectSettings | undefined;
  if (given) {
    const s = JSON.parse(given) as Omit<ProjectSettings, "publish"> & { publish?: Partial<PublishSettings> };
    settings = { ...s, publish: { ...DEFAULT_PUBLISH, ...s.publish } };
  }
  const b = await webBuild(resolve(project!), resolve(entry!), resolve(out!), argv.includes("--publish"), settings);
  if (!b.ok) {
    if (argv.includes("--json")) process.stdout.write(JSON.stringify(b) + "\n");
    process.stderr.write(`og build: ${b.reason}\n`);
    process.exit(1);
  }
  if (argv.includes("--json")) process.stdout.write(JSON.stringify(b) + "\n");
  else {
    process.stdout.write(table(b));
    if (b.downloads) process.stdout.write(downloadLines(b.downloads, b.files[0]?.codec ?? "brotli-11"));
    if (b.probe) process.stdout.write(`probe: ${b.probe.build} build, ${b.probe.members.length} members: ${b.probe.members.join(", ")}\n`);
    process.stdout.write(`${BUILD_MANIFEST}: ${b.manifest!.build} build of ${b.manifest!.game.name}, engine ${b.manifest!.engine.version}, isolation ${b.manifest!.crossOriginIsolation}, version ${b.manifest!.version.slice(0, 16)}\n`);
  }
}
