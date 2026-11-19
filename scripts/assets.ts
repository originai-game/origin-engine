// The repository's binary assets live outside Git, in the platform's engine
// asset store, addressed by their SHA-256 (docs/ci.md §The asset store). A
// directory that owns pinned files keeps an `og-assets.json` beside them:
//
//   { "version": 1, "files": [{ "path", "sha256", "size", "visibility": "public"|"private", "source"?, "licence"? }] }
//
// `path` is relative to the manifest's directory and stays inside it. The
// manifest names no location: every pin is read from
// <OG_ASSETS_URL>/api/engine-assets/sha256/<sha256>. The tool keeps a marked
// block in the same directory's .gitignore listing the pinned paths.
//
//   bun scripts/assets.ts ensure [dir...]                    install every pin under dir (default: the repository), from the cache, the mirror, then the store
//   bun scripts/assets.ts verify [dir...]                    ask the store that every pin is present at its size (no download)
//   bun scripts/assets.ts pin <dir> [--private] <path...>    pin files of dir as they are now, in dir/og-assets.json (created if missing)
//   bun scripts/assets.ts push [dir...]                      upload every pinned file the store lacks; refuses a file that is not its pin
//   bun scripts/assets.ts status [dir...]                    list pinned files that are missing or modified
//
// Every machine shares one cache, $ORIGIN_ENGINE_CACHE/blobs/sha256/<ab>/<hex>
// (default ~/.cache/origin-engine), so a verified cache works offline.
// OG_ASSETS_MIRROR (a directory or URL laid out as sha256/<ab>/<hex>) is read
// before the store. Reading public objects needs no credentials; push and
// private objects use OG_ASSETS_KEY, else OG_PLATFORM_DEVELOPMENT_KEY.
import { createHash, randomBytes } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join, normalize, relative, resolve } from "node:path";

type Visibility = "public" | "private";
interface Pin { path: string; sha256: string; size: number; visibility: Visibility; source?: string; licence?: string }

const MANIFEST = "og-assets.json";
const STORE = (process.env.OG_ASSETS_URL ?? "https://origingame.ai").replace(/\/+$/, "");
const KEY = process.env.OG_ASSETS_KEY || process.env.OG_PLATFORM_DEVELOPMENT_KEY || undefined;
const CACHE = join(process.env.ORIGIN_ENGINE_CACHE || join(homedir(), ".cache", "origin-engine"), "blobs");
const MIRROR = process.env.OG_ASSETS_MIRROR?.replace(/\/+$/, "");
const REPO = resolve(import.meta.dir, "..");
/** Directories a walk never enters (build output, installs, Git). */
const SKIP = new Set([".git", "node_modules", "target", ".og", "cooked", "og-out", "dist"]);
const PARALLEL = 6;
const VERIFY_BATCH = 1000;
const BEGIN = "# og-assets: the files og-assets.json pins, installed by `bun scripts/assets.ts ensure` (written by the tool)";
const END = "# og-assets end";

function fail(message: string): never {
  console.error(`assets: ${message}`);
  process.exit(1);
}
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const address = (sha: string) => `sha256/${sha.slice(0, 2)}/${sha}`;
const storeUrl = (sha: string) => `${STORE}/api/engine-assets/sha256/${sha}`;
const shown = (path: string) => relative(process.cwd(), path) || ".";

class Manifest {
  readonly dir: string;
  readonly file: string;
  pins: Pin[] = [];

  constructor(dir: string, create = false) {
    this.dir = resolve(dir);
    this.file = join(this.dir, MANIFEST);
    if (!existsSync(this.file)) {
      if (!create) fail(`${shown(this.file)} does not exist`);
      return;
    }
    const read = JSON.parse(readFileSync(this.file, "utf8")) as { version?: unknown; files?: unknown };
    if (read.version !== 1 || !Array.isArray(read.files)) fail(`${shown(this.file)}: expected { "version": 1, "files": [...] }`);
    for (const pin of read.files as Pin[]) {
      if (typeof pin.path !== "string" || !/^[a-f0-9]{64}$/.test(pin.sha256) || !Number.isSafeInteger(pin.size) || pin.size < 0
        || (pin.visibility !== "public" && pin.visibility !== "private")) {
        fail(`${shown(this.file)}: a pin needs path, sha256, size and visibility: ${JSON.stringify(pin)}`);
      }
      this.at(pin.path);
    }
    this.pins = read.files as Pin[];
  }

  /** A pinned path on disk, refused when it leaves the manifest's directory. */
  at(path: string): string {
    const clean = normalize(path);
    if (isAbsolute(clean) || clean.startsWith("..") || path.includes("\\") || clean === ".") fail(`${shown(this.file)}: ${path} is not a path within ${shown(this.dir)}`);
    return join(this.dir, clean);
  }

  write(): void {
    this.pins.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
    writeFileSync(this.file, `${JSON.stringify({ version: 1, files: this.pins }, null, 2)}\n`);
    ignore(this.dir, this.pins.map((p) => p.path));
  }
}

