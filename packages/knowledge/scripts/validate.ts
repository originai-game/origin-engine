// Validates @origingame/engine-knowledge against the OriginGame platform's
// knowledge rules (rules/platform.json, transcribed from the platform with
// scripts/sync-rules.ts; each rule names its platform source), and the
// package's own contract: a generated wiki with its guides, and a scaffold
// overlay whose pin and distribution list match the project template.
//
//   bun scripts/validate.ts            problems, then the packs and hashes
//   bun scripts/validate.ts --bundle   the payload as the platform's
//                                      readProductionBundle shape (text only)

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, posix, relative } from "node:path";
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
import { ART_KINDS } from "./brief.ts";

export interface Rules {
  payload: { roots: string[] };
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

/**
 * The guides the wiki carries (generated from crates/og-api/guides/ by `og api
 * gen` as wiki/guide-<name>.md): workflow and thresholds read through `og api
 * find` and `og api show`, held to the API (every `og.*` name and
 * `Type.field` exists), to their own section references and to the `.ogt`
 * line shape.
 */
export const GUIDE_PREFIX = "guide-";

/** The og-cli source that cites the guides by section, in the engine repository. */
export const GUIDE_CITERS = ["crates/og-cli/src/migrate.rs"] as const;

/** What a scaffolded project must contain (scaffold/README.md describes each). */
export const TEMPLATE_FILES = ["project.ogt", "src/main.ts", "package.json", "tsconfig.json", "tests/smoke.og.ts"] as const;

/** The keys project.ogt must set (scaffold/README.md). */
export const PROJECT_KEYS = ["name", "id", "engine", "entry", "tick", "seed", "publish"] as const;

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

  guides(roots, rules, problems);
  skinWord(roots, problems);
  apiNames(roots, problems);
  looks(roots, problems);
  problems.push(...ogtShape(guideTexts(roots)));
  scaffold(roots, rules, manifest.overlay, files, byPath, problems);
  return problems;
}

function packageJson(roots: Roots, problems: string[]): void {
  const p = JSON.parse(readFileSync(join(roots.pkg, "package.json"), "utf8")) as { name?: string; version?: string; files?: string[]; scripts?: Record<string, string> };
  if (p.name !== "@origingame/engine-knowledge") problems.push(`package.json: name is ${p.name}; the package is @origingame/engine-knowledge (design §9)`);
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(p.version ?? "")) problems.push(`package.json: version ${p.version} is not semver`);
  for (const f of ["wiki", "looks", "scaffold", "rules", "knowledge.json"]) {
    if (!p.files?.some((x) => x === f || x === `${f}/`)) problems.push(`package.json: files does not publish ${f}`);
  }
  if (!p.scripts?.test) problems.push("package.json: no test script, so scripts/check.sh bun-test would skip the package");
}

