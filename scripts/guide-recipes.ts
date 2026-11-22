// The guides' recipes (AGENTS.md §Ownership; docs/ci.md, the guide-recipes
// step): every ```ts recipe block in crates/og-api/guides/*.md is a whole
// src/main.ts. Each is written into a project of its own, type-checked and
// run headless twice by `og run --json`; it fails on a check error, a throw,
// an og.assert violation, an assertion site that never fired, no assertion
// at all, or two runs that end on different hashes. Every failure names the
// guide's file and line.
//
//   bun scripts/guide-recipes.ts [--og <bin>] [--out <dir>] [guide.md ...]
//
// A block's info string may set how long it runs: ```ts recipe ticks=300
// (default 120). The projects go under --out (default target/guide-recipes),
// inside the checkout so `@origingame/engine` resolves.
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, relative, resolve } from "node:path";

export interface Recipe {
  /** The guide, as given. */
  readonly guide: string;
  /** The guide's line of the block's first code line (1-based). */
  readonly line: number;
  readonly ticks: number;
  readonly source: string;
}

const DEFAULT_TICKS = 120;

/** The ```ts recipe blocks of a guide's text, in order. */
export function extract(guide: string, text: string): Recipe[] {
  const lines = text.split("\n");
  const out: Recipe[] = [];
  for (let i = 0; i < lines.length; i++) {
    const open = /^```ts\s+recipe\b(.*)$/.exec(lines[i]!);
    if (!open) continue;
    const ticksArg = /\bticks=(\d+)\b/.exec(open[1]!);
    const ticks = ticksArg ? Number(ticksArg[1]) : DEFAULT_TICKS;
    const body: string[] = [];
    let j = i + 1;
    for (; j < lines.length && !/^```\s*$/.test(lines[j]!); j++) body.push(lines[j]!);
    if (j === lines.length) throw new Error(`${guide}:${i + 1}: a recipe block that never closes`);
    out.push({ guide, line: i + 2, ticks, source: `${body.join("\n")}\n` });
    i = j;
  }
  return out;
}

/** The guide's place for line `n` of the recipe's src/main.ts. */
export function at(recipe: Recipe, n: number | null | undefined): string {
  return `${recipe.guide}:${n ? recipe.line + n - 1 : recipe.line - 1}`;
}

const PROJECT = `type project
version 1
---
# A guide recipe's project, written by scripts/guide-recipes.ts.
name "Recipe"
id "recipe"
engine "0.9.0"
entry "src/main.ts"
tick 60
seed 1
`;

const TSCONFIG = {
  compilerOptions: {
    target: "ES2022", module: "ESNext", moduleResolution: "Bundler", lib: ["ES2022"], types: [],
    strict: true, exactOptionalPropertyTypes: true, noFallthroughCasesInSwitch: true,
    isolatedModules: true, verbatimModuleSyntax: true, allowImportingTsExtensions: true, noEmit: true,
  },
  include: ["src/**/*.ts", ".og/types/**/*.d.ts"],
};

interface Diagnostic { file: string | null; line: number | null; message: string }
interface Site { kind: string; name: string; location: string; fired: boolean }
interface Violation { kind: string; name: string; location: string; tick: number; message: string }
interface Report {
  diagnostics?: Diagnostic[];
  hash?: string;
  assertions?: { sites: Site[]; uncovered: Site[]; violations: Violation[] };
}

/** The last JSON line `og run --json` printed that carries `key`. */
function last(stdout: string, key: string): Report | undefined {
  let found: Report | undefined;
  for (const line of stdout.split("\n")) {
    if (!line.startsWith("{")) continue;
    try {
      const v = JSON.parse(line) as Record<string, unknown>;
      if (key in v) found = v as Report;
    } catch { /* a partial line */ }
  }
  return found;
}

/** `src/main.ts:N` (or an absolute path ending so) in `text`, as the guide's place. */
function relocate(recipe: Recipe, text: string): string {
  return text.replace(/(?:[^\s()]*\/)?src\/main\.ts:(\d+)(?::\d+)?/g, (_m, n: string) => at(recipe, Number(n)));
}

