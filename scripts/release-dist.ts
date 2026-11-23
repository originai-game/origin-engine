// The download channel's files of one version (docs/release.md §1, §3):
// per target an archive `origin-engine-<v>-<target>.tar.gz` of the dist
// layout, then `manifest.json`, the launcher's `install.sh` and
// `install.ps1`, `SHA256SUMS` over all of them
// (what the platform's upload takes) and `SHA256SUMS.minisig`.
//
//   bun scripts/release-dist.ts <dir> --version <v> [--registry] [--revision <sha>] [--targets <t,...>]
//   bun scripts/release-dist.ts <dir> --version <v> --sign    the files beside the archives again, and the signature,
//                                                               over the archives in place (they must be the ones
//                                                               SHA256SUMS names: the install test's passes stand)
//
// <dir> (dist/release/<v>/) holds the npm-shaped tarballs packages/engine's
// pack.sh writes: origingame-engine-<v>.tgz, origingame-engine-<platform>-<v>.tgz
// (darwin-arm64 and linux-x64 from the Mac, win32-x64 copied in from
// studio-win), origingame-create-game-<v>.tgz and origingame-design-<v>.tgz.
// They are build intermediates, never uploaded. `--registry` first downloads
// the ones a version published to npm before 2026-11-05 (every version from
// 0.5.0 and 0.7.8-dev.17) from the registry, each checked against its
// integrity there, so a backfilled archive carries the bytes that version
// shipped with. tsrs (`og check`'s checker, a third-party npm package) comes
// from the registry at the version the engine package's optionalDependencies
// name, integrity-checked too.
//
// An archive is made for every target whose platform tarball is in <dir>
// (`--targets` restricts them). Its root directory
// `origin-engine-<v>-<target>/` is the layout the runner, `og` and
// origin-agent's engine_scaffold read, usable unpacked with no npm:
//
//   origin-engine.json   the scaffold overlay's origin-engine.json + build {package, version, base?, revision, target, triple,
//                        og, addon, wasm, wasmPublish, wasmLanes}; `target` is the node platform
//                        (linux-x64, darwin-arm64, win32-x64), `triple` the archive's target; `base` (a dev build
//                        dev.<date>.<n> only, as in manifest.json) the release it is based on: the newest version
//                        of the engine package's changelog.json
//   overlay/             the engine knowledge's game-scaffold/origin-engine-ts
//   files/<path>         the overlay's distribution.json binaries, from create-game's template/, sha256-verified
//   bin/og, bin/og.exe   the og tool
//   engine/              @origingame/engine as packed, with native/origin-engine.<platform>.node
//   create/              @origingame/create-game as packed (`og new` runs its bin/create.ts, which reads engine/knowledge's brief.ts)
//   design/              @origingame/design as packed (when <dir> has it; npm never had it)
//   node_modules/@tsrs/<tsrs platform>/   the type checker og check finds beside the engine (none on Windows: tsgo)
//
// Beside the target archives, one archive per downloadable effect pack
// (docs/release.md §1, docs/vfx-packs.md §5): every packages/vfx-packs/<name>/
// with an extension.json, at the version's revision, as `vfx/<name>.tar.gz`
// (root directory `<name>/`, no preview/, the same bytes on every build:
// packages/engine/src/node/vfx-pack.ts's packArchive), named in manifest.json's
// `vfx` and listed in SHA256SUMS. No target archive carries packages/vfx-packs/
// (checkPacksOutside).
//
// Beside them, the look catalogue and its previews (docs/release.md §1 The
// looks): `looks/catalogue.json`, the engine package's
// knowledge/looks/catalogue.json byte for byte, and `previews/<slug>.webp`
// for each of its styles, which `scripts/release.sh previews` drew on the GPU
// host with this version's own linux-x64 og and addon (render.ts --gpu) and
// recorded in `looks-render.json`. Archiving refuses previews that record
// names another version, commit, og or addon, a tier other than the
// recording tier (vulkan), or files whose bytes it did not record: a version
// never ships previews it did not draw. Named in manifest.json's `looks`
// and listed in SHA256SUMS; a backfill (--registry) has none.
//
// The local record `release.json` (the commit, whether the tree was dirty,
// the source, and the install test's passes) stays beside them for
// `scripts/release.sh publish` to read; it is not uploaded.
//
// Signing (the owner, 2026-11-05): OG_ENGINE_MINISIGN_KEY, the project
// secret of origingame/origin-engine holding the unencrypted minisign secret
// key file's content, written to a 0600 temporary file for minisign and
// removed after; never a key file in the repository. Without it SHA256SUMS
// stays unsigned and publish refuses it. The signature is checked after
// signing against the public key docs/release.md §1 states
// (OG_ENGINE_MINISIGN_PUBKEY overrides it for a test key).

import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, copyFileSync, cpSync, existsSync, linkSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { MAX_ARCHIVE, archivePath, packArchive, type VfxEntry } from "../packages/engine/src/node/vfx-pack.ts";
import { installers, pinnedKey } from "../packages/launcher/build.ts";
import { VERSION_RE } from "../packages/launcher/og.ts";

export const TARGETS = {
  "x86_64-unknown-linux-gnu": { platform: "linux-x64", tsrs: "linux-x64-gnu", exe: "og" },
  "aarch64-apple-darwin": { platform: "darwin-arm64", tsrs: "darwin-arm64", exe: "og" },
  "x86_64-pc-windows-msvc": { platform: "win32-x64", tsrs: null, exe: "og.exe" },
} as const;
export type Target = keyof typeof TARGETS;
/** A release `x.y.z`, a dev build `dev.<YYYYMMDD>.<n>` (docs/release.md §1a), or a dev build of the scheme before 0.9.0, `x.y.z-dev.n`. */
export const VERSION_RE = /^(?:\d+\.\d+\.\d+(?:-dev\.\d+)?|dev\.\d{8}\.[1-9]\d*)$/;
export const isDev = (v: string) => v.startsWith("dev.") || v.includes("-dev.");
import { DOWNLOADS } from "../packages/engine/src/node/minisign.ts";
export { DOWNLOADS };
const OVERLAY = "game-scaffold/origin-engine-ts";
const ROOT = resolve(import.meta.dir, "..");
const REGISTRY = "https://registry.npmjs.org";