/** Rewrites the tool's block in dir/.gitignore to list `paths`, keeping every other line. */
function ignore(dir: string, paths: string[]): void {
  const file = join(dir, ".gitignore");
  const lines = existsSync(file) ? readFileSync(file, "utf8").split("\n") : [];
  const begin = lines.indexOf(BEGIN), end = lines.indexOf(END);
  const kept = begin >= 0 && end > begin ? [...lines.slice(0, begin), ...lines.slice(end + 1)] : lines;
  while (kept.length && kept[kept.length - 1] === "") kept.pop();
  const block = paths.length ? [BEGIN, ...paths.map((p) => `/${p.replace(/[*?[\\]/g, "\\$&")}`), END] : [];
  const out = [...kept, ...(kept.length && block.length ? [""] : []), ...block];
  writeFileSync(file, out.length ? `${out.join("\n")}\n` : "");
}

/** Every manifest at or under `dirs`, never following a directory symlink. */
function manifests(dirs: string[]): Manifest[] {
  const found: Manifest[] = [];
  const walk = (dir: string) => {
    if (existsSync(join(dir, MANIFEST))) found.push(new Manifest(dir));
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory() && !SKIP.has(entry.name)) walk(join(dir, entry.name));
    }
  };
  for (const dir of dirs.length ? dirs : [REPO]) {
    if (!existsSync(dir) || !statSync(dir).isDirectory()) fail(`${dir} is not a directory`);
    walk(resolve(dir));
  }
  return found;
}

const matches = (path: string, pin: Pin) =>
  existsSync(path) && statSync(path).size === pin.size && sha256(readFileSync(path)) === pin.sha256;

function install(path: string, bytes: Uint8Array): void {
  mkdirSync(join(path, ".."), { recursive: true });
  const part = `${path}.${randomBytes(6).toString("hex")}.part`;
  writeFileSync(part, bytes);
  renameSync(part, path);
}

