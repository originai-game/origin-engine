#!/usr/bin/env bun
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { CREATION_CHOICES, creationBlock, inferCreationChoices, parseCreationInput } from "./choices.ts";
import type { CreationChoices, CreationInput } from "./choices.ts";

export { CREATION_CHOICES, creationBlock, creationSchema, inferCreationChoices, parseCreationInput } from "./choices.ts";
export type { CreationChoices, CreationInput } from "./choices.ts";
export { canonicalTag, isRtl, localeScript } from "./languages.ts";
export type { LocaleScript } from "./languages.ts";
export { PSEUDO_LOCALE, pseudoMessage, pseudoTable } from "./pseudo-locale.ts";
export { ART_KINDS, ART_RESOURCES, artBrief, authoredArt, colourName, MOODS, parseLook, parsePalette, presetLook, readLook, styleArtBriefs } from "./brief.ts";
export type { ArtBlocks, ArtBrief, ArtKind, ArtResource, ArtValue, AuthoredArt, BriefLook, BriefOptions, BriefPalette, BriefReference, BriefStyle, ImageParams, ModelParams, Mood, PaletteColour, ReadLookOptions, StyleArtBrief, StyleArtRequests } from "./brief.ts";
export { audioBrief, MUSIC_STYLES, readAudioChoices, SFX_PRESETS } from "./brief.ts";
export type { AudioBrief, AudioBriefInput, AudioFormat, MusicCueBrief, MusicRole, MusicStyle, SfxBrief, SfxPreset, SfxPresetBrief } from "./brief.ts";
export { RIG_TEMPLATES, rigParts } from "./brief.ts";
export type { RigJoint, RigPart, RigTemplate } from "./brief.ts";

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
}

export function catalogueFile(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const candidate of [join(here, "..", "looks", "catalogue.json"), join(here, "..", "..", "knowledge", "looks", "catalogue.json")]) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`no style catalogue next to ${here}; reinstall the engine distribution`);
}

export function readCatalogue(file = catalogueFile()): Style[] {
  return (JSON.parse(readFileSync(file, "utf8")) as { styles: Style[] }).styles;
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

/** Resolve the agent's choices without writing any project files. */
export function resolveCreationChoices(premise: string, input: CreationInput = {}, profile?: Profile, catalogue?: string): CreationChoices {
  const asked = inferCreationChoices(premise, input);
  const styles = readCatalogue(catalogue);
  const effectiveProfile = profile ?? (asked.view.endsWith("2d") ? "2d" : "3d");
  const slug = input.look ?? pickStyle(premise, styles, effectiveProfile, asked.view)?.style.slug ?? (effectiveProfile === "2d" ? "16bit-pixel" : "faceted-low-poly");
  const style = styles.find(s => s.slug === slug && s.look.presetReady);
  if (!style) throw new Error(`no look preset '${slug}'`);
  return inferCreationChoices(premise, { ...input, look: slug, ui: parseCreationInput({ ui: input.ui ?? style.ui }).ui! }, parseCreationInput({ mood: style.look.mood }));
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
  copy(template, dir);
  const look = `looks/${id}.ogt`;
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  writeFileSync(join(dir, "assets", look), `type look\nversion 1\n---\nextends ${JSON.stringify(choices.look)}\nmood ${JSON.stringify(choices.mood)}\nui ${JSON.stringify(choices.ui)}\n`);
  written.push(`assets/${look}`);
  const manifest = join(dir, "project.ogt");
  writeFileSync(manifest, readFileSync(manifest, "utf8")
    .replace(/^name ".*"$/m, () => `name ${JSON.stringify(name)}`)
    .replace(/^id ".*"$/m, () => `id ${JSON.stringify(id)}`) + `\nlook ${JSON.stringify(look)}\n\n${creationBlock(choices)}`);
  const pkg = join(dir, "package.json");
  const data = JSON.parse(readFileSync(pkg, "utf8"));
  data.name = id;
  writeFileSync(pkg, `${JSON.stringify(data, null, 2)}\n`);
  return { dir, name, id, files: written, choices };
}

const USAGE = `usage: create [dir] [--name "Name"] [--id slug] [--premise "one line"] [--look slug] [--force]

Writes one empty Origin Engine project (default: ./origin-game).
Choices are constraints and hints in project.ogt's create block.
The look, mood and UI also configure its look file; no gameplay is generated.
  --genre --view --look --mood --ui --platform --input --players --lang --music
           creation choices; --lang en,zh accepts comma-separated tags
  --no-prompt  accept inferred choices without prompting
  --force     overwrite skeleton files in a nonempty directory`;

function main(argv: string[]): number {
  let dir: string | undefined, name: string | undefined, id: string | undefined, premise: string | undefined;
  let force = false, ask = true;
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
    else if (a === "--look" || a === "--lang" || Object.keys(CREATION_CHOICES).some(key => a === `--${key}`)) choices[a.slice(2)] = a === "--lang" ? value().split(",") : value();
    else if (a === "-h" || a === "--help") { console.log(USAGE); return 0; }
    else if (a.startsWith("-")) throw new Error(`unknown option ${a}. ${USAGE}`);
    else if (dir === undefined) dir = a;
    else throw new Error(`one directory only, got '${dir}' and '${a}'`);
  }
  if (ask && process.stdin.isTTY && process.stdout.isTTY && typeof globalThis.prompt === "function") {
    premise ??= globalThis.prompt("What is the game, in one line (genre, mood, setting)?") ?? "";
    for (const key of ["genre", "view", "look", "mood", "ui", "platform", "input", "players", "lang", "music"] as const) {
      if (choices[key] !== undefined) continue;
      const suggested = resolveCreationChoices(premise, parseCreationInput(choices))[key];
      const values = key === "look" ? readCatalogue().map(s => s.slug).join(", ") : key === "lang" ? "comma-separated language tags" : CREATION_CHOICES[key].join(", ");
      for (;;) {
        const answer = globalThis.prompt(`${key}: ${values}`, Array.isArray(suggested) ? suggested.join(",") : String(suggested));
        if (answer === null) throw new Error("creation cancelled");
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
  console.log(`Created ${made.name} (${made.id}) in ${relative(process.cwd(), made.dir) || "."}: ${made.files.length} files.\nCreation choices are in project.ogt. Build the game with the engine API, then run og check and og run .`);
  return 0;
}

if (import.meta.main) {
  try { process.exit(main(process.argv.slice(2))); }
  catch (e) { console.error(`create: ${e instanceof Error ? e.message : String(e)}`); process.exit(1); }
}