export const archiveName = (v: string, t: Target) => `origin-engine-${v}-${t}.tar.gz`;
const sha256 = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const say = (s: string) => process.stdout.write(`release-dist: ${s}\n`);

function die(why: string): never {
  process.stderr.write(`release-dist: ${why}\n`);
  process.exit(1);
}

interface RegistryVersion {
  version: string;
  dist: { tarball: string; integrity: string };
  optionalDependencies?: Record<string, string>;
}

async function registryDoc(name: string): Promise<{ versions: Record<string, RegistryVersion>; time: Record<string, string> }> {
  const r = await fetch(`${REGISTRY}/${name.replace("/", "%2f")}`);
  if (!r.ok) die(`the registry answered ${r.status} for ${name}`);
  return (await r.json()) as { versions: Record<string, RegistryVersion>; time: Record<string, string> };
}

/** Downloads `name@version`'s tarball to `to` unless it is there with the registry's integrity. */
async function fetchTarball(name: string, version: string, to: string): Promise<RegistryVersion> {
  const doc = await registryDoc(name);
  const meta = doc.versions[version] ?? die(`${name}@${version} is not on the registry`);
  const [alg, want] = meta.dist.integrity.split("-", 2) as [string, string];
  const ok = (p: string) => existsSync(p) && createHash(alg).update(readFileSync(p)).digest("base64") === want;
  if (!ok(to)) {
    const r = await fetch(meta.dist.tarball);
    if (!r.ok) die(`downloading ${meta.dist.tarball}: ${r.status}`);
    mkdirSync(dirname(to), { recursive: true });
    writeFileSync(to, new Uint8Array(await r.arrayBuffer()));
    if (!ok(to)) die(`${meta.dist.tarball} does not match the registry's integrity ${meta.dist.integrity}`);
    say(`downloaded ${name}@${version} (${statSync(to).size} bytes, ${meta.dist.integrity.slice(0, 20)}…)`);
  }
  return meta;
}

/** Unpacks an npm tarball's `package/` into `to` (replacing it). */
function unpackTgz(tgz: string, to: string) {
  rmSync(to, { recursive: true, force: true });
  mkdirSync(dirname(to), { recursive: true });
  const tmp = mkdtempSync(join(dirname(to), ".unpack-"));
  execFileSync("tar", ["-xzf", tgz, "-C", tmp], { stdio: ["ignore", "ignore", "inherit"] });
  const inner = readdirSync(tmp);
  if (inner.length !== 1) die(`${tgz} holds ${inner.join(", ")}, not one package directory`);
  renameSync(join(tmp, inner[0]!), to);
  rmSync(tmp, { recursive: true, force: true });
}

function git(...a: string[]): string {
  return execFileSync("git", a, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
}

/** The commit a version was built from: a release's tag, a dev build `x.y.z-dev.<run number>`'s release.yml run (gh). */
function revisionOf(version: string): string {
  if (version.startsWith("dev.")) die(`a dev build names its exact commit: pass --revision <sha> for ${version}`);
  try {
    return git("rev-parse", `v${version}^{commit}`);
  } catch {}
  const dev = /-dev\.(\d+)$/.exec(version);
  if (dev) {
    const runs = JSON.parse(execFileSync("gh", ["run", "list", "-R", "originai-game/origin-engine", "-w", "release.yml", "-L", "200", "--json", "number,headSha,conclusion"], { encoding: "utf8" })) as { number: number; headSha: string; conclusion: string }[];
    const run = runs.find((r) => r.number === Number(dev[1]) && r.conclusion === "success");
    if (run) return run.headSha;
  }
  die(`no tag v${version} and no release.yml run for it: pass --revision <sha>`);
}

interface Layout {
  package: string;
  version: string;
  /** A dev build `dev.<date>.<n>`: the release it is based on. */
  base?: string;
  revision: string;
  target: string;
  triple: Target;
  og: string;
  addon: string;
  wasm: string;
  wasmPublish: string;
  wasmLanes: string;
}

/** Writes the scaffold overlay of an unpacked engine package (its knowledge, through its own lib.ts) into `to`. */
async function writeOverlay(engine: string, to: string) {
  const lib = join(engine, "knowledge/scripts/lib.ts");
  if (!existsSync(lib)) die(`${engine} carries no knowledge/scripts/lib.ts`);
  const { defaultRoots, payload } = (await import(lib)) as {
    defaultRoots(pkg: string): unknown;
    payload(roots: unknown): { path: string; bytes: Uint8Array; symlink: boolean }[];
  };
  const prefix = `${OVERLAY}/`;
  let n = 0;
  for (const f of payload(defaultRoots(join(engine, "knowledge")))) {
    if (f.symlink || !f.path.startsWith(prefix)) continue;
    const out = join(to, f.path.slice(prefix.length));
    mkdirSync(dirname(out), { recursive: true });
    writeFileSync(out, f.bytes);
    n++;
  }
  if (!existsSync(join(to, "project.ogt"))) die(`the engine's knowledge has no ${OVERLAY}/project.ogt (${n} files under it)`);
}

/** Copies a directory tree as it is, without .DS_Store. */
function copyTree(from: string, to: string) {
  cpSync(from, to, { recursive: true, filter: (p) => basename(p) !== ".DS_Store" });
}

/** Keep every asset path, but store identical bytes once in tar and on disk. */
export function deduplicateAssets(root: string): number {
  const first = new Map<string, string>();
  let saved = 0;
  const walk = (directory: string) => {
    if (!existsSync(directory)) return;
    for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (entry.isFile()) {
        const stat = statSync(path);
        if (!stat.size) continue;
        const key = `${stat.mode & 0o777}:${sha256(path)}`;
        const original = first.get(key);
        if (original) {
          rmSync(path);
          linkSync(original, path);
          saved += stat.size;
        } else first.set(key, path);
      }
    }
  };
  for (const directory of ["engine/fonts", "engine/styles", "create/fonts", "create/styles"]) walk(join(root, directory));
  return saved;
}

