#!/usr/bin/env bun
// `og`, the launcher (docs/release.md §1): the command an author installs
// once (install.sh, install.ps1: ~/.origin/bin/og) and runs for every
// engine version. It reads the version a project pins (project.ogt's
// `engine "<v>"`), makes sure that version is in the cache, and runs that
// version's own og with OG_ENGINE_PACKAGE set to it, so two games pinned to
// two versions each run theirs, and a version never changes under a game.
//
//   og <command> [args...]           the pinned version's og (outside a project: the newest installed release,
//                                    else the latest one published)
//   og new <dir> [--engine <v>] [...] scaffold a game with that version's create (default: the latest release)
//   og upgrade [project] [--to <v>] [...]
//                                    move a game to <v> (default: the latest release): that version's og upgrade,
//                                    which edits project.ogt's pin and lists what to change
//   og engine install <v|latest>     download, verify and unpack a version
//   og engine list                   the installed versions
//   og engine prune [--keep <v>]... [--dry-run]
//                                    remove installed versions but the newest release, the current project's pin and --keep
//   og engine path [<v>]             the installed dist of a version (the pinned one by default)
//
// The cache is `$ORIGIN_ENGINE_CACHE` (default ~/.cache/origin-engine), each
// version at engines/<version>/<target>/, the archive's root directory. An
// install downloads <downloads>/<v>/SHA256SUMS and SHA256SUMS.minisig,
// verifies the signature with the engine's public key, downloads the
// target's archive and checks its sha256 against the signed list, unpacks it
// into a temporary directory beside the cache and renames it into place: an
// installed version is complete or absent, two launchers installing at once
// leave one copy, and nothing writes into it after. Its files keep the
// archive's modes rather than being made read-only: every version's own
// tools copy files out of it (create's template, og build's wasm, snippets
// and basis/) and keep the modes they copy, so read-only files would make a
// new game's project.ogt unwritable and a second build fail. `$ORIGIN_ENGINE_DOWNLOADS`
// (default https://origingame.ai/downloads/engine) may be a URL or a local
// directory; `$ORIGIN_ENGINE_PUBKEY` replaces the key for a test channel.
// `$OG_ENGINE_VERSION` runs a version whatever the project pins.
//
// An install reports its progress as events (InstallEvent, docs/release.md
// §1 The launcher's events): JSON Lines on stdout with --json (`og engine
// install --json`, or any command given --json that has to install its
// engine first), JSON Lines on stderr when stderr is not a terminal, and
// prose on a terminal. Downloads retry network failures with backoff
// (ORIGIN_ENGINE_RETRIES, default 5 attempts; ORIGIN_ENGINE_RETRY_MS, default
// 1000, doubling) and resume an interrupted archive with a Range request
// from cache/tmp/; nothing reaches engines/ before its sha256 verifies.
//
// It needs bun (the engine runs on it) and tar (Windows 10 and later carry
// tar.exe). Two files: og.ts and the minisign verifier it shares with the
// engine (packages/engine/src/node/minisign.ts); install.sh and install.ps1
// carry both (packages/launcher/build.ts).

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { closeSync, existsSync, lstatSync, mkdirSync, mkdtempSync, openSync, readdirSync, readFileSync, readlinkSync, readSync, renameSync, rmSync, statSync, symlinkSync, writeFileSync, writeSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { DOWNLOADS, ENGINE_PUBKEY, MinisignError, verifyMinisign } from "../engine/src/node/minisign.ts";

export const LAUNCHER_VERSION = "2";
/**
 * An engine version: a release `x.y.z`, a dev build `dev.<YYYYMMDD>.<n>`
 * (the n-th of that UTC day, its own number unrelated to the release it is
 * based on), or a dev build of the scheme before 0.9.0, `x.y.z-dev.n`.
 */
export const VERSION_RE = /^(?:\d+\.\d+\.\d+(?:-dev\.\d+)?|dev\.\d{8}\.[1-9]\d*)$/;
/** Whether `v` is a dev build (either scheme), never what latest/ or a release pin moves to by itself. */
export const isDev = (v: string) => v.startsWith("dev.") || v.includes("-dev.");
const TARGETS: Record<string, string> = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};

