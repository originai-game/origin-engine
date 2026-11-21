#!/usr/bin/env bun
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { CREATION_CHOICES, creationBlock, FORM_VIEWS, formOf, GAME_ONLY, inferCreationChoices, parseCreationInput } from "./choices.ts";
import type { View } from "./choices.ts";
import { creationDocument } from "./creation.ts";
import type { PreviewFile } from "./creation.ts";
import type { CreationChoices, CreationInput } from "./choices.ts";
import { profileOf } from "./knowledge.ts";
import { fontPackFor } from "./languages.ts";
import type { FontPackLanguages } from "./languages.ts";

export { CREATION_CHOICES, creationBlock, creationSchema, defaultView, FILM_GENRE, FORM_VIEWS, formFields, formOf, GAME_ONLY, GENRE_VIEWS, inferCreationChoices, parseCreationInput } from "./choices.ts";
export type { CreationChoices, CreationInput, Form, Genre, View } from "./choices.ts";
export { creationCompat, creationDocument, lookForms, previewSlots } from "./creation.ts";
export type { CreationCompat, CreationDocument, PreviewFile, PreviewSlot } from "./creation.ts";
export { canonicalTag, fontPackFor, isRtl, localeScript } from "./languages.ts";
export type { FontPackLanguages, LocaleScript } from "./languages.ts";
export { PSEUDO_LOCALE, pseudoMessage, pseudoTable } from "./pseudo-locale.ts";
export { ART_KINDS, ART_RESOURCES, artBrief, authoredArt, colourName, MOODS, parseLook, parsePalette, presetLook, profileOf, readLook, styleArtBriefs } from "./knowledge.ts";
export type { ArtBlocks, ArtBrief, ArtKind, ArtResource, ArtValue, AuthoredArt, BriefLook, BriefOptions, BriefPalette, BriefReference, BriefStyle, ImageParams, ModelParams, Mood, PaletteColour, ReadLookOptions, StyleArtBrief, StyleArtRequests } from "./knowledge.ts";
export { audioBrief, MUSIC_STYLES, readAudioChoices, SFX_PRESETS } from "./knowledge.ts";
export type { AudioBrief, AudioBriefInput, AudioFormat, MusicCueBrief, MusicRole, MusicStyle, SfxBrief, SfxPreset, SfxPresetBrief } from "./knowledge.ts";
export { RIG_TEMPLATES, rigParts } from "./knowledge.ts";
export type { RigJoint, RigPart, RigTemplate } from "./knowledge.ts";

export const SKIPPED = new Set(["cooked", "dist", "node_modules", "og-out", ".og"]);
export const RENAMED: ReadonlyMap<string, string> = new Map([["gitattributes", ".gitattributes"], ["gitignore", ".gitignore"]]);

export interface CreateOptions {
  readonly dir: string;
  readonly name?: string;
  readonly id?: string;
  readonly force?: boolean;
  readonly choices?: CreationInput;
  readonly look?: string;
  readonly premise?: string;
  readonly catalogue?: string;
}

export interface Created {
  readonly dir: string;
  readonly name: string;
  readonly id: string;
  readonly choices: CreationChoices;
  readonly files: readonly string[];
}

export type Profile = "2d" | "3d";
export interface Style {
  readonly slug: string;
  readonly name: string;
  readonly category: string;
  readonly descriptor: string;
  readonly premise: Readonly<Record<string, string>>;
  readonly look: { readonly presetReady: boolean; readonly support: string; readonly profiles: readonly Profile[]; readonly views: readonly string[]; readonly mood: string; readonly missing: readonly string[] };
  readonly ui: string;
  readonly preview?: string;
  readonly variants?: readonly { readonly name: string; readonly title: string; readonly descriptor: string; readonly mood: string }[];
}

/** The font packs: templates/fonts in a checkout, the package's fonts/ once packed. */
export function fontPacksDir(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const candidate of [join(here, "..", "..", "..", "templates", "fonts"), join(here, "..", "fonts")]) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`no font packs next to ${here}; reinstall the engine distribution`);
}

/**
 * In an engine checkout, the files `templates/og-assets.json` pins under
 * `area` (such as `fonts/latin`) that are not installed: they live in the
 * asset store until `bun scripts/assets.ts ensure templates` fetches them. A
 * packed create (fonts/ beside bin/) has no manifest and misses nothing.
 */