/** The layout's required parts, as origin-agent's readEngineLayout checks them. */
export function checkLayout(dir: string, t: Target): void {
  const { platform, tsrs, exe } = TARGETS[t];
  const need = [
    `bin/${exe}`,
    "origin-engine.json",
    "overlay/project.ogt",
    "engine/package.json",
    "engine/src/node/run.ts",
    "engine/knowledge/knowledge.json",
    "engine/knowledge/scripts/lib.ts",
    "engine/styles/shared/LICENSES.md",
    "engine/styles/shared/InstrumentSans-Regular.ttf",
    "engine/styles/cjk/NotoSansSC-Regular.ttf",
    "engine/dist/web-publish/origin-engine_bg.wasm",
    "engine/dist/web-publish/origin-engine-lanes_bg.wasm",
    "engine/dist/web-publish/origin-engine-lanes.js",
    `engine/native/origin-engine.${platform}.node`,
    "create/bin/create.ts",
    "create/template/project.ogt",
    ...(tsrs ? [`node_modules/@tsrs/${tsrs}/${exe === "og.exe" ? "tsrs.exe" : "tsrs"}`] : []),
  ];
  const missing = need.filter((p) => !existsSync(join(dir, p)));
  if (!statSync(join(dir, "engine/dist/web-publish/snippets"), { throwIfNoEntry: false })?.isDirectory()) missing.push("engine/dist/web-publish/snippets/");
  if (missing.length) die(`${dir} lacks ${missing.join(", ")}`);
}

/** The downloadable packs in `packs` (packages/vfx-packs/ or a copy): each subdirectory with an extension.json, sorted. */
export function vfxPacks(packs: string): string[] {
  if (!existsSync(packs)) return [];
  return readdirSync(packs, { withFileTypes: true })
    .filter((e) => e.isDirectory() && existsSync(join(packs, e.name, "extension.json")))
    .map((e) => e.name)
    .sort();
}

/**
 * Writes `<dir>/vfx/<name>.tar.gz` for each pack in `packs` (replacing
 * `<dir>/vfx/`) and returns manifest.json's `vfx` section. Throws on a pack
 * whose manifest is not a vfx pack of its directory's name.
 */
export function writeVfxArchives(dir: string, version: string, packs: string): Record<string, VfxEntry> {
  rmSync(join(dir, "vfx"), { recursive: true, force: true });
  const out: Record<string, VfxEntry> = {};
  for (const name of vfxPacks(packs)) {
    const m = JSON.parse(readFileSync(join(packs, name, "extension.json"), "utf8")) as { kind?: string; name?: string; version?: string; minEngine?: string };
    if (m.kind !== "vfx" || m.name !== name || !/^\d+\.\d+\.\d+$/.test(m.version ?? "")) throw new Error(`${join(packs, name)}/extension.json is not the vfx pack ${name} at an x.y.z version`);
    const bytes = packArchive(join(packs, name), name);
    if (bytes.length > MAX_ARCHIVE) throw new Error(`${archivePath(name)} is ${bytes.length} bytes; og vfx add takes at most ${MAX_ARCHIVE}`);
    mkdirSync(join(dir, "vfx"), { recursive: true });
    writeFileSync(join(dir, archivePath(name)), bytes);
    out[name] = { url: `${DOWNLOADS}/${version}/${archivePath(name)}`, sha256: createHash("sha256").update(bytes).digest("hex"), size: bytes.length, version: m.version!, minEngine: m.minEngine ?? null };
    say(`${archivePath(name)}  ${bytes.length} bytes  sha256 ${out[name]!.sha256}`);
  }
  return out;
}

/**
 * A target's layout carries no downloadable pack: no `vfx-packs` directory
 * in engine/, overlay/ or files/, no engine/extensions/<pack> of a
 * downloadable pack's name (a pack is bundled or downloadable), and no entry
 * of the engine package's `files` that reaches packages/vfx-packs/.
 */
export function checkPacksOutside(root: string, downloadable: string[]): void {
  const found: string[] = [];
  const walk = (rel: string) => {
    for (const e of readdirSync(join(root, rel), { withFileTypes: true })) {
      const path = `${rel}/${e.name}`;
      if (e.name === "vfx-packs") found.push(path);
      else if (e.isDirectory()) walk(path);
    }
  };
  for (const top of ["engine", "overlay", "files"]) if (existsSync(join(root, top))) walk(top);
  for (const name of downloadable) if (existsSync(join(root, "engine/extensions", name))) found.push(`engine/extensions/${name} (${name} is a downloadable pack)`);
  const pkg = join(root, "engine/package.json");
  const files = existsSync(pkg) ? ((JSON.parse(readFileSync(pkg, "utf8")) as { files?: string[] }).files ?? []) : [];
  for (const f of files) if (f.includes("vfx-packs") || f.split("/").includes("..")) found.push(`engine/package.json files: ${f}`);
  if (found.length) throw new Error(`${basename(root)} carries downloadable vfx packs (they ship as vfx/<name>.tar.gz only): ${found.join(", ")}`);
}

/** Installs the pinned files under `dir` (its og-assets.json; docs/ci.md §The asset store), so a pack reads every file. */
function ensurePins(dir: string): string {
  const r = spawnSync("bun", [join(ROOT, "scripts/assets.ts"), "ensure", dir], { cwd: ROOT, stdio: ["ignore", "ignore", "inherit"] });
  if (r.status !== 0) die(`the pinned files in ${dir} are not all available`);
  return dir;
}

