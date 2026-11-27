// Validates @origingame/engine-knowledge against the OriginGame platform's
// knowledge rules (rules/platform.json, transcribed from the platform with
// scripts/sync-rules.ts; each rule names its platform source), and the
// package's own contract: a generated API reference, and a scaffold
// overlay whose pin and distribution list match the project template.
//
//   bun scripts/validate.ts            problems, then the packs and hashes
//   bun scripts/validate.ts --bundle   the payload as the platform's
//                                      readProductionBundle shape (text only)

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, posix, relative } from "node:path";
import {
  defaultRoots,
  isText,
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
import { ART_KINDS, MOODS } from "./brief.ts";
import { CREATION_CHOICES, LOOK_ZH, VARIANT_ZH } from "./creation.ts";

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

/** What a scaffolded project must contain (scaffold/README.md describes each). */
export const TEMPLATE_FILES = ["og-project.json", "project.ts", "src/main.ts", "package.json", "tsconfig.json", "tests/smoke.og.ts"] as const;

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

  skinWord(roots, problems);
  looks(roots, problems);
  scaffold(roots, rules, manifest.overlay, files, byPath, problems);
  return problems;
}

function packageJson(roots: Roots, problems: string[]): void {
  const p = JSON.parse(readFileSync(join(roots.pkg, "package.json"), "utf8")) as { name?: string; version?: string; files?: string[]; scripts?: Record<string, string> };
  if (p.name !== "@origingame/engine-knowledge") problems.push(`package.json: name is ${p.name}; the package is @origingame/engine-knowledge (design §9)`);
  if (!/^(?:\d+\.\d+\.\d+(-[\w.]+)?|dev\.\d{8}\.\d+)$/.test(p.version ?? "")) problems.push(`package.json: version ${p.version} is not semver`);
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

  // Identity is ordinary data. TS configuration is checked by og, never
  // interpreted with a second project parser in the packaging tools.
  const metadata = byPath.get(`${prefix}og-project.json`);
  if (metadata && isText(metadata.bytes)) {
    try {
      const value = JSON.parse(decoder.decode(metadata.bytes));
      if (value?.version !== 1 || typeof value.name !== "string" || !value.name) problems.push(`${prefix}og-project.json: requires version 1 and a nonempty name`);
      if (value?.engine !== engineVersion) problems.push(`${prefix}og-project.json: engine ${JSON.stringify(value?.engine)}, packages/engine is ${engineVersion}`);
      if (value && Object.keys(value).some(key => !["version", "engine", "name", "look"].includes(key))) problems.push(`${prefix}og-project.json: unknown identity field`);
    } catch (e) {
      problems.push(`${prefix}og-project.json: ${e instanceof Error ? e.message : String(e)}`);
    }
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
 * What a sentence that says "skin" must also name for the word to be one of
 * its other meanings: a glTF rig's skin (joints, morphs, a model), mesh
 * preparation (deformation, welding, LODs), the
 * character controllers' collision gap (the `skin` field, off a surface),
 * a garment/cloth backstop behind the pose skin, a 2D skeleton's skin (a rig2d
 * `skin` block swapping a slot's attachment, `og.Skeleton2D.skin`, an
 * armature's setup pose), or a surface (a skin tone, peach skin, a crowd's or
 * a tile set's looks).
 */
export const SKIN_ELSEWHERE = /gltf|joint|skelet|morph|\brig\b|rig2d|bone|mesh|model|deform|weld|\blods?\b|clips?\b|anim|`skins?`|slot|attach|armature|\bswap\b|setup pose|backstop|\bpose\b|garment|cloth|surface|\bgap\b|collid|\btones?\b|peach|crowd|tiles/i;

/**
 * A wiki page never calls a UI style a "skin": og-ui's word is
 * style (`og.load.style`, `UiDesc.style`, `styles/<name>/theme.json` or `pack.json`).
 * Require UI context rather than treating every unexplained use of skin as
 * UI terminology. Skeletal declarations and their comments are not UI styles.
 */
function skinWord(roots: Roots, problems: string[]): void {
  // The API reference og api reads from the package.
  const texts: [string, string][] = [];
  const wikiDir = join(roots.pkg, "wiki");
  for (const n of wikiPages(wikiDir)) texts.push([`wiki/${n}`, readFileSync(join(wikiDir, n), "utf8")]);
  for (const [name, text] of texts) {
    for (const paragraph of text.split(/\n\s*\n/)) {
      // Sentences, and a table's cells, of the paragraph's joined lines.
      for (const sentence of paragraph.replace(/\s*\n\s*/g, " ").split(/(?<=[.;!?])\s+|\s\|\s/)) {
        const ui = /\b(?:ui|og-ui|user interface|style|theme|panel|widget|button|menu|hud)\b/i.test(sentence);
        if (/\bskins?\b/i.test(sentence) && ui && !SKIN_ELSEWHERE.test(sentence)) {
          problems.push(`${name}: calls a UI style a skin ("${sentence.trim().slice(0, 120)}"); og-ui's word is style (og.load.style, styles/<name>/)`);
        }
      }
    }
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
  /** The preset drawn other ways (`variant: "<name>"`, templates/looks/variants/<slug>/<name>.ts): each one's own text, which the brief and the critic read in place of the style's. */
  variants?: Variant[];
}

/** One variant of a style: its layer's name and the text a brief and a critic read for it. */
interface Variant {
  name: string;
  title: string;
  descriptor: string;
  mood: string;
  checks: string[];
  intended: string;
  premise: Record<string, string>;
  palette: { size: string; rules: string };
  assets: Record<string, string>;
  artBible: { direction: string; avoid: string[] };
  authored?: { kind: string; what: string }[];
}

/** What `look.support` may say: full (every primitive draws), partial or planned (`missing` names what is still to land), content (the look is authored art). */
export const SUPPORT = ["full", "partial", "planned", "content"] as const;
/** What `look.profiles` may name: the project profiles a style draws for (create's premise match never offers a style outside the game's). */
export const PROFILES = ["2d", "3d"] as const;
/** What `look.layers` may name (docs/art.md §2.6): the look's render resources, the cook's asset conformance, the runtime pieces, authored art. */
export const LAYERS = ["frame", "conform", "runtime", "authored"] as const;
/** What `look.views` may name: the creation choice's views (scripts/creation.ts); a `-2d` view needs the 2d profile, the rest 3d. */
export const VIEWS = CREATION_CHOICES.view;
/** The art-bible placeholders. `look` is og-project.json's canonical look ref, never the project id or a guessed filename. */
export const BIBLE_FIELDS = ["title", "premise", "camera", "platform", "name", "descriptor", "direction", "palette", "assets", "ui", "avoid", "look", "slug", "checks"] as const;
const PREMISE = ["genre", "mood", "era", "audience"];
const ASSET_SPECS = ["logicalResolution", "spriteScale", "faceBudget", "outlinePx", "textureDensity"];

/**
 * The style catalogue (looks/catalogue.json of the engine package): each
 * style complete (three checks, the premise, palette, asset specs, an art
 * bible direction), its preset path `templates/looks/<slug>.ts` (a
 * checked `file.look` when presetReady), its support honest (`missing` names
 * the primitives a style that is not `full` lacks, and is empty for one
 * that is), the profiles it draws for (`look.profiles`, 2d and/or 3d), every `Resource.field` it names a member of the generated API,
 * no text phrased as an imitation, each variant complete with its layer
 * (`templates/looks/variants/<slug>/<name>.ts`, every such file a variant
 * of the catalogue) and its mood the layer's, and every creation view with
 * a look to pick.
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
  const choiceUi: readonly string[] = CREATION_CHOICES.ui;
  for (const p of packs) {
    if (!choiceUi.includes(p)) problems.push(`${at}: ui names ${p}, which scripts/creation.ts CREATION_CHOICES.ui / CHOICE_LABELS.ui lacks`);
    const templates = join(roots.pkg, "..", "..", "templates");
    if (p !== "default" && existsSync(templates) && !existsSync(join(templates, "themes", p, "theme.ts")) && !existsSync(join(templates, "styles", p, "pack.ts"))) problems.push(`${at}: ui names ${p}, which is no checked engine theme or built-in style`);
  }
  for (const p of choiceUi) {
    if (!packs.includes(p)) problems.push(`scripts/creation.ts: CREATION_CHOICES.ui names ${p}, which ${at} ui does not list`);
  }
  const themesDir = join(roots.pkg, "..", "..", "templates", "themes");
  if (existsSync(themesDir)) {
    for (const e of readdirSync(themesDir, { withFileTypes: true })) {
      if (e.isDirectory() && existsSync(join(themesDir, e.name, "theme.ts")) && !packs.includes(e.name)) {
        problems.push(`${at}: templates/themes/${e.name}/theme.ts is missing from ui`);
      }
    }
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
    if (look.preset !== `templates/looks/${s.slug}.ts`) problems.push(`${id}: look.preset is ${look.preset}, expected templates/looks/${s.slug}.ts`);
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
    const checkedPath = join(roots.pkg, "presets", `${s.slug}.json`);
    const checked = existsSync(checkedPath) ? JSON.parse(readFileSync(checkedPath, "utf8")) as {looks: {variant: string | null; mood: string | null; look: {ui: string | null; mood: string | null}}[]} : undefined;
    if (look.presetReady) {
      if (!LOOK_ZH[s.slug]) problems.push(`${id}: scripts/creation.ts LOOK_ZH lacks Chinese copy for ${s.slug}`);
      if (existsSync(join(repo, "templates")) && !existsSync(preset)) problems.push(`${id}: presetReady, but ${look.preset} does not exist`);
      const base = checked?.looks.find(l => l.variant === null && l.mood === null)?.look;
      if (!base) problems.push(`${id}: missing checked preset; run og api gen`);
      else {
        if (base.ui !== (s.ui === "default" ? null : `@themes/${s.ui}`)) problems.push(`${id}: checked ui ${base.ui}; catalogue says ${s.ui}`);
        if (base.mood !== look.mood) problems.push(`${id}: checked mood ${base.mood}; catalogue says ${look.mood}`);
      }
    }
    const variants = Array.isArray(s.variants) ? s.variants : [];
    const names = new Set<string>();
    for (const v of variants) {
      const vid = `${id}: variant ${v.name ?? "?"}`;
      if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(v.name ?? "")) problems.push(`${vid}: name is not lowercase-hyphen`);
      if (names.has(v.name)) problems.push(`${vid}: a second variant with this name`);
      names.add(v.name);
      if (look.presetReady && !VARIANT_ZH[`${s.slug}.${v.name}`]) problems.push(`${vid}: scripts/creation.ts VARIANT_ZH lacks Chinese copy for ${s.slug}.${v.name}`);
      for (const k of ["title", "descriptor"] as const) if (!text(v[k])) problems.push(`${vid}: no ${k}`);
      if (!Array.isArray(v.checks) || v.checks.length !== 3 || !v.checks.every(text)) problems.push(`${vid}: needs exactly three checks a critic can judge on a frame`);
      if (!text(v.intended) || !/ (is|are) the style\b/.test(v.intended)) problems.push(`${vid}: needs intended, one sentence naming the traits that look like defects but are the style`);
      for (const k of PREMISE) if (!text(v.premise?.[k])) problems.push(`${vid}: premise has no ${k}`);
      for (const k of ASSET_SPECS) if (!text(v.assets?.[k])) problems.push(`${vid}: assets has no ${k}`);
      if (!text(v.palette?.size) || !text(v.palette?.rules)) problems.push(`${vid}: palette needs a size and rules`);
      if (!text(v.artBible?.direction) || !Array.isArray(v.artBible?.avoid) || v.artBible.avoid.length === 0) problems.push(`${vid}: artBible needs a direction and what to avoid`);
      if (!(MOODS as readonly string[]).includes(v.mood)) problems.push(`${vid}: mood ${v.mood} is none of ${MOODS.join(", ")}`);
      for (const a of v.authored ?? []) if (!(ART_KINDS as readonly string[]).includes(a.kind) || !text(a.what)) problems.push(`${vid}: authored names ${a.kind}, which is not an art brief kind with what it must be`);
      const layer = join(repo, "templates", "looks", "variants", s.slug, `${v.name}.ts`);
      if (existsSync(join(repo, "templates"))) {
        if (!existsSync(layer)) problems.push(`${vid}: templates/looks/variants/${s.slug}/${v.name}.ts does not exist`);
      }
      const mood = checked?.looks.find(l => l.variant === v.name && l.mood === null)?.look.mood;
      if (mood !== v.mood) problems.push(`${vid}: checked mood ${mood}; catalogue says ${v.mood}`);
    }
    const variantDir = join(repo, "templates", "looks", "variants", s.slug);
    if (existsSync(variantDir)) {
      for (const f of readdirSync(variantDir)) if (f.endsWith(".ts") && !names.has(f.slice(0, -3))) problems.push(`${id}: templates/looks/variants/${s.slug}/${f} is no variant of the catalogue's`);
    }
    const all = JSON.stringify(s);
    if (/in the style of|™|®/i.test(all)) problems.push(`${id}: phrased as an imitation (\"in the style of\", a trademark sign); describe the look in its own terms`);
  }
  if (seen.size === 0) problems.push(`${at}: no styles`);
  const offered = (cat.styles ?? []).filter((s) => s.look?.presetReady);
  for (const v of VIEWS) {
    if (seen.size > 0 && !offered.some((s) => s.look.views?.includes(v))) problems.push(`${at}: no look with a preset suits the ${v} view; a new game of that view has no look to pick`);
  }
  const variantsDir = join(repo, "templates", "looks", "variants");
  if (existsSync(variantsDir)) {
    for (const d of readdirSync(variantsDir, { withFileTypes: true })) if (d.isDirectory() && !seen.has(d.name)) problems.push(`${at}: templates/looks/variants/${d.name}/ is no style's`);
  }
  artDoc(roots, problems);
}

/**
 * docs/art.md's restatements of the catalogue and the moods, read back
 * against their sources: §2.6's looks list names every catalogue style with
 * the UI pack its `ui` line carries (and only ones a creation offers),
 * §2.10's moods sentence names MOODS (brief.ts's, held to og-bundle's by its
 * tests), and §2.11's variants table lists exactly each style's variants, in
 * the catalogue's order. Checked, not emitted: the prose around them is
 * written by hand. The published package carries no repository docs, so a
 * missing docs/art.md is not a problem.
 */
export function artDoc(roots: Roots, problems: string[]): void {
  const docAt = join(roots.pkg, "..", "..", "docs", "art.md");
  if (!existsSync(docAt)) return;
  const doc = readFileSync(docAt, "utf8");
  const catAt = join(roots.pkg, "looks", "catalogue.json");
  let styles: { slug: string; ui?: string; look?: { presetReady?: boolean }; variants?: { name: string }[] }[] = [];
  try {
    styles = (JSON.parse(readFileSync(catAt, "utf8")) as { styles?: typeof styles }).styles ?? [];
  } catch {
    return; // looks() reports the catalogue's own problems.
  }

  const moods = [...(/one of ([^)]*?) \(og-bundle's `MOODS`/.exec(doc)?.[1] ?? "").matchAll(/`([a-z-]+)`/g)].map((m) => m[1]!);
  if (moods.join() !== MOODS.join()) problems.push(`docs/art.md §2.10 names the moods ${moods.join(", ") || "none"}; the look takes ${MOODS.join(", ")} (og-bundle's MOODS)`);

  const list = /Twelve looks[\s\S]*?line names: ([\s\S]*?)\./.exec(doc)?.[1];
  if (!list) {
    problems.push("docs/art.md §2.6 has no looks list naming every catalogue style with the UI pack its `ui` line names");
  } else {
    const named: { slug: string; ui: string }[] = [];
    for (const group of list.split(";")) {
      const g = /(.+) \(([a-z0-9-]+)\)/.exec(group.replace(/\s+/g, " ").trim());
      if (!g) {
        problems.push(`docs/art.md §2.6's looks list names styles as \`slug, slug (ui pack)\`: \`${group.trim()}\` names none`);
        continue;
      }
      for (const slug of g[1]!.split(/, | and /)) named.push({ slug: slug.trim(), ui: g[2]! });
    }
    const at = "docs/art.md §2.6's looks list";
    const known = new Map(styles.map((s) => [s.slug, s]));
    for (const { slug, ui } of named) {
      const s = known.get(slug);
      if (!s) problems.push(`${at} names ${slug}, which the catalogue has no style for`);
      else if (s.ui !== ui) problems.push(`${at} names ${slug} under the ${ui} UI pack; the catalogue says ui "${s.ui}"`);
      else if (!s.look?.presetReady) problems.push(`${at} names ${slug}, which no creation offers (its look has no preset; creationLooks)`);
    }
    const missing = styles.filter((s) => !named.some((n) => n.slug === s.slug)).map((s) => s.slug);
    if (missing.length) problems.push(`${at} does not name ${missing.join(", ")}`);
  }

  const tableAt = doc.indexOf("| Look | Variants |");
  const rows = tableAt < 0 ? [] : [...doc.slice(tableAt).matchAll(/^\| ([a-z0-9-]+) \| ([a-z0-9-, -]*[a-z0-9-])?\s*\|$/gm)].map((m) => ({ slug: m[1]!, variants: (m[2] ?? "").split(", ").filter(Boolean) }));
  const at = "docs/art.md §2.11's variants table";
  if (tableAt < 0) problems.push(`${at}: the doc has none`);
  const withVariants = styles.filter((s) => (s.variants?.length ?? 0) > 0);
  for (const row of rows) {
    const s = styles.find((x) => x.slug === row.slug);
    if (!s) problems.push(`${at} has a row for ${row.slug}, which the catalogue has no style for`);
    else if (row.variants.join() !== (s.variants ?? []).map((v) => v.name).join()) problems.push(`${at} lists ${row.slug}'s variants ${row.variants.join(", ") || "none"}; the catalogue says ${(s.variants ?? []).map((v) => v.name).join(", ") || "none"}`);
  }
  const unlisted = withVariants.filter((s) => !rows.some((r) => r.slug === s.slug)).map((s) => s.slug);
  if (unlisted.length) problems.push(`${at} has no row for ${unlisted.join(", ")}`);
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