export function missingPins(packs: string, area: string): string[] {
  const manifest = join(packs, "..", "og-assets.json");
  if (!existsSync(manifest)) return [];
  const files = (JSON.parse(readFileSync(manifest, "utf8")) as { files?: { path: string }[] }).files ?? [];
  return files.map((f) => f.path).filter((p) => p.startsWith(`${area}/`) && !existsSync(join(packs, "..", p)));
}

export function catalogueFile(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const candidate of [join(here, "..", "..", "knowledge", "looks", "catalogue.json"), join(here, "..", "looks", "catalogue.json")]) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`no style catalogue next to ${here}; reinstall the engine distribution`);
}

export function readCatalogue(file = catalogueFile()): Style[] {
  return (JSON.parse(readFileSync(file, "utf8")) as { styles: Style[] }).styles;
}

/** The looks a new game may pick: the catalogue's styles with a preset. */
export function creationLooks(styles: readonly Style[] = readCatalogue()): string[] {
  return styles.filter(s => s.look.presetReady).map(s => s.slug);
}

const STOP = new Set(["the", "and", "for", "with", "about", "into", "from", "that", "this", "game", "games", "player", "players", "where", "who", "your", "you", "its", "set", "lot", "very", "like", "one", "two", "all", "any", "out", "their", "them", "has", "have", "are", "was", "but", "not", "own"]);
const SPELLING: Readonly<Record<string, string>> = { cozy: "cosy", color: "colour", colors: "colours", gray: "grey", scifi: "sci", "sci-fi": "sci", watercolour: "watercolor", "8-bit": "8bit", "16-bit": "16bit", "1-bit": "1bit" };

export function words(text: string): string[] {
  const spelt = (w: string) => SPELLING[w] ?? w;
  return (text.toLowerCase().match(/[a-z0-9]+(?:-[a-z0-9]+)*/g) ?? [])
    .flatMap((w) => (SPELLING[w] ? [SPELLING[w]!] : w.includes("-") ? [w, ...w.split("-").map(spelt)] : [spelt(w)]))
    .map((w) => (w.length > 4 && w.endsWith("s") && !w.endsWith("ss") ? w.slice(0, -1) : w))
    .filter((w) => w.length >= 3 && !STOP.has(w));
}

/** Suggest a compatible preset. Explicit look choices bypass ranking. */
export function pickStyle(premise: string, styles: readonly Style[], profile: Profile = "3d", view?: string): { style: Style; score: number } | null {
  const asked = new Set(words(premise));
  if (asked.size === 0) return null;
  const weights: [(s: Style) => string, number][] = [
    [(s) => s.premise.genre ?? "", 3], [(s) => s.premise.mood ?? "", 2],
    [(s) => s.premise.era ?? "", 1], [(s) => s.premise.audience ?? "", 1],
    [(s) => `${s.name} ${s.descriptor}`, 1],
  ];
  let best: { style: Style; score: number; full: boolean } | null = null;
  for (const style of styles) {
    if (!style.look.presetReady || !style.look.profiles.includes(profile)) continue;
    if (view !== undefined && !style.look.views.includes(view)) continue;
    let score = 0;
    for (const [field, weight] of weights) {
      const have = new Set(words(field(style)));
      for (const w of asked) if (have.has(w)) score += weight;
    }
    const full = style.look.support === "full";
    if (score > 0 && (!best || score > best.score || (score === best.score && full && !best.full))) best = { style, score, full };
  }
  return best && { style: best.style, score: best.score };
}

export function slug(name: string): string {
  return name.normalize("NFKD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
}

export function templateDir(): string {
  return join(dirname(fileURLToPath(import.meta.url)), "../template");
}

/**
 * The skeleton's files, relative to template/: the `template/` entries of
 * the package manifest's `files`, which is also what a release packs. A file
 * left in template/ of a checkout (a run's output, a stray asset) is not in
 * the manifest, so it is never part of a new project.
 */
export function skeletonFiles(): string[] {
  const manifest = join(templateDir(), "../package.json");
  const files = (JSON.parse(readFileSync(manifest, "utf8")) as { files?: string[] }).files ?? [];
  const skeleton = files.filter((f) => f.startsWith("template/")).map((f) => f.slice("template/".length)).sort();
  if (!skeleton.length) throw new Error(`${manifest} lists no template/ file in "files"`);
  return skeleton;
}

/** The film skeleton, what `form film` adds over template/ (og api show film): templates/film in a checkout, the package's film/ once packed. */
export function filmDir(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const candidate of [join(here, "..", "..", "..", "templates", "film"), join(here, "..", "film")]) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`no film skeleton next to ${here}; reinstall the engine distribution`);
}

