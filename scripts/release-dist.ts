// The download channel's files of one version (docs/release.md §1, §3):
// per target an archive `origin-engine-<v>-<target>.tar.gz` of the dist
// layout, then `manifest.json`, for a release (not a dev build) the
// launcher's `install.sh` and `install.ps1`, `SHA256SUMS` over all of them
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
//   origin-engine.json   the scaffold overlay's origin-engine.json + build {package, version, revision, target, triple,
//                        og, addon, wasm, wasmPublish, wasmPublish2d, wasmLanes}; `target` is the node platform
//                        (linux-x64, darwin-arm64, win32-x64), `triple` the archive's target
//   overlay/             the engine knowledge's game-scaffold/origin-engine-ts
//   files/<path>         the overlay's distribution.json binaries, from create-game's template/, sha256-verified
//   archetypes/<name>/   every archetype create-game ships, as stored (`gitignore`, `gitattributes`)
//   bin/og, bin/og.exe   the og tool
//   engine/              @origingame/engine as packed, with native/origin-engine.<platform>.node
//   create/              @origingame/create-game as packed (`og new` runs its bin/create.ts; brief.ts)
//   design/              @origingame/design as packed (when <dir> has it; npm never had it)
//   node_modules/@tsrs/<tsrs platform>/   the type checker og check finds beside the engine (none on Windows: tsgo)
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
import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { installers, pinnedKey } from "../packages/launcher/build.ts";

export const TARGETS = {
  "x86_64-unknown-linux-gnu": { platform: "linux-x64", tsrs: "linux-x64-gnu", exe: "og" },
  "aarch64-apple-darwin": { platform: "darwin-arm64", tsrs: "darwin-arm64", exe: "og" },
  "x86_64-pc-windows-msvc": { platform: "win32-x64", tsrs: null, exe: "og.exe" },
} as const;
export type Target = keyof typeof TARGETS;
export const VERSION_RE = /^\d+\.\d+\.\d+(-dev\.\d+)?$/;
export const DOWNLOADS = "https://origingame.ai/downloads/engine";
const OVERLAY = "game-scaffold/origin-engine-ts";
const ARCHETYPES = ["runner", "match3", "topdown"];
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

/** The commit a version was built from: a release's tag, a dev build's release.yml run (gh). */
function revisionOf(version: string): string {
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
  revision: string;
  target: string;
  triple: Target;
  og: string;
  addon: string;
  wasm: string;
  wasmPublish: string;
  wasmPublish2d: string;
  wasmLanes: string;
}

/** Writes the scaffold overlay of an unpacked engine package (its knowledge, through its own lib.ts) into `to`. */
async function writeOverlay(engine: string, to: string, overlay = OVERLAY) {
  const lib = join(engine, "knowledge/scripts/lib.ts");
  if (!existsSync(lib)) die(`${engine} carries no knowledge/scripts/lib.ts`);
  const { defaultRoots, payload } = (await import(lib)) as {
    defaultRoots(pkg: string): unknown;
    payload(roots: unknown): { path: string; bytes: Uint8Array; symlink: boolean }[];
  };
  const prefix = `${overlay}/`;
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
    "engine/dist/web-publish/origin-engine_bg.wasm",
    "engine/dist/web-publish-2d/origin-engine_bg.wasm",
    "engine/dist/web-publish/origin-engine-lanes_bg.wasm",
    "engine/dist/web-publish/origin-engine-lanes.js",
    `engine/native/origin-engine.${platform}.node`,
    "create/bin/create.ts",
    ...(tsrs ? [`node_modules/@tsrs/${tsrs}/${exe === "og.exe" ? "tsrs.exe" : "tsrs"}`] : []),
    ...ARCHETYPES.map((a) => `archetypes/${a}/project.ogt`),
  ];
  const missing = need.filter((p) => !existsSync(join(dir, p)));
  if (!statSync(join(dir, "engine/dist/web-publish/snippets"), { throwIfNoEntry: false })?.isDirectory()) missing.push("engine/dist/web-publish/snippets/");
  if (missing.length) die(`${dir} lacks ${missing.join(", ")}`);
}