/** The bytes at `url`, `null` when it holds none. */
async function download(url: string, headers: Record<string, string> = {}): Promise<Uint8Array | null> {
  const response = await fetch(url, { headers });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`answered ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
}

/** The pinned bytes from the cache, the mirror or the store, each checked; the cache keeps what was fetched. */
async function obtain(pin: Pin): Promise<Uint8Array> {
  const cached = join(CACHE, address(pin.sha256));
  if (matches(cached, pin)) return new Uint8Array(readFileSync(cached));
  const sources: [string, () => Promise<Uint8Array | null>][] = [];
  if (MIRROR) {
    const at = /^https?:\/\//.test(MIRROR) ? `${MIRROR}/${address(pin.sha256)}` : join(MIRROR, address(pin.sha256));
    sources.push([`the mirror ${MIRROR}`, async () => (at.startsWith("http") ? download(at) : existsSync(at) ? new Uint8Array(readFileSync(at)) : null)]);
  }
  sources.push(pin.visibility === "private" && !KEY
    ? ["the store", async () => { throw new Error("a private object needs OG_ASSETS_KEY or OG_PLATFORM_DEVELOPMENT_KEY"); }]
    : ["the store", () => download(storeUrl(pin.sha256), pin.visibility === "private" ? { authorization: `Bearer ${KEY}` } : {})]);
  const tried: string[] = ["not in the cache"];
  for (const [where, read] of sources) {
    try {
      const bytes = await read();
      if (bytes && bytes.length === pin.size && sha256(bytes) === pin.sha256) {
        try { install(cached, bytes); } catch { /* an unwritable cache only costs the next fetch */ }
        return bytes;
      }
      tried.push(bytes ? `${where} holds other bytes` : `${where} has none`);
    } catch (e) {
      tried.push(`${where}: ${(e as Error).message ?? e}`);
    }
  }
  throw new Error(tried.join("; "));
}

/** Runs `job` over `items`, a few at once. */
async function each<T>(items: readonly T[], job: (item: T) => Promise<void>): Promise<void> {
  let next = 0;
  await Promise.all(Array.from({ length: Math.min(PARALLEL, items.length) }, async () => {
    while (next < items.length) await job(items[next++]!);
  }));
}

type Located = { manifest: Manifest; pin: Pin; path: string };
const located = (list: Manifest[]): Located[] => list.flatMap((manifest) => manifest.pins.map((pin) => ({ manifest, pin, path: manifest.at(pin.path) })));

/** The store's answer for each address: its size and visibility, absent when it has none (or hides a private one). */
async function present(hashes: string[]): Promise<Map<string, { size: number; visibility: Visibility }>> {
  const found = new Map<string, { size: number; visibility: Visibility }>();
  const unique = [...new Set(hashes)];
  for (let i = 0; i < unique.length; i += VERIFY_BATCH) {
    const response = await fetch(`${STORE}/api/engine-assets/verify`, {
      method: "POST",
      headers: { "content-type": "application/json", ...(KEY ? { authorization: `Bearer ${KEY}` } : {}) },
      body: JSON.stringify({ objects: unique.slice(i, i + VERIFY_BATCH).map((sha) => ({ sha256: sha })) }),
    }).catch((e) => fail(`could not reach ${STORE}: ${e}`));
    if (!response.ok) fail(`${STORE}/api/engine-assets/verify answered ${response.status}: ${(await response.text()).slice(0, 300)}`);
    const { objects } = (await response.json()) as { objects: { sha256: string; present: boolean; size?: number; visibility?: Visibility }[] };
    for (const o of objects) if (o.present) found.set(o.sha256, { size: o.size!, visibility: o.visibility! });
  }
  return found;
}

function finish(problems: string[], done: string): void {
  if (problems.length) fail(`${problems.length} pinned file${problems.length === 1 ? "" : "s"}:\n  ${problems.sort().join("\n  ")}`);
  console.log(`assets: ${done}`);
}

const [command, ...rest] = process.argv.slice(2);
if (command === "ensure") {
  const all = located(manifests(rest));
  const problems: string[] = [];
  let fetched = 0;
  await each(all, async ({ pin, path }) => {
    if (matches(path, pin)) return;
    try {
      install(path, await obtain(pin));
      fetched++;
    } catch (e) {
      problems.push(`${shown(path)}: ${(e as Error).message}`);
    }
  });
  finish(problems, `${all.length} pinned files ready (${fetched} installed)`);
} else if (command === "verify") {
  const all = located(manifests(rest));
  const stored = await present(all.map((l) => l.pin.sha256));
  const problems = all.flatMap(({ pin, path }) => {
    const at = stored.get(pin.sha256);
    if (!at) return [`${shown(path)}: the store has no ${pin.sha256}${pin.visibility === "private" && !KEY ? " (or hides it: a private object needs a key)" : ""}`];
    return at.size === pin.size ? [] : [`${shown(path)}: the store holds ${at.size} bytes, the pin ${pin.size}`];
  });
  finish(problems, `the store holds all ${all.length} pinned files`);
} else if (command === "pin" && rest.length >= 2) {
  const [dir, ...args] = rest;
  const visibility: Visibility = args.includes("--private") ? "private" : "public";
  const paths = args.filter((a) => a !== "--private");
  if (!paths.length) fail("name the files to pin");
  const manifest = new Manifest(dir!, true);
  for (const path of paths) {
    const file = manifest.at(path);
    if (!existsSync(file) || !statSync(file).isFile()) fail(`${shown(file)} is not a file`);
    const bytes = new Uint8Array(readFileSync(file));
    const name = normalize(path);
    const old = manifest.pins.find((p) => p.path === name);
    manifest.pins = manifest.pins.filter((p) => p.path !== name);
    // A file pinned again keeps its provenance.
    manifest.pins.push({ ...old, path: name, sha256: sha256(bytes), size: bytes.length, visibility });
    install(join(CACHE, address(sha256(bytes))), bytes);
  }
  manifest.write();
  console.log(`assets: pinned ${paths.length} files in ${shown(manifest.file)}; push them before committing it`);
} else if (command === "push") {
  if (!KEY) fail("set OG_ASSETS_KEY or OG_PLATFORM_DEVELOPMENT_KEY to push (a platform key the store lets write)");
  const all = located(manifests(rest));
  const problems = all.filter(({ pin, path }) => !matches(path, pin)).map(({ path }) => `${shown(path)} is not its pin: pin it again or restore it (bun scripts/assets.ts ensure)`);
  if (problems.length) finish(problems, "");
  const stored = await present(all.map((l) => l.pin.sha256));
  // Each address once, as public when any pin of it is public.
  const wanted = new Map<string, Located>();
  for (const l of all) if (!wanted.has(l.pin.sha256) || l.pin.visibility === "public") wanted.set(l.pin.sha256, l);
  const puts = [...wanted.values()].filter(({ pin }) => {
    const at = stored.get(pin.sha256);
    return !at || (pin.visibility === "public" && at.visibility === "private");
  });
  await each(puts, async ({ pin, path }) => {
    const response = await fetch(`${storeUrl(pin.sha256)}?visibility=${pin.visibility}`, {
      method: "PUT",
      headers: { authorization: `Bearer ${KEY}`, "content-type": "application/octet-stream", "content-length": String(pin.size) },
      body: readFileSync(path),
    }).catch((e) => ({ ok: false, status: 0, text: async () => String(e) }) as const);
    if (!response.ok) problems.push(`${shown(path)}: the store answered ${response.status}: ${(await response.text()).slice(0, 300)}`);
  });
  finish(problems, `${all.length} pinned files in the store (${puts.length} uploaded)`);
} else if (command === "status") {
  const problems = located(manifests(rest)).flatMap(({ pin, path }) =>
    !existsSync(path) ? [`${shown(path)}: missing (bun scripts/assets.ts ensure)`] : matches(path, pin) ? [] : [`${shown(path)}: modified (pin it again, or ensure to restore it)`]);
  finish(problems, "every pinned file is its pin");
} else {
  fail("usage: bun scripts/assets.ts ensure [dir...] | verify [dir...] | pin <dir> [--private] <path...> | push [dir...] | status [dir...]");
}
