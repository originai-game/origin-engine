// The one version of the five packages, written everywhere it occurs or
// checked equal everywhere (docs/release.md §3).
//
//   bun scripts/release-bump.ts <version>   write it
//   bun scripts/release-bump.ts --check     exit 1 naming every occurrence that differs from packages/engine's
//   bun scripts/release-bump.ts --print     print the current version
//
// What carries the version: the three package.json of packages/ (and
// @origingame/engine's optionalDependencies on its platform packages), the
// scaffold's origin-engine.json (engine.version and template.version), every
// project's `engine "<v>"` line and `"@origingame/engine": "<v>"` dependency
// (templates/, templates/archetypes/, examples/, the test fixtures and
// benches), the inline project.ogt texts of the tests and scripts, and the
// workspace crates' Cargo.toml (og --version prints og-cli's). Cargo.lock
// follows through `cargo update --workspace --offline`, run by release.sh.
//
// packages/engine/changelog.json (the machine-readable changelog the
// platform reads): a bump moves its `unreleased` entry into `versions` as
// the new version, dated today (UTC), its kind from the numbers (patch in
// the same minor, minor otherwise), and refuses a patch that carries a
// break; --check requires an entry for the current version.

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
  for (const p of ["packages/engine/package.json", "packages/create/package.json", "packages/knowledge/package.json"]) {
    s.push({ path: p, re: /("version":\s*")([^"]+)(")/g });
  }
  s.push({ path: "packages/engine/package.json", re: /("@origingame\/engine-(?:darwin-arm64|linux-x64|win32-x64)":\s*")([^"]+)(")/g });
  s.push({ path: "packages/knowledge/scaffold/origin-engine.json", re: /("version":\s*")([^"]+)(")/g });
  // Projects: project.ogt's engine line and package.json's dependency.
  const projectDirs = ["templates", "examples", "tools/bench", "crates/og-tests/reference-scene", "packages/engine/tests/fixtures"];
  for (const d of projectDirs) {
    for (const p of walk(d, (n) => n === "project.ogt")) s.push({ path: p, re: /(^engine\s+")([^"]+)(")/gm });
    for (const p of walk(d, (n) => n === "package.json")) {
      if (read(p).includes('"@origingame/engine"')) s.push({ path: p, re: /("@origingame\/engine":\s*")([^"]+)(")/g });
    }
  }
  // Inline project.ogt texts in tests, benches and scripts.
  for (const p of [...walk("packages/engine/tests", (n) => n.endsWith(".ts")), ...walk("packages/engine/bench", (n) => n.endsWith(".ts")), "scripts/sound-presets.sh"]) {
    if (read(p).includes('engine "')) s.push({ path: p, re: /(engine \\?")(\d+\.\d+\.\d+[^"\\]*)(\\?")/g });
  }
  // The workspace crates.
  for (const p of walk("crates", (n) => n === "Cargo.toml")) {
    if (/^version = "/m.test(read(p))) s.push({ path: p, re: /(^version = ")([^"]+)(")/m });
  }
  return s;
}

const CHANGELOG = "packages/engine/changelog.json";

interface ChangeEntry {
  version?: string;
  date?: string;
  kind?: "patch" | "minor";
  simulation: "unchanged" | "changed";
  breaking: unknown[];
  added: unknown[];
  fixed: unknown[];
}
interface Changelog {
  schema: number;
  unreleased: ChangeEntry;
  versions: ChangeEntry[];
}

const minorOf = (v: string) => v.split(".").slice(0, 2).join(".");

/** Problems with changelog.json for `version` (an entry, well formed); empty when fine. */
function changelogProblems(log: Changelog, version: string): string[] {
  const out: string[] = [];
  const entry = log.versions.find((e) => e.version === version);
  if (!entry) out.push(`${CHANGELOG}: no entry for ${version}`);
  for (const e of [log.unreleased, ...log.versions]) {
    const name = e.version ?? "unreleased";
    if (e.simulation !== "unchanged" && e.simulation !== "changed") out.push(`${CHANGELOG} ${name}: simulation is "unchanged" or "changed"`);
    if (e.version !== undefined && e.kind !== "patch" && e.kind !== "minor") out.push(`${CHANGELOG} ${name}: kind is "patch" or "minor"`);
    if (e.kind === "patch" && e.breaking.length > 0) out.push(`${CHANGELOG} ${name}: a patch has no breaking entries`);
  }
  return out;
}

/** Moves the unreleased entry into versions as `target`. */
function releaseChangelog(now: string, target: string): string | null {
  const log = JSON.parse(read(CHANGELOG)) as Changelog;
  if (log.versions.some((e) => e.version === target)) return null;
  const kind = minorOf(now) === minorOf(target) ? "patch" : "minor";
  const u = log.unreleased;
  if (kind === "patch" && u.breaking.length > 0) {
    return `${CHANGELOG}: unreleased has ${u.breaking.length} breaking entr${u.breaking.length === 1 ? "y" : "ies"}; ${target} is a patch of ${now}: release a minor (0.x.0) instead (docs/release.md §3)`;
  }
  const entry: ChangeEntry = { version: target, date: new Date().toISOString().slice(0, 10), kind, ...u };
  log.versions.unshift(entry);
  log.unreleased = { simulation: "unchanged", breaking: [], added: [], fixed: [] };
  writeFileSync(join(ROOT, CHANGELOG), `${JSON.stringify(log, null, 2)}\n`);
  return null;
}

function main(args: string[]): number {
  const want = args.find((a) => !a.startsWith("--"));
  const check = args.includes("--check");
  const now = current();
  if (args.includes("--print")) {
    process.stdout.write(`${now}\n`);
    return 0;
  }
  if (!check && (!want || !SEMVER.test(want))) {
    process.stderr.write("usage: bun scripts/release-bump.ts <version> | --check | --print\n");
    return 2;
  }
  const target = check ? now : want!;
  if (!check) {
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
    let hit = 0;
    const after = before.replace(site.re, (_m, a: string, v: string, c: string) => {
      hit++;
      if (v !== target) wrong.push(`${site.path}: ${v}`);
      return `${a}${target}${c}`;
    });
    if (hit === 0) wrong.push(`${site.path}: no version found by ${site.re}`);
    if (!check && after !== before) {
      writeFileSync(join(ROOT, site.path), after);
      changed++;
    }
  }
  if (check) {
    wrong.push(...changelogProblems(JSON.parse(read(CHANGELOG)) as Changelog, now));
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

process.exit(main(process.argv.slice(2)));