/** Assembles one target's layout under `work` and packs it into `<dir>/<archive>`. */
async function assemble(o: { dir: string; work: string; version: string; revision: string; t: Target; common: string; tsrsTgz: string | null }): Promise<Layout> {
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
  if (existsSync(join(o.common, "overlay-2d"))) {
    copyTree(join(o.common, "overlay-2d"), join(root, "overlay-2d"));
    copyTree(join(o.common, "files-2d"), join(root, "files-2d"));
  }
  copyTree(join(o.common, "archetypes"), join(root, "archetypes"));

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
    revision: o.revision,
    target: platform,
    triple: o.t,
    og: sha256(join(root, "bin", exe)),
    addon: sha256(join(root, "engine/native", `origin-engine.${platform}.node`)),
    wasm: sha256(join(root, "engine/dist/web/origin-engine_bg.wasm")),
    wasmPublish: sha256(join(root, "engine/dist/web-publish/origin-engine_bg.wasm")),
    wasmPublish2d: sha256(join(root, "engine/dist/web-publish-2d/origin-engine_bg.wasm")),
    wasmLanes: sha256(join(root, "engine/dist/web-publish/origin-engine-lanes_bg.wasm")),
  };
  const overlayManifest = JSON.parse(readFileSync(join(root, "overlay/origin-engine.json"), "utf8")) as Record<string, unknown>;
  writeFileSync(join(root, "origin-engine.json"), `${JSON.stringify({ ...overlayManifest, build }, null, 2)}\n`);
  checkLayout(root, o.t);

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
 * manifest.json; for a release (never a dev build) the launcher's install.sh
 * and install.ps1, which move on the site only with a newer release; and
 * SHA256SUMS over all of them, signed. Returns the archives' targets.
 */