/** A site's place in the guide; a default invariant's (`Health.hp`) after the recipe's. */
function place(recipe: Recipe, location: string): string {
  const m = /^src\/main\.ts:(\d+)/.exec(location);
  return m ? at(recipe, Number(m[1])) : `${at(recipe, null)} (${location})`;
}

/** Runs one recipe in `dir`; the failures, each starting with its guide place. */
export function check(recipe: Recipe, dir: string, og: string): string[] {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(join(dir, "src"), { recursive: true });
  writeFileSync(join(dir, "project.ogt"), PROJECT);
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(TSCONFIG, null, 2)}\n`);
  writeFileSync(join(dir, "src", "main.ts"), recipe.source);
  const failures: string[] = [];
  const hashes: string[] = [];
  for (let run = 0; run < 2; run++) {
    const r = Bun.spawnSync([og, "run", dir, "--headless", "--json", "--ticks", String(recipe.ticks)], { stdout: "pipe", stderr: "pipe" });
    const stdout = r.stdout.toString(), stderr = r.stderr.toString();
    const diagnostics = last(stdout, "diagnostics")?.diagnostics ?? [];
    for (const d of diagnostics) {
      failures.push(`${d.file === "src/main.ts" ? at(recipe, d.line) : at(recipe, null)}: ${d.file && d.file !== "src/main.ts" ? `${d.file}: ` : ""}${d.message.split("\n")[0]}`);
    }
    if (diagnostics.length > 0) return failures;
    const report = last(stdout, "assertions");
    if (report?.assertions) {
      const { sites, uncovered, violations } = report.assertions;
      for (const v of violations) failures.push(`${place(recipe, v.location)}: og.assert.${v.kind} "${v.name}" violated at tick ${v.tick}: ${v.message}`);
      for (const u of uncovered) failures.push(`${place(recipe, u.location)}: og.assert.${u.kind} "${u.name}" never fired in ${recipe.ticks} ticks`);
      if (!sites.some((s) => s.location.startsWith("src/main.ts:"))) failures.push(`${at(recipe, null)}: the recipe asserts nothing (no og.assert site)`);
    }
    if (failures.length === 0 && (r.exitCode !== 0 || !report?.assertions)) {
      const why = relocate(recipe, (stderr.trim() || stdout.trim()).split("\n").slice(0, 6).join("\n    "));
      failures.push(`${at(recipe, null)}: og run exited ${r.exitCode}: ${why}`);
    }
    if (failures.length > 0) return failures;
    hashes.push(report!.hash ?? "");
  }
  if (hashes[0] !== hashes[1]) failures.push(`${at(recipe, null)}: two runs ended on different hashes (${hashes[0]}, ${hashes[1]})`);
  return failures;
}

function main(argv: string[]): number {
  const root = resolve(import.meta.dir, "..");
  let og = process.env.OG_BIN ?? join(process.env.OG_TARGET_DIR ?? join(root, "target"), "debug", "og");
  let out = join(root, "target", "guide-recipes");
  const guides: string[] = [];
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--og") og = argv[++i]!;
    else if (argv[i] === "--out") out = resolve(argv[++i]!);
    else guides.push(argv[i]!);
  }
  if (guides.length === 0) {
    const dir = join(root, "crates", "og-api", "guides");
    for (const f of readdirSync(dir).sort()) if (f.endsWith(".md")) guides.push(relative(process.cwd(), join(dir, f)));
  }
  let count = 0;
  const failures: string[] = [];
  for (const guide of guides) {
    let recipes: Recipe[];
    try {
      recipes = extract(guide, readFileSync(guide, "utf8"));
    } catch (e) {
      failures.push((e as Error).message);
      continue;
    }
    for (const recipe of recipes) {
      count++;
      const dir = join(out, `${basename(guide, ".md")}-${recipe.line}`);
      failures.push(...check(recipe, dir, og));
    }
  }
  for (const f of failures) console.error(`guide-recipes: ${f}`);
  console.log(`guide-recipes: ${count} recipe(s) in ${guides.length} guide(s), ${failures.length} failure(s)`);
  return failures.length > 0 ? 1 : 0;
}

if (import.meta.main) process.exit(main(process.argv.slice(2)));