export interface Config {
  cache: string;
  downloads: string;
  pubkey: string;
  target: string;
}

export function config(env = process.env): Config {
  const target = TARGETS[`${process.platform}-${process.arch}`];
  if (!target) fail(`the engine ships no build for ${process.platform}-${process.arch} (it ships ${Object.keys(TARGETS).join(", ")})`, "unsupported_target");
  return {
    cache: resolve(env.ORIGIN_ENGINE_CACHE || join(homedir(), ".cache", "origin-engine")),
    downloads: (env.ORIGIN_ENGINE_DOWNLOADS || DOWNLOADS).replace(/\/+$/, ""),
    pubkey: env.ORIGIN_ENGINE_PUBKEY || ENGINE_PUBKEY,
    target,
  };
}

/**
 * Why a launcher command failed: stable codes a tool branches on. `retryable`
 * says whether running the same command again may succeed.
 */
export type ErrorCode =
  | "network" // the channel did not answer, or answered an HTTP error other than 404
  | "not_found" // no such version (or no archive for this target)
  | "checksum" // the archive does not match the signed SHA256SUMS
  | "signature" // SHA256SUMS is not signed by the engine's key
  | "disk_full" // no space left (or a quota) in the cache
  | "unpack" // the archive does not unpack into this version's dist
  | "no_pin" // the project pins no engine version, or not a version
  | "usage" // the command or a version string is wrong
  | "unsupported_target" // the engine ships no build for this platform
  | "internal"; // anything else: the launcher's own bug
const RETRYABLE: ReadonlySet<ErrorCode> = new Set(["network", "checksum"]);

export class LauncherError extends Error {
  constructor(message: string, readonly code: ErrorCode = "usage", readonly retryable = RETRYABLE.has(code)) {
    super(message);
  }
}
function fail(why: string, code: ErrorCode = "usage", retryable?: boolean): never {
  throw new LauncherError(why, code, retryable);
}
const note = (s: string) => process.stderr.write(`og: ${s}\n`);

/** An install's progress, in order: resolve, download (repeated) with retries, verify, unpack, installed; or error. */
export type InstallEvent =
  | { event: "resolve"; version: string; target: string }
  | { event: "download"; bytes: number; total: number | null }
  | { event: "retry"; attempt: number; max: number; reason: string; delayMs: number }
  | { event: "verify" }
  | { event: "unpack" }
  | { event: "installed"; version: string; path: string; cached: boolean }
  | { event: "error"; code: ErrorCode; message: string; retryable: boolean };
export type Report = (e: InstallEvent) => void;
export const silent: Report = () => {};

/** Events as JSON Lines on a descriptor (1 stdout, 2 stderr), written synchronously so none is lost at exit. */
export const jsonLines = (fd: number): Report => (e) => {
  writeSync(fd, `${JSON.stringify(e)}\n`);
};

/** Events as prose on stderr, the download as a percentage redrawn in place on a terminal. */
function prose(): Report {
  let version = "";
  let started = false;
  const tty = Boolean(process.stderr.isTTY);
  const mb = (n: number) => (n / 1e6).toFixed(1);
  return (e) => {
    if (e.event === "resolve") version = e.version;
    else if (e.event === "download") {
      if (!started) note(`downloading engine ${version} for ${config().target}`);
      started = true;
      if (tty && e.total) process.stderr.write(`\rog: ${Math.floor((e.bytes / e.total) * 100)}% (${mb(e.bytes)} of ${mb(e.total)} MB)${e.bytes === e.total ? "\n" : ""}`);
    } else if (e.event === "retry") note(`${e.reason}; retrying in ${Math.round(e.delayMs / 100) / 10} s (attempt ${e.attempt} of ${e.max})`);
    else if (e.event === "installed" && !e.cached) note(`installed engine ${e.version} in ${e.path}`);
    else if (e.event === "error") note(e.message);
  };
}

