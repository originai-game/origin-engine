#!/usr/bin/env bun
// `npm create @origingame/game` (and `bun create`): scaffolds the Origin
// Engine project template (templates/game) into a directory, naming the game.
//
//   create [dir] [--name "Name"] [--id slug] [--profile 2d|3d] [--archetype runner|match3|...]
//          [--premise "one line"] [--look <slug>] [--force]
//
// The template comes from ./template/ (the 3D template, templates/game) or
// ./template-2d/ (the 2D starter, templates/game-2d) in the published
// package (prepack: scripts/pack-template.ts), or from ../../templates/ in
// the engine repository; an archetype (a complete small game to start from)
// from ./archetypes/<name>/ or ../../templates/archetypes/<name>/. The 2D
// starter is written for --profile 2d, or for a premise that says 2D
// (platformer, side-scroller, metroidvania, pixel art) without saying 3D.
//
// Every game gets its look, `assets/looks/<id>.ogt`, named by GameDesc.look
// in the entry and by project.ogt's `look` line (the look whose art contract
// the cook reads, docs/art.md §2.1): it extends the style catalogue's preset
// given by --look, or the one whose premise best matches --premise (asked
// for on a terminal when neither is given), else the template's own. The
// catalogue is
// ./looks/catalogue.json in the published package (prepack) or
// ../../knowledge/looks/catalogue.json in the engine repository.

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// The art brief (docs/art.md §2.2): a game's look compiled into the request
// for one asset. bin/brief.ts is a copy of the knowledge package's
// scripts/brief.ts (scripts/pack-template.ts writes it; the tests hold it
// equal); README §The art brief.
export { ART_KINDS, ART_RESOURCES, artBrief, colourName, parseLook, parsePalette, presetLook, readLook } from "./brief.ts";
export type { ArtBlocks, ArtBrief, ArtKind, ArtResource, ArtValue, BriefLook, BriefOptions, BriefPalette, BriefReference, BriefStyle, ImageParams, ModelParams, PaletteColour, ReadLookOptions } from "./brief.ts";

/** Directories of a template copy that are build output, never scaffolded. */
export const SKIPPED = new Set(["cooked", "dist", "node_modules", "og-out", ".og"]);

/**
 * Template files stored under another name and renamed in the scaffold: the
 * template's Git attributes route media through LFS in a game's repository,
 * but as `.gitattributes` they would do the same in the engine repository,
 * which stores the template's media as plain blobs; and `.gitignore`, which
 * npm drops from every tarball, so the published package carries it as
 * `gitignore` (scripts/pack-template.ts renames it at pack; the engine
 * repository keeps the dotted name, and a copy taken from there scaffolds
 * the same).
 */
export const RENAMED: ReadonlyMap<string, string> = new Map([
  ["gitattributes", ".gitattributes"],
  ["gitignore", ".gitignore"],
]);

export interface CreateOptions {
  readonly dir: string;
  readonly name?: string;
  readonly id?: string;
  readonly force?: boolean;
  /** Start from an archetype (`runner`, `match3`, `topdown`, `tower-defense`, `roguelike`, `idle`, `rhythm`, `third-person`, `arena`: templates/archetypes/README.md) instead of the project template. */
  readonly archetype?: string;
  /** The template's profile: `2d` writes the 2D starter (templates/game-2d), `3d` the 3D template (templates/game). Unset, a premise that says 2D picks `2d`; else `3d`. Not for an archetype, which has its own. */
  readonly profile?: Profile;
  /** The template directory; found next to this file when omitted. */
  readonly template?: string;
  /** A style catalogue slug the game's look extends (og-taste §12). */
  readonly look?: string;
  /** The game's premise in a line: the look extends the catalogue style that fits it best, unless `look` is given. */
  readonly premise?: string;
  /** The catalogue file; found next to this file when omitted. */
  readonly catalogue?: string;
}

/** The game's look as written. */
export interface CreatedLook {
  /** The look file, relative to the game (`assets/looks/<id>.ogt`). */
  readonly file: string;
  /** Its asset id, as GameDesc.look names it (`looks/<id>.ogt`). */
  readonly id: string;
  /** The catalogue slug it extends; null for a look that extends no preset. */
  readonly extends: string | null;
  /** How the preset was chosen. */
  readonly chosen: "flag" | "premise" | "template" | "none";
}