/**
 * Resolve the agent's choices without writing any project files. The look
 * must draw the form (a 2D look for a 2D game, catalogue `views`), and the
 * variant must be one of the look's.
 */
export function resolveCreationChoices(premise: string, input: CreationInput = {}, catalogue?: string): CreationChoices {
  const styles = readCatalogue(catalogue);
  let asked = inferCreationChoices(premise, input);
  // A look chosen without a form or view gives the form, when the inferred one is not one it draws.
  const chosen = input.look === undefined ? undefined : styles.find(s => s.slug === input.look);
  if (chosen && input.form === undefined && input.view === undefined && asked.view !== undefined && !chosen.look.views.includes(asked.view)) {
    asked = inferCreationChoices(premise, { ...input, form: formOf(chosen.look.views[0] as View) });
  }
  const view = asked.view;
  const slug = input.look ?? (asked.form === "film" ? "cinematic-realism" : pickStyle(premise, styles, profileOf(asked.form), view)?.style.slug ?? (asked.form === "2d" ? "pixel" : "faceted-low-poly"));
  const style = styles.find(s => s.slug === slug && s.look.presetReady);
  if (!style) throw new Error(`no look preset '${slug}'; pick one of ${creationLooks(styles).join(", ")}`);
  if (view !== undefined && !style.look.views.includes(view)) {
    const fits = creationLooks(styles).filter(l => styles.find(s => s.slug === l)!.look.views.includes(view));
    throw new Error(`look ${slug} does not draw ${asked.form} (${view}); it draws ${style.look.views.join(", ")}. For ${view} pick one of ${fits.join(", ")}`);
  }
  const variants = (style.variants ?? []).map(v => v.name);
  if (input.variant !== undefined && !variants.includes(input.variant)) throw new Error(`look ${slug} has no variant '${input.variant}'; ${variants.length ? `pick one of ${variants.join(", ")}` : "it has none"}`);
  return inferCreationChoices(premise, { ...input, look: slug, ui: parseCreationInput({ ui: input.ui ?? style.ui }).ui! });
}

/** The font packs in `packs`: each directory with an extension.json, and its languages. */
function readPacks(packs: string): ({ dir: string } & FontPackLanguages)[] {
  return readdirSync(packs, { withFileTypes: true })
    .filter((e) => e.isDirectory() && existsSync(join(packs, e.name, "extension.json")))
    .map((e) => ({ dir: join(packs, e.name), ...(JSON.parse(readFileSync(join(packs, e.name, "extension.json"), "utf8")) as FontPackLanguages) }));
}

