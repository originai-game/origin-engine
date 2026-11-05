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
// It needs bun (the engine runs on it) and tar (Windows 10 and later carry
// tar.exe). One file with no dependencies: install.sh and install.ps1 carry
// it whole (packages/launcher/build.ts).

import { spawnSync } from "node:child_process";
import { createHash, createPublicKey, verify } from "node:crypto";
import { existsSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, readlinkSync, renameSync, rmSync, statSync, symlinkSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, resolve } from "node:path";

export const LAUNCHER_VERSION = "1";
/** The engine's release key (docs/release.md §1; the owner, 2026-11-05). */
export const ENGINE_PUBKEY = "RWSJfZkSrBHM9kOnWQNC0WoGXEFI+9NMCj8+Wy6WJDkYl9pgPGkoCoxt";
export const DOWNLOADS = "https://origingame.ai/downloads/engine";
export const VERSION_RE = /^\d+\.\d+\.\d+(-dev\.\d+)?$/;
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
  if (!target) fail(`the engine ships no build for ${process.platform}-${process.arch} (it ships ${Object.keys(TARGETS).join(", ")})`);
  return {
    cache: resolve(env.ORIGIN_ENGINE_CACHE || join(homedir(), ".cache", "origin-engine")),
    downloads: (env.ORIGIN_ENGINE_DOWNLOADS || DOWNLOADS).replace(/\/+$/, ""),
    pubkey: env.ORIGIN_ENGINE_PUBKEY || ENGINE_PUBKEY,
    target,
  };
}

export class LauncherError extends Error {}
function fail(why: string): never {
  throw new LauncherError(why);
}
const note = (s: string) => process.stderr.write(`og: ${s}\n`);

// ---------------------------------------------------------------- minisign

const ED25519_SPKI = Buffer.from("302a300506032b6570032100", "hex");

/**
 * Verifies a minisign signature (`minisign -S`'s .minisig: the prehashed
 * `ED` algorithm, BLAKE2b-512 of the file, or the legacy `Ed`) of `data`
 * with a public key line, including the trusted comment's global
 * signature. Returns the trusted comment; throws on any mismatch.
 */
export function verifyMinisign(data: Uint8Array, signature: string, pubkey: string): string {
  const pk = Buffer.from(pubkey.trim(), "base64");
  if (pk.length !== 42 || pk.toString("latin1", 0, 2) !== "Ed") fail("the engine's public key is not a minisign Ed25519 key");
  const lines = signature.split(/\r?\n/);
  const sig = Buffer.from(lines[1] ?? "", "base64");
  const trusted = lines[2] ?? "";
  const global = Buffer.from(lines[3] ?? "", "base64");
  if (sig.length !== 74 || !trusted.startsWith("trusted comment: ") || global.length !== 64) fail("SHA256SUMS.minisig is not a minisign signature");
  const alg = sig.toString("latin1", 0, 2);
  if (alg !== "ED" && alg !== "Ed") fail(`SHA256SUMS.minisig uses the unknown algorithm ${alg}`);
  if (!sig.subarray(2, 10).equals(pk.subarray(2, 10))) fail(`SHA256SUMS.minisig is signed by key ${keyId(sig.subarray(2, 10))}, not the engine's ${keyId(pk.subarray(2, 10))}`);
  const key = createPublicKey({ key: Buffer.concat([ED25519_SPKI, pk.subarray(10)]), format: "der", type: "spki" });
  const message = alg === "ED" ? createHash("blake2b512").update(data).digest() : Buffer.from(data);
  if (!verify(null, message, key, sig.subarray(10))) fail("SHA256SUMS.minisig does not verify: the file was changed or not signed by the engine's key");
  const comment = trusted.slice("trusted comment: ".length);
  if (!verify(null, Buffer.concat([sig.subarray(10), Buffer.from(comment, "utf8")]), key, global)) fail("SHA256SUMS.minisig's trusted comment does not verify");
  return comment;
}
const keyId = (b: Uint8Array) => Buffer.from(b).reverse().toString("hex").toUpperCase();

// ---------------------------------------------------------------- downloads