/** Every page of the generated wiki, as its path under wiki/ (`physics.md`, `physics/record-CharacterController.md`). */
function wikiPages(dir: string, at = ""): string[] {
  if (!existsSync(join(dir, at))) return [];
  const out: string[] = [];
  for (const e of readdirSync(join(dir, at), { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const rel = at ? `${at}/${e.name}` : e.name;
    if (e.isDirectory()) out.push(...wikiPages(dir, rel));
    else if (e.name.endsWith(".md")) out.push(rel);
  }
  return out;
}

/**
 * The generated wiki: every page carries the generated header, every link
 * lands on a page, every page but the index is linked from another (module
 * pages from the index, item pages from their module's contents), and the
 * `og api find` index ships beside them.
 */
function wiki(roots: Roots, problems: string[]): void {
  const dir = join(roots.pkg, "wiki");
  if (!existsSync(join(dir, "index.md"))) {
    problems.push("wiki/index.md: missing; run `og api gen --wiki` in the engine repository");
    return;
  }
  if (!existsSync(join(dir, "search.tsv"))) problems.push("wiki/search.tsv: missing (the `og api find` index); run `og api gen --wiki`");
  const pages = wikiPages(dir);
  const have = new Set(pages);
  const linked = new Set<string>();
  for (const name of pages) {
    const text = readFileSync(join(dir, name), "utf8");
    if (!text.startsWith(GENERATED)) problems.push(`wiki/${name}: not generated from api.ogt (no generated header); wiki pages are never hand-written: run \`og api gen --wiki\``);
    const base = posix.dirname(name);
    for (const m of text.matchAll(/\]\(((?:\.\.\/)*[a-z0-9_\/-]+\.md)[#)]/gi)) {
      const target = posix.normalize(posix.join(base, m[1]!));
      if (target === name) continue;
      linked.add(target);
      if (!have.has(target)) problems.push(`wiki/${name}: links to ${target}, which does not exist; run \`og api gen --wiki\``);
    }
  }
  for (const name of pages) if (name !== "index.md" && !linked.has(name)) problems.push(`wiki/${name}: not linked from any wiki page (a page for a name the IDL no longer has?); run \`og api gen --wiki\``);
}

/** Tokens a text names as tools, as the platform's production test reads them. */
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

/** The generated wiki's module pages (wiki/<module>.md, index.md left out), by module name. */
function wikiModules(roots: Roots): Set<string> {
  const dir = join(roots.pkg, "wiki");
  if (!existsSync(dir)) return new Set();
  return new Set(readdirSync(dir).filter((n) => n.endsWith(".md") && n !== "index.md").map((n) => n.slice(0, -3)));
}

/** The modules a text names as `og api show <module>`: a lowercase word, not a dotted name such as `og.physics.castRay`. */
export function shownModules(text: string): Set<string> {
  return new Set([...text.matchAll(/\bog api show ([a-z][a-z0-9]*)(?![\w.])/g)].map((m) => m[1]!));
}

/**
 * What a guide says of the API it points to: a `reference/<page>.md`
 * pointer reaches no page `og api` serves
 * nothing (`og api show` and `og api find` answer instead), and every
 * `og api show <module>` names a module the wiki has a page for.
 */
function apiPointers(at: string, text: string, modules: ReadonlySet<string>): string[] {
  const problems: string[] = [];
  for (const page of new Set([...text.matchAll(/\breference\/([a-z0-9-]+\.md)\b/g)].map((m) => m[1]!))) {
    problems.push(`${at}: names reference/${page}, which no reader opens: name \`og api show <module>\` for a module, \`og api show <exact name>\` for one name, or \`og api find <words>\``);
  }
  for (const m of shownModules(text)) {
    if (!modules.has(m)) problems.push(`${at}: names \`og api show ${m}\`, and the generated wiki has no module ${m} (wiki/index.md lists them; write a type or function as its full name, og.<name>)`);
  }
  return problems;
}

/** The guide pages of the wiki, by guide name (wiki/guide-<name>.md). */
export function guideTexts(roots: Roots): Map<string, string> {
  const dir = join(roots.pkg, "wiki");
  const out = new Map<string, string>();
  if (!existsSync(dir)) return out;
  for (const n of readdirSync(dir).sort()) {
    if (n.startsWith(GUIDE_PREFIX) && n.endsWith(".md")) out.set(n.slice(GUIDE_PREFIX.length, -3), readFileSync(join(dir, n), "utf8"));
  }
  return out;
}

/** A guide page's numbered sections, as `og api gen` heads them (`### \`testing §2\` Title`). */
function guideSections(text: string): Set<string> {
  return new Set([...text.matchAll(/^### `[a-z0-9-]+ §(\d+)` /gm)].map((m) => m[1]!));
}

/**
 * The guides as one set: each names no tool a Session lacks, no unmounted
 * wiki page and no module the IDL does not generate; every section
 * reference (`testing §2`, `§2` for its own) names a section that exists;
 * every `three-to-og §N` the og-cli source cites (GUIDE_CITERS) is one.
 */
function guides(roots: Roots, rules: Rules, problems: string[]): void {
  const texts = guideTexts(roots);
  const modules = wikiModules(roots);
  const sections = new Map([...texts].map(([g, t]) => [g, guideSections(t)]));
  for (const [g, text] of texts) {
    const at = `wiki/${GUIDE_PREFIX}${g}.md`;
    for (const t of namedTools(text, rules)) problems.push(`${at}: names \`${t}\`, which is no tool a Session can be offered; rename it or put it in a code block if it is not a tool (crates/og-api/guides/${g}.md)`);
    problems.push(...apiPointers(at, text, modules));
    problems.push(...sectionRefs(at, g, text, sections));
  }
  const repo = join(roots.pkg, "..", "..");
  if (!existsSync(join(repo, "crates"))) return; // the published package: the sources are not beside it
  for (const citer of GUIDE_CITERS) {
    const path = join(repo, citer);
    if (!existsSync(path)) {
      problems.push(`${citer}: missing; GUIDE_CITERS in scripts/validate.ts names it`);
      continue;
    }
    for (const m of readFileSync(path, "utf8").matchAll(/\b([a-z0-9]+(?:-[a-z0-9]+)+) §(\d+)/g)) {
      const have = sections.get(m[1]!);
      if (have && !have.has(m[2]!)) problems.push(`${citer}: cites ${m[1]} §${m[2]}, which wiki/${GUIDE_PREFIX}${m[1]}.md does not have (its sections: ${[...have].join(", ")})`);
    }
  }
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
  // The published package's scaffold/template carries text only (scripts/pack.ts): the binaries it lists are @origingame/create-game's.
  const published = roots.template === join(roots.pkg, "scaffold", "template");
  if (!published) for (const rel of listed.keys()) if (!binaries.some((b) => b.path === prefix + rel)) problems.push(`${prefix}${rules.scaffold.distributionFile}: lists ${rel}, which the template does not have`);
}

/**
 * Every section reference of one text (`testing §2`, or `§2` for its own:
 * any guide `sections` holds, or an `og-*` name) names a numbered section
 * that exists: `at` names the text in the messages, `own` the text itself.
 */
export function sectionRefs(at: string, own: string, text: string, sections: ReadonlyMap<string, ReadonlySet<string>>): string[] {
  const problems: string[] = [];
  const prose = text.replace(/^```[\s\S]*?^```/gm, "");
  const owner = new RegExp(`(og(?:-[a-z0-9]+)+${[...sections.keys()].map((n) => `|${n}`).join("")}) §([A-Za-z][\\w ]*)`, "g");
  for (const m of prose.matchAll(owner)) problems.push(`${at}: refers to ${m[1]} §${m[2]!.split(" ")[0]} by name; refer to a guide's sections by their numbers (## N.)`);
  for (const m of prose.matchAll(/§(\d+)/g)) {
    // The text this § belongs to: walk back over a list of §s ("og-core
    // §9–§11", "art-review §4, §5") to the word before it.
    const before = prose.slice(0, m.index).split(/\s+/);
    let i = before.length - 1;
    while (i >= 0 && (before[i] === "" || /^\(?§\d+(\.\d+)?[–,;)]*$/.test(before[i]!) || before[i] === "and" || before[i] === "or")) i--;
    const word = (before[i] ?? "").replace(/^[(`*]+/, "");
    let target: string | undefined = own;
    if (/^og-[a-z0-9]+(?:-[a-z0-9]+)*$/.test(word) || sections.has(word)) target = word;
    else if (/\.md\b|\//.test(word) || /^design/.test(word)) target = undefined;
    if (target === undefined) continue;
    const have = sections.get(target);
    if (!have) problems.push(`${at}: refers to ${target} §${m[1]}, and there is no guide ${target}`);
    else if (!have.has(m[1]!)) problems.push(`${at}: refers to ${target === own ? "its own" : target} §${m[1]}, which ${target === own ? "it does not have" : "does not exist there"} (its sections: ${[...have].join(", ") || "none numbered"})`);
  }
  return problems;
}

/**
 * What a sentence that says "skin" must also name for the word to be one of
 * its other meanings: a glTF rig's skin (joints, morphs, a model), the
 * character controllers' collision gap (the `skin` field, off a surface),
 * a 2D skeleton's skin (a rig2d `skin` block swapping a slot's attachment,
 * `og.Skeleton2D.skin`, an armature's setup pose), or a surface (a skin
 * tone, peach skin, a crowd's or a tile set's looks).
 */
export const SKIN_ELSEWHERE = /gltf|joint|skelet|morph|\brig\b|bone|mesh|model|clips?\b|anim|`skins?`|slot|attach|armature|\bswap\b|setup pose|surface|\bgap\b|collid|\btones?\b|peach|crowd|tiles/i;

/**
 * A wiki page never calls a UI style a "skin": og-ui's word is
 * style (`type style`, `og.load.style`, `UiDesc.style`, `styles/<name>/`).
 * Each sentence holding `skin` or `skins` must name one of the word's
 * other meanings (SKIN_ELSEWHERE).
 */
function skinWord(roots: Roots, problems: string[]): void {
  // The wiki og api reads from the package, its guide pages included.
  const texts: [string, string][] = [];
  const wikiDir = join(roots.pkg, "wiki");
  for (const n of wikiPages(wikiDir)) texts.push([`wiki/${n}`, readFileSync(join(wikiDir, n), "utf8")]);
  for (const [name, text] of texts) {
    for (const paragraph of text.split(/\n\s*\n/)) {
      // Sentences, and a table's cells, of the paragraph's joined lines.
      for (const sentence of paragraph.replace(/\s*\n\s*/g, " ").split(/(?<=[.;!?])\s+|\s\|\s/)) {
        if (/\bskins?\b/i.test(sentence) && !SKIN_ELSEWHERE.test(sentence)) {
          problems.push(`${name}: calls a UI style a skin ("${sentence.trim().slice(0, 120)}"); og-ui's word is style (type style, og.load.style, styles/<name>/)`);
        }
      }
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

/** `og.` names a guide may write that are not API: each with why. */
export const NOT_API: Record<string, string> = {};

/** `` `key: value` `` spans a guide may write whose key is no field of the generated API: each with why. */
export const NOT_FIELDS: Record<string, string> = {
  runs: "PlaytestDesc.runs (packages/engine/src/playtest.ts)",
  container: "the CSS container shorthand (document stylesheets)",
  transition: "the CSS transition shorthand (document stylesheets)",
  gap: "the CSS gap property (document stylesheets)",
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
 * Every API name a guide writes resolves in the generated API: `og.<x>` is a
 * root function, a component, resource or event token or a module
 * namespace; `og.<module>.<y>` one of the module's functions or values;
 * `og.<Token>.<field>` and `<Type>.<field>` (for a struct, desc, component,
 * resource or event the IDL declares) one of its fields. Code blocks are
 * read too: an example is API as much as prose is. A file name such as
 * `away.og.ts` is not a name.
 */
function apiNames(roots: Roots, problems: string[]): void {
  const api = readApiNames(roots.engine);
  if (!api) return;
  for (const [g, text] of guideTexts(roots)) {
    const at = `wiki/${GUIDE_PREFIX}${g}.md`;
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
    for (const b of bad) problems.push(`${at}: names \`${b}\`, which the generated API (api.ogt) does not have; check the name with og api find or in packages/engine/types/index.d.ts`);
  }
}

/** A style catalogue entry (looks/catalogue.json of the engine package). */
interface Style {
  slug: string;
  preview: string;
  name: string;
  category: string;
  descriptor: string;
  checks: string[];
  /** The traits that look like defects but are the style, one sentence a blind critic reads beside the checks. */
  intended: string;
  premise: Record<string, string>;
  look: {
    preset: string;
    presetReady: boolean;
    /** Being rebuilt: games that build on it keep cooking, and no new game is offered it. */
    frozen?: boolean;
    support: string;
    layers: string[];
    profiles: string[];
    views: string[];
    mood: string;
    primitives: string[];
    missing: string[];
    authored?: { kind: string; what: string }[];
  };
  ui: string;
  palette: { size: string; rules: string };
  assets: Record<string, string>;
  artBible: { direction: string; avoid: string[] };
}

/** What `look.support` may say: full (every primitive draws), partial or planned (`missing` names what is still to land), content (the look is authored art). */
export const SUPPORT = ["full", "partial", "planned", "content"] as const;
/** What `look.profiles` may name: the project profiles a style draws for (create's premise match never offers a style outside the game's). */
export const PROFILES = ["2d", "3d"] as const;
/** What `look.layers` may name (docs/art.md §2.6): the look's render resources, the cook's asset conformance, the runtime pieces, authored art. */
export const LAYERS = ["frame", "conform", "runtime", "authored"] as const;
/** What `look.views` may name: the creation choice's views (create's choices.ts, held equal by its tests); a `-2d` view needs the 2d profile, the rest 3d. */
export const VIEWS = ["side-2d", "top-down-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"] as const;
/** What `look.mood` may name: the moods a look takes (og-bundle's MOODS, create's choices.ts; docs/art.md §2.10). */
export const MOODS = ["bright", "dark", "warm", "cold", "soft", "high-contrast"] as const;
/** The placeholders the art-bible template may use. */
export const BIBLE_FIELDS = ["title", "premise", "camera", "platform", "name", "descriptor", "direction", "palette", "assets", "ui", "avoid", "game", "slug", "checks"] as const;
const PREMISE = ["genre", "mood", "era", "audience"];
const ASSET_SPECS = ["logicalResolution", "spriteScale", "faceBudget", "outlinePx", "textureDensity"];

/**
 * The style catalogue (looks/catalogue.json of the engine package): each
 * style complete (three checks, the premise, palette, asset specs, an art
 * bible direction), its preset path `templates/looks/<slug>.ogt` (a
 * `type look` file when presetReady), its support honest (`missing` names
 * the primitives a style that is not `full` lacks, and is empty for one
 * that is), the profiles it draws for (`look.profiles`, 2d and/or 3d), every `Resource.field` it names a member of the generated API,
 * no text phrased as an imitation, and every creation view left with a
 * style that is not frozen (`look.frozen`).
 */
function looks(roots: Roots, problems: string[]): void {
  const path = join(roots.pkg, "looks", "catalogue.json");
  const at = "looks/catalogue.json";
  if (!existsSync(path)) {
    problems.push(`${at}: missing; the art brief (scripts/brief.ts) and create read the style catalogue from it`);
    return;
  }
  let cat: { schemaVersion?: number; categories?: Record<string, string>; ui?: string[]; artBibleTemplate?: string; styles?: Style[] };
  try {
    cat = JSON.parse(readFileSync(path, "utf8"));
  } catch (e) {
    problems.push(`${at}: not JSON (${(e as Error).message})`);
    return;
  }
  if (cat.schemaVersion !== 3) problems.push(`${at}: schemaVersion ${cat.schemaVersion}, expected 3`);
  const categories = Object.keys(cat.categories ?? {});
  const packs = cat.ui ?? [];
  for (const p of packs) {
    if (p !== "default" && existsSync(join(roots.pkg, "..", "..", "templates")) && !existsSync(join(roots.pkg, "..", "..", "templates", "styles", p, "style.ogt"))) problems.push(`${at}: ui names ${p}, which is not a built-in style (templates/styles/${p}/style.ogt)`);
  }
  for (const m of (cat.artBibleTemplate ?? "").matchAll(/\{(\w+)\}/g)) {
    if (!(BIBLE_FIELDS as readonly string[]).includes(m[1]!)) problems.push(`${at}: artBibleTemplate uses {${m[1]}}, which is none of ${BIBLE_FIELDS.join(", ")}`);
  }
  if (!cat.artBibleTemplate) problems.push(`${at}: no artBibleTemplate`);
  const schemaPath = join(roots.engine, "schema", "api.schema.json");
  const defs = existsSync(schemaPath) ? ((JSON.parse(readFileSync(schemaPath, "utf8")) as { $defs?: Record<string, { properties?: Record<string, unknown> }> }).$defs ?? {}) : undefined;
  const repo = join(roots.pkg, "..", "..");
  const seen = new Set<string>();
  const text = (s: unknown) => typeof s === "string" && s.trim().length > 0;
  for (const [i, s] of (cat.styles ?? []).entries()) {
    const id = `${at}: ${s.slug ?? `styles[${i}]`}`;
    if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(s.slug ?? "")) problems.push(`${id}: slug is not lowercase-hyphen`);
    if (seen.has(s.slug)) problems.push(`${id}: a second style with this slug`);
    seen.add(s.slug);
    for (const k of ["name", "descriptor", "ui"] as const) if (!text(s[k])) problems.push(`${id}: no ${k}`);
    if (s.preview !== `previews/${s.slug}.webp`) problems.push(`${id}: preview is ${s.preview}, expected previews/${s.slug}.webp`);
    else if (!existsSync(join(roots.pkg, s.preview))) problems.push(`${id}: ${s.preview} does not exist (a 960x540 picture of the style as a game)`);
    if (!categories.includes(s.category)) problems.push(`${id}: category ${s.category} is none of ${categories.join(", ")}`);
    if (!packs.includes(s.ui)) problems.push(`${id}: ui ${s.ui} is none of the built-in styles (${packs.join(", ")})`);
    if (!Array.isArray(s.checks) || s.checks.length !== 3 || !s.checks.every(text)) problems.push(`${id}: needs exactly three checks a critic can judge on a frame`);
    // What a critic must not hold against the style: said once, the same for every critic, and blind
    // like the checks (a blind critic reads it beside them), so it never names the style.
    if (!text(s.intended) || !/ (is|are) the style\b/.test(s.intended)) problems.push(`${id}: needs intended, one sentence naming the traits that look like defects but are the style ("… are the style, not …")`);
    else if ([s.name, s.slug].some((n) => text(n) && s.intended.toLowerCase().includes(n.toLowerCase()))) problems.push(`${id}: intended names the style; a blind critic reads it`);
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
    const profiles = Array.isArray(look.profiles) ? look.profiles : [];
    if (profiles.length === 0 || new Set(profiles).size !== profiles.length || !profiles.every((p) => (PROFILES as readonly string[]).includes(p))) {
      problems.push(`${id}: look.profiles must name the profiles the style draws for, one or both of ${PROFILES.join(", ")} (create matches a premise only among the game's)`);
    }
    const layers = Array.isArray(look.layers) ? look.layers : [];
    if (!layers.includes("frame") || new Set(layers).size !== layers.length || !layers.every((l) => (LAYERS as readonly string[]).includes(l))) problems.push(`${id}: look.layers must include frame and name only ${LAYERS.join(", ")}, each once`);
    if (layers.includes("authored") !== (look.support === "content")) problems.push(`${id}: support ${look.support} disagrees with look.layers (content exactly when authored art is needed)`);
    if ("family" in look) problems.push(`${id}: look.family is gone (schema 3): the sample scene is tools/art-samples/scenes.json's, not a reader's`);
    const views = Array.isArray(look.views) ? look.views : [];
    if (views.length === 0 || new Set(views).size !== views.length || !views.every((v) => (VIEWS as readonly string[]).includes(v))) problems.push(`${id}: look.views must name the views the style suits, each once, among ${VIEWS.join(", ")}`);
    for (const v of views) if (!profiles.includes(v.endsWith("-2d") ? "2d" : "3d")) problems.push(`${id}: look.views names ${v}, which its profiles (${profiles.join(", ")}) do not draw`);
    if (!(MOODS as readonly string[]).includes(look.mood)) problems.push(`${id}: look.mood ${look.mood} is none of ${MOODS.join(", ")}`);
    const authored = Array.isArray(look.authored) ? look.authored : [];
    if (layers.includes("authored") !== authored.length > 0) problems.push(`${id}: look.authored names the art exactly when look.layers needs authored art`);
    for (const a of authored) {
      if (!(ART_KINDS as readonly string[]).includes(a.kind)) problems.push(`${id}: look.authored names ${a.kind}, which is not an art brief kind`);
      if (!text(a.what)) problems.push(`${id}: look.authored's ${a.kind} says not what the art must be`);
    }
    if (new Set(authored.map((a) => a.kind)).size !== authored.length) problems.push(`${id}: look.authored names a kind twice`);
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
      else if (existsSync(preset)) {
        const body = readFileSync(preset, "utf8");
        if (!/^type look\s*$/m.test(body)) problems.push(`${id}: ${look.preset} is not an .ogt of type look`);
        // The preset dresses the UI in the style the catalogue gives it.
        const ui = /^ui[ \t]+"([^"]+)"/m.exec(body)?.[1];
        if (ui !== s.ui) problems.push(`${id}: ${look.preset} has ${ui ? `ui "${ui}"` : "no ui line"}; the catalogue says ui "${s.ui}"`);
        const mood = /^mood[ \t]+"([^"]+)"/m.exec(body)?.[1];
        if (mood !== look.mood) problems.push(`${id}: ${look.preset} has ${mood ? `mood "${mood}"` : "no mood line"}; the catalogue says mood "${look.mood}"`);
      }
    }
    const all = JSON.stringify(s);
    if (/in the style of|™|®/i.test(all)) problems.push(`${id}: phrased as an imitation (\"in the style of\", a trademark sign); describe the look in its own terms`);
  }
  if (seen.size === 0) problems.push(`${at}: no styles`);
  for (const s of cat.styles ?? []) {
    if (s.look?.frozen !== undefined && s.look.frozen !== true) problems.push(`${at}: ${s.slug}: look.frozen is true or absent`);
  }
  const offered = (cat.styles ?? []).filter((s) => s.look?.presetReady && !s.look.frozen);
  for (const v of VIEWS) {
    if (seen.size > 0 && !offered.some((s) => s.look.views?.includes(v))) problems.push(`${at}: every style of the ${v} view is frozen; a new game of that view has no look to pick`);
  }
}

/**
 * Every `.ogt` example a guide writes in a code block (```text or a bare
 * fence, as a whole file or a fragment) holds to og-core's parser
 * (crates/og-core/src/ogt.rs `parse`) line by line, so an author who copies
 * it cooks: a `{` ends its key's line, a `}` stands alone on its own, the
 * blocks close, and a file that opens with `type` has `version` and `---`
 * next. A block with no brace and no `type` line is not checked (shell,
 * prose). Strings and `#` comments are read as the parser reads them.
 */
export function ogtShape(guides: ReadonlyMap<string, string>): string[] {
  const problems: string[] = [];
  for (const [guide, md] of guides) {
    for (const m of md.matchAll(/^( *)```([\w-]*)\n([\s\S]*?)^ *```/gm)) {
      if (m[2] !== "text" && m[2] !== "") continue;
      const indent = m[1]!.length;
      const lines = m[3]!.split("\n").map((l) => l.slice(Math.min(indent, l.length - l.trimStart().length)));
      const words = lines.map((l) => {
        const out: string[] = [];
        for (const t of l.matchAll(/"(?:[^"\\]|\\.)*"|\S+/g)) {
          if (t[0].startsWith("#")) break;
          out.push(t[0]);
        }
        return out;
      });
      const typed = words.find((w) => w.length > 0)?.[0] === "type";
      if (!typed && !words.some((w) => w.includes("{") || w.includes("}"))) continue;
      const first = md.slice(0, m.index).split("\n").length + 1;
      const at = (i: number) => `wiki/${GUIDE_PREFIX}${guide}.md:${first + i}`;
      if (typed) {
        const head = words.map((w, i) => [w, i] as const).filter(([w]) => w.length > 0).slice(0, 3);
        if (head[1]?.[0][0] !== "version" || head[2]?.[0].join(" ") !== "---") problems.push(`${at(head[0]![1])}: an .ogt example opens with \`type\` but not \`version <n>\` and \`---\` after it; the parser refuses a file without its header`);
      }
      let depth = 0;
      words.forEach((w, i) => {
        const open = w.indexOf("{");
        if (open >= 0 && open !== w.length - 1) problems.push(`${at(i)}: \`${lines[i]!.trim()}\` opens a block mid-line; the .ogt parser refuses it ('{' must end its line): put each line of the block on its own`);
        if (w.includes("}") && w.length !== 1) problems.push(`${at(i)}: \`${lines[i]!.trim()}\` closes a block beside other words; the .ogt parser refuses it ('}' must be alone on its line)`);
        if (open >= 0) depth++;
        if (w.includes("}")) depth--;
        if (depth < 0) {
          problems.push(`${at(i)}: an unmatched '}'`);
          depth = 0;
        }
      });
      if (depth > 0) problems.push(`${at(lines.length - 1)}: an .ogt example leaves ${depth} block(s) open`);
    }
  }
  return problems;
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
