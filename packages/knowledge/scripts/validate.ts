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
export const SKILLS = ["og-ai", "og-audio", "og-build", "og-core", "og-feel", "og-game", "og-i18n", "og-platform-sdk", "og-taste", "og-ui"] as const;

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
  coherence(files, problems);
  apiNames(roots, files, problems);
  looks(roots, files, problems);
  grammar(roots, files, problems);
  scaffold(roots, rules, manifest.overlay, files, byPath, problems);
  return problems;
}

function packageJson(roots: Roots, problems: string[]): void {
  const p = JSON.parse(readFileSync(join(roots.pkg, "package.json"), "utf8")) as { name?: string; version?: string; files?: string[]; scripts?: Record<string, string> };
  if (p.name !== "@origingame/engine-knowledge") problems.push(`package.json: name is ${p.name}; the package is @origingame/engine-knowledge (design §9)`);
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(p.version ?? "")) problems.push(`package.json: version ${p.version} is not semver`);
  for (const f of ["wiki", "skills", "looks", "scaffold", "rules", "knowledge.json"]) {
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
/**
 * The skills as one set: the wiki index's grouping (a `### \`og-x\`` heading
 * over its modules' rows, from each module's `skill` in api.ogt) names
 * shipped skills; the skill a module is grouped under names its page
 * (`reference/<module>.md`) and no other skill does (they point to that
 * skill); and every section reference (`og-feel §4`, or `§4` for the skill's
 * own) names a numbered `## N.` section that exists.
 */
function coherence(files: readonly PayloadFile[], problems: string[]): void {
  const textOf = (path: string) => {
    const f = files.find((x) => x.path === path);
    return f && isText(f.bytes) ? decoder.decode(f.bytes) : undefined;
  };
  const index = textOf("game/skills/og-game/reference/index.md");
  if (index === undefined) return;
  const home = new Map<string, string>();
  let group: string | undefined;
  for (const line of index.slice(0, index.indexOf("\n## Units")).split("\n")) {
    const heading = /^### `([^`]+)`$/.exec(line);
    if (heading) {
      group = heading[1]!;
      if (!(SKILLS as readonly string[]).includes(group)) problems.push(`wiki/index.md: groups modules under ${group}, which is no skill this package ships; fix the modules' \`skill\` in api.ogt`);
      continue;
    }
    const row = /^\| \[`([a-z0-9]+)`\]\(([a-z0-9]+)\.md\)/.exec(line);
    if (row && group) home.set(row[1]!, group);
  }
  const texts = new Map<string, string>();
  for (const s of SKILLS) {
    const t = textOf(`game/skills/${s}/SKILL.md`);
    if (t !== undefined) texts.set(s, t);
  }
  const sections = new Map([...texts].map(([s, t]) => [s, new Set([...t.matchAll(/^## (\d+)\. /gm)].map((m) => m[1]!))]));
  for (const [module, skill] of home) {
    const t = texts.get(skill);
    if (t !== undefined && !t.includes(`reference/${module}.md`)) problems.push(`skills/${skill}/SKILL.md: teaches the ${module} module (wiki/index.md) but never names reference/${module}.md`);
  }
  for (const [skill, t] of texts) {
    for (const m of new Set([...t.matchAll(/\breference\/([a-z0-9-]+)\.md\b/g)].map((x) => x[1]!))) {
      const owner = home.get(m);
      if (owner && owner !== skill) problems.push(`skills/${skill}/SKILL.md: names reference/${m}.md, which ${owner} teaches; point to ${owner} instead, so each module is taught in one place`);
    }
    const prose = t.replace(/^```[\s\S]*?^```/gm, "");
    for (const m of prose.matchAll(/(og-[a-z]+(?:-[a-z]+)*) §([A-Za-z][\w ]*)/g)) problems.push(`skills/${skill}/SKILL.md: refers to ${m[1]} §${m[2]!.split(" ")[0]} by name; refer to a skill's sections by their numbers (## N.)`);
    for (const m of prose.matchAll(/§(\d+)/g)) {
      // The skill this § belongs to: walk back over a list of §s ("og-feel
      // §9–§11", "og-taste §4, §10") to the word before it.
      const before = prose.slice(0, m.index).split(/\s+/);
      let i = before.length - 1;
      while (i >= 0 && (before[i] === "" || /^\(?§\d+(\.\d+)?[–,;)]*$/.test(before[i]!) || before[i] === "and" || before[i] === "or")) i--;
      const word = (before[i] ?? "").replace(/^[(`*]+/, "");
      let target: string | undefined = skill;
      if (/^og-[a-z]+(?:-[a-z]+)*$/.test(word)) target = word;
      else if (/\.md\b|\//.test(word) || /^design/.test(word)) target = undefined;
      if (target === undefined) continue;
      const have = sections.get(target);
      if (!have) problems.push(`skills/${skill}/SKILL.md: refers to ${target} §${m[1]}, and there is no skill ${target}`);
      else if (!have.has(m[1]!)) problems.push(`skills/${skill}/SKILL.md: refers to ${target === skill ? "its own" : target} §${m[1]}, which ${target === skill ? "it does not have" : "does not exist there"} (its sections: ${[...have].join(", ") || "none numbered"})`);
    }
  }
}

/** The names the generated API answers to, read from packages/engine/schema/api.schema.json. */
export interface ApiNames {
  /** `og.<name>`: root functions, component, resource and event tokens, and module namespaces. */
  root: Set<string>;
  /** `og.<module>.<name>`: a namespace module's functions and values. */
  members: Map<string, Set<string>>;
  /** `<Type>.<field>` and `og.<Token>.<field>`: the fields of every struct, desc, component, resource and event. */
  fields: Map<string, Set<string>>;
  /** Every field name of any of them. */
  anyField: Set<string>;
}

/** `og.` names a skill may write that are not API: each with why. */
export const NOT_API: Record<string, string> = {
  "og.settings": "the platform store key settings are saved under (og-platform-sdk §2)",
  "og.AudioPlayback": "an engine-owned component dump_entity shows while a voice plays; game code never names it",
  "og.AudioStream": "an engine-owned component dump_entity shows on a streamed voice; game code never names it",
  "og.physics.queries": "the engine system dump_world lists that records the colliders a system's queries see",
  "og.events": "the key of a glTF animation's extras the cook reads a clip's markers from (og-game §9), a name in the glTF file",
  "og.TimeScale": "the engine-owned resource that keeps og.Time's scale and hit stop; dumps list it and the probe's get and set reach it, game code reads og.Time",
};

/** `` `key: value` `` spans a skill may write whose key is no field of the generated API: each with why. */
export const NOT_FIELDS: Record<string, string> = {
  runs: "PlaytestDesc.runs (packages/engine/src/playtest.ts)",
  then: "PlaytestDesc.then (packages/engine/src/playtest.ts)",
  store: "PlaytestRun.store (packages/engine/src/playtest.ts)",
};

export function readApiNames(engine: string): ApiNames | undefined {
  const path = join(engine, "schema", "api.schema.json");
  if (!existsSync(path)) return undefined;
  const schema = JSON.parse(readFileSync(path, "utf8")) as {
    "x-og-modules"?: { name: string; place: string; functions?: { name: string }[]; values?: { name: string }[] }[];
    $defs?: Record<string, { "x-og-kind"?: string; properties?: Record<string, unknown> }>;
  };
  const names: ApiNames = { root: new Set(), members: new Map(), fields: new Map(), anyField: new Set() };
  for (const m of schema["x-og-modules"] ?? []) {
    const fns = [...(m.functions ?? []), ...(m.values ?? [])].map((f) => f.name);
    if (m.place === "root") for (const f of fns) names.root.add(f);
    if (m.place === "namespace") {
      names.root.add(m.name);
      names.members.set(m.name, new Set(fns));
    }
  }
  for (const [name, def] of Object.entries(schema.$defs ?? {})) {
    const kind = def["x-og-kind"] ?? "";
    if (["component", "resource", "event"].includes(kind)) names.root.add(name);
    if (def.properties) names.fields.set(name, new Set(Object.keys(def.properties)));
    for (const k of Object.keys(def.properties ?? {})) names.anyField.add(k);
  }
  return names;
}

/**
 * Every API name a skill writes resolves in the generated API: `og.<x>` is a
 * root function, a component, resource or event token or a module
 * namespace; `og.<module>.<y>` one of the module's functions or values;
 * `og.<Token>.<field>` and `<Type>.<field>` (for a struct, desc, component,
 * resource or event the IDL declares) one of its fields. Code blocks are
 * read too: an example is API as much as prose is. A file name such as
 * `away.og.ts` is not a name.
 */
function apiNames(roots: Roots, files: readonly PayloadFile[], problems: string[]): void {
  const api = readApiNames(roots.engine);
  if (!api) return;
  for (const f of files) {
    const m = /^game\/skills\/([^/]+)\/SKILL\.md$/.exec(f.path);
    if (!m || !isText(f.bytes)) continue;
    const at = `skills/${m[1]}/SKILL.md`;
    const text = decoder.decode(f.bytes);
    const bad = new Set<string>();
    for (const x of text.matchAll(/(?<![\w./@-])og\.([A-Za-z_]\w*)(?:\.([A-Za-z_]\w*))?/g)) {
      const [whole, head, member] = [x[0], x[1]!, x[2]];
      if (whole in NOT_API || `og.${head}` in NOT_API) continue;
      if (!api.root.has(head)) {
        bad.add(`og.${head}`);
        continue;
      }
      if (member === undefined) continue;
      const ok = api.members.has(head) ? api.members.get(head)!.has(member) : (api.fields.get(head)?.has(member) ?? false);
      if (!ok) bad.add(whole);
    }
    for (const x of text.matchAll(/(?<![\w./@-])([A-Z][A-Za-z0-9]*)\.([a-z]\w*)\b/g)) {
      const have = api.fields.get(x[1]!);
      if (have && !have.has(x[2]!)) bad.add(x[0]);
    }
    // A component, resource or event's value written beside its token
    // (`world.insert(e, og.AudioSource, { sound, loop: true })`): each key one of its fields.
    for (const x of text.matchAll(/(?<![\w./@-])og\.([A-Z]\w*)\s*,\s*\{([^{}]*)\}/g)) {
      const have = api.fields.get(x[1]!);
      if (!have) continue;
      for (const part of x[2]!.split(",")) {
        const key = /^\s*([A-Za-z_]\w*)\s*(?::|$)/.exec(part)?.[1];
        if (key && !have.has(key)) bad.add(`og.${x[1]}.${key}`);
      }
    }
    // A field set in prose, `` `looping: true` ``: its key is some type's field.
    for (const x of text.matchAll(/`([a-z]\w*): [^`]+`/g)) {
      if (!api.anyField.has(x[1]!) && !(x[1]! in NOT_FIELDS)) bad.add(x[1]!);
    }
    for (const b of bad) problems.push(`${at}: names \`${b}\`, which the generated API (api.ogt) does not have; check the name in reference/ or packages/engine/types/index.d.ts`);
  }
}

/** A style catalogue entry (looks/catalogue.json; og-taste §12). */
interface Style {
  slug: string;
  name: string;
  category: string;
  descriptor: string;
  checks: string[];
  premise: Record<string, string>;
  look: { preset: string; presetReady: boolean; support: string; primitives: string[]; missing: string[] };
  skin: string;
  palette: { size: string; rules: string };
  assets: Record<string, string>;
  artBible: { direction: string; avoid: string[] };
}

/** What `look.support` may say: full (every primitive draws), partial or planned (`missing` names what is still to land), content (the look is authored art). */
export const SUPPORT = ["full", "partial", "planned", "content"] as const;
/** The placeholders the art-bible template may use. */
export const BIBLE_FIELDS = ["title", "premise", "camera", "platform", "name", "descriptor", "direction", "palette", "assets", "skin", "avoid", "game", "slug", "checks"] as const;
const PREMISE = ["genre", "mood", "era", "audience"];
const ASSET_SPECS = ["logicalResolution", "spriteScale", "faceBudget", "outlinePx", "textureDensity"];

/**
 * The style catalogue (looks/catalogue.json, mounted beside og-taste): each
 * style complete (three checks, the premise, palette, asset specs, an art
 * bible direction), its preset path `templates/looks/<slug>.ogt` (a
 * `type look` file when presetReady), its support honest (`missing` names
 * the primitives a style that is not `full` lacks, and is empty for one
 * that is), every `Resource.field` it names a member of the generated API,
 * og-taste naming every slug, and no text phrased as an imitation.
 */
function looks(roots: Roots, files: readonly PayloadFile[], problems: string[]): void {
  const path = join(roots.pkg, "looks", "catalogue.json");
  const at = "looks/catalogue.json";
  if (!existsSync(path)) {
    problems.push(`${at}: missing; og-taste §12 reads the style catalogue from it`);
    return;
  }
  let cat: { schemaVersion?: number; categories?: Record<string, string>; skins?: string[]; artBibleTemplate?: string; styles?: Style[] };
  try {
    cat = JSON.parse(readFileSync(path, "utf8"));
  } catch (e) {
    problems.push(`${at}: not JSON (${(e as Error).message})`);
    return;
  }
  if (cat.schemaVersion !== 1) problems.push(`${at}: schemaVersion ${cat.schemaVersion}, expected 1`);
  const categories = Object.keys(cat.categories ?? {});
  const skins = cat.skins ?? [];
  for (const m of (cat.artBibleTemplate ?? "").matchAll(/\{(\w+)\}/g)) {
    if (!(BIBLE_FIELDS as readonly string[]).includes(m[1]!)) problems.push(`${at}: artBibleTemplate uses {${m[1]}}, which is none of ${BIBLE_FIELDS.join(", ")}`);
  }
  if (!cat.artBibleTemplate) problems.push(`${at}: no artBibleTemplate`);
  const schemaPath = join(roots.engine, "schema", "api.schema.json");
  const defs = existsSync(schemaPath) ? ((JSON.parse(readFileSync(schemaPath, "utf8")) as { $defs?: Record<string, { properties?: Record<string, unknown> }> }).$defs ?? {}) : undefined;
  const repo = join(roots.pkg, "..", "..");
  const taste = files.find((f) => f.path === "game/skills/og-taste/SKILL.md");
  const tasteText = taste && isText(taste.bytes) ? decoder.decode(taste.bytes) : "";
  const seen = new Set<string>();
  const text = (s: unknown) => typeof s === "string" && s.trim().length > 0;
  for (const [i, s] of (cat.styles ?? []).entries()) {
    const id = `${at}: ${s.slug ?? `styles[${i}]`}`;
    if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(s.slug ?? "")) problems.push(`${id}: slug is not lowercase-hyphen`);
    if (seen.has(s.slug)) problems.push(`${id}: a second style with this slug`);
    seen.add(s.slug);
    for (const k of ["name", "descriptor", "skin"] as const) if (!text(s[k])) problems.push(`${id}: no ${k}`);
    if (!categories.includes(s.category)) problems.push(`${id}: category ${s.category} is none of ${categories.join(", ")}`);
    if (!skins.includes(s.skin)) problems.push(`${id}: skin ${s.skin} is none of the packs (${skins.join(", ")})`);
    if (!Array.isArray(s.checks) || s.checks.length !== 3 || !s.checks.every(text)) problems.push(`${id}: needs exactly three checks a critic can judge on a frame`);
    for (const k of PREMISE) if (!text(s.premise?.[k])) problems.push(`${id}: premise has no ${k}`);
    for (const k of ASSET_SPECS) if (!text(s.assets?.[k])) problems.push(`${id}: assets has no ${k}`);
    if (!text(s.palette?.size) || !text(s.palette?.rules)) problems.push(`${id}: palette needs a size and rules`);
    if (!text(s.artBible?.direction) || !Array.isArray(s.artBible?.avoid) || s.artBible.avoid.length === 0) problems.push(`${id}: artBible needs a direction and what to avoid`);
    const look = s.look;
    if (!look) {
      problems.push(`${id}: no look`);
      continue;
    }
    if (look.preset !== `templates/looks/${s.slug}.ogt`) problems.push(`${id}: look.preset is ${look.preset}, expected templates/looks/${s.slug}.ogt`);
    if (!(SUPPORT as readonly string[]).includes(look.support)) problems.push(`${id}: look.support ${look.support} is none of ${SUPPORT.join(", ")}`);
    const missing = look.missing ?? [];
    if (look.support === "full" && missing.length > 0) problems.push(`${id}: support full, yet it names missing primitives`);
    if (look.support !== "full" && !missing.some(text)) problems.push(`${id}: support ${look.support} must name what is missing (the primitive or the authored art)`);
    if (!Array.isArray(look.primitives) || look.primitives.length === 0) problems.push(`${id}: look.primitives names none`);
    if (defs) {
      for (const p of look.primitives ?? []) {
        const m = /^([A-Z]\w*)\.(\w+)$/.exec(p);
        if (m && !defs[m[1]!]?.properties?.[m[2]!]) problems.push(`${id}: names ${p}, which the generated API does not have`);
      }
    }
    const preset = join(repo, look.preset);
    if (look.presetReady) {
      if (existsSync(join(repo, "templates")) && !existsSync(preset)) problems.push(`${id}: presetReady, but ${look.preset} does not exist`);
      else if (existsSync(preset) && !/^type look\s*$/m.test(readFileSync(preset, "utf8"))) problems.push(`${id}: ${look.preset} is not an .ogt of type look`);
    }
    if (tasteText && !tasteText.includes(`\`${s.slug}\``)) problems.push(`skills/og-taste/SKILL.md: the style catalogue (§12) does not list \`${s.slug}\``);
    else if (tasteText) {
      // §12's row for the style says what the catalogue says: family, name, support, skin.
      const row = new RegExp(`^\\| \`${s.slug.replace(/[-]/g, "\\-")}\` \\| ([^|]+) \\| ([^|]+) \\| ([^|]+) \\| \`([^\`]+)\` \\|$`, "m").exec(tasteText);
      const want = [cat.categories?.[s.category] ?? s.category, s.name, look.support, s.skin];
      const have = row?.slice(1, 5).map((x) => x.trim());
      if (!have) problems.push(`skills/og-taste/SKILL.md: §12's table has no row for \`${s.slug}\` (| slug | family | style | support | skin |)`);
      else if (have.join("|") !== want.join("|")) problems.push(`skills/og-taste/SKILL.md: §12's row for \`${s.slug}\` reads ${have.join(", ")}; the catalogue says ${want.join(", ")}`);
    }
    const all = JSON.stringify(s);
    if (/in the style of|™|®/i.test(all)) problems.push(`${id}: phrased as an imitation (\"in the style of\", a trademark sign); describe the look in its own terms`);
  }
  if (seen.size === 0) problems.push(`${at}: no styles`);
}

/**
 * A text format the skills write out line by line, held to the parser that
 * reads it: every `.ogt` block of that `type` in a skill (a ```text code
 * block) and the parser's source.
 */
export interface Grammar {
  /** The block's `type` line. */
  readonly type: string;
  /** For `type import`: the nested block (`tilemap { ... }`) whose lines are this grammar. */
  readonly block?: string;
  /** The parser, relative to the engine repository. */
  readonly source: string;
  /** The part of the source that parses it: from the first match to the second. */
  readonly region?: readonly [RegExp, RegExp];
  /** Keys a block may write that the parser accepts without naming them: each with why. */
  readonly unnamed?: Readonly<Record<string, string>>;
  /** Keys whose next words are names the author chooses (a state, a route, a motion), not grammar: how many. */
  readonly names?: Readonly<Record<string, number>>;
  /** The keys the parser reads, from its region; default: its match arms' first keys (`"grid" =>`). */
  readonly reads?: (region: string) => string[];
  /** The skill that writes the grammar out (default og-game), for the messages. */
  readonly home?: string;
}

const quotedIn = (s: string) => [...s.matchAll(/"([A-Za-z_]\w*)"/g)].map((m) => m[1]!);

/**
 * Every key a parser reads by name: each alternative of a match arm
 * (`"enter" | "exit" =>`, `Some("all") =>`), a block lookup (`.get("agent")`,
 * `.get_all("box")`, `f32_of(agent, "radius"`, `opt_f32(b, "reach"`), a key
 * compared (`.key != "child"`), a `matches!` list, an option list
 * (`check_options(e, &["speed"])`) and the list of keys og.ai writes itself.
 */
export function readKeys(region: string): string[] {
  const id = '"([A-Za-z_]\\w*)"';
  const one = [
    new RegExp(`${id}(?=(?:\\s*\\|\\s*"[A-Za-z_]\\w*")*\\s*=>)`, "g"),
    new RegExp(`Some\\(${id}\\)\\s*=>`, "g"),
    new RegExp(`\\.get(?:_all)?\\(${id}\\)`, "g"),
    new RegExp(`(?:_of|opt_f32)\\([^,()]*(?:\\([^()]*\\))?,\\s*${id}`, "g"),
    new RegExp(`\\.key\\s*[!=]=\\s*${id}`, "g"),
  ];
  const lists = [/matches!\(\s*[^,]+,\s*((?:"\w+"\s*\|?\s*)+)\)/g, /check_options\(\w+,\s*&\[([^\]]*)\]/g, /BUILTIN_KEYS: &\[&str\] = &\[([^\]]*)\]/g];
  return [...one.flatMap((re) => [...region.matchAll(re)].map((m) => m[1]!)), ...lists.flatMap((re) => [...region.matchAll(re)].flatMap((m) => quotedIn(m[1]!)))];
}

/**
 * The grammars the skills write out (og-game §8 tilesets and maps, §9 a
 * model's animation settings and the animation graph, §10 emitters; og-ai
 * §1 the navmesh, §3 behaviours) and the parsers they come from.
 */
export const GRAMMARS: readonly Grammar[] = [
  { type: "tileset", source: "crates/og-tilemap/src/tileset.rs" },
  { type: "tilemap", source: "crates/og-tilemap/src/map.rs", region: [/pub fn parse\(text/, /^pub fn world_to_cell/m] },
  { type: "particles", source: "crates/og-cook/src/fx.rs", unnamed: { key: "colorOverLife's lines: the parser reads their five numbers whatever the key" } },
  { type: "import", block: "tilemap", source: "crates/og-cook/src/cook/tilemap.rs", region: [/pub fn parse\(block/, /pub fn describe/] },
  { type: "import", block: "animation", source: "crates/og-cook/src/cook/anim.rs", region: [/pub fn parse\(block/, /^fn asset_path/m] },
  {
    type: "animgraph",
    source: "crates/og-anim/src/graph.rs",
    region: [/pub fn from_document/, /\/\/\/ The slot of a parameter/],
    reads: readKeys,
    names: { param: 1, clip: 1, blend1d: 2, blend2d: 3, child: 1, state: 2, entry: 1, transition: 2, when: 1, weight: 1 },
  },
  { type: "navmesh", source: "crates/og-cook/src/cook/e6.rs", region: [/pub\(super\) fn cook_navmesh/, /og_nav::bake::bake\(/], reads: readKeys, home: "og-ai" },
  {
    type: "behaviour",
    source: "crates/og-ai/src/asset.rs",
    reads: readKeys,
    home: "og-ai",
    // A label, a route, a state, a key, an operand or a speed's field are the author's; a command's arguments too.
    names: { selector: 1, sequence: 1, steps: 1, parallel: 1, route: 1, patrol: 1, state: 1, initial: 1, to: 1, key: 1, set: 1, check: 1, move_to: 1, chase: 1, flee: 1, speed: 1, command: Infinity },
  },
];

/** The Tiled fields a skill's `<!-- grammar: tiled -->` table names, and the importer that reads them. */
export const TILED = {
  source: "crates/og-cook/src/cook/tilemap.rs",
  region: [/pub fn import_tiled/, /^fn tiled_data/m] as const,
  /** Fields the importer reads that are structure, not settings: each with why. */
  structural: { layers: "a group layer's children, flattened", objects: "an object layer's objects" } as Record<string, string>,
};

/** Keys after which the next word is a name the author chooses, not grammar. */
const NAMING = new Set(["terrain", "layer", "object", "kind", "prop"]);

interface Line {
  readonly words: string[];
  readonly depth: number;
  readonly text: string;
}

/** An `.ogt` block's lines as tokens: strings dropped, comments cut, numbers kept as written. */
function ogtLines(block: string): Line[] {
  const out: Line[] = [];
  let depth = 0;
  for (const raw of block.split("\n")) {
    const words: string[] = [];
    const re = /"(?:[^"\\]|\\.)*"|\S+/g;
    let opens = false;
    for (const m of raw.matchAll(re)) {
      const t = m[0];
      if (t.startsWith("#")) break;
      if (t === "{") opens = true;
      else if (t === "}") depth--;
      else words.push(t.startsWith('"') ? '""' : t);
    }
    out.push({ words, depth, text: raw });
    if (opens) depth++;
  }
  return out;
}

/** The ```text blocks of a markdown file. */
function textBlocks(md: string): string[] {
  return [...md.matchAll(/^```text\n([\s\S]*?)^```/gm)].map((m) => m[1]!);
}

/**
 * Every problem between the grammars the skills write and their parsers:
 * a bare word a block writes (a key, or a word value such as `blob47`)
 * that the parser never names in quotes, and a key the parser matches
 * (`"grid" =>`) that no block of its type writes, in code or comment. So a
 * key renamed, added or dropped on either side fails here.
 */
export function grammarDrift(skills: ReadonlyMap<string, string>, read: (path: string) => string | undefined): string[] {
  const problems: string[] = [];
  for (const g of GRAMMARS) {
    let source = read(g.source);
    if (source === undefined) continue;
    if (g.region) {
      const a = g.region[0].exec(source)?.index ?? -1;
      const b = a < 0 ? -1 : source.slice(a).search(g.region[1]);
      if (a < 0 || b < 0) {
        problems.push(`${g.source}: the ${g.type} parser's region (${g.region[0]} to ${g.region[1]}) is not found; update GRAMMARS in scripts/validate.ts`);
        continue;
      }
      source = source.slice(a, a + b);
    }
    const home = g.home ?? "og-game";
    const quoted = new Set(quotedIn(source));
    const arms = new Set(g.reads ? g.reads(source) : [...source.matchAll(/"([A-Za-z_]\w*)"\s*(?:\|[^=]*)?=>/g)].map((m) => m[1]!));
    const written: string[] = [];
    let blocks = 0;
    for (const [skill, text] of skills) {
      for (const block of textBlocks(text)) {
        const lines = ogtLines(block);
        if (lines[0]?.words.join(" ") !== `type ${g.type}`) continue;
        let body = lines.slice(lines.findIndex((l) => l.words[0] === "---") + 1);
        if (g.block) {
          const start = body.findIndex((l) => l.depth === 0 && l.words[0] === g.block && l.text.includes("{"));
          if (start < 0) continue;
          const end = body.findIndex((l, i) => i > start && l.depth === 0);
          body = body.slice(start + 1, end < 0 ? undefined : end);
        }
        blocks++;
        written.push(body.map((l) => l.text).join("\n"));
        for (const l of body) {
          l.words.forEach((w, i) => {
            if (!/^[A-Za-z_]\w*$/.test(w) || w === "true" || w === "false") return;
            if (i > 0 && NAMING.has(l.words[i - 1]!)) return;
            if (i > 0 && i <= (g.names?.[l.words[0]!] ?? 0)) return;
            if (i === 0 && g.unnamed && w in g.unnamed) return;
            if (!quoted.has(w)) problems.push(`skills/${skill}/SKILL.md: a \`type ${g.type}\` block${g.block ? `'s ${g.block} block` : ""} writes \`${w}\` (${l.text.trim()}), which ${g.source} does not read; fix the grammar to the parser's`);
          });
        }
      }
    }
    if (blocks === 0) {
      problems.push(`skills: no skill writes a \`type ${g.type}\` block${g.block ? ` with ${/^[aeiou]/.test(g.block) ? "an" : "a"} ${g.block} block` : ""}; ${home}'s grammar of it is gone`);
      continue;
    }
    const all = written.join("\n");
    for (const k of arms) {
      if (!new RegExp(`(?<![\\w-])${k}(?![\\w-])`).test(all)) problems.push(`${g.source}: reads \`${k}\`, which no \`type ${g.type}\`${g.block ? ` ${g.block}` : ""} block in the skills writes or names; add it to ${home}'s grammar`);
    }
  }

  // Tiled: the table after `<!-- grammar: tiled -->`.
  const importer = read(TILED.source);
  if (importer !== undefined) {
    const a = TILED.region[0].exec(importer)?.index ?? -1;
    const b = a < 0 ? -1 : importer.slice(a).search(TILED.region[1]);
    const region = a < 0 || b < 0 ? undefined : importer.slice(a, a + b);
    if (!region) problems.push(`${TILED.source}: the Tiled importer's region is not found; update TILED in scripts/validate.ts`);
    const named = new Set<string>();
    let tables = 0;
    for (const [skill, text] of skills) {
      for (const m of text.matchAll(/<!-- grammar: tiled -->\n((?:\|.*\n?)+)/g)) {
        tables++;
        for (const row of m[1]!.split("\n").slice(2)) {
          const first = row.split("|")[1] ?? "";
          for (const n of first.matchAll(/`([^`]+)`/g)) {
            named.add(n[1]!);
            if (region && !region.includes(`"${n[1]}"`)) problems.push(`skills/${skill}/SKILL.md: the Tiled table names \`${n[1]}\`, which ${TILED.source}'s Tiled importer does not read`);
          }
        }
      }
    }
    if (tables === 0) problems.push("skills: no skill has the Tiled table (`<!-- grammar: tiled -->`); og-game §8's list of what the cook reads of a Tiled map is gone");
    else if (region) {
      const read = [
        ...region.matchAll(/\bprops\s*\.get\("(\w+)"\)/g),
        ...region.matchAll(/\bf\(l, "(\w+)"/g),
        ...region.matchAll(/\bl\s*\.get\("(\w+)"\)/g),
      ].map((m) => m[1]!);
      for (const k of new Set(read)) {
        if (!named.has(k) && !(k in TILED.structural)) problems.push(`${TILED.source}: the Tiled importer reads \`${k}\`, which og-game's Tiled table does not name; add a row`);
      }
    }
  }
  return problems;
}

function grammar(roots: Roots, files: readonly PayloadFile[], problems: string[]): void {
  const repo = join(roots.pkg, "..", "..");
  if (!existsSync(join(repo, "crates"))) return; // the published package: the parsers are not beside it
  const skills = new Map<string, string>();
  for (const f of files) {
    const m = /^game\/skills\/([^/]+)\/SKILL\.md$/.exec(f.path);
    if (m && isText(f.bytes)) skills.set(m[1]!, decoder.decode(f.bytes));
  }
  problems.push(...grammarDrift(skills, (p) => (existsSync(join(repo, p)) ? readFileSync(join(repo, p), "utf8") : undefined)));
}

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