/** packages/vfx-packs/ as the version was built: the checkout's when packed here, else the revision's (none if it has none). */
function vfxSource(revision: string, here: boolean, work: string): string {
  if (here) return ensurePins(join(ROOT, "packages/vfx-packs"));
  const to = join(work, "vfx-packs-src");
  mkdirSync(to, { recursive: true });
  try {
    git("cat-file", "-e", `${revision}^{commit}`);
  } catch {
    die(`the commit ${revision} is not in this clone: git fetch it, so its packages/vfx-packs/ can be archived`);
  }
  if (!git("ls-tree", "-d", "--name-only", revision, "packages/vfx-packs")) return join(to, "packages/vfx-packs");
  const tar = spawnSync("sh", ["-c", 'git archive "$1" packages/vfx-packs | tar -xf - -C "$2"', "sh", revision, to], { cwd: ROOT, stdio: ["ignore", "ignore", "inherit"] });
  if (tar.status !== 0) die(`git archive ${revision} packages/vfx-packs failed`);
  // A revision whose packs are pinned archives only their og-assets.json; one from before keeps its files in Git.
  const packs = join(to, "packages/vfx-packs");
  return existsSync(join(packs, "og-assets.json")) ? ensurePins(packs) : packs;
}

/** Assembles one target's layout under `work` and packs it into `<dir>/<archive>`. */
async function assemble(o: { dir: string; work: string; version: string; base?: string; revision: string; t: Target; common: string; tsrsTgz: string | null; downloadable: string[] }): Promise<Layout> {
  const { platform, tsrs, exe } = TARGETS[o.t];
  const name = `origin-engine-${o.version}-${o.t}`;
  const root = join(o.work, name);
  rmSync(root, { recursive: true, force: true });
  mkdirSync(join(root, "bin"), { recursive: true });
  copyTree(join(o.common, "engine"), join(root, "engine"));
  copyTree(join(o.common, "create"), join(root, "create"));
  if (existsSync(join(o.common, "design"))) copyTree(join(o.common, "design"), join(root, "design"));
  copyTree(join(o.common, "overlay"), join(root, "overlay"));
  copyTree(join(o.common, "files"), join(root, "files"));

  const plat = join(o.work, `platform-${platform}`);
  unpackTgz(join(o.dir, `origingame-engine-${platform}-${o.version}.tgz`), plat);
  mkdirSync(join(root, "engine/native"), { recursive: true });
  copyFileSync(join(plat, "origin-engine.node"), join(root, "engine/native", `origin-engine.${platform}.node`));
  if (!existsSync(join(plat, exe))) die(`origingame-engine-${platform}-${o.version}.tgz has no ${exe}`);
  copyFileSync(join(plat, exe), join(root, "bin", exe));
  chmodSync(join(root, "bin", exe), 0o755);
  rmSync(plat, { recursive: true, force: true });

  if (tsrs && o.tsrsTgz) unpackTgz(o.tsrsTgz, join(root, "node_modules/@tsrs", tsrs));

  const build: Layout = {
    package: "@origingame/engine",
    version: o.version,
    ...(o.base ? { base: o.base } : {}),
    revision: o.revision,
    target: platform,
    triple: o.t,
    og: sha256(join(root, "bin", exe)),
    addon: sha256(join(root, "engine/native", `origin-engine.${platform}.node`)),
    wasm: sha256(join(root, "engine/dist/web/origin-engine_bg.wasm")),
    wasmPublish: sha256(join(root, "engine/dist/web-publish/origin-engine_bg.wasm")),
    wasmLanes: sha256(join(root, "engine/dist/web-publish/origin-engine-lanes_bg.wasm")),
  };
  const overlayManifest = JSON.parse(readFileSync(join(root, "overlay/origin-engine.json"), "utf8")) as Record<string, unknown>;
  writeFileSync(join(root, "origin-engine.json"), `${JSON.stringify({ ...overlayManifest, build }, null, 2)}\n`);
  checkLayout(root, o.t);
  checkPacksOutside(root, o.downloadable);
  say(`${o.t}: shared asset hardlinks avoid ${deduplicateAssets(root)} repeated bytes`);

  // One portable archive: ustar/pax entries owned by root, no macOS metadata (bsdtar) or xattrs.
  const archive = join(o.dir, archiveName(o.version, o.t));
  const bsd = execFileSync("tar", ["--version"], { encoding: "utf8" }).includes("bsdtar");
  const flags = bsd ? ["--no-mac-metadata", "--no-xattrs", "--uid", "0", "--gid", "0", "--uname", "root", "--gname", "root"] : ["--owner=0", "--group=0", "--numeric-owner", "--no-xattrs"];
  rmSync(archive, { force: true });
  const r = spawnSync("tar", [...flags, "-czf", archive, "-C", o.work, name], { stdio: ["ignore", "ignore", "inherit"], env: { ...process.env, COPYFILE_DISABLE: "1" } });
  if (r.status !== 0) die(`tar of ${name} failed`);
  rmSync(root, { recursive: true, force: true });
  say(`${basename(archive)}  ${statSync(archive).size} bytes  sha256 ${sha256(archive)}`);
  return build;
}

interface LookFile {
  url: string;
  sha256: string;
  size: number;
}
/** manifest.json's `looks`: the catalogue, and each style's preview with the version that drew it. */
export interface Looks {
  catalogue: LookFile;
  previews: Record<string, LookFile & { engine: string }>;
}
/** What render.ts --record writes for `scripts/release.sh previews`. */
interface RenderRecord {
  engine: string;
  revision: string;
  dirty: boolean;
  tier: string | null;
  og: string;
  addon: string | null;
  previews: Record<string, { sha256: string; size: number }>;
}
/** The tier the previews are drawn on: the GPU host's, which records the reference frames (docs/ci.md §Reference frames). */
const PREVIEW_TIER = "vulkan";
export const RENDER_RECORD = "looks-render.json";

/** One preview a version publishes: create-game's PreviewSlot (bin/creation.ts), or a catalogue look's before it. */
export interface Slot {
  path: string;
  kind: "look" | "variant" | "ui";
  look?: string;
  variant?: string;
  pack?: string;
}
/** What a version publishes of its looks: manifest.json's `looks` and `creation`, and every file they bring, for SHA256SUMS. */
export interface LooksRelease {
  looks: Looks;
  creation: LookFile | null;
  files: string[];
}
/** The files below `root`, relative to it with `/`, sorted. */
function filesUnder(root: string, rel = ""): string[] {
  if (!existsSync(join(root, rel))) return [];
  return readdirSync(join(root, rel), { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? filesUnder(root, `${rel}${e.name}/`) : [`${rel}${e.name}`])).sort();
}