/** A project's profile (`project.ogt`'s `profile` line): 2D or 3D. */
export type Profile = "2d" | "3d";

/** The profile the game was written in, and how it was chosen. */
export interface CreatedProfile {
  readonly value: Profile;
  /** `flag` for --profile, `premise` for a premise that says 2D, `archetype` for an archetype's own, `template` for the 3D template by default. */
  readonly chosen: "flag" | "premise" | "archetype" | "template";
}

export interface Created {
  readonly dir: string;
  readonly name: string;
  readonly id: string;
  readonly files: readonly string[];
  readonly profile: CreatedProfile;
  readonly look: CreatedLook;
  /** What the game's author should know, such as reference frames a look will move. */
  readonly notes: readonly string[];
}

/** A style catalogue entry: the fields create reads (packages/knowledge/looks/catalogue.json). */
export interface Style {
  readonly slug: string;
  readonly name: string;
  readonly category: string;
  readonly descriptor: string;
  readonly premise: Readonly<Record<string, string>>;
  /** `profiles`: the project profiles the look draws for, `2d`, `3d` or both (a flipbook or hand-pixelled style is 2D only, a style of meshes and lit materials 3D only). */
  readonly look: { readonly presetReady: boolean; readonly support: string; readonly profiles: readonly Profile[]; readonly missing: readonly string[] };
  readonly ui: string;
}

/** The style catalogue this copy of the package reads. */
export function catalogueFile(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const candidate of [join(here, "..", "looks", "catalogue.json"), join(here, "..", "..", "knowledge", "looks", "catalogue.json")]) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`no style catalogue next to ${here}: expected ../looks/catalogue.json (published package) or ../../knowledge/looks/catalogue.json (engine repository)`);
}

export function readCatalogue(file = catalogueFile()): Style[] {
  return (JSON.parse(readFileSync(file, "utf8")) as { styles: Style[] }).styles;
}

const STOP = new Set(["the", "and", "for", "with", "about", "into", "from", "that", "this", "game", "games", "player", "players", "where", "who", "your", "you", "its", "set", "lot", "very", "like", "one", "two", "all", "any", "out", "their", "them", "has", "have", "are", "was", "but", "not", "own"]);
const SPELLING: Readonly<Record<string, string>> = { cozy: "cosy", color: "colour", colors: "colours", gray: "grey", scifi: "sci", "sci-fi": "sci", watercolour: "watercolor", "8-bit": "8bit", "16-bit": "16bit", "1-bit": "1bit" };

/** Lowercase words, spelling unified and a plural's s dropped: `Cozy Platformers` -> cosy, platformer. */
export function words(text: string): string[] {
  const spelt = (w: string) => SPELLING[w] ?? w;
  return (text.toLowerCase().match(/[a-z0-9]+(?:-[a-z0-9]+)*/g) ?? [])
    .flatMap((w) => (SPELLING[w] ? [SPELLING[w]!] : w.includes("-") ? [w, ...w.split("-").map(spelt)] : [spelt(w)]))
    .map((w) => (w.length > 4 && w.endsWith("s") && !w.endsWith("ss") ? w.slice(0, -1) : w))
    .filter((w) => w.length >= 3 && !STOP.has(w));
}

/**
 * The catalogue style whose premise fits a game's premise best: each word of
 * the premise found in a style's genre counts 3, in its mood 2, in its era
 * or audience 1, in its name or descriptor 1. Ties go to the style whose
 * look the engine draws whole (support `full`), then to catalogue order.
 * Null when no word matches any style. Only styles with a preset that draw
 * for the project's profile (`look.profiles`, 3D unless `2d`) are chosen: a
 * 3D party premise never lands on a flipbook style, a 2D platformer never on
 * a style of lit meshes. `--look` is not weighed here and may pick any.
 */
