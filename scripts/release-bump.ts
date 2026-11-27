// The one version of the five packages, written everywhere it occurs or
// checked equal everywhere (docs/release.md §3).
//
//   bun scripts/release-bump.ts <version>   write it
//   bun scripts/release-bump.ts --dev dev.<YYYYMMDD>.<n>
//                                           write the dev build dev.<YYYYMMDD>.<n> (the n-th of that UTC day; docs/release.md §1a)
//                                           over the release the checkout is at, the fragments folded into unreleased; the
//                                           release workflow's dev run does this in its checkout, never committed
//   bun scripts/release-bump.ts --next-dev  print the next dev build of today (UTC): dev.<YYYYMMDD>.<n>, n one past the
//                                           last of that day the downloads publish ($ORIGIN_ENGINE_DOWNLOADS, default
//                                           https://origingame.ai/downloads/engine); two runs before a publish get the same name,
//                                           and the platform publishes only the first
//   bun scripts/release-bump.ts --check     exit 1 naming every occurrence that differs from packages/engine's
//   bun scripts/release-bump.ts --print     print the current version
//
// What carries the version: the three package.json of packages/ (and
// @origingame/engine's optionalDependencies on its platform packages), the
// scaffold's origin-engine.json (engine.version and template.version), every
// project's og-project.json engine and `"@origingame/engine": "<v>"` dependency
// (templates/, the former test sample tests/fixtures/ and the benches),
// and the workspace
// crates' Cargo.toml (og --version prints og-cli's). Cargo.lock
// follows through `cargo update --workspace --offline`, run by release.sh.
// Cargo takes only semver, so a dev build's crates carry it as build
// metadata of the release it is based on (`0.9.0+dev.20261119.1`); that
// base is the newest version in changelog.json, which a dev build never
// adds to.
//
// packages/engine/changelog.json (the one changelog, which the platform
// reads): a bump first folds changelog/unreleased/'s fragments into
// `unreleased` (scripts/changelog.ts), then moves `unreleased` into `versions` as
// the new version, dated today (UTC), its kind from the numbers (patch in
// the same minor, minor otherwise). A patch may carry breaks: its breaking
// list says what breaks; --check requires an entry for the current version.

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = join(import.meta.dir, "..");
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

function read(p: string): string {
  return readFileSync(join(ROOT, p), "utf8");
}

function current(): string {
  return (JSON.parse(read("packages/engine/package.json")) as { version: string }).version;
}

/** Every file under `dir` (relative to ROOT) whose name passes `keep`, skipping build output. */
function walk(dir: string, keep: (name: string) => boolean, out: string[] = []): string[] {
  const full = join(ROOT, dir);
  if (!existsSync(full)) return out;
  for (const name of readdirSync(full).sort()) {
    if (["node_modules", "target", "dist", "cooked", "og-out", ".og", "references", "pkg", "scaffold"].includes(name)) continue;
    const p = join(dir, name);
    if (statSync(join(ROOT, p)).isDirectory()) walk(p, keep, out);
    else if (keep(name)) out.push(p);
  }
  return out;
}

interface Site {
  readonly path: string;
  /** Finds the version(s) in the text; replaces them. */
  readonly re: RegExp;
}