/**
 * Writes `<dir>/looks/catalogue.json` (the engine package's catalogue's
 * bytes) and returns manifest.json's `looks` over it and the previews in
 * `<dir>/previews/`, one per slot (a look's under its slug, a variant's under
 * `<slug>.<variant>`; ui previews are named by creation.json only),
 * after checking them against `<dir>/looks-render.json`: drawn by this
 * version at this revision from a clean templates/looks/, on the recording
 * tier, with the og and addon `linux` names (the linux-x64 archive's sha256),
 * every slot's file the one recorded and no file for no slot. Throws on any
 * difference. `filled` is what creationDocument takes.
 */
export function writeLooks(dir: string, version: string, revision: string, engine: string, linux: { og: string; addon: string }, slots: Slot[]): { looks: Looks; filled: Record<string, LookFile & { engine: string }> } {
  const file = join(dir, RENDER_RECORD);
  const where = "scripts/release.sh previews on the GPU host (docs/release.md §3)";
  if (!existsSync(file)) throw new Error(`no ${RENDER_RECORD} in ${dir}: the previews are drawn by ${where}`);
  const r = JSON.parse(readFileSync(file, "utf8")) as RenderRecord;
  const wrong: string[] = [];
  if (r.engine !== version) wrong.push(`drawn by engine ${r.engine}, not ${version}`);
  if (r.revision !== revision) wrong.push(`drawn at ${r.revision}, not ${revision}`);
  if (r.dirty) wrong.push("drawn from a dirty templates/looks/");
  if (r.tier !== PREVIEW_TIER) wrong.push(`drawn on ${r.tier ?? "no renderer"}, not ${PREVIEW_TIER} (the GPU host)`);
  if (r.og !== linux.og || r.addon !== linux.addon) wrong.push("drawn with another og or addon than the linux-x64 archive's");
  const paths = new Set(slots.map((s) => s.path));
  const extra = filesUnder(join(dir, "previews")).map((f) => `previews/${f}`).filter((f) => !paths.has(f));
  if (extra.length) wrong.push(`${extra.join(", ")}: no preview slot names them`);
  const previews: Looks["previews"] = {};
  const filled: Record<string, LookFile & { engine: string }> = {};
  for (const slot of slots) {
    const at = join(dir, slot.path);
    if (!/^previews\/(?:ui\/)?[a-z0-9-]+(?:\.[a-z0-9-]+)?\.webp$/.test(slot.path)) {
      wrong.push(`${slot.path} is not a preview path the downloads serve`);
      continue;
    }
    if (!existsSync(at)) {
      wrong.push(`no ${slot.path}`);
      continue;
    }
    const digest = sha256(at);
    if (r.previews[slot.path]?.sha256 !== digest) wrong.push(`${slot.path} is not the preview ${RENDER_RECORD} recorded`);
    filled[slot.path] = { url: `${DOWNLOADS}/${version}/${slot.path}`, sha256: digest, size: statSync(at).size, engine: r.engine };
    if (slot.kind === "look" || slot.kind === "variant") previews[slot.variant ? `${slot.look}.${slot.variant}` : slot.look!] = filled[slot.path]!;
  }
  if (wrong.length) throw new Error(`the previews in ${dir} are not this version's (draw them again: ${where}):\n  ${wrong.join("\n  ")}`);
  mkdirSync(join(dir, "looks"), { recursive: true });
  copyFileSync(join(engine, "knowledge/looks/catalogue.json"), join(dir, "looks/catalogue.json"));
  const at = join(dir, "looks/catalogue.json");
  return { looks: { catalogue: { url: `${DOWNLOADS}/${version}/looks/catalogue.json`, sha256: sha256(at), size: statSync(at).size }, previews }, filled };
}

/** Writes `<dir>/creation.json`, the creation document as create-game wrote it, and returns manifest.json's `creation`. */
export function writeCreation(dir: string, version: string, doc: unknown): LookFile {
  const at = join(dir, "creation.json");
  writeFileSync(at, `${JSON.stringify(doc, null, 2)}\n`);
  return { url: `${DOWNLOADS}/${version}/creation.json`, sha256: sha256(at), size: statSync(at).size };
}

/** A version's looks files, as SHA256SUMS lists them. */
const LOOK_FILE = /^(?:looks\/catalogue\.json|previews\/.+\.webp|creation\.json)$/;

/**
 * The preview slots of a packed create-game (`create/bin/create.ts`'s
 * previewSlots over its own catalogue) and its creationDocument; a
 * create-game from before them has a slot per catalogue look and no document.
 */
export async function creationOf(create: string, engine: string): Promise<{ slots: Slot[]; document: ((previews: Record<string, LookFile & { engine: string }>, catalogue: string) => unknown) | null }> {
  const mod = (await import(join(create, "bin/create.ts"))) as { previewSlots?: (catalogue: string) => Slot[]; creationDocument?: (p: object, catalogue: string) => unknown };
  if (mod.previewSlots && mod.creationDocument) return { slots: mod.previewSlots(join(create, "looks/catalogue.json")), document: mod.creationDocument };
  const styles = (JSON.parse(readFileSync(join(engine, "knowledge/looks/catalogue.json"), "utf8")) as { styles: { slug: string }[] }).styles;
  return { slots: styles.map((s) => ({ path: `previews/${s.slug}.webp`, kind: "look", look: s.slug })), document: null };
}