export function pickStyle(premise: string, styles: readonly Style[], profile: Profile = "3d"): { style: Style; score: number } | null {
  const asked = new Set(words(premise));
  if (asked.size === 0) return null;
  const weights: [(s: Style) => string, number][] = [
    [(s) => s.premise.genre ?? "", 3],
    [(s) => s.premise.mood ?? "", 2],
    [(s) => s.premise.era ?? "", 1],
    [(s) => s.premise.audience ?? "", 1],
    [(s) => `${s.name} ${s.descriptor}`, 1],
  ];
  let best: { style: Style; score: number; full: boolean } | null = null;
  for (const style of styles) {
    if (!style.look.presetReady || !style.look.profiles.includes(profile)) continue;
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

/** `My Game!` -> `my-game`. */
export function slug(name: string): string {
  return name
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/** `my-game` -> `My Game`. */
function title(id: string): string {
  return id
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((w) => w[0]!.toUpperCase() + w.slice(1))
    .join(" ");
}

/** Words of a premise that say the game is 2D (platformer, side-scroller, metroidvania, pixel art, sprites, tiles). */
const TWO_D = new Set(["platformer", "platforming", "side-scroller", "side-scrolling", "sidescroller", "sidescrolling", "scroller", "metroidvania", "pixel", "pixel-art", "pixelart", "sprite", "tilemap", "tile", "8bit", "16bit", "1bit"]);
const THREE_D = new Set(["third-person", "first-person", "fps", "voxel", "lowpoly", "low-poly"]);

/**
 * The profile a premise asks for: `2d` when it uses a 2D word (TWO_D, or
 * `2D` itself) and no 3D word (THREE_D, or `3D`), else null: the template's
 * own, 3D, applies.
 */
export function premiseProfile(premise: string): Profile | null {
  const asked = new Set(words(premise));
  const says = (w: string) => new RegExp(`(^|[^a-z0-9])${w}($|[^a-z0-9])`, "i").test(premise);
  if (says("3d") || [...THREE_D].some((w) => asked.has(w))) return null;
  return says("2d") || [...TWO_D].some((w) => asked.has(w)) ? "2d" : null;
}

/** The `profile "2d"` line of a project manifest; `3d` when it has none. */
function profileOf(dir: string): Profile {
  const m = /^profile[ \t]+"(2d|3d)"/m.exec(readFileSync(join(dir, "project.ogt"), "utf8"));
  return m?.[1] === "2d" ? "2d" : "3d";
}

/** The template this copy of the package scaffolds: the archetype's, or the project template of a profile (3D unless `2d`). */
export function templateDir(archetype?: string, profile: Profile = "3d"): string {
  const here = dirname(fileURLToPath(import.meta.url));
  if (archetype !== undefined) {
    const roots = [join(here, "..", "archetypes"), join(here, "..", "..", "..", "templates", "archetypes")];
    for (const root of roots) {
      if (existsSync(join(root, archetype, "project.ogt"))) return join(root, archetype);
    }
    const known = roots.filter(existsSync).flatMap((r) => readdirSync(r).filter((d) => existsSync(join(r, d, "project.ogt"))));
    throw new Error(`no archetype '${archetype}': choose one of ${[...new Set(known)].sort().join(", ") || "(none shipped)"}`);
  }
  const packed = profile === "2d" ? "template-2d" : "template";
  const source = profile === "2d" ? "game-2d" : "game";
  // In the engine repository the live template wins: a ./template/ left by an
  // earlier prepack is stale as soon as templates/ changes. A published
  // package has no templates/ beside it, so it scaffolds from its packed copy.
  for (const candidate of [join(here, "..", "..", "..", "templates", source), join(here, "..", packed)]) {
    if (existsSync(join(candidate, "project.ogt"))) return candidate;
  }
  throw new Error(`no ${profile} project template next to ${here}: expected ../${packed} (published package) or ../../../templates/${source} (engine repository)`);
}

function ogtString(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")}"`;
}

/** Replace the value of a top-level `key "..."` line of an .ogt document. */
function setOgt(text: string, key: string, value: string): string {
  const line = new RegExp(`^${key}[ \\t]+"(?:[^"\\\\]|\\\\.)*"[ \\t]*$`, "m");
  if (!line.test(text)) throw new Error(`the template's project.ogt has no \`${key} "..."\` line`);
  return text.replace(line, `${key} ${ogtString(value)}`);
}

/**
 * project.ogt naming the game's look (`look "looks/<id>.ogt"`, the look whose
 * art contract the cook reads): a template's own line is replaced, else the
 * line is added after `entry` (or at the end).
 */
function projectLook(text: string, lookId: string): string {
  const line = `look ${ogtString(lookId)}`;
  const own = /^look[ \t]+"(?:[^"\\]|\\.)*"[ \t]*$/m;
  if (own.test(text)) return text.replace(own, line);
  const block = `# The game's look: its art contract the cook reads, GameDesc.look's default.\n${line}`;
  const entry = /^entry[ \t]+"(?:[^"\\]|\\.)*"[ \t]*$/m.exec(text);
  if (!entry) return `${text.replace(/\n*$/, "\n")}${block}\n`;
  const at = entry.index + entry[0].length;
  return `${text.slice(0, at)}\n${block}${text.slice(at)}`;
}

