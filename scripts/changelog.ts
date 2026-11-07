// The engine's one changelog, packages/engine/changelog.json (docs/release.md §3),
// and the fragments merges record it with.
//
// A merge never edits changelog.json: it adds one new file under
// changelog/unreleased/ (any name of lowercase letters, digits, `.`, `-`
// and `_`, such as its branch and topic), so two concurrent landings never
// touch the same lines. A fragment is part of an entry:
//
//   { "simulation": "changed",                      optional; omit when a game's ticks cannot differ
//     "breaking": [{ "member", "change", "fix", "find"?: [identifier], "codemod"?: {...} }],
//     "added": [{ "member", "summary" }],
//     "fixed": [{ "member", "summary" } | "text"] }  every list optional, at least one item in all
//
//   bun scripts/changelog.ts check            shape-check every fragment and changelog.json (the quick gate runs it)
//   bun scripts/changelog.ts fold             fold every fragment into changelog.json's unreleased, in name order,
//                                             and delete them; run it at any time, and release-bump runs it first
//   bun scripts/changelog.ts notes [<version>] the release notes, Markdown, from changelog.json (unreleased and the
//                                             fragments when no version is named)

import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const ROOT = join(import.meta.dir, "..");
export const CHANGELOG = "packages/engine/changelog.json";
export const FRAGMENTS = "changelog/unreleased";
const NAME = /^[a-z0-9][a-z0-9._-]*\.json$/;

export interface Breaking {
  member: string;
  change: string;
  fix: string;
  find?: string[];
  codemod?: Record<string, unknown>;
}
export type Note = { member: string; summary: string } | string;
export interface Entry {
  version?: string;
  date?: string;
  kind?: "patch" | "minor";
  simulation: "unchanged" | "changed";
  breaking: Breaking[];
  added: Note[];
  fixed: Note[];
}
export interface Changelog {
  schema: number;
  about?: string;
  unreleased: Entry;
  versions: Entry[];
}
export interface Fragment {
  simulation?: "changed" | "unchanged";
  breaking?: Breaking[];
  added?: Note[];
  fixed?: Note[];
}

const isText = (v: unknown): v is string => typeof v === "string" && v.trim().length > 0;
const isObject = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

function noteProblems(where: string, v: unknown): string[] {
  if (isText(v)) return [];
  if (isObject(v) && isText(v.member) && isText(v.summary) && Object.keys(v).length === 2) return [];
  return [`${where}: write { "member": "...", "summary": "..." }`];
}

function breakingProblems(where: string, v: unknown): string[] {
  if (!isObject(v)) return [`${where}: write { "member", "change", "fix", "find"?, "codemod"? }`];
  const out: string[] = [];
  for (const k of ["member", "change", "fix"]) if (!isText(v[k])) out.push(`${where}: "${k}" is a non-empty string`);
  if (v.find !== undefined && !(Array.isArray(v.find) && v.find.every(isText))) out.push(`${where}: "find" is a list of identifiers`);
  if (v.codemod !== undefined && !isObject(v.codemod)) out.push(`${where}: "codemod" is an object`);
  for (const k of Object.keys(v)) if (!["member", "change", "fix", "find", "codemod"].includes(k)) out.push(`${where}: unknown key "${k}"`);
  return out;
}

/** Problems with one fragment's parsed JSON; empty when it is well formed. */
export function fragmentProblems(name: string, f: unknown): string[] {
  const where = `${FRAGMENTS}/${name}`;
  if (!NAME.test(name)) return [`${where}: name it with lowercase letters, digits, ".", "-" and "_", ending in .json`];
  if (!isObject(f)) return [`${where}: a fragment is a JSON object`];
  const out: string[] = [];
  for (const k of Object.keys(f)) {
    if (!["simulation", "breaking", "added", "fixed"].includes(k)) out.push(`${where}: unknown key "${k}" (simulation, breaking, added, fixed)`);
  }
  if (f.simulation !== undefined && f.simulation !== "changed" && f.simulation !== "unchanged") out.push(`${where}: "simulation" is "changed" or "unchanged"`);
  let items = 0;
  for (const k of ["breaking", "added", "fixed"] as const) {
    const list = f[k];
    if (list === undefined) continue;
    if (!Array.isArray(list)) {
      out.push(`${where}: "${k}" is a list`);
      continue;
    }
    items += list.length;
    list.forEach((v, i) => out.push(...(k === "breaking" ? breakingProblems(`${where} ${k}[${i}]`, v) : noteProblems(`${where} ${k}[${i}]`, v))));
  }
  if (items === 0 && f.simulation !== "changed") out.push(`${where}: records nothing (add a breaking, added or fixed item)`);
  return out;
}

/** Problems with changelog.json itself (its entries' shape). */
export function changelogProblems(log: Changelog): string[] {
  const out: string[] = [];
  if (log.schema !== 1) out.push(`${CHANGELOG}: schema is 1`);
  for (const e of [log.unreleased, ...log.versions]) {
    const name = `${CHANGELOG} ${e.version ?? "unreleased"}`;
    if (e.simulation !== "unchanged" && e.simulation !== "changed") out.push(`${name}: simulation is "unchanged" or "changed"`);
    if (e.version !== undefined && e.kind !== "patch" && e.kind !== "minor") out.push(`${name}: kind is "patch" or "minor"`);
    if (e.kind === "patch" && e.breaking.length > 0) out.push(`${name}: a patch has no breaking entries`);
    e.breaking.forEach((v, i) => out.push(...breakingProblems(`${name} breaking[${i}]`, v)));
    e.added.forEach((v, i) => out.push(...noteProblems(`${name} added[${i}]`, v)));
    e.fixed.forEach((v, i) => out.push(...noteProblems(`${name} fixed[${i}]`, v)));
  }
  return out;
}

