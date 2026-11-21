// A downloadable effect pack (docs/release.md §1, docs/vfx-packs.md §5):
// its archive `<version>/vfx/<name>.tar.gz` beside the engine archives,
// listed in the version's signed SHA256SUMS and named in its manifest.json.
// scripts/release-dist.ts writes the archives with `packArchive`; `og vfx add
// <name>` runs this file to fetch one:
//
//   bun src/node/vfx-pack.ts fetch <name> <engine version> <into>
//
// It downloads <downloads>/<version>/SHA256SUMS and SHA256SUMS.minisig,
// verifies the signature with the engine's public key (src/node/minisign.ts,
// the one implementation the launcher shares), checks manifest.json and the
// archive
// against the signed sums, and unpacks the archive into <into>/<name>/:
// regular files and directories only, every path under <name>/ with no `..`
// and no absolute path, within a size cap. og then installs that directory
// as it installs a local one. `$ORIGIN_ENGINE_DOWNLOADS` (default
// https://origingame.ai/downloads/engine; a URL, file:// or a directory) and
// `$ORIGIN_ENGINE_PUBKEY` (a test key) are the launcher's.

import { createHash } from "node:crypto";
import { existsSync, lstatSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { gunzipSync, gzipSync } from "node:zlib";

import { DOWNLOADS, ENGINE_PUBKEY, MinisignError, verifyMinisign } from "./minisign.ts";
export { DOWNLOADS, ENGINE_PUBKEY, MinisignError, verifyMinisign };

// The node members this file uses (the package type-checks without node's declarations: src/env.d.ts).
declare module "node:fs" {
  interface Dirent { name: string; isDirectory(): boolean; isFile(): boolean; isSymbolicLink(): boolean }
  export function readdirSync(path: string, options: { withFileTypes: true }): Dirent[];
  export function writeFileSync(path: string, data: Uint8Array, options: { flag: string }): void;
}
declare module "node:zlib" {
  export function gzipSync(data: Uint8Array, options?: { level?: number }): Uint8Array;
  export function gunzipSync(data: Uint8Array, options?: { maxOutputLength?: number }): Uint8Array;
}

/** What one pack may weigh: the archive, and its files unpacked. */
export const MAX_ARCHIVE = 64 << 20;
export const MAX_UNPACKED = 256 << 20;
const MAX_ENTRIES = 20_000;
/** Every entry's mtime in an archive, so two builds of one pack are the same bytes. */
const MTIME = 0;
const PREVIEW = "preview";
const NAME_RE = /^[a-z][a-z0-9-]*$/;

export const archivePath = (name: string) => `vfx/${name}.tar.gz`;
const sha256 = (b: Uint8Array) => createHash("sha256").update(b).digest("hex");
const utf8 = new TextEncoder();
const text8 = new TextDecoder();
function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

export class PackError extends Error {}
function fail(why: string): never {
  throw new PackError(why);
}

// ---------------------------------------------------------------- the archive

/** The files a pack installs, sorted: what og vfx add copies (no hidden files, nothing under preview/); refuses links. */
export function packFiles(dir: string): string[] {
  const out: string[] = [];
  const walk = (rel: string) => {
    for (const e of readdirSync(join(dir, rel), { withFileTypes: true })) {
      if (e.name.startsWith(".")) continue;
      const path = rel ? `${rel}/${e.name}` : e.name;
      if (path === PREVIEW) continue;
      const st = lstatSync(join(dir, path));
      if (st.isSymbolicLink()) fail(`${join(dir, path)}: a pack holds no symlinks`);
      if (st.isDirectory()) walk(path);
      else if (st.isFile()) out.push(path);
      else fail(`${join(dir, path)}: not a regular file`);
    }
  };
  walk("");
  return out.sort();
}

function header(path: string, size: number, dir: boolean): Uint8Array {
  const h = new Uint8Array(512);
  const enc = utf8;
  const put = (at: number, len: number, s: string) => {
    const b = enc.encode(s);
    if (b.length > len) fail(`${path}: too long for a tar header`);
    h.set(b, at);
  };
  const octal = (at: number, len: number, n: number) => put(at, len, `${n.toString(8).padStart(len - 1, "0")}\0`);
  let name = path;
  let prefix = "";
  if (enc.encode(path).length > 100) {
    const cut = path.lastIndexOf("/", path.length - 2);
    prefix = path.slice(0, cut);
    name = path.slice(cut + 1);
  }
  put(0, 100, name);
  octal(100, 8, dir ? 0o755 : 0o644);
  octal(108, 8, 0);
  octal(116, 8, 0);
  octal(124, 12, size);
  octal(136, 12, MTIME);
  h.fill(0x20, 148, 156);
  h[156] = dir ? 0x35 : 0x30;
  put(257, 6, "ustar\0");
  put(263, 2, "00");
  put(265, 32, "root");
  put(297, 32, "root");
  put(345, 155, prefix);
  put(148, 8, `${h.reduce((a: number, b: number) => a + b, 0).toString(8).padStart(6, "0")}\0 `);
  return h;
}

/**
 * The pack in `dir` as `vfx/<name>.tar.gz`'s bytes: ustar entries under
 * `<name>/`, sorted, mode 0644 or 0755, owner 0:0 (root), mtime 0, its
 * files as packFiles names them; the same bytes on every run.
 */
export function packArchive(dir: string, name: string): Uint8Array {
  const files = packFiles(dir);
  const dirs = new Set<string>([name]);
  for (const f of files) for (let at = f.indexOf("/"); at >= 0; at = f.indexOf("/", at + 1)) dirs.add(`${name}/${f.slice(0, at)}`);
  const entries = [...[...dirs].map((d) => ({ path: `${d}/`, file: null as string | null })), ...files.map((f) => ({ path: `${name}/${f}`, file: f }))];
  entries.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  const parts: Uint8Array[] = [];
  for (const e of entries) {
    const body = e.file === null ? new Uint8Array(0) : new Uint8Array(readFileSync(join(dir, e.file)));
    parts.push(header(e.path, body.length, e.file === null), body, new Uint8Array((512 - (body.length % 512)) % 512));
  }
  parts.push(new Uint8Array(1024));
  const gz = new Uint8Array(gzipSync(concat(parts), { level: 9 }));
  gz.fill(0, 4, 8); // the gzip header's mtime
  gz[9] = 3; // and its OS byte (unix), whichever host packs it
  return gz;
}

/**
 * Unpacks a pack archive into `into/<name>/` (`into` empty): only regular
 * files and directories, every path `<name>/...` with no empty, `.` or `..`
 * component, no absolute path or backslash, no duplicate, within the caps.
 */
export function unpackArchive(gz: Uint8Array, name: string, into: string, required: readonly string[] = ["extension.json"]): void {
  let tar: Uint8Array;
  try {
    tar = gunzipSync(gz, { maxOutputLength: MAX_UNPACKED + MAX_ENTRIES * 1024 });
  } catch (e) {
    fail(`${archivePath(name)} is not a gzip archive within the ${MAX_UNPACKED >> 20} MiB cap (${e instanceof Error ? e.message : e})`);
  }
  const text = (at: number, len: number) => {
    const s = tar.subarray(at, at + len);
    const end = s.indexOf(0);
    return text8.decode(s.subarray(0, end < 0 ? len : end));
  };
  const octal = (at: number, len: number) => {
    const s = text(at, len).trim();
    if (!/^[0-7]+$/.test(s)) fail(`${archivePath(name)}: a corrupt tar header`);
    return Number.parseInt(s, 8);
  };
  let total = 0;
  let entries = 0;
  let at = 0;
  for (;;) {
    if (at + 512 > tar.length) fail(`${archivePath(name)}: the tar ends inside an entry`);
    const h = tar.subarray(at, at + 512);
    if (h.every((b) => b === 0)) break;
    if (++entries > MAX_ENTRIES) fail(`${archivePath(name)} has more than ${MAX_ENTRIES} entries`);
    let sum = 0;
    for (let i = 0; i < 512; i++) sum += i >= 148 && i < 156 ? 0x20 : h[i]!;
    if (octal(at + 148, 8) !== sum) fail(`${archivePath(name)}: a tar header's checksum does not match`);
    if (!text(at + 257, 6).startsWith("ustar")) fail(`${archivePath(name)}: not a ustar archive`);
    const prefix = text(at + 345, 155);
    const path = (prefix ? `${prefix}/` : "") + text(at, 100);
    const type = String.fromCharCode(h[156]!);
    const size = octal(at + 124, 12);
    at += 512;
    const kind = type === "0" || type === "\0" ? "file" : type === "5" ? "dir" : null;
    if (!kind) fail(`${archivePath(name)}: ${path} is a ${{ "1": "hard link", "2": "symlink" }[type] ?? `tar entry of type '${type}'`}; a pack holds only regular files`);
    const rel = kind === "dir" ? path.replace(/\/$/, "") : path;
    const parts = rel.split("/");
    if (rel.startsWith("/") || rel.includes("\\") || parts.some((p) => p === "" || p === "." || p === "..") || parts[0] !== name) fail(`${archivePath(name)}: the unsafe path ${JSON.stringify(path)} (every entry is under ${name}/)`);
    if (kind === "dir" && size !== 0) fail(`${archivePath(name)}: the directory ${path} has a size`);
    total += size;
    if (total > MAX_UNPACKED) fail(`${archivePath(name)} unpacks past the ${MAX_UNPACKED >> 20} MiB cap`);
    if (at + size > tar.length) fail(`${archivePath(name)}: the tar ends inside ${path}`);
    try {
      if (kind === "dir") {
        mkdirSync(join(into, rel), { recursive: true });
        continue;
      }
      mkdirSync(dirname(join(into, rel)), { recursive: true });
      writeFileSync(join(into, rel), tar.subarray(at, at + size), { flag: "wx" });
    } catch (e) {
      fail(`${archivePath(name)}: cannot write ${path} (${e instanceof Error ? e.message : e}); an entry is there twice or under a file`);
    }
    at += size + ((512 - (size % 512)) % 512);
  }
  for (const file of required) if (!existsSync(join(into, name, file))) fail(`${archivePath(name)} has no ${name}/${file}`);
}

// ---------------------------------------------------------------- the download

export interface VfxEntry {
  url: string;
  sha256: string;
  size: number;
  version: string;
  minEngine: string | null;
}

/** The lines `<sha256>  <path>` of a SHA256SUMS, by path. */
export function parseSums(text: string): Map<string, string> {
  const sums = new Map<string, string>();
  for (const line of text.split("\n")) {
    const m = /^([0-9a-f]{64}) [ *](.+)$/.exec(line.trim());
    if (m) sums.set(m[2]!, m[1]!);
  }
  return sums;
}

async function get(base: string, path: string, cap: number): Promise<Uint8Array> {
  let bytes: Uint8Array;
  if (/^https?:\/\//.test(base)) {
    const r = await fetch(`${base}/${path}`);
    if (!r.ok) fail(`${base}/${path}: ${r.status} ${r.statusText}`);
    if (Number(r.headers.get("content-length") ?? 0) > cap) fail(`${base}/${path} is larger than ${cap >> 20} MiB`);
    bytes = new Uint8Array(await r.arrayBuffer());
  } else {
    const file = join(base.replace(/^file:\/\//, ""), path);
    if (!existsSync(file)) fail(`${file}: no such file`);
    bytes = new Uint8Array(readFileSync(file));
  }
  if (bytes.length > cap) fail(`${base}/${path} is larger than ${cap >> 20} MiB`);
  return bytes;
}

/** Fetches engine `version`'s pack `name`, verified, into `into/<name>/`; returns its manifest entry. */
export async function fetchPack(o: { name: string; version: string; into: string; downloads?: string; pubkey?: string }): Promise<VfxEntry> {
  const { name, version, into } = o;
  if (!NAME_RE.test(name)) fail(`\`${name}\` is not a pack name (lowercase letters, digits and -)`);
  if (!/^(?:\d+\.\d+\.\d+(?:-dev\.\d+)?|dev\.\d{8}\.[1-9]\d*)$/.test(version)) fail(`${version} is not an engine version`);
  const base = `${(o.downloads || process.env.ORIGIN_ENGINE_DOWNLOADS || DOWNLOADS).replace(/\/+$/, "")}/${version}`;
  const sums = await get(base, "SHA256SUMS", 1 << 20);
  const sig = text8.decode(await get(base, "SHA256SUMS.minisig", 1 << 16));
  verifyMinisign(sums, sig, o.pubkey || process.env.ORIGIN_ENGINE_PUBKEY || ENGINE_PUBKEY);
  const signed = parseSums(text8.decode(sums));
  const manifestBytes = await get(base, "manifest.json", 1 << 20);
  if (signed.get("manifest.json") !== sha256(manifestBytes)) fail(`${base}/manifest.json does not match the signed SHA256SUMS`);
  const manifest = JSON.parse(text8.decode(manifestBytes)) as { version?: string; vfx?: Record<string, VfxEntry> };
  if (manifest.version !== version) fail(`${base}/manifest.json is version ${manifest.version}`);
  const entry = manifest.vfx?.[name];
  const offered = Object.keys(manifest.vfx ?? {});
  if (!entry) fail(`engine ${version} has no downloadable vfx pack \`${name}\` (${offered.length ? `it has ${offered.join(", ")}` : "it has none"})`);
  const file = archivePath(name);
  const want = signed.get(file);
  if (!want) fail(`the signed SHA256SUMS does not list ${file}`);
  if (entry.sha256 !== want) fail(`manifest.json's sha256 of ${file} is not the signed SHA256SUMS's`);
  const archive = await get(base, file, MAX_ARCHIVE);
  const got = sha256(archive);
  if (got !== want) fail(`${file} is ${got}; the signed SHA256SUMS says ${want}`);
  unpackArchive(archive, name, into);
  const ext = JSON.parse(readFileSync(join(into, name, "extension.json"), "utf8")) as { name?: string; kind?: string; version?: string };
  if (ext.name !== name || ext.kind !== "vfx" || ext.version !== entry.version) fail(`${file} holds ${ext.kind} ${ext.name} ${ext.version}, not vfx ${name} ${entry.version}`);
  return entry;
}

if (import.meta.main) {
  const [verb, name, version, into] = process.argv.slice(2);
  if (verb !== "fetch" || !name || !version || !into) {
    process.stderr.write("usage: bun src/node/vfx-pack.ts fetch <name> <engine version> <into>\n");
    process.exit(2);
  }
  try {
    const entry = await fetchPack({ name, version, into });
    process.stderr.write(`downloaded ${name} ${entry.version} (${entry.size} bytes, sha256 ${entry.sha256.slice(0, 12)}…, signed SHA256SUMS of engine ${version})\n`);
  } catch (e) {
    process.stderr.write(`${e instanceof PackError || e instanceof MinisignError ? e.message : e instanceof Error ? e.stack : e}\n`);
    process.exit(1);
  }
}