/** Every file under `root`, relative, skipping build output. */
function files(root: string, at = root): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(at).sort()) {
    const path = join(at, entry);
    if (statSync(path).isDirectory()) {
      if (SKIPPED.has(entry) && at === root) continue;
      out.push(...files(root, path));
    } else out.push(relative(root, path));
  }
  return out;
}

/** The entry file a project manifest names (`entry "src/main.ts"`). */
function entryOf(dir: string): string | undefined {
  const m = /^entry[ \t]+"([^"]+)"/m.exec(readFileSync(join(dir, "project.ogt"), "utf8"));
  return m?.[1];
}

const LOOK_KEY = /^([ \t]*)look:[ \t]*"looks\/([^"]+)\.ogt",?[ \t]*$/m;

/** A look file's `extends` and its lines after `---` without the header comment, `name` and `extends`. */
function lookBody(text: string): { extends: string | null; body: string[] } {
  const lines = text.split("\n");
  const at = lines.indexOf("---");
  const rest = at < 0 ? lines : lines.slice(at + 1);
  const ext = /^extends[ \t]+"([^"]+)"/m.exec(rest.join("\n"))?.[1] ?? null;
  // The header comment goes (create writes its own); comments inside stay.
  let first = 0;
  while (first < rest.length && (rest[first]!.trim() === "" || rest[first]!.startsWith("#"))) first++;
  const body = rest.slice(first).filter((l) => !/^(name|extends)[ \t]/.test(l));
  while (body.length > 0 && body[body.length - 1]!.trim() === "") body.pop();
  return { extends: ext, body };
}

/** The game's look file. */
function lookText(name: string, style: Style | undefined, slug: string | null, id: string, body: readonly string[]): string {
  const head = [
    "type look",
    "version 1",
    "---",
    `# The look of ${name} (og-taste §12): the frame's art direction as data.`,
    "# GameDesc.look in the entry names it; it is applied after the first scene",
    "# spawns and before start runs, so start may still change any of it.",
  ];
  if (slug && style) {
    head.push(
      `# It extends the style catalogue's ${slug} preset (${style.name}),`,
      style.look.support === "full" ? "# whose primitives all draw today." : `# support ${style.look.support}: ${style.look.missing.join("; ")}.`,
      `# The preset dresses the UI in the built-in ${style.ui} style; a \`ui "<slug>"\``,
      `# line here picks another, or the game's own assets/styles/<name>/style.ogt.`,
    );
  } else if (slug) head.push(`# It extends ${slug}.`);
  else head.push("# It extends no preset yet: the frame is the engine's defaults plus the lines below.");
  head.push(
    "# A block below (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog,",
    "# Sky, Environment, and the art contract's ArtPalette, ArtPixels, ArtTexels,",
    "# ArtShape, ArtLine, ArtMotion, ArtCamera) wins over the preset field by",
    "# field. project.ogt's `look` line names this file: the cook reads its art",
    "# contract. For another style, set `extends` to another catalogue slug and",
    "# run og cook.",
    `name ${ogtString(name)}`,
  );
  if (slug) head.push(`extends ${ogtString(slug)}`);
  return `${[...head, ...body].join("\n")}\n`;
}

