// Materialises the mounts knowledge.json declares into the platform's
// knowledge tree, `knowledge/` of an origin-agent checkout (before the
// agent-repo split, OriginGame's packages/game-knowledge/production):
// skills as game/skills/<name>/ (the guides as og-game's guides/), the
// project template plus origin-engine.json and distribution.json as the
// scaffold overlay game-scaffold/<overlay>/.
//
//   bun scripts/mount.ts <origin-agent checkout> [--check] [--commit <rev> | --package <dir>]
//
// The package owns, in the platform tree, exactly the directories it mounts:
// game/skills/<each of its skills>/ and game-scaffold/<overlay>/. Within
// them the result is byte-for-byte the package's text payload: files are
// written only when their bytes differ, files the package no longer has are
// removed, and nothing outside them is touched, so a second run changes
// nothing. Binary template files (models, textures, audio, fonts) are never
// written: the platform payload is UTF-8 text only and distribution.json
// names them with their digests. The package is validated against the
// platform rules first; an invalid package mounts nothing.
//
// --check writes nothing and exits 1 when the platform tree differs, naming
// each added, changed and removed path. --commit mounts the package as
// committed at <rev> (a `git archive` of packages/knowledge, templates/game
// and packages/engine/package.json) instead of the working tree, so work in
// progress in this checkout never reaches the platform. --package mounts the
// knowledge of an installed @origingame/engine (the directory `bun install`
// put it in; the package carries this directory as knowledge/): what the
// platform does at a pinned version (docs/release.md §4 L2).
//
// origin-agent's scripts/knowledge/mount-engine.ts imports `installed`,
// `mount` and `summary` from the installed package and passes its own
// knowledge root: their signatures are a public contract (AGENTS.md), and
// changing them is a break.

import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { defaultRoots, isText, PACKAGE_ROOT, payload, walk, type Roots } from "./lib.ts";
import { validate } from "./validate.ts";

/** The platform's knowledge tree, relative to an origin-agent checkout. */
export const PRODUCTION = "knowledge";

export interface Change {
  readonly kind: "added" | "changed" | "removed";
  readonly path: string;
}

export interface MountResult {
  /** The platform's production root. */
  readonly production: string;
  /** The directories the package owns there, production-relative, each ending in `/`. */
  readonly owned: readonly string[];
  /** Text files mounted (written or already identical). */
  readonly files: number;
  /** Template binaries left to distribution.json. */
  readonly binaries: readonly string[];
  readonly changes: readonly Change[];
}

/** The production-relative directories the payload occupies: game/skills/<name>/ and game-scaffold/<overlay>/. */
export function ownedRoots(paths: readonly string[]): string[] {
  const owned = new Set<string>();
  for (const p of paths) {
    const parts = p.split("/");
    if (parts[0] === "game" && parts[1] === "skills" && parts.length > 3) owned.add(`game/skills/${parts[2]}/`);
    else if (parts[0] === "game-scaffold" && parts.length > 2) owned.add(`game-scaffold/${parts[1]}/`);
    else throw new Error(`${p}: a mount outside game/skills/<name>/ and game-scaffold/<overlay>/; fix knowledge.json`);
  }
  return [...owned].sort();
}

function executable(path: string): boolean {
  return (statSync(path).mode & 0o111) !== 0;
}

/** Compares (and with `write`, materialises) the package's payload in `production`. */
export function mount(roots: Roots, production: string, write: boolean): MountResult {
  for (const top of ["game/skills", "game-scaffold"]) {
    if (!existsSync(join(production, top)) || !statSync(join(production, top)).isDirectory()) {
      throw new Error(`${production}: no ${top}/; pass the root of an origin-agent checkout (its ${PRODUCTION}/ holds game/ and game-scaffold/)`);
    }
  }
  const problems = validate(roots);
  if (problems.length) throw new Error(`the knowledge package is invalid against the platform rules; nothing mounted:\n${problems.join("\n")}`);

  const all = payload(roots);
  const text = all.filter((f) => isText(f.bytes));
  const binaries = all.filter((f) => !isText(f.bytes)).map((f) => f.path);
  const owned = ownedRoots(all.map((f) => f.path));
  const wanted = new Map(text.map((f) => [f.path, f]));
  const changes: Change[] = [];

  for (const f of text) {
    const target = join(production, f.path);
    const exists = existsSync(target);
    if (exists && lstatSync(target).isDirectory()) throw new Error(`${f.path}: a directory in the platform tree where the package has a file`);
    const same = exists && Buffer.compare(readFileSync(target), Buffer.from(f.bytes)) === 0;
    const mode = executable(f.source) ? 0o755 : 0o644;
    // Windows chmod cannot represent POSIX group/other or executable bits.
    const modeSame = exists && (process.platform === "win32" || (statSync(target).mode & 0o777) === mode);
    if (same && modeSame) continue;
    changes.push({ kind: exists ? "changed" : "added", path: f.path });
    if (write) {
      mkdirSync(dirname(target), { recursive: true });
      if (!same) writeFileSync(target, f.bytes);
      chmodSync(target, mode);
    }
  }

  for (const root of owned) {
    const dir = join(production, root);
    for (const full of walk(dir)) {
      const path = `${root}${full.slice(dir.length).split("\\").join("/")}`;
      if (wanted.has(path)) continue;
      changes.push({ kind: "removed", path });
      if (write) rmSync(full, { force: true });
    }
    if (write) pruneEmpty(dir);
  }

  changes.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  return { production, owned, files: text.length, binaries, changes };
}