async function fetchBytes(c: Config, path: string): Promise<Uint8Array> {
  const base = c.downloads;
  if (/^https?:\/\//.test(base)) {
    const r = await fetch(`${base}/${path}`);
    if (!r.ok) fail(`${base}/${path}: ${r.status} ${r.statusText}`);
    return new Uint8Array(await r.arrayBuffer());
  }
  const file = join(base.replace(/^file:\/\//, ""), path);
  if (!existsSync(file)) fail(`${file}: no such file`);
  return new Uint8Array(readFileSync(file));
}

/** The newest published release (latest/manifest.json). */
export async function latest(c: Config): Promise<string> {
  const m = JSON.parse(new TextDecoder().decode(await fetchBytes(c, "latest/manifest.json"))) as { version?: string };
  if (!m.version || !VERSION_RE.test(m.version)) fail(`${c.downloads}/latest/manifest.json names no version`);
  return m.version;
}

// ---------------------------------------------------------------- the cache

export const distDir = (c: Config, v: string) => join(c.cache, "engines", v, c.target);
const ogExe = (dist: string) => join(dist, "bin", process.platform === "win32" ? "og.exe" : "og");
const isInstalled = (c: Config, v: string) => existsSync(join(distDir(c, v), "origin-engine.json"));

/** Installs version `v` unless it is installed; returns its dist directory. */
export async function install(c: Config, v: string, quiet = false): Promise<string> {
  if (!VERSION_RE.test(v)) fail(`${v} is not an engine version (x.y.z or x.y.z-dev.n)`);
  const dist = distDir(c, v);
  if (isInstalled(c, v)) return dist;
  const name = `origin-engine-${v}-${c.target}`;
  const sums = await fetchBytes(c, `${v}/SHA256SUMS`);
  const sig = new TextDecoder().decode(await fetchBytes(c, `${v}/SHA256SUMS.minisig`));
  verifyMinisign(sums, sig, c.pubkey);
  const line = new TextDecoder().decode(sums).split("\n").find((l) => l.trim().endsWith(`  ${name}.tar.gz`));
  if (!line) fail(`engine ${v} has no ${c.target} archive (SHA256SUMS lists none)`);
  const want = line.trim().split(/\s+/)[0]!;
  if (!quiet) note(`downloading engine ${v} for ${c.target}`);
  const archive = await fetchBytes(c, `${v}/${name}.tar.gz`);
  const got = createHash("sha256").update(archive).digest("hex");
  if (got !== want) fail(`${name}.tar.gz is ${got}; the signed SHA256SUMS says ${want}`);
  const tmpRoot = join(c.cache, "tmp");
  mkdirSync(tmpRoot, { recursive: true });
  const tmp = mkdtempSync(join(tmpRoot, `${v}-`));
  try {
    writeFileSync(join(tmp, "archive.tar.gz"), archive);
    const r = spawnSync("tar", ["-xzf", "archive.tar.gz"], { cwd: tmp, encoding: "utf8" });
    if (r.error || r.status !== 0) fail(`unpacking ${name}.tar.gz failed: ${r.error?.message ?? r.stderr.trim()}`);
    const root = join(tmp, name);
    const build = (JSON.parse(readFileSync(join(root, "origin-engine.json"), "utf8")) as { build?: { version?: string } }).build;
    if (build?.version !== v) fail(`${name}.tar.gz holds engine ${build?.version}, not ${v}`);
    if (!existsSync(ogExe(root))) fail(`${name}.tar.gz has no ${ogExe(root).slice(root.length + 1)}`);
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
  if (!quiet) note(`installed engine ${v} in ${dist}`);
  return dist;
}

/** The installed versions, newest first. */
export function installed(c: Config): string[] {
  const dir = join(c.cache, "engines");
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter((v) => VERSION_RE.test(v) && isInstalled(c, v)).sort(compareVersions).reverse();
}

/** Orders versions; a dev build sorts before the release of its numbers. */
export function compareVersions(a: string, b: string): number {
  const parse = (v: string) => {
    const [core, dev] = v.split("-dev.");
    return [...core!.split(".").map(Number), dev === undefined ? Number.POSITIVE_INFINITY : Number(dev)];
  };
  const x = parse(a);
  const y = parse(b);
  for (let i = 0; i < 4; i++) if (x[i] !== y[i]) return x[i]! < y[i]! ? -1 : 1;
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
  if (r.error) fail(`cannot start ${ogExe(dist)} (${r.error.message})`);
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

async function resolveVersion(c: Config, v: string): Promise<string> {
  return v === "latest" ? latest(c) : v;
}

/** The version a command runs: OG_ENGINE_VERSION, the project's pin, the newest installed release, the latest published. */
async function versionFor(c: Config, root: string | undefined): Promise<string> {
  const forced = process.env.OG_ENGINE_VERSION;
  if (forced) return forced;
  if (root) {
    const pin = pinOf(root);
    if (!pin) fail(`${join(root, "project.ogt")} pins no engine (an \`engine "<version>"\` line)`);
    if (!VERSION_RE.test(pin)) fail(`${join(root, "project.ogt")} pins engine "${pin}", which is not a version (x.y.z)`);
    return pin;
  }
  const releases = installed(c).filter((v) => !v.includes("-dev."));
  return releases[0] ?? (await latest(c));
}

async function engineCommand(c: Config, args: string[]): Promise<number> {
  const [sub, ...rest] = args;
  switch (sub) {
    case "install": {
      if (!rest[0]) fail("usage: og engine install <version|latest>");
      const v = await resolveVersion(c, rest[0]);
      const was = isInstalled(c, v);
      const dist = await install(c, v);
      if (was) process.stdout.write(`engine ${v} is installed: ${dist}\n`);
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
      const newest = all.find((v) => !v.includes("-dev."));
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
      fail("usage: og engine install <version|latest> | list | path [<version>] | prune [--keep <version>]... [--dry-run]");
  }
}

/** create's options that take a value (packages/create/bin/create.ts). */
const CREATE_VALUES = new Set(["--name", "--id", "--profile", "--archetype", "--premise", "--look"]);

export async function main(argv: string[]): Promise<number> {
  const c = config();
  const [cmd, ...rest] = argv;
  if (cmd === "engine") return engineCommand(c, rest);
  if (cmd === "--launcher-version") {
    process.stdout.write(`og launcher ${LAUNCHER_VERSION}\n`);
    return 0;
  }
  if (cmd === "new") {
    const v = await resolveVersion(c, optValue(rest, "--engine") ?? process.env.OG_ENGINE_VERSION ?? "latest");
    const dist = await install(c, v);
    const args = without(rest, "--engine");
    const create = join(dist, "create", "bin", "create.ts");
    const r = spawnSync("bun", [create, ...args], { stdio: "inherit" });
    if (r.error) fail(`cannot start bun (${r.error.message}); the engine runs on bun: https://bun.sh`);
    if (r.status !== 0) return r.status ?? 1;
    const dir = args.find((a, i) => !a.startsWith("-") && !CREATE_VALUES.has(args[i - 1] ?? ""));
    const root = dir ? projectRoot(resolve(dir)) : undefined;
    if (root) linkEngine(root, dist);
    return 0;
  }
  if (cmd === "upgrade") {
    // The target version's og does the upgrade: its changelog knows every version up to it.
    const v = await resolveVersion(c, optValue(rest, "--to") ?? "latest");
    const dist = await install(c, v);
    const status = exec(dist, ["upgrade", ...without(rest, "--to"), "--to", v]);
    const root = projectOf(rest);
    if ((status === 0 || status === 3) && root && pinOf(root) === v) linkEngine(root, dist);
    return status;
  }
  const root = projectOf(rest);
  const v = await versionFor(c, root);
  const dist = await install(c, v);
  if (root) linkEngine(root, dist);
  return exec(dist, argv);
}

if (import.meta.main) {
  try {
    process.exit(await main(process.argv.slice(2)));
  } catch (e) {
    if (e instanceof LauncherError) {
      note(e.message);
      process.exit(1);
    }
    throw e;
  }
}