/** Signs SHA256SUMS with the release key (minisign), when OG_ENGINE_MINISIGN_KEY is set; checks the signature. */
function sign(dir: string, version: string, revision: string): boolean {
  const secret = process.env.OG_ENGINE_MINISIGN_KEY?.trim();
  if (!secret) {
    rmSync(join(dir, "SHA256SUMS.minisig"), { force: true });
    say("no OG_ENGINE_MINISIGN_KEY: SHA256SUMS is unsigned (publish refuses it)");
    return false;
  }
  const tmp = mkdtempSync(join(tmpdir(), "og-sign-"));
  try {
    const key = join(tmp, "key");
    writeFileSync(key, `${secret}\n`, { mode: 0o600 });
    const r = spawnSync("minisign", ["-S", "-s", key, "-m", join(dir, "SHA256SUMS"), "-x", join(dir, "SHA256SUMS.minisig"), "-t", `origin-engine ${version} ${revision}`], { input: "", encoding: "utf8" });
    if (r.error) die(`minisign did not start (${r.error.message}): brew install minisign, or apt-get install minisign`);
    if (r.status !== 0) die(`minisign -S failed (OG_ENGINE_MINISIGN_KEY must be an unencrypted key): ${r.stderr.trim()}`);
  } finally {
    rmSync(tmp, { recursive: true, force: true });
  }
  const pub = process.env.OG_ENGINE_MINISIGN_PUBKEY ?? pinnedKey();
  if (!pub) {
    say("signed SHA256SUMS; docs/release.md states no public key yet to check it against");
    return true;
  }
  const v = spawnSync("minisign", ["-V", "-q", "-P", pub, "-m", join(dir, "SHA256SUMS"), "-x", join(dir, "SHA256SUMS.minisig")], { encoding: "utf8" });
  if (v.status !== 0) die(`SHA256SUMS.minisig does not verify with the public key ${pub}: ${v.stderr.trim()}`);
  say(`signed SHA256SUMS; verified with ${pub}`);
  return true;
}

/**
 * What the platform's upload takes beside the archives (docs/release.md §3):
 * manifest.json; the launcher's install.sh and install.ps1, a dev build's
 * too, so `<version>/install.sh` installs a launcher that installs that
 * version (the site's root copies move only with a newer release, and an
 * older launcher refuses a newer version's name); and SHA256SUMS over all
 * of them, signed. A dev build `dev.<date>.<n>` records
 * the release it is based on (`base`) beside its exact `revision`; `looks`
 * (null for a backfill) names the catalogue, previews and creation document. Returns
 * the archives' targets.
 */