/** Where a command's install events go (InstallEvent): stdout with --json, else prose on a terminal and JSON Lines on any other stderr. */
export function reporterFor(argv: string[], stderrIsTTY = Boolean(process.stderr.isTTY)): { report: Report; stdout: boolean } {
  if (argv.includes("--json")) return { report: jsonLines(1), stdout: true };
  if (argv[0] === "engine" || stderrIsTTY) return { report: prose(), stdout: false };
  return { report: jsonLines(2), stdout: false };
}

// ---------------------------------------------------------------- downloads

const env = (name: string, fallback: number) => {
  const n = Number(process.env[name]);
  return Number.isFinite(n) && process.env[name] !== "" && process.env[name] !== undefined ? n : fallback;
};
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const isUrl = (base: string) => /^https?:\/\//.test(base);

/** An HTTP answer that is not a success, as its error: 404 not_found; 408, 429 and 5xx retryable network; the rest network. */
function httpError(url: string, r: Response): LauncherError {
  const why = `${url}: ${r.status} ${r.statusText}`;
  if (r.status === 404) return new LauncherError(why, "not_found");
  return new LauncherError(why, "network", r.status === 408 || r.status === 429 || r.status >= 500);
}

/** A thrown value as the launcher's error: a refused connection or a reset stream is a retryable network error, a full disk disk_full. */
function asLauncherError(e: unknown): LauncherError {
  if (e instanceof LauncherError) return e;
  if (e instanceof MinisignError) return new LauncherError(e.message, "signature");
  const err = e as { code?: string; name?: string; message?: string };
  if (err?.code === "ENOSPC" || err?.code === "EDQUOT") return new LauncherError(`no space left in the engine cache: ${err.message}`, "disk_full");
  if (err?.name === "AbortError" || err?.name === "TimeoutError" || err instanceof TypeError || /^(ECONN|ENOTFOUND|EAI_|ETIMEDOUT|EPIPE|ENETUNREACH|EHOSTUNREACH)/.test(err?.code ?? ""))
    return new LauncherError(`network: ${err?.message ?? String(e)}`, "network", true);
  return new LauncherError(err?.message ?? String(e), "internal");
}

/** Runs `f` until it succeeds, retrying a retryable error with doubling delays and reporting each retry. */
async function withRetry<T>(report: Report, f: () => Promise<T>): Promise<T> {
  const max = Math.max(1, env("ORIGIN_ENGINE_RETRIES", 5));
  for (let attempt = 1; ; attempt++) {
    try {
      return await f();
    } catch (e) {
      const err = asLauncherError(e);
      if (!err.retryable || attempt >= max) throw err;
      const delayMs = Math.min(30_000, env("ORIGIN_ENGINE_RETRY_MS", 1000) * 2 ** (attempt - 1));
      report({ event: "retry", attempt: attempt + 1, max, reason: err.message, delayMs });
      await sleep(delayMs);
    }
  }
}

/** A fetch that is aborted once no bytes arrive for ORIGIN_ENGINE_IDLE_MS (default 30 s): a stalled connection is retried, not waited on. */
async function fetchIdle(url: string, init: RequestInit = {}): Promise<{ response: Response; body: AsyncIterable<Uint8Array>; done: () => void }> {
  const idle = env("ORIGIN_ENGINE_IDLE_MS", 30_000);
  const abort = new AbortController();
  let timer = setTimeout(() => abort.abort(), idle);
  const poke = () => {
    clearTimeout(timer);
    timer = setTimeout(() => abort.abort(), idle);
  };
  const response = await fetch(url, { ...init, signal: abort.signal });
  poke();
  const reader = response.body?.getReader();
  const body = {
    async *[Symbol.asyncIterator]() {
      if (!reader) return;
      for (;;) {
        const { value, done } = await reader.read();
        if (done) return;
        poke();
        yield value;
      }
    },
  };
  return { response, body, done: () => clearTimeout(timer) };
}