export function create(options: CreateOptions): Created {
  const choices = resolveCreationChoices(options.premise ?? "", {
    ...options.choices, ...(options.look === undefined ? {} : { look: options.look }),
  }, undefined, options.catalogue);
  const dir = resolve(options.dir);
  const id = options.id ?? slug(options.name ?? basename(dir));
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) throw new Error(`'${id}' is not a game id: use lowercase letters, digits and single hyphens`);
  const name = options.name ?? id.split("-").map(w => w[0]!.toUpperCase() + w.slice(1)).join(" ");
  if (existsSync(dir)) {
    if (!statSync(dir).isDirectory()) throw new Error(`${dir} is a file; pick another directory`);
    if (readdirSync(dir).length > 0 && !options.force) throw new Error(`${dir} is not empty; pick an empty directory, or pass --force`);
  }
  // Before anything is written: a pack copied without its pinned font makes a game that does not cook.
  const packs = fontPacksDir();
  for (const name of new Set(choices.lang.map((l) => fontPackFor(l, readPacks(packs))).filter((n) => n !== undefined))) {
    const missing = missingPins(packs, `fonts/${name}`);
    if (missing.length) throw new Error(`${missing.length} pinned file(s) of the checkout's templates/fonts/${name} are not installed (${missing[0]}); run bun scripts/assets.ts ensure templates`);
  }
  // The film's placeholder clip is pinned like the fonts (templates/og-assets.json).
  if (choices.form === "film") {
    const missing = missingPins(filmDir(), "film");
    if (missing.length) throw new Error(`${missing.length} pinned file(s) of the checkout's templates/film are not installed (${missing[0]}); run bun scripts/assets.ts ensure templates`);
  }
  const template = templateDir();
  const written: string[] = [];
  function copy(from: string, to: string) {
    mkdirSync(to, { recursive: true });
    for (const item of readdirSync(from, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      if (SKIPPED.has(item.name) || item.isSymbolicLink()) continue;
      const source = join(from, item.name), target = join(to, RENAMED.get(item.name) ?? item.name);
      if (item.isDirectory()) copy(source, target);
      else {
        cpSync(source, target);
        written.push(relative(dir, target));
      }
    }
  }
  for (const file of skeletonFiles()) {
    const target = join(dir, RENAMED.get(file) ?? file);
    mkdirSync(dirname(target), { recursive: true });
    cpSync(join(template, file), target);
    written.push(relative(dir, target));
  }
  // The font packs its languages are written in, as `og font add` installs
  // them (assets/fonts/<pack>/); og.i18n puts them in the fallback chain.
  const available = readPacks(packs);
  for (const name of new Set(choices.lang.map((l) => fontPackFor(l, available)).filter((n) => n !== undefined))) {
    copy(available.find((p) => p.name === name)!.dir, join(dir, "assets", "fonts", name));
  }
  // A film adds its story, segments, placeholder clip and entry (og api show film).
  if (choices.form === "film") copy(filmDir(), dir);
  const look = `looks/${id}.ogt`;
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  writeFileSync(join(dir, "assets", look), `type look\nversion 1\n---\nextends ${JSON.stringify(choices.look)}\n${choices.variant === undefined ? "" : `variant ${JSON.stringify(choices.variant)}\n`}ui ${JSON.stringify(choices.ui)}\n`);
  written.push(`assets/${look}`);
  const manifest = join(dir, "project.ogt");
  writeFileSync(manifest, readFileSync(manifest, "utf8")
    .replace(/^name ".*"$/m, () => `name ${JSON.stringify(name)}`)
    .replace(/^id ".*"$/m, () => `id ${JSON.stringify(id)}`) + `\nlook ${JSON.stringify(look)}\n\n${creationBlock(choices)}`);
  const pkg = join(dir, "package.json");
  const data = JSON.parse(readFileSync(pkg, "utf8"));
  data.name = id;
  writeFileSync(pkg, `${JSON.stringify(data, null, 2)}\n`);
  return { dir, name, id, files: [...new Set(written)], choices };
}

const USAGE = `usage: create [dir] [--name "Name"] [--id slug] [--premise "one line"] [--form 2d|2.5d|3d|film] [--look slug] [--force]
       create --creation [--previews previews.json]

Writes one empty Origin Engine project (default: ./origin-game).
Choices are constraints and hints in project.ogt's create block.
The look, variant and UI also configure its look file, which draws in the
look's own mood; no gameplay is generated.
  --form      2d, 2.5d or 3d (a game, its view one of the form's), or film: an
              interactive film (og.film), which adds a Yarn story, two segments, a
              placeholder clip, the picture and the film-choices and subtitles screens;
              --genre interactive-film is the same; view and players do not apply to a film
  --genre --view --look --variant --ui --platform --input --players --lang --music
              creation choices; --lang en,zh accepts comma-separated tags. A look must
              draw the form; a variant must be one of the look's. --mood is refused:
              a look draws in its own mood (write mood "<word>" in the look file)
  --no-prompt accept inferred choices without prompting (it never prompts without a terminal)
  --force     overwrite skeleton files in a nonempty directory
  --creation  print the creation document (every option, label, preview slot, compat rule
              and default; packages/create/README.md) as JSON and write nothing.
              --previews fills each slot from a JSON map of path -> { url, sha256, size, engine }`;

const FIELDS = ["form", "genre", "view", "look", "variant", "ui", "platform", "input", "players", "lang", "music"] as const;