/** Removes empty directories under `dir` (not `dir` itself). */
function pruneEmpty(dir: string): void {
  if (!existsSync(dir)) return;
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (!lstatSync(full).isDirectory()) continue;
    pruneEmpty(full);
    if (readdirSync(full).length === 0) rmdirSync(full);
  }
}

/** What a mount reads from the engine repository. */
export const SOURCES = ["packages/knowledge", "templates/game", "packages/engine/package.json"] as const;

const git = (repo: string, ...args: string[]) =>
  execFileSync("git", ["-C", repo, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();

/** The working tree's short revision, `+dirty` when the mounted sources differ from it, or `unversioned`. */
export function revision(repo: string): string {
  try {
    const head = git(repo, "rev-parse", "--short", "HEAD");
    return git(repo, "status", "--porcelain", "--", ...SOURCES) ? `${head}+dirty` : head;
  } catch {
    return "unversioned";
  }
}

/** The package as committed at `rev`, exported into a temporary directory: its roots, short revision and a cleanup. */
export function committed(repo: string, rev: string): { roots: Roots; revision: string; cleanup: () => void } {
  const short = git(repo, "rev-parse", "--short", `${rev}^{commit}`);
  const dir = mkdtempSync(join(tmpdir(), "og-knowledge-mount-"));
  execFileSync("sh", ["-c", 'git -C "$1" archive "$2" -- "$3" "$4" "$5" | tar -x -C "$6"', "sh", repo, short, ...SOURCES, dir], {
    stdio: ["ignore", "ignore", "pipe"],
  });
  return { roots: defaultRoots(join(dir, "packages/knowledge")), revision: short, cleanup: () => rmSync(dir, { recursive: true, force: true }) };
}

/** An installed @origingame/engine (the directory `bun install` put it in, or its knowledge/): the knowledge's roots and `@origingame/engine@<version>`. */
export function installed(dir: string): { roots: Roots; revision: string; cleanup: () => void } {
  const manifest = join(dir, "package.json");
  if (!existsSync(manifest)) throw new Error(`${dir}: no package.json; --package takes the installed @origingame/engine directory`);
  const pkg = JSON.parse(readFileSync(manifest, "utf8")) as { name?: string; version?: string };
  let knowledge = dir;
  if (pkg.name === "@origingame/engine") knowledge = join(dir, "knowledge");
  else if (pkg.name !== "@origingame/engine-knowledge") throw new Error(`${dir}: ${pkg.name ?? "no name"}, not @origingame/engine`);
  if (!existsSync(join(knowledge, "knowledge.json"))) throw new Error(`${knowledge}: no knowledge.json; the published @origingame/engine carries the knowledge in knowledge/`);
  const roots = defaultRoots(knowledge);
  if (!existsSync(join(roots.engine, "package.json"))) throw new Error(`${roots.engine}: no @origingame/engine package.json`);
  const engine = JSON.parse(readFileSync(join(roots.engine, "package.json"), "utf8")) as { name?: string; version?: string };
  return { roots, revision: `${engine.name}@${engine.version} (knowledge)`, cleanup: () => {} };
}

export function summary(r: MountResult, rev: string, check: boolean): string {
  const count = (k: Change["kind"]) => r.changes.filter((c) => c.kind === k).length;
  const lines = [
    `${check ? "check" : "mount"}: engine knowledge ${rev} -> ${r.production}`,
    `owned: ${r.owned.join(", ")}`,
    `text files: ${r.files}; binaries left to distribution.json: ${r.binaries.length}`,
    `${check ? "differences" : "changes"}: ${count("added")} added, ${count("changed")} changed, ${count("removed")} removed`,
    ...r.changes.map((c) => `  ${c.kind.padEnd(7)} ${c.path}`),
  ];
  return `${lines.join("\n")}\n`;
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  const check = args.includes("--check");
  const ci = args.indexOf("--commit");
  const commit = ci >= 0 ? args[ci + 1] : undefined;
  const pi = args.indexOf("--package");
  const pkgDir = pi >= 0 ? args[pi + 1] : undefined;
  const positional = args.filter((a, i) => !a.startsWith("--") && !(ci >= 0 && i === ci + 1) && !(pi >= 0 && i === pi + 1));
  if ((ci >= 0 && !commit) || (pi >= 0 && !pkgDir) || (commit && pkgDir) || positional.length !== 1 || args.some((a) => a.startsWith("--") && a !== "--check" && a !== "--commit" && a !== "--package")) {
    process.stderr.write("usage: bun scripts/mount.ts <origin-agent checkout> [--check] [--commit <rev> | --package <dir>]\n");
    process.exit(2);
  }
  const repo = join(PACKAGE_ROOT, "..", "..");
  const production = join(resolve(positional[0]!), PRODUCTION);
  let code = 0;
  let source: { roots: Roots; revision: string; cleanup: () => void } | undefined;
  try {
    source = commit ? committed(repo, commit) : pkgDir ? installed(resolve(pkgDir)) : { roots: defaultRoots(), revision: revision(repo), cleanup: () => {} };
    const result = mount(source.roots, production, !check);
    process.stdout.write(summary(result, source.revision, check));
    if (check && result.changes.length) code = 1;
  } catch (e) {
    process.stderr.write(`${e instanceof Error ? e.message : String(e)}\n`);
    code = 1;
  } finally {
    source?.cleanup();
  }
  process.exit(code);
}
