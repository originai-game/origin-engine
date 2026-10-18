// Validates @origingame/engine-knowledge against the OriginGame platform's
// knowledge rules (rules/platform.json, transcribed from the platform with
// scripts/sync-rules.ts; each rule names its platform source), and the
// package's own contract: a generated wiki, the skills in SKILLS, and a scaffold
// overlay whose pin and distribution list match the project template.
//
//   bun scripts/validate.ts            problems, then the packs and hashes
//   bun scripts/validate.ts --bundle   the payload as the platform's
//                                      readProductionBundle shape (text only)

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import {
  defaultRoots,
  frontMatter,
  isText,
  ogtTopLevel,
  packName,
  packs,
  payload,
  readManifest,
  sha256,
  TEMPLATE_SKIP,
  walk,
  type PayloadFile,
  type Roots,
} from "./lib.ts";

export interface Rules {
  payload: { roots: string[] };
  skills: { file: string; namePattern: string; existing: string[] };
  tools: { snake: string; notTools: string[]; retired: string[]; known: string[] };
  scaffold: {
    pinFile: string;
    pin: { schema: number; probe: { protocol: number; wire: number } };
    distributionFile: string;
    distributionEntry: string[];
    existingOverlays: string[];
  };
}

export function readRules(pkg: string): Rules {
  return JSON.parse(readFileSync(join(pkg, "rules/platform.json"), "utf8")) as Rules;
}

/** The skills this package ships. */
export const SKILLS = ["og-build", "og-core", "og-feel", "og-game", "og-taste", "og-ui"] as const;

/** What a scaffolded project must contain (scaffold/README.md describes each). */
export const TEMPLATE_FILES = ["project.ogt", "src/main.ts", "package.json", "tsconfig.json", "tests/smoke.og.ts"] as const;

/** The keys project.ogt must set (scaffold/README.md). */
export const PROJECT_KEYS = ["name", "id", "engine", "entry", "profile", "tick", "seed", "scenes", "scene", "publish"] as const;

const GENERATED = "<!-- Generated from crates/og-api/api.ogt";
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

/** Every problem found, each naming the file and the fix; empty when the package is valid. */
export function validate(roots: Roots = defaultRoots()): string[] {
  const problems: string[] = [];
  const rules = readRules(roots.pkg);
  const manifest = readManifest(roots.pkg);
  const files = payload(roots);
  const byPath = new Map(files.map((f) => [f.path, f]));

  packageJson(roots, problems);
  wiki(roots, problems);

  // The platform payload: production roots, regular files, UTF-8 text.
  for (const f of files) {
    if (!rules.payload.roots.some((r) => f.path.startsWith(`${r}/`))) {
      problems.push(`${f.path}: outside the platform's production roots (${rules.payload.roots.join(", ")}); fix its mount in knowledge.json`);
    }
    if (f.symlink) problems.push(`${f.path}: a symbolic link (${f.source}); the platform bundle takes regular files only`);
    const binaryAllowed = f.path.startsWith(`game-scaffold/${manifest.overlay}/`);
    if (!binaryAllowed && !isText(f.bytes)) {
      problems.push(`${f.path}: not UTF-8 text; the platform payload is text only (packages/game-knowledge/scripts/production-bundle.mjs)`);
    }
  }

  skills(rules, files, problems);
  scaffold(roots, rules, manifest.overlay, files, byPath, problems);
  return problems;
}

function packageJson(roots: Roots, problems: string[]): void {
  const p = JSON.parse(readFileSync(join(roots.pkg, "package.json"), "utf8")) as { name?: string; version?: string; files?: string[]; scripts?: Record<string, string> };
  if (p.name !== "@origingame/engine-knowledge") problems.push(`package.json: name is ${p.name}; the package is @origingame/engine-knowledge (design §9)`);
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(p.version ?? "")) problems.push(`package.json: version ${p.version} is not semver`);
  for (const f of ["wiki", "skills", "scaffold", "rules", "knowledge.json"]) {
    if (!p.files?.some((x) => x === f || x === `${f}/`)) problems.push(`package.json: files does not publish ${f}`);
  }
  if (!p.scripts?.test) problems.push("package.json: no test script, so scripts/check.sh bun-test would skip the package");
}