function main(argv: string[]): number {
  let dir: string | undefined, name: string | undefined, id: string | undefined, premise: string | undefined;
  let force = false, ask = true, document = false, previews: string | undefined;
  let choices: Record<string, unknown> = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    const value = () => {
      const v = argv[++i];
      if (v === undefined) throw new Error(`${a} needs a value. ${USAGE}`);
      return v;
    };
    if (a === "--name") name = value();
    else if (a === "--id") id = value();
    else if (a === "--premise") premise = value();
    else if (a === "--force") force = true;
    else if (a === "--no-prompt") ask = false;
    else if (a === "--choices") continue;
    else if (a === "--creation") document = true;
    else if (a === "--previews") previews = value();
    else if (a === "--mood") throw new Error('--mood is no longer a creation choice: a look draws in its own mood (its preset\'s or variant\'s); to change it, write mood "<word>" in the game\'s look file');
    else if (a === "--look" || a === "--variant" || a === "--lang" || Object.keys(CREATION_CHOICES).some(key => a === `--${key}`)) choices[a.slice(2)] = a === "--lang" ? value().split(",") : value();
    else if (a === "-h" || a === "--help") { console.log(USAGE); return 0; }
    else if (a.startsWith("-")) throw new Error(`unknown option ${a}. ${USAGE}`);
    else if (dir === undefined) dir = a;
    else throw new Error(`one directory only, got '${dir}' and '${a}'`);
  }
  if (document) {
    const filled = previews === undefined ? undefined : JSON.parse(readFileSync(previews, "utf8")) as Record<string, PreviewFile>;
    process.stdout.write(`${JSON.stringify(creationDocument(filled), null, 2)}\n`);
    return 0;
  }
  if (ask && process.stdin.isTTY && process.stdout.isTTY && typeof globalThis.prompt === "function") {
    premise ??= globalThis.prompt("What is the game, in one line (genre, mood, setting)?") ?? "";
    for (const key of FIELDS) {
      if (choices[key] !== undefined) continue;
      const now = resolveCreationChoices(premise, parseCreationInput(choices));
      if (now.form === "film" && ((GAME_ONLY as readonly string[]).includes(key) || key === "genre")) continue;
      const styles = readCatalogue();
      const variants = (styles.find(s => s.slug === now.look)?.variants ?? []).map(v => v.name);
      if (key === "variant" && !variants.length) continue;
      if (key !== "look" && key !== "lang" && key !== "variant" && CREATION_CHOICES[key].length === 1) {
        choices[key] = CREATION_CHOICES[key][0];
        continue;
      }
      const suggested = now[key];
      const values = key === "look" ? creationLooks().join(", ") : key === "variant" ? `${variants.join(", ")} (empty: none)` : key === "lang" ? "comma-separated language tags" : key === "view" ? FORM_VIEWS[now.form].join(", ") : CREATION_CHOICES[key].join(", ");
      for (;;) {
        const answer = globalThis.prompt(`${key}: ${values}`, Array.isArray(suggested) ? suggested.join(",") : String(suggested ?? ""));
        if (answer === null) throw new Error("creation cancelled");
        if (key === "variant" && answer === "") break;
        try {
          const next = { ...choices, ...parseCreationInput({ [key]: key === "lang" ? answer.split(",") : answer }) };
          resolveCreationChoices(premise, parseCreationInput(next));
          choices = next;
          break;
        } catch (error) { console.error(error instanceof Error ? error.message : String(error)); }
      }
    }
  }
  const made = create({ dir: dir ?? id ?? (name ? slug(name) : "origin-game"), ...(name === undefined ? {} : { name }), ...(id === undefined ? {} : { id }), ...(premise === undefined ? {} : { premise }), choices: parseCreationInput(choices), force });
  console.log(`Created ${made.name} (${made.id}) in ${relative(process.cwd(), made.dir) || "."}: ${made.files.length} files.\nCreation choices are in project.ogt. ${made.choices.form === "film" ? "Write the story and segments (og api show film), replace the placeholder clip" : "Build the game with the engine API"}, then run og check and og run .`);
  return 0;
}

if (import.meta.main) {
  try { process.exit(main(process.argv.slice(2))); }
  catch (e) { console.error(`create: ${e instanceof Error ? e.message : String(e)}`); process.exit(1); }
}