function finish(dir: string, version: string, revision: string, releasedAt: string): { archives: Target[]; signed: boolean } {
  const archives = (Object.keys(TARGETS) as Target[]).filter((t) => existsSync(join(dir, archiveName(version, t))));
  const channel = version.includes("-dev.") ? "dev" : "release";
  const manifest = {
    version,
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
  };
  writeFileSync(join(dir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  const files = [...archives.map((t) => archiveName(version, t)), "manifest.json"];
  rmSync(join(dir, "install.sh"), { force: true });
  rmSync(join(dir, "install.ps1"), { force: true });
  if (channel === "release") {
    const { sh, ps1 } = installers();
    writeFileSync(join(dir, "install.sh"), sh, { mode: 0o755 });
    writeFileSync(join(dir, "install.ps1"), ps1);
    files.push("install.sh", "install.ps1");
  }
  writeFileSync(join(dir, "SHA256SUMS"), files.map((f) => `${sha256(join(dir, f))}  ${f}\n`).join(""));
  for (const t of archives) {
    const size = statSync(join(dir, archiveName(version, t))).size;
    if (size > UPLOAD_LIMIT) die(`${archiveName(version, t)} is ${size} bytes; the platform's upload takes at most ${UPLOAD_LIMIT}`);
  }
  return { archives, signed: sign(dir, version, revision) };
}

/** The platform ingress's request body limit (210 MiB). */
const UPLOAD_LIMIT = 210 * 1024 * 1024;

/** `--sign`: manifest.json, the installers and SHA256SUMS again over the archives in place, signed; release.json keeps its passes. */
function signAgain(dir: string, version: string) {
  const recordFile = join(dir, "release.json");
  if (!existsSync(recordFile)) die(`no ${recordFile}: archive the version first (scripts/release.sh pack)`);
  const record = JSON.parse(readFileSync(recordFile, "utf8")) as { version: string; revision: string; releasedAt?: string; signed?: boolean };
  if (record.version !== version) die(`${recordFile} is version ${record.version}`);
  const releasedAt = record.releasedAt ?? (JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8")) as { releasedAt: string }).releasedAt;
  // The archives must be the ones the install test passed: the sums of the last run name them.
  const before = new Map(readFileSync(join(dir, "SHA256SUMS"), "utf8").split("\n").filter(Boolean).map((l) => [l.split(/\s+/)[1]!, l.split(/\s+/)[0]!] as const));
  for (const t of Object.keys(TARGETS) as Target[]) {
    const f = archiveName(version, t);
    if (existsSync(join(dir, f)) && before.get(f) !== sha256(join(dir, f))) die(`${f} is not the archive SHA256SUMS named; archive again (and verify) instead`);
  }
  const { archives, signed } = finish(dir, version, record.revision, releasedAt);
  writeFileSync(recordFile, `${JSON.stringify({ ...record, releasedAt, targets: archives, signed }, null, 2)}\n`);
  say(`${version}: SHA256SUMS over ${archives.length} archive(s), manifest.json${version.includes("-dev.") ? "" : ", install.sh, install.ps1"}${signed ? "; signed" : "; unsigned"}`);
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
  if (!VERSION_RE.test(version)) die(`${version} is not a release (x.y.z) or a dev build (x.y.z-dev.n)`);
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
  const dirty = again ? again.dirty : here && git("status", "--porcelain") !== "";
  const work = mkdtempSync(join(dir, ".work-"));
  try {
    // What every target shares, unpacked once.
    const common = join(work, "common");
    mkdirSync(common, { recursive: true });
    unpackTgz(tgz("engine"), join(common, "engine"));
    const engineJson = JSON.parse(readFileSync(join(common, "engine/package.json"), "utf8")) as { version: string; optionalDependencies?: Record<string, string> };
    if (engineJson.version !== version) die(`${tgz("engine")} is version ${engineJson.version}`);
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
    // Old versions remain backfillable; new versions carry both profile overlays.
    const knowledge = JSON.parse(readFileSync(join(common, "engine/knowledge/knowledge.json"), "utf8"));
    if (knowledge.overlays?.["2d"]) {
      await writeOverlay(join(common, "engine"), join(common, "overlay-2d"), `game-scaffold/${knowledge.overlays["2d"]}`);
      const dist2d = JSON.parse(readFileSync(join(common, "overlay-2d/distribution.json"), "utf8")) as { files: { path: string; sha256: string }[] };
      mkdirSync(join(common, "files-2d"), { recursive: true });
      for (const f of dist2d.files) {
        const source = join(common, "create/template-2d", f.path);
        if (!existsSync(source) || sha256(source) !== f.sha256) die(`template-2d/${f.path} does not match distribution.json`);
        mkdirSync(dirname(join(common, "files-2d", f.path)), { recursive: true });
        copyFileSync(source, join(common, "files-2d", f.path));
      }
    }
    for (const a of readdirSync(join(common, "create/archetypes"))) {
      if (existsSync(join(common, "create/archetypes", a, "project.ogt"))) copyTree(join(common, "create/archetypes", a), join(common, "archetypes", a));
    }

    const builds: Partial<Record<Target, Layout>> = {};
    for (const t of targets) {
      const tsrs = TARGETS[t].tsrs;
      const tsrsVersion = tsrs ? engineJson.optionalDependencies?.[`@tsrs/${tsrs}`] : undefined;
      let tsrsTgz: string | null = null;
      if (tsrs && tsrsVersion) {
        tsrsTgz = join(dir, ".tsrs", `tsrs-${tsrs}-${tsrsVersion}.tgz`);
        await fetchTarball(`@tsrs/${tsrs}`, tsrsVersion, tsrsTgz);
      } else if (tsrs) say(`@origingame/engine@${version} names no @tsrs/${tsrs}: the ${t} archive has no tsrs`);
      builds[t] = await assemble({ dir, work, version, revision, t, common, tsrsTgz });
    }

    const { archives, signed } = finish(dir, version, revision, releasedAt);
    // The local record publish reads; the install test adds its passes (new archives clear them).
    writeFileSync(recordFile, `${JSON.stringify({ version, revision, releasedAt, dirty, source: registry ? "registry" : "build", builtAt: new Date().toISOString(), targets: archives, signed, verified: {} }, null, 2)}\n`);
    if (dirty) say("note: the working tree is dirty; publish refuses these until a clean tree is packed");
    say(`${version}: ${archives.length} archive(s) (${archives.join(", ")}), manifest.json, SHA256SUMS${signed ? ", SHA256SUMS.minisig" : ""}`);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

if (import.meta.main) await main();