function wiki(roots: Roots, problems: string[]): void {
  const dir = join(roots.pkg, "wiki");
  const index = join(dir, "index.md");
  if (!existsSync(index)) {
    problems.push("wiki/index.md: missing; run `og api gen --wiki` in the engine repository");
    return;
  }
  const pages = readdirSync(dir).filter((n) => n.endsWith(".md")).sort();
  for (const name of pages) {
    const text = readFileSync(join(dir, name), "utf8");
    if (!text.startsWith(GENERATED)) problems.push(`wiki/${name}: not generated from api.ogt (no generated header); wiki pages are never hand-written: run \`og api gen --wiki\``);
  }
  const indexText = readFileSync(index, "utf8");
  const linked = new Set([...indexText.matchAll(/\]\(([a-z0-9_-]+\.md)[#)]/gi)].map((m) => m[1]!));
  for (const name of linked) if (!pages.includes(name)) problems.push(`wiki/index.md: links to ${name}, which does not exist; run \`og api gen --wiki\``);
  for (const name of pages) if (name !== "index.md" && !linked.has(name)) problems.push(`wiki/${name}: not linked from wiki/index.md (a page for a module the IDL no longer has?); run \`og api gen --wiki\``);
}

/** Tokens a skill names as tools, as the platform's production test reads them. */
function namedTools(text: string, rules: Rules): string[] {
  const snake = new RegExp(rules.tools.snake);
  const known = new Set(rules.tools.known);
  const families = new Set([...known].filter((n) => snake.test(n)).map((n) => n.split("_")[0]!));
  const notTools = new Set(rules.tools.notTools);
  const prose = text.replace(/^```[\s\S]*?^```/gm, "");
  const description = /^description: (.*)$/m.exec(text)?.[1] ?? "";
  const named = [...[...prose.matchAll(/`([^`\s]+)`/g)].map((m) => m[1]!), ...(description.match(/\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+\b/g) ?? [])]
    .filter((t) => snake.test(t) && families.has(t.split("_")[0]!) && !notTools.has(t));
  const unknown = named.filter((t) => !known.has(t));
  for (const m of text.matchAll(/(?:tools\.call|origin\.execute)\(\s*['"]([^'"]+)['"]/g)) if (!known.has(m[1]!)) unknown.push(m[0]);
  for (const r of rules.tools.retired) for (const m of text.matchAll(new RegExp(`\\b${r}\\b`, "gi"))) unknown.push(m[0]);
  return [...new Set(unknown)];
}

function skills(rules: Rules, files: readonly PayloadFile[], problems: string[]): void {
  const pattern = new RegExp(rules.skills.namePattern);
  const pages = new Set(
    files.map((f) => /^game\/skills\/og-game\/reference\/([^/]+)$/.exec(f.path)?.[1]).filter((n): n is string => n !== undefined),
  );
  const found = new Set<string>();
  for (const f of files) {
    const m = /^game\/skills\/([^/]+)\/(.+)$/.exec(f.path);
    if (!m || m[2] !== rules.skills.file) continue;
    const dir = m[1]!;
    found.add(dir);
    if (!isText(f.bytes)) continue;
    const text = decoder.decode(f.bytes);
    const fm = frontMatter(text);
    if (!fm) {
      problems.push(`${f.path}: no YAML front matter between --- lines; a skill starts with name and description`);
      continue;
    }
    if (fm.name !== dir) problems.push(`${f.path}: front matter name is "${fm.name ?? ""}", its directory is "${dir}"; they must be equal`);
    if (!pattern.test(dir)) problems.push(`${f.path}: skill name "${dir}" is not lowercase-hyphen`);
    if (!fm.description) problems.push(`${f.path}: no description; the runner lists skills by it`);
    if (rules.skills.existing.includes(dir)) problems.push(`${f.path}: the platform already has a skill named ${dir}; installing this one would replace it`);
    for (const t of namedTools(text, rules)) problems.push(`${f.path}: names \`${t}\`, which is no tool a Session can be offered; rename it or put it in a code block if it is not a tool`);
    // reference/ is the generated wiki, mounted under og-game: a page a
    // skill names must be one the IDL generates.
    for (const page of new Set([...text.matchAll(/\breference\/([a-z0-9-]+\.md)\b/g)].map((m) => m[1]!))) {
      if (!pages.has(page)) {
        problems.push(`${f.path}: names reference/${page}, which the generated wiki does not have (the wiki's pages: reference/index.md lists them; a module's page is reference/<module>.md)`);
      }
    }
  }
  for (const s of SKILLS) if (!found.has(s)) problems.push(`skills/${s}/SKILL.md: missing`);
}

function scaffold(roots: Roots, rules: Rules, overlay: string, files: readonly PayloadFile[], byPath: Map<string, PayloadFile>, problems: string[]): void {
  const prefix = `game-scaffold/${overlay}/`;
  if (rules.scaffold.existingOverlays.includes(overlay)) {
    problems.push(`knowledge.json: overlay "${overlay}" is already a platform scaffold overlay; preparing knowledge would replace it`);
  }
  for (const f of TEMPLATE_FILES) if (!byPath.has(prefix + f)) problems.push(`${prefix}${f}: the template has no ${f} (${join(relative(roots.pkg, roots.template), f)}); scaffold/README.md says every project starts with it`);
  if (!files.some((f) => f.path.startsWith(`${prefix}assets/`))) problems.push(`${prefix}assets/: the template has no assets`);

  // The pin.
  const pinFile = byPath.get(prefix + rules.scaffold.pinFile);
  let pin: { schema?: number; distribution?: string; engine?: { package?: string; version?: string }; probe?: { protocol?: number; wire?: number } } = {};
  if (!pinFile) problems.push(`${prefix}${rules.scaffold.pinFile}: missing; the overlay pins its engine there`);
  else pin = JSON.parse(decoder.decode(pinFile.bytes));
  if (pinFile) {
    if (pin.schema !== rules.scaffold.pin.schema) problems.push(`${prefix}${rules.scaffold.pinFile}: schema ${pin.schema}, the platform reads ${rules.scaffold.pin.schema}`);
    if (!pin.distribution) problems.push(`${prefix}${rules.scaffold.pinFile}: no distribution`);
    if (pin.probe?.protocol !== rules.scaffold.pin.probe.protocol || pin.probe?.wire !== rules.scaffold.pin.probe.wire) {
      problems.push(`${prefix}${rules.scaffold.pinFile}: probe ${JSON.stringify(pin.probe)}, the platform speaks protocol ${rules.scaffold.pin.probe.protocol} wire ${rules.scaffold.pin.probe.wire}`);
    }
  }
  const engineVersion = (JSON.parse(readFileSync(join(roots.engine, "package.json"), "utf8")) as { version: string }).version;
  if (pinFile && pin.engine?.version !== engineVersion) problems.push(`${prefix}${rules.scaffold.pinFile}: pins engine ${pin.engine?.version}, packages/engine is ${engineVersion}`);

  // project.ogt.
  const project = byPath.get(`${prefix}project.ogt`);
  if (project && isText(project.bytes)) {
    const { type, entries } = ogtTopLevel(decoder.decode(project.bytes));
    if (type !== "project") problems.push(`${prefix}project.ogt: type ${type}, expected project`);
    for (const k of PROJECT_KEYS) if (!entries.has(k)) problems.push(`${prefix}project.ogt: no \`${k}\``);
    if (entries.has("engine") && entries.get("engine") !== engineVersion) problems.push(`${prefix}project.ogt: engine "${entries.get("engine")}", packages/engine is ${engineVersion}`);
    const entry = entries.get("entry");
    if (entry && !byPath.has(prefix + entry)) problems.push(`${prefix}project.ogt: entry ${entry} does not exist in the template`);
  }

  // Binaries: never in the text payload; each named in distribution.json with its digest.
  const distFile = byPath.get(prefix + rules.scaffold.distributionFile);
  const listed = new Map<string, Record<string, unknown>>();
  if (!distFile) problems.push(`${prefix}${rules.scaffold.distributionFile}: missing; run \`bun scripts/scaffold.ts --write\``);
  else {
    const dist = JSON.parse(decoder.decode(distFile.bytes)) as { schema?: number; files?: Record<string, unknown>[] };
    for (const e of dist.files ?? []) {
      for (const k of rules.scaffold.distributionEntry) if (!(k in e)) problems.push(`${prefix}${rules.scaffold.distributionFile}: ${String(e.path)} has no ${k}`);
      listed.set(String(e.path), e);
    }
  }
  const binaries = files.filter((f) => f.path.startsWith(prefix) && !isText(f.bytes));
  for (const b of binaries) {
    const rel = b.path.slice(prefix.length);
    const e = listed.get(rel);
    if (!e) problems.push(`${b.path}: a binary file distribution.json does not list; run \`bun scripts/scaffold.ts --write\``);
    else if (e.sha256 !== sha256(b.bytes) || e.bytes !== b.bytes.length) problems.push(`${b.path}: differs from its distribution.json digest; run \`bun scripts/scaffold.ts --write\` after a deliberate asset change`);
  }
  for (const rel of listed.keys()) if (!binaries.some((b) => b.path === prefix + rel)) problems.push(`${prefix}${rules.scaffold.distributionFile}: lists ${rel}, which the template does not have`);
}

/** The bundle the platform's production reader would build: text files only. */
export function bundle(roots: Roots = defaultRoots()): { schemaVersion: 1; files: Record<string, string> } {
  const files: Record<string, string> = {};
  for (const f of payload(roots)) if (isText(f.bytes)) files[f.path] = decoder.decode(f.bytes);
  return { schemaVersion: 1, files };
}

export { packName, walk, TEMPLATE_SKIP };

if (import.meta.main) {
  const roots = defaultRoots();
  if (process.argv.includes("--bundle")) {
    process.stdout.write(`${JSON.stringify(bundle(roots))}\n`);
  } else {
    const problems = validate(roots);
    for (const p of problems) process.stderr.write(`${p}\n`);
    if (problems.length) {
      process.stderr.write(`\nengine knowledge: ${problems.length} problem(s).\n`);
      process.exit(1);
    }
    const hashes = packs(payload(roots));
    process.stdout.write(`engine knowledge valid against the platform rules: ${Object.entries(hashes).map(([n, h]) => `${n} ${h}`).join(", ")}\n`);
  }
}