function sites(): Site[] {
  const s: Site[] = [];
  for (const p of ["packages/engine/package.json", "packages/create/package.json", "packages/design/package.json", "packages/knowledge/package.json"]) {
    s.push({ path: p, re: /("version":\s*")([^"]+)(")/g });
  }
  s.push({ path: "packages/engine/package.json", re: /("@origingame\/engine-(?:darwin-arm64|linux-x64|win32-x64)":\s*")([^"]+)(")/g });
  s.push({ path: "packages/engine/package.json", re: /("@origingame\/create-game":\s*")([^"]+)(")/g });
  s.push({ path: "packages/knowledge/scaffold/origin-engine.json", re: /("version":\s*")([^"]+)(")/g });
  // Projects: the authoritative JSON pin and package.json's dependency.
  const projectDirs = ["packages/create/template", "packages/engine/sheets", "templates", "tests/fixtures", "crates/og-tests/reference-scene", "packages/engine/tests/fixtures", "showcase"];
  for (const d of projectDirs) {
    for (const p of walk(d, (n) => n === "og-project.json")) s.push({ path: p, re: /("engine":\s*")([^"]+)(")/g });
    for (const p of walk(d, (n) => n === "package.json")) {
      if (read(p).includes('"@origingame/engine"')) s.push({ path: p, re: /("@origingame\/engine":\s*")([^"]+)(")/g });
    }
  }
  // The workspace crates.
  for (const p of walk("crates", (n) => n === "Cargo.toml")) {
    if (/^version = "/m.test(read(p))) s.push({ path: p, re: /(^version = ")([^"]+)(")/m });
  }
  return s;
}

import { type Changelog, type Entry as ChangeEntry, CHANGELOG, check as checkFragments, fold, readLog, writeLog } from "./changelog.ts";

const minorOf = (v: string) => v.split(".").slice(0, 2).join(".");
/** A dev build `dev.<YYYYMMDD>.<n>`, or one of the scheme before 0.9.0, `x.y.z-dev.n`. */
const isDev = (v: string) => v.startsWith("dev.") || /-dev\.\d+$/.test(v);
const DATED = /^dev\.(\d{4})(\d{2})(\d{2})\.[1-9]\d*$/;

/** Whether `v` is dev.<YYYYMMDD>.<n> with a real date. */
function datedDev(v: string): boolean {
  const m = DATED.exec(v);
  if (!m) return false;
  const d = new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3])));
  return d.getUTCFullYear() === Number(m[1]) && d.getUTCMonth() === Number(m[2]) - 1 && d.getUTCDate() === Number(m[3]);
}

/** What the crates' Cargo.toml carry for `v`: a dated dev build as build metadata of the release it is based on. */
const cargoVersion = (v: string, base: string) => (v.startsWith("dev.") ? `${base}+${v}` : v);

/** Problems with the changelog for `version` (an entry, well formed, its fragments too); empty when fine. */
function changelogProblems(log: Changelog, version: string): string[] {
  const out: string[] = [];
  // A dev build (`--dev`) carries unreleased: it has no entry of its own.
  if (!log.versions.some((e) => e.version === version) && !isDev(version)) out.push(`${CHANGELOG}: no entry for ${version}`);
  out.push(...checkFragments());
  return out;
}

/** Moves the unreleased entry into versions as `target`. */
function releaseChangelog(now: string, target: string): string | null {
  const problems = checkFragments();
  if (problems.length) return `the changelog fragments do not fold:\n${problems.map((p) => `  ${p}`).join("\n")}`;
  if (readLog().versions.some((e) => e.version === target)) return null;
  const kind = minorOf(now) === minorOf(target) ? "patch" : "minor";
  fold();
  const log = readLog();
  const entry: ChangeEntry = { version: target, date: new Date().toISOString().slice(0, 10), kind, ...log.unreleased };
  log.versions.unshift(entry);
  log.unreleased = { simulation: "unchanged", breaking: [], added: [], fixed: [] };
  writeLog(log);
  return null;
}

/** The next dev build of today (UTC) the downloads have not published. */
async function nextDev(): Promise<string> {
  const base = (process.env.ORIGIN_ENGINE_DOWNLOADS || "https://origingame.ai/downloads/engine").replace(/\/+$/, "");
  const day = new Date().toISOString().slice(0, 10).replaceAll("-", "");
  for (let n = 1; ; n++) {
    const r = await fetch(`${base}/dev.${day}.${n}/manifest.json`, { method: "HEAD" });
    if (r.status === 404) return `dev.${day}.${n}`;
    if (!r.ok) throw new Error(`${base}/dev.${day}.${n}/manifest.json answered ${r.status}`);
  }
}

function main(args: string[]): number {
  const dev = args.includes("--dev");
  const want = args.find((a) => !a.startsWith("--"));
  const check = args.includes("--check");
  const now = current();
  if (args.includes("--print")) {
    process.stdout.write(`${now}\n`);
    return 0;
  }
  if (dev ? !datedDev(want ?? "") : !check && (!want || !SEMVER.test(want))) {
    process.stderr.write("usage: bun scripts/release-bump.ts <version> | --dev dev.<YYYYMMDD>.<n> | --check | --print\n");
    return 2;
  }
  const target = check ? now : want!;
  // The release a dated dev build is based on: the newest in the changelog, where the checkout is.
  const base = readLog().versions[0]?.version ?? "";
  if (dev && (isDev(now) || now !== base)) {
    process.stderr.write(`release-bump: a dev build is made over a release; the checkout is at ${now}, the changelog's newest release is ${base}\n`);
    return 1;
  }
  if (dev) {
    // The dev package ships changelog.json with what the merges recorded since the release.
    const folded = fold();
    if (folded.problems.length) {
      process.stderr.write(`release-bump: the changelog fragments do not fold:\n${folded.problems.map((p) => `  ${p}`).join("\n")}\n`);
      return 1;
    }
  }
  if (!check && !dev) {
    const refused = releaseChangelog(now, target);
    if (refused) {
      process.stderr.write(`release-bump: ${refused}\n`);
      return 1;
    }
  }
  const wrong: string[] = [];
  let changed = 0;
  for (const site of sites()) {
    const before = read(site.path);
    const want = site.path.endsWith("Cargo.toml") ? cargoVersion(target, base) : target;
    let hit = 0;
    const after = before.replace(site.re, (_m, a: string, v: string, c: string) => {
      hit++;
      if (v !== want) wrong.push(`${site.path}: ${v}`);
      return `${a}${want}${c}`;
    });
    if (hit === 0) wrong.push(`${site.path}: no version found by ${site.re}`);
    if (!check && after !== before) {
      writeFileSync(join(ROOT, site.path), after);
      changed++;
    }
  }
  if (check) {
    wrong.push(...changelogProblems(readLog(), now));
    if (wrong.length) {
      process.stderr.write(`release-bump --check: ${wrong.length} occurrence(s) differ from packages/engine's ${now}:\n${wrong.map((w) => `  ${w}`).join("\n")}\n`);
      return 1;
    }
    process.stdout.write(`release-bump --check: every occurrence is ${now}\n`);
    return 0;
  }
  process.stdout.write(`release-bump: ${now} -> ${target}, ${changed} file(s) written (${wrong.length} occurrence(s) moved); run \`cargo update --workspace --offline\` for Cargo.lock\n`);
  return 0;
}

if (process.argv.includes("--next-dev")) process.stdout.write(`${await nextDev()}\n`);
else process.exit(main(process.argv.slice(2)));