/** The README section that tells the game's author where the look is and how to change it. */
function lookReadme(look: CreatedLook, style: Style | undefined, entry: string, references: boolean): string {
  const how = look.chosen === "premise" ? ", chosen from the game's premise" : look.chosen === "flag" ? ", chosen with `--look`" : "";
  const lines = [
    "",
    "## The look",
    "",
    "The game's art direction is data, in one file a reader can diff:",
    `\`${look.file}\` (an \`.ogt\` of \`type look\`), named by`,
    `\`look: "${look.id}"\` in the \`og.game({...})\` of \`${entry}\` and by`,
    `\`look "${look.id}"\` in \`project.ogt\`. It is applied`,
    "after the first scene spawns and before `start` runs, and sets tone",
    "mapping and exposure, the post stack, the stylisation (`Stylise`),",
    "material defaults (`MaterialStyle`: toon, outlines, flat shading), fog,",
    "sky, the environment map and the UI style (its `ui` line).",
    "",
    "It is also the art contract every asset is made to: the palette",
    "(`ArtPalette`), the pixel grid (`ArtPixels`), texel density",
    "(`ArtTexels`), the shape language (`ArtShape`), the line (`ArtLine`),",
    "animation timing (`ArtMotion`) and the camera (`ArtCamera`). The cook",
    "reads it from `project.ogt`'s `look` line; a field left at zero",
    "constrains nothing.",
    "",
  ];
  if (look.extends && style) {
    lines.push(`It extends \`${look.extends}\`${how}: "${style.name}" in the style`, `catalogue. ${style.descriptor} Its UI style is \`${style.ui}\`.`, "");
  } else if (look.extends) lines.push(`It extends \`${look.extends}\`${how}.`, "");
  else lines.push("It extends no preset yet: the frame is the engine's defaults plus what", "the file sets.", "");
  lines.push(
    "To change it:",
    "",
    "- **Another style**: set `extends \"<slug>\"` to another entry of the style",
    "  catalogue (the og-taste skill's §12 lists all 37 with the premises each",
    "  fits and three checks to hold the frames to), then `og cook`.",
    "- **Tune it**: add a block after `extends`; a field written there wins",
    "  over the preset's, field by field:",
    "",
    "  ```text",
    "  RenderSettings {",
    "    toneMapping agx",
    "    exposure 1.1",
    "  }",
    "  ```",
    "",
    "- **Switch at a tick** (a flashback, an area's mood): `og.load.look` a",
    "  second look file and `og.render.applyLook(world, look)`.",
    "- **Check it**: `og run --headless --commands \"1:screenshot:look.png;1.1:quit\"`",
    "  and read the frame against the style's checks.",
  );
  if (references) {
    lines.push(
      "",
      "The frames in `references/` show the look they were recorded with: after",
      "changing the look, record them again with `OG_FX_CAPTURE=1 og test .` and",
      "read the new frames before committing them.",
    );
  }
  return `${lines.join("\n")}\n`;
}

/** Scaffold the template into `options.dir`. Throws with the fix on refusal. */
export function create(options: CreateOptions): Created {
  const dir = resolve(options.dir);
  const id = options.id ?? slug(options.name ?? basename(dir));
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id)) {
    throw new Error(`'${id}' is not a game id: use lowercase letters, digits and single hyphens, as \`--id my-game\``);
  }
  const name = options.name ?? title(id);
  if (existsSync(dir)) {
    if (!statSync(dir).isDirectory()) throw new Error(`${dir} is a file; pick another directory`);
    if (readdirSync(dir).length > 0 && !options.force) {
      throw new Error(`${dir} is not empty; pick an empty directory, or pass --force to write the template over it`);
    }
  }
  if (options.profile !== undefined && options.archetype !== undefined) {
    throw new Error(`--profile and --archetype together: an archetype has its own profile (${options.archetype}); drop one`);
  }
  const asked = options.profile ?? (options.premise !== undefined ? premiseProfile(options.premise) : null);
  const template = options.template ?? templateDir(options.archetype, asked ?? "3d");
  const profile: CreatedProfile = {
    value: profileOf(template),
    chosen: options.archetype !== undefined ? "archetype" : options.profile !== undefined ? "flag" : asked !== null ? "premise" : "template",
  };
  const choice = chooseStyle(options, profile.value);
  const list = files(template);
  const written: string[] = [];
  mkdirSync(dir, { recursive: true });
  for (const file of list) {
    const to = RENAMED.get(file) ?? file;
    mkdirSync(dirname(join(dir, to)), { recursive: true });
    cpSync(join(template, file), join(dir, to));
    written.push(to);
  }
  const manifest = join(dir, "project.ogt");
  writeFileSync(manifest, setOgt(setOgt(readFileSync(manifest, "utf8"), "name", name), "id", id));
  const pkgPath = join(dir, "package.json");
  if (existsSync(pkgPath)) {
    const pkg = JSON.parse(readFileSync(pkgPath, "utf8")) as Record<string, unknown>;
    pkg.name = id;
    writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
  }
  const { look, notes } = writeLook(dir, name, id, choice, written);
  return { dir, name, id, files: written, profile, look, notes };
}