async function fetchBytes(c: Config, path: string, report: Report = silent): Promise<Uint8Array> {
  const base = c.downloads;
  if (!isUrl(base)) {
    const file = join(base.replace(/^file:\/\//, ""), path);
    if (!existsSync(file)) fail(`${file}: no such file`, "not_found");
    return new Uint8Array(readFileSync(file));
  }
  return withRetry(report, async () => {
    const url = `${base}/${path}`;
    const { response, body, done } = await fetchIdle(url);
    try {
      if (!response.ok) throw httpError(url, response);
      const parts: Uint8Array[] = [];
      for await (const chunk of body) parts.push(chunk);
      return new Uint8Array(Buffer.concat(parts));
    } finally {
      done();
    }
  });
}

/**
 * Downloads `path` into `part`, resuming from what `part` already holds with
 * a Range request (a server that ignores it sends the whole file, which
 * replaces the part). Reports the bytes every 1 % or 500 ms, and once at
 * the end. A failure leaves the part for the next attempt.
 */
async function download(c: Config, path: string, part: string, report: Report): Promise<void> {
  const base = c.downloads;
  if (!isUrl(base)) {
    const file = join(base.replace(/^file:\/\//, ""), path);
    if (!existsSync(file)) fail(`${file}: no such file`, "not_found");
    const bytes = readFileSync(file);
    writeFileSync(part, bytes);
    report({ event: "download", bytes: bytes.length, total: bytes.length });
    return;
  }
  const url = `${base}/${path}`;
  let have = existsSync(part) ? statSync(part).size : 0;
  const { response, body, done } = await fetchIdle(url, have > 0 ? { headers: { Range: `bytes=${have}-` } } : {});
  let fd: number | undefined;
  try {
    if (response.status === 416) {
      // The part is not a prefix of this file (larger, or of another upload): start over.
      rmSync(part, { force: true });
      fail(`${url}: the partial download does not fit the file; starting over`, "network", true);
    }
    if (!response.ok) throw httpError(url, response);
    let total: number | null;
    const range = /^bytes (\d+)-\d+\/(\d+|\*)$/.exec(response.headers.get("content-range") ?? "");
    if (response.status === 206 && range && Number(range[1]) === have) {
      total = range[2] === "*" ? null : Number(range[2]);
      fd = openSync(part, "a");
    } else {
      have = 0;
      const length = Number(response.headers.get("content-length"));
      total = Number.isFinite(length) && length > 0 ? length : null;
      fd = openSync(part, "w");
    }
    let shownBytes = -1;
    let shownAt = 0;
    const show = (final: boolean) => {
      const now = Date.now();
      const step = total ? total / 100 : Number.POSITIVE_INFINITY;
      if (final || shownBytes < 0 || have - shownBytes >= step || now - shownAt >= 500) {
        report({ event: "download", bytes: have, total });
        shownBytes = have;
        shownAt = now;
      }
    };
    show(false);
    for await (const chunk of body) {
      writeSync(fd, chunk);
      have += chunk.length;
      show(false);
    }
    if (total !== null && have !== total) fail(`${url}: the connection ended at ${have} of ${total} bytes`, "network", true);
    if (shownBytes !== have) show(true);
  } finally {
    done();
    if (fd !== undefined) closeSync(fd);
  }
}

/** A file's sha256, read in chunks. */
function sha256File(file: string): string {
  const hash = createHash("sha256");
  const fd = openSync(file, "r");
  try {
    const buf = Buffer.alloc(1 << 20);
    for (let n; (n = readSync(fd, buf, 0, buf.length, null)) > 0; ) hash.update(buf.subarray(0, n));
  } finally {
    closeSync(fd);
  }
  return hash.digest("hex");
}

/** The newest published release (latest/manifest.json). */
export async function latest(c: Config, report: Report = silent): Promise<string> {
  const m = JSON.parse(new TextDecoder().decode(await fetchBytes(c, "latest/manifest.json", report))) as { version?: string };
  if (!m.version || !VERSION_RE.test(m.version) || isDev(m.version)) fail(`${c.downloads}/latest/manifest.json names no release`, "not_found");
  return m.version;
}

// ---------------------------------------------------------------- the cache

export const distDir = (c: Config, v: string) => join(c.cache, "engines", v, c.target);
const ogExe = (dist: string) => join(dist, "bin", process.platform === "win32" ? "og.exe" : "og");
const isInstalled = (c: Config, v: string) => existsSync(join(distDir(c, v), "origin-engine.json"));

/**
 * Holds `lock` (a directory naming its owner's pid) while `f` runs: one
 * download of a version at a time, so two launchers never write one part.
 * A lock whose owner is gone is taken over.
 */
async function locked<T>(lock: string, f: () => Promise<T>): Promise<T> {
  for (;;) {
    try {
      mkdirSync(lock);
      writeFileSync(join(lock, "pid"), String(process.pid));
      break;
    } catch (e) {
      if ((e as { code?: string }).code !== "EEXIST") throw e;
      const owner = Number(existsSync(join(lock, "pid")) ? readFileSync(join(lock, "pid"), "utf8") : NaN);
      let alive = true;
      try {
        if (owner > 0) process.kill(owner, 0);
      } catch {
        alive = false;
      }
      if (!alive) rmSync(lock, { recursive: true, force: true });
      else await sleep(200);
    }
  }
  try {
    return await f();
  } finally {
    rmSync(lock, { recursive: true, force: true });
  }
}

/** Installs version `v` unless it is installed, reporting its progress; returns its dist directory. */
export async function install(c: Config, v: string, report: Report = silent): Promise<string> {
  if (!VERSION_RE.test(v)) fail(`${v} is not an engine version (x.y.z, or a dev build dev.YYYYMMDD.n)`);
  const dist = distDir(c, v);
  report({ event: "resolve", version: v, target: c.target });
  if (isInstalled(c, v)) {
    report({ event: "installed", version: v, path: dist, cached: true });
    return dist;
  }
  const tmpRoot = join(c.cache, "tmp");
  try {
    mkdirSync(tmpRoot, { recursive: true });
    const name = `origin-engine-${v}-${c.target}`;
    await locked(join(tmpRoot, `${name}.lock`), async () => {
      if (isInstalled(c, v)) return;
      const sums = await fetchBytes(c, `${v}/SHA256SUMS`, report);
      const sig = new TextDecoder().decode(await fetchBytes(c, `${v}/SHA256SUMS.minisig`, report));
      verifyMinisign(sums, sig, c.pubkey);
      const line = new TextDecoder().decode(sums).split("\n").find((l) => l.trim().endsWith(`  ${name}.tar.gz`));
      if (!line) fail(`engine ${v} has no ${c.target} archive (SHA256SUMS lists none)`, "not_found");
      const want = line.trim().split(/\s+/)[0]!;
      const part = join(tmpRoot, `${name}.tar.gz.part`);
      // A download and its check are one attempt: a part that does not match is deleted and fetched again.
      await withRetry(report, async () => {
        await download(c, `${v}/${name}.tar.gz`, part, report);
        report({ event: "verify" });
        const got = sha256File(part);
        if (got !== want) {
          rmSync(part, { force: true });
          fail(`${name}.tar.gz is ${got}; the signed SHA256SUMS says ${want}`, "checksum");
        }
      });
      report({ event: "unpack" });
      const tmp = mkdtempSync(join(tmpRoot, `${v}-`));
      try {
        const r = spawnSync("tar", ["-xzf", part], { cwd: tmp, encoding: "utf8" });
        if (r.error || r.status !== 0) {
          const why = r.error?.message ?? r.stderr.trim();
          fail(`unpacking ${name}.tar.gz failed: ${why}`, /no space left|disk quota/i.test(why) ? "disk_full" : "unpack");
        }
        const root = join(tmp, name);
        const build = (JSON.parse(readFileSync(join(root, "origin-engine.json"), "utf8")) as { build?: { version?: string } }).build;
        if (build?.version !== v) fail(`${name}.tar.gz holds engine ${build?.version}, not ${v}`, "unpack");
        if (!existsSync(ogExe(root))) fail(`${name}.tar.gz has no ${ogExe(root).slice(root.length + 1)}`, "unpack");
        mkdirSync(dirname(dist), { recursive: true });
        try {
          renameSync(root, dist);
        } catch (e) {
          // Another launcher installed it meanwhile: keep that one.
          if (!isInstalled(c, v)) throw e;
        }
      } finally {
        rmSync(tmp, { recursive: true, force: true });
      }
      rmSync(part, { force: true });
    });
  } catch (e) {
    throw asLauncherError(e);
  }
  report({ event: "installed", version: v, path: dist, cached: false });
  return dist;
}

/** The release an installed dev build `dev.<date>.<n>` is based on (its build record's `base`). */
export function baseOf(c: Config, v: string): string | undefined {
  try {
    return (JSON.parse(readFileSync(join(distDir(c, v), "origin-engine.json"), "utf8")) as { build?: { base?: string } }).build?.base;
  } catch {
    return undefined;
  }
}

/** The installed versions, newest first. */
export function installed(c: Config): string[] {
  const dir = join(c.cache, "engines");
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((v) => VERSION_RE.test(v) && isInstalled(c, v))
    .sort((a, b) => compareVersions(a, b, (v) => baseOf(c, v)))
    .reverse();
}

/**
 * Orders versions. A dev build `dev.<date>.<n>` sorts after the release it
 * is based on (`base`) and before the next one, by date then n; its own
 * numbers never compare with a release's, and one whose base is unknown
 * sorts before every release. A dev build `x.y.z-dev.n` sorts before the
 * release of its numbers.
 */
export function compareVersions(a: string, b: string, base: (v: string) => string | undefined = () => undefined): number {
  const key = (v: string): number[] => {
    const dated = /^dev\.(\d{8})\.(\d+)$/.exec(v);
    if (dated) {
      const on = base(v);
      const release = on && /^\d+\.\d+\.\d+$/.test(on) ? on.split(".").map(Number) : [-1, -1, -1];
      return [...release, 2, Number(dated[1]), Number(dated[2])];
    }
    const [core, dev] = v.split("-dev.");
    return [...core!.split(".").map(Number), dev === undefined ? 1 : 0, dev === undefined ? 0 : Number(dev), 0];
  };
  const x = key(a);
  const y = key(b);
  for (let i = 0; i < x.length; i++) if (x[i] !== y[i]) return x[i]! < y[i]! ? -1 : 1;
  return 0;
}

// ---------------------------------------------------------------- projects

/** The project root (the directory holding project.ogt) at or above `from`. */
export function projectRoot(from: string): string | undefined {
  let dir = resolve(from);
  if (existsSync(dir) && statSync(dir).isFile()) dir = dirname(dir);
  for (;;) {
    if (existsSync(join(dir, "project.ogt"))) return dir;
    const up = dirname(dir);
    if (up === dir) return undefined;
    dir = up;
  }
}

/** project.ogt's `engine "<v>"`. */
export function pinOf(root: string): string | undefined {
  return /^engine\s+"([^"]+)"\s*$/m.exec(readFileSync(join(root, "project.ogt"), "utf8"))?.[1];
}

/** The project a command is about: the first argument naming a path inside one, else the working directory's. */
export function projectOf(args: string[], cwd = process.cwd()): string | undefined {
  for (const a of args) {
    if (a.startsWith("-")) continue;
    const p = isAbsolute(a) ? a : join(cwd, a);
    if (existsSync(p)) {
      const root = projectRoot(p);
      if (root) return root;
    }
  }
  return projectRoot(cwd);
}

/**
 * Points `<project>/.og/engine` at a dist's engine/ (a symlink; a junction on
 * Windows): what the project's tsconfig paths and an editor read. The
 * version's own og keeps it too (from 0.7.8); this covers the versions
 * before.
 */
export function linkEngine(root: string, dist: string) {
  const link = join(root, ".og", "engine");
  const target = join(dist, "engine");
  try {
    if (lstatSync(link).isSymbolicLink() && resolve(dirname(link), readlinkSync(link)) === resolve(target)) return;
    if (!lstatSync(link).isSymbolicLink()) return; // a directory og dev made in an engine checkout: not ours
    rmSync(link, { force: true });
  } catch {}
  mkdirSync(dirname(link), { recursive: true });
  symlinkSync(target, link, process.platform === "win32" ? "junction" : "dir");
}

// ---------------------------------------------------------------- commands

function exec(dist: string, args: string[], cwd = process.cwd()): number {
  const r = spawnSync(ogExe(dist), args, { cwd, stdio: "inherit", env: { ...process.env, OG_ENGINE_PACKAGE: join(dist, "engine") } });
  if (r.error) fail(`cannot start ${ogExe(dist)} (${r.error.message})`, "internal");
  return r.status ?? 1;
}

const optValue = (args: string[], name: string): string | undefined => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const without = (args: string[], name: string): string[] => {
  const i = args.indexOf(name);
  return i < 0 ? args : [...args.slice(0, i), ...args.slice(i + 2)];
};

async function resolveVersion(c: Config, v: string, report: Report = silent): Promise<string> {
  return v === "latest" ? latest(c, report) : v;
}

/** The version a command runs: OG_ENGINE_VERSION, the project's pin, the newest installed release, the latest published. */
async function versionFor(c: Config, root: string | undefined, report: Report = silent): Promise<string> {
  const forced = process.env.OG_ENGINE_VERSION;
  if (forced) return forced;
  if (root) {
    const pin = pinOf(root);
    if (!pin) fail(`${join(root, "project.ogt")} pins no engine (an \`engine "<version>"\` line)`, "no_pin");
    if (!VERSION_RE.test(pin)) fail(`${join(root, "project.ogt")} pins engine "${pin}", which is not a version (x.y.z, or a dev build dev.YYYYMMDD.n)`, "no_pin");
    return pin;
  }
  const releases = installed(c).filter((v) => !isDev(v));
  return releases[0] ?? (await latest(c, report));
}

async function engineCommand(c: Config, args: string[], out: { report: Report; stdout: boolean }): Promise<number> {
  const [sub, ...rest] = args;
  switch (sub) {
    case "install": {
      // A version, else (in a project) the version it runs: its pin.
      const given = rest.find((a) => !a.startsWith("-"));
      const root = projectOf([]);
      if (!given && !root) fail("usage: og engine install [<version>|latest] [--json] (the version, or run it in a project to install its pin)");
      const v = given ? await resolveVersion(c, given, out.report) : await versionFor(c, root, out.report);
      const was = isInstalled(c, v);
      const dist = await install(c, v, out.report);
      if (was && !out.stdout) process.stdout.write(`engine ${v} is installed: ${dist}\n`);
      return 0;
    }
    case "list": {
      const root = projectOf([]);
      const pin = root ? pinOf(root) : undefined;
      for (const v of installed(c)) process.stdout.write(`${v}${v === pin ? "  (this project's pin)" : ""}  ${distDir(c, v)}\n`);
      return 0;
    }
    case "path": {
      const v = rest[0] ?? (await versionFor(c, projectOf([])));
      if (!isInstalled(c, v)) fail(`engine ${v} is not installed (og engine install ${v})`);
      process.stdout.write(`${distDir(c, v)}\n`);
      return 0;
    }
    case "prune": {
      const keep = new Set<string>();
      for (let i = 0; i < rest.length; i++) if (rest[i] === "--keep" && rest[i + 1]) keep.add(rest[++i]!);
      const dry = rest.includes("--dry-run");
      const all = installed(c);
      const newest = all.find((v) => !isDev(v));
      if (newest) keep.add(newest);
      const root = projectOf([]);
      const pin = root ? pinOf(root) : undefined;
      if (pin) keep.add(pin);
      for (const v of all) {
        if (keep.has(v)) continue;
        process.stdout.write(`${dry ? "would remove" : "removing"} engine ${v}\n`);
        if (!dry) {
          rmSync(distDir(c, v), { recursive: true, force: true });
          const dir = join(c.cache, "engines", v);
          if (existsSync(dir) && readdirSync(dir).length === 0) rmSync(dir, { recursive: true });
        }
      }
      process.stdout.write(`kept ${[...keep].filter((v) => all.includes(v)).join(", ") || "nothing"}\n`);
      return 0;
    }
    default:
      fail("usage: og engine install [<version>|latest] [--json] | list | path [<version>] | prune [--keep <version>]... [--dry-run]");
  }
}

/** create's options that take a value (packages/create/bin/create.ts). */
const CREATE_VALUES = new Set(["--name", "--id", "--profile", "--archetype", "--premise", "--look"]);

export async function main(argv: string[]): Promise<number> {
  const out = reporterFor(argv);
  try {
    return await command(argv, out);
  } catch (e) {
    const err = asLauncherError(e);
    out.report({ event: "error", code: err.code, message: err.message, retryable: err.retryable });
    // The prose too, for a reader of stderr when the events went to stdout.
    if (out.stdout) note(err.message);
    if (err.code === "internal" && e instanceof Error && e.stack) process.stderr.write(`${e.stack}\n`);
    return 1;
  }
}

async function command(argv: string[], out: { report: Report; stdout: boolean }): Promise<number> {
  const c = config();
  const [cmd, ...rest] = argv;
  if (cmd === "engine") return engineCommand(c, rest, out);
  if (cmd === "--launcher-version") {
    process.stdout.write(`og launcher ${LAUNCHER_VERSION}\n`);
    return 0;
  }
  if (cmd === "new") {
    const v = await resolveVersion(c, optValue(rest, "--engine") ?? process.env.OG_ENGINE_VERSION ?? "latest", out.report);
    const dist = await install(c, v, out.report);
    const args = without(rest, "--engine");
    const create = join(dist, "create", "bin", "create.ts");
    const r = spawnSync("bun", [create, ...args], { stdio: "inherit" });
    if (r.error) fail(`cannot start bun (${r.error.message}); the engine runs on bun: https://bun.sh`, "internal");
    if (r.status !== 0) return r.status ?? 1;
    const dir = args.find((a, i) => !a.startsWith("-") && !CREATE_VALUES.has(args[i - 1] ?? ""));
    const root = dir ? projectRoot(resolve(dir)) : undefined;
    if (root) linkEngine(root, dist);
    return 0;
  }
  if (cmd === "upgrade") {
    // The target version's og does the upgrade: its changelog knows every version up to it.
    const v = await resolveVersion(c, optValue(rest, "--to") ?? "latest", out.report);
    const dist = await install(c, v, out.report);
    const status = exec(dist, ["upgrade", ...without(rest, "--to"), "--to", v]);
    const root = projectOf(rest);
    if ((status === 0 || status === 3) && root && pinOf(root) === v) linkEngine(root, dist);
    return status;
  }
  const root = projectOf(rest);
  const v = await versionFor(c, root, out.report);
  const dist = await install(c, v, out.report);
  if (root) linkEngine(root, dist);
  return exec(dist, argv);
}

if (import.meta.main) process.exit(await main(process.argv.slice(2)));