/** The fragment files under `root`, in name order (the order they fold in). */
export function fragmentNames(root = ROOT): string[] {
  const dir = join(root, FRAGMENTS);
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((n) => !n.startsWith(".") && n !== "README.md")
    .sort();
}

export function readLog(root = ROOT): Changelog {
  return JSON.parse(readFileSync(join(root, CHANGELOG), "utf8")) as Changelog;
}

export function writeLog(log: Changelog, root = ROOT): void {
  writeFileSync(join(root, CHANGELOG), `${JSON.stringify(log, null, 2)}\n`);
}

/** Every problem with the fragments and changelog.json under `root`. */
export function check(root = ROOT): string[] {
  const out: string[] = [];
  for (const name of fragmentNames(root)) {
    let f: unknown;
    try {
      f = JSON.parse(readFileSync(join(root, FRAGMENTS, name), "utf8"));
    } catch (e) {
      out.push(`${FRAGMENTS}/${name}: not JSON (${(e as Error).message})`);
      continue;
    }
    out.push(...fragmentProblems(name, f));
  }
  out.push(...changelogProblems(readLog(root)));
  return out;
}

/** Adds `fragments` to `entry` in order: a "changed" simulation wins, the lists append. */
export function foldInto(entry: Entry, fragments: Fragment[]): Entry {
  const next: Entry = { ...entry, breaking: [...entry.breaking], added: [...entry.added], fixed: [...entry.fixed] };
  for (const f of fragments) {
    if (f.simulation === "changed") next.simulation = "changed";
    next.breaking.push(...(f.breaking ?? []));
    next.added.push(...(f.added ?? []));
    next.fixed.push(...(f.fixed ?? []));
  }
  return next;
}

/**
 * Folds every fragment under `root` into changelog.json's unreleased and
 * deletes them. Refuses, folding nothing, when any fragment or the log is
 * malformed. Returns the problems (empty when it folded) and the count.
 */
export function fold(root = ROOT): { problems: string[]; folded: number } {
  const problems = check(root);
  if (problems.length) return { problems, folded: 0 };
  const names = fragmentNames(root);
  if (names.length === 0) return { problems: [], folded: 0 };
  const log = readLog(root);
  const fragments = names.map((n) => JSON.parse(readFileSync(join(root, FRAGMENTS, n), "utf8")) as Fragment);
  log.unreleased = foldInto(log.unreleased, fragments);
  writeLog(log, root);
  for (const n of names) rmSync(join(root, FRAGMENTS, n));
  return { problems: [], folded: names.length };
}

const line = (n: Note) => (typeof n === "string" ? `- ${n}` : `- **${n.member}.** ${n.summary}`);

/** One version's release notes in Markdown. */
export function notes(e: Entry, title: string): string {
  const out = [`## ${title}`, ""];
  const tick = e.simulation === "changed" ? "may step differently from the version before" : "steps as the version before";
  out.push(`${e.kind === "minor" ? "A minor release: a game may need edits to upgrade" : e.kind === "patch" ? "A patch release: compatible" : "Unreleased"}; a game's simulation ${tick}.`, "");
  if (e.breaking.length) {
    out.push("### Breaking", "");
    for (const b of e.breaking) out.push(`- **${b.member}.** ${b.change} *Fix:* ${b.fix}`);
    out.push("");
  }
  if (e.added.length) out.push("### Added", "", ...e.added.map(line), "");
  if (e.fixed.length) out.push("### Fixed", "", ...e.fixed.map(line), "");
  return `${out.join("\n").trimEnd()}\n`;
}

function main(args: string[]): number {
  const [verb, arg] = args;
  if (verb === "check") {
    const problems = check();
    if (problems.length) {
      process.stderr.write(`changelog: ${problems.length} problem(s):\n${problems.map((p) => `  ${p}`).join("\n")}\n`);
      return 1;
    }
    process.stdout.write(`changelog: ${fragmentNames().length} fragment(s) and ${CHANGELOG} well formed\n`);
    return 0;
  }
  if (verb === "fold") {
    const { problems, folded } = fold();
    if (problems.length) {
      process.stderr.write(`changelog fold: nothing folded; ${problems.length} problem(s):\n${problems.map((p) => `  ${p}`).join("\n")}\n`);
      return 1;
    }
    process.stdout.write(`changelog fold: ${folded} fragment(s) folded into ${CHANGELOG}'s unreleased\n`);
    return 0;
  }
  if (verb === "notes") {
    const log = readLog();
    if (!arg) {
      const fragments = fragmentNames().map((n) => JSON.parse(readFileSync(join(ROOT, FRAGMENTS, n), "utf8")) as Fragment);
      process.stdout.write(notes(foldInto(log.unreleased, fragments), "Unreleased"));
      return 0;
    }
    const e = log.versions.find((v) => v.version === arg);
    if (!e) {
      process.stderr.write(`changelog notes: ${CHANGELOG} has no version ${arg}\n`);
      return 1;
    }
    process.stdout.write(notes(e, `${e.version} — ${e.date}`));
    return 0;
  }
  process.stderr.write("usage: bun scripts/changelog.ts check | fold | notes [<version>]\n");
  return 2;
}

if (import.meta.main) process.exit(main(process.argv.slice(2)));