/**
 * The game's look: `assets/looks/<id>.ogt`, carrying the template's own look
 * (the file its entry names) with the preset `options.look` or the premise
 * chose, and the entry's GameDesc.look pointed at it (inserted after
 * `og.game({` when the template names none).
 */
/** The preset `--look` names or the premise fits for the project's profile; throws for a slug the catalogue has no preset for. */
function chooseStyle(options: CreateOptions, profile: Profile): { styles: Style[]; style: Style | undefined; chosen: CreatedLook["chosen"]; notes: string[] } {
  const styles = readCatalogue(options.catalogue);
  if (options.look !== undefined) {
    const style = styles.find((s) => s.slug === options.look);
    if (!style || !style.look.presetReady) {
      throw new Error(`no look preset '${options.look}': choose one of the style catalogue's slugs (og-taste §12): ${styles.filter((s) => s.look.presetReady).map((s) => s.slug).join(", ")}`);
    }
    return { styles, style, chosen: "flag", notes: [] };
  }
  if (options.premise !== undefined && options.premise.trim() !== "") {
    const pick = pickStyle(options.premise, styles, profile);
    if (pick) return { styles, style: pick.style, chosen: "premise", notes: [] };
    return { styles, style: undefined, chosen: "none", notes: [`no catalogue style matched the premise "${options.premise}" for a ${profile.toUpperCase()} game; the look keeps the template's (og-taste §1 chooses one by genre, mood, era and audience: pass --look <slug>)`] };
  }
  return { styles, style: undefined, chosen: "none", notes: [] };
}

