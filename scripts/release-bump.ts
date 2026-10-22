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
  s.push({ path: "packages/engine/package.json", re: /("@origingame\/engine-(?:darwin-arm64|linux-x64)":\s*")([^"]+)(")/g });
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