export function finish(dir: string, version: string, revision: string, releasedAt: string, vfx: Record<string, VfxEntry>, looks: LooksRelease | null, base?: string): { archives: Target[]; signed: boolean } {
  const archives = (Object.keys(TARGETS) as Target[]).filter((t) => existsSync(join(dir, archiveName(version, t))));
  const channel = isDev(version) ? "dev" : "release";
  if (version.startsWith("dev.") && !(base && /^\d+\.\d+\.\d+$/.test(base) && /^[0-9a-f]{40}$/.test(revision))) die(`${version} must record its base release and its full commit id (base ${base}, revision ${revision})`);
  const manifest = {
    version,
    ...(version.startsWith("dev.") ? { base } : {}),
    releasedAt,
    revision,
    channel,
    checksums: "SHA256SUMS",
    signature: "SHA256SUMS.minisig",
    targets: Object.fromEntries(
      archives.map((t) => {
        const f = join(dir, archiveName(version, t));
        return [t, { url: `${DOWNLOADS}/${version}/${archiveName(version, t)}`, sha256: sha256(f), size: statSync(f).size }];
      }),
    ),
    vfx,
    ...(looks ? { looks: looks.looks } : {}),
    ...(looks?.creation ? { creation: looks.creation } : {}),
  };
  writeFileSync(join(dir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  const files = [...archives.map((t) => archiveName(version, t)), ...Object.keys(vfx).map(archivePath), ...wasmFiles(dir), ...(looks?.files ?? []), "manifest.json"];
  if (!VERSION_RE.test(version)) die(`the launcher the installers carry (packages/launcher/og.ts VERSION_RE) refuses ${version}; a version is published with a launcher that installs it`);
  const { sh, ps1 } = installers();
  writeFileSync(join(dir, "install.sh"), sh, { mode: 0o755 });
  writeFileSync(join(dir, "install.ps1"), ps1);
  files.push("install.sh", "install.ps1");
  writeFileSync(join(dir, "SHA256SUMS"), files.map((f) => `${sha256(join(dir, f))}  ${f}\n`).join(""));
  for (const t of archives) {
    const size = statSync(join(dir, archiveName(version, t))).size;
    if (size > UPLOAD_LIMIT) die(`${archiveName(version, t)} is ${size} bytes; the platform's upload takes at most ${UPLOAD_LIMIT}`);
  }
  return { archives, signed: sign(dir, version, revision) };
}

/**
 * The engine wasm's split modules (docs/web-export.md §Modules), served from
 * the version's directory: each `dist/<host>/modules[-lanes]/` of the engine
 * package under `wasm/<variant>/<module>.<hash12>.wasm` (the platform's
 * route), its variant as scripts/wasm-crates.txt names it. Each listing in
 * the package gets the `base` they are served from, so a game built with this
 * engine fetches them there (og build's default). Returns the files written,
 * relative to `dir`.
 */
export const WASM_HOSTS: Record<string, string> = {
  "web-publish/modules": "publish",
  "web-publish/modules-lanes": "lanes",
};
export function writeWasmModules(engine: string, dir: string, version: string): string[] {
  rmSync(join(dir, "wasm"), { recursive: true, force: true });
  const files: string[] = [];
  for (const [from, variant] of Object.entries(WASM_HOSTS)) {
    const listing = join(engine, "dist", from, "modules.json");
    if (!existsSync(listing)) continue;
    const listed = JSON.parse(readFileSync(listing, "utf8")) as Record<string, { file: string; bytes: number; sha256: string; base?: string }>;
    const base = `${DOWNLOADS}/${version}/wasm/${variant}/`;
    for (const [m, entry] of Object.entries(listed)) {
      const source = join(engine, "dist", dirname(from), entry.file);
      if (sha256(source) !== entry.sha256) die(`${source} is not the module ${listing} names`);
      const name = `wasm/${variant}/${basename(entry.file)}`;
      if (!/^wasm\/[a-z0-9-]+\/[a-z0-9]+\.[0-9a-f]{12}\.wasm$/.test(name)) die(`${name} is not a name the platform's wasm route serves`);
      mkdirSync(join(dir, "wasm", variant), { recursive: true });
      copyFileSync(source, join(dir, name));
      files.push(name);
      listed[m] = { ...entry, base };
    }
    writeFileSync(listing, `${JSON.stringify(listed, null, 2)}\n`);
  }
  return files.sort();
}

/** The `wasm/<variant>/*.wasm` files of a version's directory, sorted. */
function wasmFiles(dir: string): string[] {
  if (!existsSync(join(dir, "wasm"))) return [];
  return readdirSync(join(dir, "wasm")).flatMap((v) => readdirSync(join(dir, "wasm", v)).map((f) => `wasm/${v}/${f}`)).sort();
}

/** The platform ingress's request body limit (210 MiB). */
const UPLOAD_LIMIT = 210 * 1024 * 1024;

/** `--sign`: manifest.json, the installers and SHA256SUMS again over the archives in place, signed; release.json keeps its passes. */
function signAgain(dir: string, version: string) {
  const recordFile = join(dir, "release.json");
  if (!existsSync(recordFile)) die(`no ${recordFile}: archive the version first (scripts/release.sh pack)`);
  const record = JSON.parse(readFileSync(recordFile, "utf8")) as { version: string; base?: string; revision: string; releasedAt?: string; signed?: boolean };
  if (record.version !== version) die(`${recordFile} is version ${record.version}`);
  const releasedAt = record.releasedAt ?? (JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as { releasedAt: string }).releasedAt;
  // The archives must be the ones the install test passed: the sums of the last run name them.
  const before = new Map(readFileSync(join(dir, "SHA256SUMS"), "utf8").split("\n").filter(Boolean).map((l) => [l.split(/\s+/)[1]!, l.split(/\s+/)[0]!] as const));
  for (const t of Object.keys(TARGETS) as Target[]) {
    const f = archiveName(version, t);
    if (existsSync(join(dir, f)) && before.get(f) !== sha256(join(dir, f))) die(`${f} is not the archive SHA256SUMS named; archive again (and verify) instead`);
  }
  // The split modules in place, as the last SHA256SUMS named them.
  for (const f of wasmFiles(dir)) if (before.get(f) !== sha256(join(dir, f))) die(`${f} is not the module SHA256SUMS named; archive again (and verify) instead`);
  // The vfx archives in place, as the last manifest named them.
  const vfx = (JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as { vfx?: Record<string, VfxEntry> }).vfx ?? {};
  for (const name of Object.keys(vfx)) {
    const f = archivePath(name);
    if (!existsSync(join(dir, f)) || before.get(f) !== sha256(join(dir, f)) || vfx[name]!.sha256 !== before.get(f)) die(`${f} is not the archive SHA256SUMS and manifest.json named; archive again (and verify) instead`);
  }
  // The catalogue and previews in place, as the last manifest named them.
  const prior = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as { looks?: Looks; creation?: LookFile };
  const files = [...before.keys()].filter((f) => LOOK_FILE.test(f)).sort();
  for (const f of files) if (!existsSync(join(dir, f)) || before.get(f) !== sha256(join(dir, f))) die(`${f} is not the file SHA256SUMS named; archive again (and verify) instead`);
  const looks: LooksRelease | null = prior.looks ? { looks: prior.looks, creation: prior.creation ?? null, files } : null;
  const { archives, signed } = finish(dir, version, record.revision, releasedAt, vfx, looks, record.base);
  writeFileSync(recordFile, `${JSON.stringify({ ...record, releasedAt, targets: archives, signed }, null, 2)}\n`);
  say(`${version}: SHA256SUMS over ${archives.length} archive(s), manifest.json, install.sh, install.ps1${signed ? "; signed" : "; unsigned"}`);
}

async function main() {
  const argv = process.argv.slice(2);
  const opt = (n: string) => {
    const i = argv.indexOf(n);
    return i >= 0 ? argv[i + 1] : undefined;
  };
  const dirArg = argv.find((a, i) => !a.startsWith("--") && !["--version", "--revision", "--targets"].includes(argv[i - 1] ?? ""));
  const version = opt("--version");
  if (!dirArg || !version) die("usage: bun scripts/release-dist.ts <dir> --version <v> [--registry] [--revision <sha>] [--targets <t,...>] | --sign");
  if (!VERSION_RE.test(version)) die(`${version} is not a release (x.y.z) or a dev build (dev.YYYYMMDD.n)`);
  const dir = resolve(dirArg);
  if (argv.includes("--sign")) return signAgain(dir, version);
  mkdirSync(dir, { recursive: true });
  // Archiving again (after the win32-x64 tarball came in, or to sign) keeps what the first run recorded.
  const recordFile = join(dir, "release.json");
  const prior = existsSync(recordFile) ? (JSON.parse(readFileSync(recordFile, "utf8")) as { version: string; revision: string; dirty: boolean; source: string; verified?: unknown }) : undefined;
  const again = prior?.version === version ? prior : undefined;
  const registry = argv.includes("--registry") || again?.source === "registry";

  const tgz = (name: string) => join(dir, `origingame-${name}-${version}.tgz`);
  let releasedAt = new Date().toISOString();
  if (registry) {
    const doc = await registryDoc("@origingame/engine");
    releasedAt = doc.time[version] ?? die(`@origingame/engine@${version} is not on the registry`);
    for (const name of ["engine", "create-game", ...Object.values(TARGETS).map((t) => `engine-${t.platform}`)]) await fetchTarball(`@origingame/${name}`, version, tgz(name));
  }
  let targets = (Object.keys(TARGETS) as Target[]).filter((t) => existsSync(tgz(`engine-${TARGETS[t].platform}`)));
  const only = opt("--targets");
  if (only) {
    const want = only.split(",") as Target[];
    for (const t of want) if (!(t in TARGETS)) die(`unknown target ${t} (${Object.keys(TARGETS).join(", ")})`);
    targets = targets.filter((t) => want.includes(t));
  }
  for (const n of ["engine", "create-game"]) if (!existsSync(tgz(n))) die(`no ${tgz(n)}`);
  if (!targets.length) die(`no platform tarball in ${dir} (origingame-engine-<platform>-${version}.tgz)`);

  // Packed here from the checkout (pack), or elsewhere (a workflow run: --revision), or published to npm (--registry).
  const checkoutVersion = (JSON.parse(readFileSync(join(ROOT, "packages/engine/package.json"), "utf8")) as { version: string }).version;
  const here = !registry && !opt("--revision") && version === checkoutVersion;
  const revision = opt("--revision") ?? again?.revision ?? (here ? git("rev-parse", "HEAD") : revisionOf(version));
  // Pinned assets are ignored by Git, so a pinned file that is not its pin is dirty too (scripts/assets.ts status).
  const dirty = again ? again.dirty : here && (git("status", "--porcelain") !== ""
    || spawnSync("bun", [join(ROOT, "scripts/assets.ts"), "status"], { cwd: ROOT, stdio: ["ignore", "ignore", "inherit"] }).status !== 0);
  const work = mkdtempSync(join(dir, ".work-"));
  try {
    // What every target shares, unpacked once.
    const common = join(work, "common");
    mkdirSync(common, { recursive: true });
    unpackTgz(tgz("engine"), join(common, "engine"));
    const engineJson = JSON.parse(readFileSync(join(common, "engine/package.json"), "utf8")) as { version: string; optionalDependencies?: Record<string, string> };
    if (engineJson.version !== version) die(`${tgz("engine")} is version ${engineJson.version}`);
    // A dev build carries the changelog up to the release it is based on, its own changes in unreleased.
    const base = version.startsWith("dev.") ? (JSON.parse(readFileSync(join(common, "engine/changelog.json"), "utf8")) as { versions: { version: string }[] }).versions[0]?.version : undefined;
    unpackTgz(tgz("create-game"), join(common, "create"));
    if (existsSync(tgz("design"))) unpackTgz(tgz("design"), join(common, "design"));
    await writeOverlay(join(common, "engine"), join(common, "overlay"));
    const distribution = JSON.parse(readFileSync(join(common, "overlay/distribution.json"), "utf8")) as { files: { path: string; sha256?: string }[] };
    for (const f of distribution.files) {
      const source = join(common, "create/template", f.path);
      if (!existsSync(source)) die(`create-game@${version} has no template/${f.path}, which the overlay's distribution.json names`);
      const digest = sha256(source);
      if (f.sha256 && digest !== f.sha256) die(`template/${f.path} is ${digest}; distribution.json pins ${f.sha256}`);
      mkdirSync(dirname(join(common, "files", f.path)), { recursive: true });
      copyFileSync(source, join(common, "files", f.path));
    }
    mkdirSync(join(common, "files"), { recursive: true });
    const vfx = writeVfxArchives(dir, version, vfxSource(revision, here, work));
    // Before the archives are assembled: each target's engine names where its modules are served.
    const wasm = writeWasmModules(join(common, "engine"), dir, version);
    if (wasm.length) say(`${wasm.length} split engine module(s) under wasm/`);
    const downloadable = Object.keys(vfx);

    const builds: Partial<Record<Target, Layout>> = {};
    for (const t of targets) {
      const tsrs = TARGETS[t].tsrs;
      const tsrsVersion = tsrs ? engineJson.optionalDependencies?.[`@tsrs/${tsrs}`] : undefined;
      let tsrsTgz: string | null = null;
      if (tsrs && tsrsVersion) {
        tsrsTgz = join(dir, ".tsrs", `tsrs-${tsrs}-${tsrsVersion}.tgz`);
        await fetchTarball(`@tsrs/${tsrs}`, tsrsVersion, tsrsTgz);
      } else if (tsrs) say(`@origingame/engine@${version} names no @tsrs/${tsrs}: the ${t} archive has no tsrs`);
      builds[t] = await assemble({ dir, work, version, base, revision, t, common, tsrsTgz, downloadable });
    }

    // The previews must be this version's own (writeLooks); a backfill ships none. The Mac's pack
    // archives before the GPU host drew them: archived again after, and publish refuses a build without.
    let looks: LooksRelease | null = null;
    if (!registry && !existsSync(join(dir, RENDER_RECORD))) say(`no previews yet: scripts/release.sh previews ${version} on the GPU host, then archive again (publish refuses until then)`);
    else if (!registry) {
      const linux = builds["x86_64-unknown-linux-gnu"] ?? die("the previews are checked against the linux-x64 archive's og and addon: archive it too");
      const { slots, document } = await creationOf(join(common, "create"), join(common, "engine"));
      try {
        const { looks: manifest, filled } = writeLooks(dir, version, revision, join(common, "engine"), linux, slots);
        rmSync(join(dir, "creation.json"), { force: true });
        const creation = document ? writeCreation(dir, version, document(filled, join(common, "create/looks/catalogue.json"))) : null;
        looks = { looks: manifest, creation, files: ["looks/catalogue.json", ...slots.map((s) => s.path).sort(), ...(creation ? ["creation.json"] : [])] };
      } catch (e) {
        die((e as Error).message);
      }
      say(`looks/catalogue.json, ${slots.length} previews drawn by ${version} on ${PREVIEW_TIER}${looks.creation ? ", creation.json" : " (this create-game has no creation document)"}`);
    }
    const { archives, signed } = finish(dir, version, revision, releasedAt, vfx, looks, base);
    // The local record publish reads; the install test adds its passes (new archives clear them).
    writeFileSync(recordFile, `${JSON.stringify({ version, ...(base ? { base } : {}), revision, releasedAt, dirty, source: registry ? "registry" : "build", builtAt: new Date().toISOString(), targets: archives, signed, looks: looks !== null, verified: {} }, null, 2)}\n`);
    if (dirty) say("note: the working tree is dirty; publish refuses these until a clean tree is packed");
    say(`${version}: ${archives.length} archive(s) (${archives.join(", ")}), manifest.json, SHA256SUMS${signed ? ", SHA256SUMS.minisig" : ""}`);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

if (import.meta.main) await main();