function writeLook(dir: string, name: string, id: string, choice: ReturnType<typeof chooseStyle>, written: string[]): { look: CreatedLook; notes: string[] } {
  const { styles, notes } = choice;
  let { style, chosen } = choice;

  const entry = entryOf(dir) ?? "src/main.ts";
  const entryPath = join(dir, entry);
  const source = existsSync(entryPath) ? readFileSync(entryPath, "utf8") : undefined;
  const named = source === undefined ? undefined : LOOK_KEY.exec(source);
  const file = `assets/looks/${id}.ogt`;
  const lookId = `looks/${id}.ogt`;
  let base: { extends: string | null; body: string[] } = { extends: null, body: [] };
  if (named) {
    const old = join(dir, "assets", "looks", `${named[2]}.ogt`);
    if (existsSync(old)) {
      base = lookBody(readFileSync(old, "utf8"));
      if (`assets/looks/${named[2]}.ogt` !== file) {
        rmSync(old);
        const listed = written.indexOf(`assets/looks/${named[2]}.ogt`);
        if (listed >= 0) written.splice(listed, 1);
      }
    }
  }
  let slug: string | null = style?.slug ?? base.extends;
  if (!style && slug) {
    style = styles.find((s) => s.slug === slug);
    if (style) chosen = "template";
  }
  mkdirSync(join(dir, "assets", "looks"), { recursive: true });
  writeFileSync(join(dir, file), lookText(name, style, slug, id, base.body));
  if (!written.includes(file)) written.push(file);
  const manifest = join(dir, "project.ogt");
  writeFileSync(manifest, projectLook(readFileSync(manifest, "utf8"), lookId));

  if (source !== undefined) {
    let next: string | undefined;
    if (named) next = source.replace(LOOK_KEY, `${named[1]}look: "${lookId}",`);
    else {
      const game = /^([ \t]*)export default og\.game\(\{[ \t]*$/m.exec(source);
      if (game) {
        const at = game.index + game[0].length;
        next = `${source.slice(0, at)}\n${game[1]}  // The art direction as data (assets/${lookId}; og-taste §12).\n${game[1]}  look: "${lookId}",${source.slice(at)}`;
      }
    }
    if (next !== undefined) writeFileSync(entryPath, next);
    else notes.push(`${entry} has no \`export default og.game({\` line: add \`look: "${lookId}"\` to the game's desc yourself`);
  }

  const references = existsSync(join(dir, "references"));
  if (references && slug !== base.extends) {
    notes.push(`the frames in references/ show the archetype's own look; with ${slug} they will differ: record them again with \`OG_FX_CAPTURE=1 og test .\``);
  }
  if (style && style.look.support !== "full") notes.push(`${style.slug} is \`${style.look.support}\`: ${style.look.missing.join("; ")} (og-taste §12)`);
  const look: CreatedLook = { file, id: lookId, extends: slug, chosen };
  const readme = join(dir, "README.md");
  const section = lookReadme(look, style, entry, references);
  writeFileSync(readme, existsSync(readme) ? `${readFileSync(readme, "utf8").replace(/\n*$/, "\n")}${section}` : `# ${name}\n${section}`);
  if (!written.includes("README.md")) written.push("README.md");
  return { look, notes };
}

const USAGE = `usage: create [dir] [--name "Name"] [--id slug] [--profile 2d|3d] [--archetype name] [--premise "one line"] [--look slug] [--force]

Scaffolds an Origin Engine game into dir (default: ./origin-game).
  --name   the game's display name (default: from the id or the directory)
  --id     the game's id, lowercase-hyphen (default: from the name)
  --profile    2d writes the 2D starter (a platformer: sprites, tilemap, 2D physics), 3d the 3D template (default: 2d for a premise that says 2D, platformer, side-scroller, metroidvania or pixel art without saying 3D; else 3d)
  --archetype  start from a complete archetype game (runner, match3, topdown, tower-defense, roguelike, idle, rhythm, third-person, arena) instead of the template; it has its own profile
  --premise    the game in one line (genre, mood, setting, audience): its look extends the style catalogue's best match among the styles drawn for the game's profile (asked for on a terminal when neither this nor --look is given)
  --look   the style catalogue slug the look extends (og-taste §12), such as anime-cel-3d or 16bit-pixel; it wins over the premise and the profile
  --no-prompt  never ask for the premise
  --force  write into a directory that is not empty`;

function main(argv: string[]): number {
  let dir: string | undefined;
  let name: string | undefined;
  let id: string | undefined;
  let force = false;
  let archetype: string | undefined;
  let profile: Profile | undefined;
  let premise: string | undefined;
  let look: string | undefined;
  let ask = true;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    const value = () => {
      const v = argv[++i];
      if (v === undefined) throw new Error(`${a} needs a value. ${USAGE}`);
      return v;
    };
    if (a === "--name") name = value();
    else if (a === "--id") id = value();
    else if (a === "--force") force = true;
    else if (a === "--archetype") archetype = value();
    else if (a === "--profile") {
      const v = value();
      if (v !== "2d" && v !== "3d") throw new Error(`--profile ${v}: use 2d or 3d. ${USAGE}`);
      profile = v;
    } else if (a === "--premise") premise = value();
    else if (a === "--look") look = value();
    else if (a === "--no-prompt") ask = false;
    else if (a === "-h" || a === "--help") {
      console.log(USAGE);
      return 0;
    } else if (a.startsWith("-")) throw new Error(`unknown option ${a}. ${USAGE}`);
    else if (dir === undefined) dir = a;
    else throw new Error(`one directory only, got '${dir}' and '${a}'. ${USAGE}`);
  }
  if (ask && premise === undefined && look === undefined && process.stdin.isTTY && process.stdout.isTTY && typeof globalThis.prompt === "function") {
    premise = globalThis.prompt("What is the game, in one line (genre, mood, setting)? It picks the look; Enter keeps the template's:") ?? undefined;
  }
  const made = create({
    dir: dir ?? (id ?? (name ? slug(name) : "origin-game")),
    ...(name ? { name } : {}),
    ...(id ? { id } : {}),
    ...(archetype ? { archetype } : {}),
    ...(profile ? { profile } : {}),
    ...(premise ? { premise } : {}),
    ...(look ? { look } : {}),
    force,
  });
  const shown = relative(process.cwd(), made.dir) || ".";
  const style = made.look.extends ? `extends ${made.look.extends}${made.look.chosen === "premise" ? " (from the premise)" : made.look.chosen === "flag" ? " (--look)" : ""}` : "extends no preset";
  const how = { flag: " (--profile)", premise: " (from the premise)", archetype: ` (the ${archetype} archetype's)`, template: "" }[made.profile.chosen];
  console.log(`Created ${made.name} (${made.id}) in ${shown}: ${made.files.length} files.
Profile: ${made.profile.value}${how}.
Look: ${made.look.file}, ${style}; README.md says how to change it.${made.notes.map((n) => `\nNote: ${n}`).join("")}

Next:
  cd ${shown}
  bun install
  og cook
  og run .`);
  return 0;
}

if (import.meta.main) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (e) {
    console.error(`create: ${e instanceof Error ? e.message : String(e)}`);
    process.exit(1);
  }
}
