// The art brief (docs/art.md §2.2): a game's look compiled into the request
// for one asset. `artBrief(look, kind, subject)` is a pure function: from the
// flattened look (its art blocks, the preset slug it extends, its resolved
// palette) and the style's catalogue entry it returns the prompt (the
// style's direction, the palette as named colours with hex, the sizes, the
// line, the shape language, what to avoid), the reference images (the
// style's preview, then the game's accepted assets the caller passes) and
// the generator's parameters (transparent and size for an image; the face
// budget and texture size for a model). `readLook(projectDir)` reads a
// project's look from disk (project.ogt's `look` line, its `extends` chain
// down to the preset, and the mood it asks of the preset) and
// `presetLook(slug)` a preset's alone, for a game not written yet (in its own
// mood, or the one `options.mood` asks). `authoredArt(look)` lists the art a
// `content` style needs made for it, which `artBrief` asks for by kind.
//
// The platform calls it: `artBrief`, `readLook`, `presetLook`, their option
// and result shapes, `ART_KINDS` and `ART_RESOURCES` are a public contract
// (README §The art brief), kept stable; a change to any of them is a break.
// @origingame/create-game re-exports them; its bin/brief.ts is a copy of
// this file (written by its scripts/pack-template.ts, held equal by its
// tests), so this file imports nothing but node's own modules.
//
//   bun scripts/brief.ts <kind> "<subject>" [--project <dir> | --look <slug>]
//                        [--reference <path>]... [--metres <n>] [--json]
//   bun scripts/brief.ts audio [--project <dir>] [--music <style> --mood <mood> --genre <genre>]
//                        [--premise "<line>"] [--json]

import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * The kinds of asset a brief is written for: the game's own (sprite to icon),
 * then the UI's art slots (docs/ui.md §13, Appendix F; `UI_KINDS` has each
 * one's spec), the names og-ui's `SlotKind` gives them.
 */
export const ART_KINDS = [
  "sprite", "tileset", "portrait", "backdrop", "prop", "character", "texture", "icon",
  "keyArt", "capsule", "logo", "appIcon", "characterSplash", "fullBody", "bust", "vnSprite", "eventCg", "itemIcon",
  "abilityIcon", "cardArt", "cardFrame", "unitPortrait", "levelThumb", "songJacket", "saveThumb", "menuBackground",
  "menuPattern", "banner", "playerBanner", "emblem", "achievement", "emote", "currency", "mapGlyph", "talentIcon",
  "tutorial", "cosmeticPreview", "styleFrame", "styleButton", "ornament", "iconSet",
] as const;
export type ArtKind = (typeof ART_KINDS)[number];

/** The moods a look takes (`mood "<word>"`, docs/art.md §2.9): og-bundle's MOODS and create's choices, held equal by the tests. */
export const MOODS = ["bright", "dark", "warm", "cold", "soft", "high-contrast"] as const;
export type Mood = (typeof MOODS)[number];

/** How each mood reads in a request: the light and colour the asset is drawn in. */
const MOOD_WORDS: Readonly<Record<Mood, string>> = {
  bright: "high-key light, open shadows and lively, clean colour",
  dark: "low-key light, most of the subject in its shadow tones, muted colour with cool shadows and a few small bright accents",
  warm: "golden light, amber highlights and warm shadows, cool colours only as small accents",
  cold: "cool blue light, blue-grey shadows and slightly drained colour, warm colours only as small accents",
  soft: "gentle low-contrast light, muted or pastel colour and lifted shadows, no harsh highlights",
  "high-contrast": "strong light against deep shadow, a wide range of values and bold, saturated colour",
};

/** How a UI slot fills its rectangle with the art (og-ui's `Fit`). */
export type UiFit = "cover" | "contain" | "fill" | "nineSlice" | "tile";

/** What an art slot of a UI kind is (docs/ui.md §13.1): og-ui's `SlotKind::info`, held equal by og-ui's tests. */
export interface UiKindSpec {
  /** The sentence that opens the prompt; `{subject}` is the subject. */
  readonly about: string;
  /** The aspects it is drawn at as `w:h`, the first the default; empty: free (the art's own). */
  readonly aspects: readonly string[];
  /** Any aspect between these two is also allowed. */
  readonly range?: readonly [string, string];
  readonly transparent: boolean;
  readonly tiling: boolean;
  /** Where the subject sits, 0..1 from the top left: cropping keeps it. */
  readonly focal: readonly [number, number];
  /** Where text or controls lie over it (x0, y0, x1, y1, 0..1): nothing important there; null: none. */
  readonly safe: readonly [number, number, number, number] | null;
  readonly fit: UiFit;
  /** The smallest CSS px size it must read at; 0: no floor. */
  readonly legiblePx: number;
  /** Style art drawn as a nine-slice with declared insets. */
  readonly nineSlice: boolean;
  /** The states drawn, one image each, left to right on one sheet (style buttons). */
  readonly states: readonly string[];
  /** Made by an image generator; false: the engine renders it (a viewport), the brief only describes it. */
  readonly generated: boolean;
  /** Pixel sizes per aspect where a store asks for exact ones (Steamworks, 2024). */
  readonly pixels?: Readonly<Record<string, readonly [number, number]>>;
}

const centre = [0.5, 0.5] as const;
const ui = (about: string, aspects: readonly string[], transparent: boolean, focal: readonly [number, number], fit: UiFit, extra: Partial<UiKindSpec> = {}): UiKindSpec => ({
  about,
  aspects,
  transparent,
  tiling: false,
  focal,
  safe: null,
  fit,
  legiblePx: 0,
  nineSlice: false,
  states: [],
  generated: true,
  ...extra,
});

/** The states a style button is drawn in. */
export const BUTTON_STATES = ["normal", "hover", "pressed", "disabled", "focused"] as const;
/** The cells (columns x rows) a grid inventory's item takes. */
export const ITEM_CELLS: readonly (readonly [number, number])[] = [[1, 1], [1, 2], [2, 3], [2, 4]];

/** Each UI kind's spec (Appendix F of docs/ui.md). */
export const UI_KINDS = {
  keyArt: ui("Key art of {subject}: the game's title image, a full-bleed scene", ["16:9", "21:9", "9:16"], false, [0.66, 0.45], "cover", { safe: [0, 0, 0.4, 1] }),
  capsule: ui("A store capsule of {subject}: the game's artwork for a store listing, its logo laid over it separately", ["92:43", "77:29", "616:353", "187:224", "2:3", "96:31"], false, [0.5, 0.45], "cover", {
    pixels: { "92:43": [920, 430], "77:29": [462, 174], "616:353": [1232, 706], "187:224": [748, 896], "2:3": [600, 900], "96:31": [3840, 1240] },
  }),
  logo: ui("The game's logo: {subject}, lettered as one mark", [], true, centre, "contain"),
  appIcon: ui("An app icon of {subject}: one bold mark filling the square", ["1:1"], false, centre, "cover", { legiblePx: 32 }),
  characterSplash: ui("A character splash of {subject}: the character large and in action, the moment they are unlocked", ["16:9", "9:16"], false, [0.5, 0.3], "cover", { safe: [0, 0.78, 1, 1] }),
  fullBody: ui("A full-body character illustration of {subject}, head to feet, standing", ["9:16", "1:2"], true, [0.5, 0.18], "contain", { range: ["1:2", "9:16"] }),
  bust: ui("A character bust of {subject}, head and shoulders, facing three-quarters", ["1:1", "3:4"], true, [0.5, 0.35], "cover"),
  vnSprite: ui("A visual-novel sprite of {subject}: the character standing, knees up, on a clear background, layered so expressions swap", ["1:2"], true, [0.5, 0.15], "contain"),
  eventCg: ui("An event illustration of {subject}: a story moment in one frame", ["16:9"], false, [0.5, 0.4], "cover", { safe: [0, 0.72, 1, 1] }),
  itemIcon: ui("An inventory item icon of {subject}: the object alone, centred, seen at a three-quarter angle", ["1:1", "1:2", "2:3"], true, centre, "contain", { legiblePx: 32 }),
  abilityIcon: ui("An ability icon of {subject}: one symbol of the effect on a filled ground", ["1:1"], false, centre, "cover", { legiblePx: 24 }),
  cardArt: ui("Card art of {subject}: the picture in a card's art window", ["4:3"], false, [0.5, 0.45], "cover"),
  cardFrame: ui("A card frame for {subject}: the border, name banner and text box of a playing card, its art window and text box left empty", ["5:7"], true, centre, "contain"),
  unitPortrait: ui("A unit portrait of {subject}: the face or the building, filling a square", ["1:1"], false, [0.5, 0.4], "cover", { legiblePx: 32 }),
  levelThumb: ui("A level thumbnail of {subject}: the place at a glance", ["16:9", "4:3"], false, centre, "cover"),
  songJacket: ui("A song jacket of {subject}: album-cover artwork", ["1:1"], false, centre, "cover"),
  saveThumb: ui("A save-slot thumbnail of {subject}: the scene where the game was saved", ["16:9"], false, centre, "cover"),
  menuBackground: ui("A menu background of {subject}: a calm scene behind the game's menus", ["16:9", "21:9"], false, [0.68, 0.45], "cover", { safe: [0, 0, 0.45, 1] }),
  menuPattern: ui("A tiling menu pattern of {subject}: a quiet repeat behind panels", ["1:1"], false, centre, "tile", { tiling: true }),
  banner: ui("An event banner of {subject}: a promotion across the top of a shop or event screen", ["2:1", "16:9", "3:1"], false, [0.68, 0.5], "cover", { range: ["16:9", "2:1"], safe: [0, 0, 0.45, 1] }),
  playerBanner: ui("A player banner of {subject}: a calling card behind a player's name and rank", ["4:1", "5:1", "1:2"], false, [0.75, 0.4], "cover", { range: ["4:1", "5:1"], safe: [0, 0.55, 0.6, 1] }),
  emblem: ui("An emblem of {subject}: a crest or badge, symmetrical, on a clear background", ["1:1"], true, centre, "contain", { legiblePx: 32 }),
  achievement: ui("An achievement icon of {subject}: one symbol of the deed on a filled ground", ["1:1"], false, centre, "cover", { legiblePx: 32 }),
  emote: ui("An emote of {subject}: a sticker, one expression, bold outline, on a clear background", ["1:1"], true, centre, "contain", { legiblePx: 28 }),
  currency: ui("A currency icon of {subject}: one coin, gem or resource, on a clear background", ["1:1"], true, centre, "contain", { legiblePx: 16 }),
  mapGlyph: ui("A map glyph of {subject}: a flat symbol for a point of interest, on a clear background", ["1:1"], true, centre, "contain", { legiblePx: 16 }),
  talentIcon: ui("A talent node icon of {subject}: one symbol on a filled ground", ["1:1"], false, centre, "cover", { legiblePx: 32 }),
  tutorial: ui("A tutorial illustration of {subject}: the action shown plainly", ["16:9", "4:3"], false, centre, "cover"),
  cosmeticPreview: ui("A cosmetic preview of {subject}: the item rendered in the game, three-quarter view", ["1:1", "3:4"], false, [0.5, 0.45], "cover", { generated: false }),
  styleFrame: ui("A UI frame of {subject}: a panel's border and ground, its centre plain so text reads on it", [], true, centre, "nineSlice", { nineSlice: true }),
  styleButton: ui("A UI button of {subject}: its border and face, its centre plain so a label reads on it", [], true, centre, "nineSlice", { nineSlice: true, states: BUTTON_STATES }),
  ornament: ui("A UI ornament of {subject}: a corner, flourish or divider, on a clear background", [], true, centre, "contain"),
  iconSet: ui("A UI icon of {subject} for the style's icon set: one flat symbol in a single colour, on a clear background", ["1:1"], true, centre, "contain", { legiblePx: 24 }),
} as const satisfies Readonly<Record<string, UiKindSpec>>;
export type UiKind = keyof typeof UI_KINDS;

/** The art contract's resources (the IDL's `art` module): the blocks of a look the brief reads. */
export const ART_RESOURCES = ["ArtPalette", "ArtPixels", "ArtTexels", "ArtShape", "ArtLine", "ArtMotion", "ArtCamera"] as const;
export type ArtResource = (typeof ART_RESOURCES)[number];

/**
 * A field's value as the look writes it: a number, `true`/`false`, a word
 * (an enum variant) or a string (a handle, an sRGB `"#rrggbb"`), or a list
 * of numbers (`spriteSizes 16 32`, `accents 3 7`, a linear colour `0.02
 * 0.02 0.03`; an empty list releases a list field).
 */
export type ArtValue = string | number | boolean | readonly number[];

/** The look's art blocks, flattened over its `extends` chain: each field the nearest look sets. */
export type ArtBlocks = { readonly [R in ArtResource]?: Readonly<Record<string, ArtValue>> };

/** One palette colour: a name a person can say and its sRGB hex. */
export interface PaletteColour {
  readonly name: string;
  /** `#rrggbb`, lowercase. */
  readonly hex: string;
}

/** The palette the look's assets sit on, read from its file. */
export interface BriefPalette {
  /** The handle the look names (`@palettes/dusk16`, `palettes/meadow.hex`). */
  readonly id: string;
  /** Which field named it: the art contract's, or (without one) the frame's quantise palette. */
  readonly from: "ArtPalette.colors" | "Stylise.palette";
  /** In file order: an accent's index is its place here. */
  readonly colours: readonly PaletteColour[];
}

/** The fields of a style catalogue entry the brief reads (packages/knowledge/looks/catalogue.json, og-taste §12). */
export interface BriefStyle {
  readonly slug: string;
  readonly name: string;
  readonly descriptor: string;
  /** `previews/<slug>.webp`, against the root of the package carrying the catalogue. */
  readonly preview: string;
  readonly palette: { readonly size: string; readonly rules: string };
  readonly assets: Readonly<Record<string, string>>;
  readonly artBible: { readonly direction: string; readonly avoid: readonly string[] };
  readonly look: {
    readonly profiles: readonly ("2d" | "3d")[];
    /** The mood the preset is drawn in (catalogue schema 3). */
    readonly mood?: Mood;
    /** A `content` style's art that conformance cannot make: each kind and what it must be (catalogue schema 3). */
    readonly authored?: readonly AuthoredArt[];
  };
}

/** Art a `content` style needs made for it (the catalogue's `look.authored`): the brief kind and what the art must be. */
export interface AuthoredArt {
  readonly kind: ArtKind;
  readonly what: string;
}

/** The look a brief is compiled from. */
export interface BriefLook {
  /** The catalogue preset the look's `extends` chain reaches; null for a look that extends no preset. */
  readonly extends: string | null;
  /** The flattened art blocks. */
  readonly art: ArtBlocks;
  /** The project's profile (project.ogt's `profile`): a prop or character of a 2D game is a sprite, of a 3D game a model. Unset: the style's only profile, else 3D. */
  readonly profile?: "2d" | "3d";
  /** The palette `ArtPalette.colors` names (else `Stylise.palette`), read; null or unset when the look names none. */
  readonly palette?: BriefPalette | null;
  /** The catalogue entry of `extends`; null or unset when there is none. */
  readonly style?: BriefStyle | null;
  /** The style's preview as the caller can open it (a path or URL); unset, the entry's `preview`. */
  readonly preview?: string;
  /** The mood the look is drawn in: the one its chain asks of the preset (`mood "<word>"`), else the preset's own; null or unset when neither names one. */
  readonly mood?: Mood | null;
}

export interface BriefOptions {
  /** The game's accepted assets of this style (paths or URLs), passed on as references after the style's preview. */
  readonly references?: readonly string[];
  /** The asset's size in metres: a prop 1, a character 2, a texture's tile 2 by default. It sets the cell of a 2D sprite (ArtPixels.unit per metre) and the texture of a model (ArtTexels.density per metre). */
  readonly metres?: number;
  /** A UI kind's aspect as `w:h`, one of `UI_KINDS[kind].aspects` (or in its range); unset, the first. */
  readonly aspect?: string;
  /** An item icon's cells, columns and rows (one of `ITEM_CELLS`); unset, 1 x 1. */
  readonly cells?: readonly [number, number];
  /** A nine-slice's insets as fractions of the art (left, top, right, bottom); unset, a quarter each. */
  readonly insets?: readonly [number, number, number, number];
}

/** What a UI kind's art must hold to fit its slot (docs/ui.md §13.3). */
export interface UiParams {
  /** The aspect drawn, `w:h`. */
  readonly aspect: string;
  /** Every aspect the kind allows (a range as `lo..hi`). */
  readonly aspects: readonly string[];
  readonly focal: readonly [number, number];
  readonly safe: readonly [number, number, number, number] | null;
  readonly fit: UiFit;
  readonly legiblePx: number;
  /** Style art: the nine-slice insets (fractions) and the states drawn, left to right; null otherwise. */
  readonly nineSlice: { readonly insets: readonly [number, number, number, number]; readonly states: readonly string[] } | null;
  /** An item's cells (columns, rows); null for other kinds. */
  readonly cells: readonly [number, number] | null;
  /** What the cook's conform pass will do to it under this look (docs/art.md §2.3); empty: nothing. */
  readonly conform: readonly string[];
  /** Made by an image generator; false: rendered by the engine. */
  readonly generated: boolean;
}

/** One reference image to hand the generator. */
export interface BriefReference {
  /** `style`: the style's preview; `accepted`: an asset of the game already accepted in the style. */
  readonly role: "style" | "accepted";
  readonly path: string;
}

/** The generator parameters for an image (sprite, tile set, portrait, backdrop, icon, texture; a prop or character of a 2D game). */
export interface ImageParams {
  /** A clear background (sprites, icons, portraits); tile sets, backdrops and textures fill the frame. */
  readonly transparent: boolean;
  /** The size to generate, in pixels: for pixel art the logical grid times `grid.scale`. */
  readonly width: number;
  readonly height: number;
  /** The pixel-art grid: the asset's size in logical pixels and the whole number each is drawn at (the cook's conformance downscales by it); null without one. */
  readonly grid: { readonly width: number; readonly height: number; readonly scale: number } | null;
  /** The image repeats edge to edge (tile sets, textures). */
  readonly tiling: boolean;
}

/** The generator parameters for a model (a prop or character of a 3D game): the remesh's budget and the retexture's size. */
export interface ModelParams {
  /** The most triangles (ArtShape.propFaces or characterFaces); null when the look sets no budget. */
  readonly faceBudget: number | null;
  /** The texture's side in texels: ArtTexels.density times the size, to a power of two, at most ArtTexels.maxSize (1024 when neither is set). */
  readonly textureSize: number;
  /** ArtShape.normals: `any`, `smooth`, `flat` or `angle`. */
  readonly normals: string;
  /** ArtShape.voxel: the cube edge in metres, 0 none. */
  readonly voxel: number;
  /** ArtShape.bevel: the chamfer radius in metres, 0 none. */
  readonly bevel: number;
  /** ArtTexels.filter and mips: `any`, `nearest`/`linear`, `full`/`none`. */
  readonly filter: string;
  readonly mips: string;
  /** The size the texture was worked out for. */
  readonly metres: number;
}

/** The request for one asset. */
export interface ArtBrief {
  readonly kind: ArtKind;
  readonly subject: string;
  /** The catalogue slug, or null. */
  readonly style: string | null;
  /** `image` for a picture generator, `model` for a 3D one. */
  readonly medium: "image" | "model";
  /** The whole request as text, one statement per line. */
  readonly prompt: string;
  /** What the asset must not show (also the prompt's last line). */
  readonly avoid: readonly string[];
  /** The palette colours the prompt names (empty when the look names no palette). */
  readonly palette: readonly PaletteColour[];
  /** The style's preview first, then the accepted assets. */
  readonly references: readonly BriefReference[];
  /** Set when `medium` is `image`. */
  readonly image?: ImageParams;
  /** Set when `medium` is `model`. */
  readonly model?: ModelParams;
  /** Set for a UI kind (`UI_KINDS`). */
  readonly ui?: UiParams;
}

// ---------------------------------------------------------------- colour

/** sRGB bytes of `#rrggbb` (the `#` optional); null when it is not one. */
function rgbOf(hex: string): [number, number, number] | null {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return null;
  const n = parseInt(m[1]!, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

const hex2 = (v: number) => Math.round(Math.min(1, Math.max(0, v)) * 255).toString(16).padStart(2, "0");
const toLinear = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const toSrgb = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);

/** OKLCH of an sRGB colour: lightness 0..1, chroma, hue in degrees. */
function oklch([r, g, b]: [number, number, number]): { l: number; c: number; h: number } {
  const [lr, lg, lb] = [r, g, b].map((v) => toLinear(v / 255)) as [number, number, number];
  const l = Math.cbrt(0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb);
  const m = Math.cbrt(0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb);
  const s = Math.cbrt(0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  return { l: L, c: Math.hypot(A, B), h: ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360 };
}

/** Hue names by OKLCH hue angle: each band runs up to its bound. */
const HUES: readonly [number, string][] = [
  [15, "pink"],
  [45, "red"],
  [70, "orange"],
  [100, "amber"],
  [118, "yellow"],
  [135, "lime"],
  [165, "green"],
  [195, "teal"],
  [225, "cyan"],
  [250, "sky blue"],
  [275, "blue"],
  [295, "indigo"],
  [315, "violet"],
  [335, "purple"],
  [350, "magenta"],
  [360, "pink"],
];

/**
 * A name a person can say for an sRGB colour (`#e88058` "orange", `#100c18`
 * "near-black"): a lightness word and a hue from its OKLCH, greys by
 * lightness with a warm or cool cast, browns and creams by their hue.
 */
export function colourName(hex: string): string {
  const rgb = rgbOf(hex);
  if (!rgb) throw new Error(`'${hex}' is not an sRGB colour #rrggbb`);
  const { l, c, h } = oklch(rgb);
  if (l < 0.2 && c < 0.06) return "near-black";
  if (l > 0.97 && c < 0.02) return "white";
  if (c < 0.03) {
    const cast = c < 0.012 ? "" : h > 30 && h < 120 ? "warm " : h > 180 && h < 300 ? "cool " : "";
    const shade = l < 0.35 ? "dark " : l > 0.8 ? "light " : "";
    return `${shade}${cast}grey`;
  }
  const hue = HUES.find(([bound]) => h < bound)?.[1] ?? "pink";
  if ((hue === "orange" || hue === "amber" || hue === "red") && l < 0.55 && c < 0.16) return l < 0.35 ? "dark brown" : "brown";
  if ((hue === "amber" || hue === "yellow") && l > 0.9 && c < 0.1) return "cream";
  const shade = l < 0.35 ? "dark " : l < 0.48 ? "deep " : l > 0.82 && c < 0.12 ? "pale " : l > 0.72 ? "light " : "";
  const muted = c < 0.07 && shade !== "pale " ? "muted " : "";
  return `${shade}${muted}${hue}`;
}

/** A linear colour (three numbers, as a look may write one) as sRGB `#rrggbb`. */
function linearHex(v: readonly number[]): string {
  return `#${v.slice(0, 3).map((x) => hex2(toSrgb(x))).join("")}`;
}

/**
 * The colours of a palette file, named: `.hex` (one `rrggbb` per line, `#`
 * optional, `;` comments) or GIMP `.gpl` (`r g b name` per colour; its names
 * are kept), as the cook reads them (og-cook `parse_palette`). Names are
 * made unique by a number (`dark purple 2`).
 */
export function parsePalette(file: string, text: string): PaletteColour[] {
  const gpl = file.toLowerCase().endsWith(".gpl");
  const out: { name: string; hex: string }[] = [];
  const lines = text.split(/\r?\n/);
  if (gpl && lines[0]?.trim() !== "GIMP Palette") throw new Error(`${file}: a .gpl palette starts with the line \`GIMP Palette\``);
  for (const [i, raw] of lines.entries()) {
    const line = raw.trim();
    if ((gpl && i === 0) || line === "" || line.startsWith(";") || (gpl && (line.startsWith("#") || /^(Name|Columns):/.test(line)))) continue;
    if (gpl) {
      const m = /^(\d+)\s+(\d+)\s+(\d+)\s*(.*)$/.exec(line);
      if (!m || [m[1], m[2], m[3]].some((v) => Number(v) > 255)) throw new Error(`${file}: line ${i + 1}: a colour is \`r g b [name]\` with 0-255 channels`);
      const hex = `#${[m[1], m[2], m[3]].map((v) => Number(v).toString(16).padStart(2, "0")).join("")}`;
      const own = m[4]!.trim();
      out.push({ name: own && !/^untitled$/i.test(own) ? own.toLowerCase() : colourName(hex), hex });
    } else {
      const h = line.replace(/^#/, "").split(/\s+/)[0]!;
      if (!rgbOf(h)) throw new Error(`${file}: line ${i + 1}: '${line}' is not an rrggbb colour`);
      const hex = `#${h.toLowerCase()}`;
      out.push({ name: colourName(hex), hex });
    }
  }
  if (out.length === 0) throw new Error(`${file}: the palette holds no colours`);
  const seen = new Map<string, number>();
  for (const c of out) seen.set(c.name, (seen.get(c.name) ?? 0) + 1);
  const count = new Map<string, number>();
  return out.map(({ name, hex }) => {
    if (seen.get(name)! < 2) return { name, hex };
    const n = (count.get(name) ?? 0) + 1;
    count.set(name, n);
    return { name: `${name} ${n}`, hex };
  });
}

// ---------------------------------------------------------------- the brief

const num = (v: ArtValue | undefined): number => (typeof v === "number" ? v : 0);
const list = (v: ArtValue | undefined): number[] => (Array.isArray(v) ? [...v] : typeof v === "number" ? [v] : []);
const word = (v: ArtValue | undefined, zero: string): string => (typeof v === "string" && v !== "" ? v : zero);
const bool = (v: ArtValue | undefined): boolean => v === true;
const lower = (s: string) => (/^[A-Z][a-z]/.test(s) ? s[0]!.toLowerCase() + s.slice(1) : s);

/** The smallest power of two at or above `n` (at least 1). */
function pow2(n: number): number {
  let p = 1;
  while (p < n) p *= 2;
  return p;
}

/** The largest power of two at or below `n`. */
function pow2Below(n: number): number {
  let p = 1;
  while (p * 2 <= n) p *= 2;
  return p;
}

/** `n` up to the next of the allowed cell sizes (the largest when it is past them all); `n` itself when none are given. */
function snapUp(n: number, sizes: readonly number[]): number {
  const sorted = [...sizes].sort((a, b) => a - b);
  if (sorted.length === 0) return Math.max(1, Math.ceil(n));
  return sorted.find((s) => s >= n) ?? sorted[sorted.length - 1]!;
}

/** The longest side image generators draw: the pixel-art grid is scaled to the whole number nearest to it. */
const GENERATED = 1024;
const TILESET = { columns: 4, rows: 4 };

const DEFAULT_METRES: Readonly<Partial<Record<ArtKind, number>>> = { sprite: 2, tileset: 1, portrait: 1, backdrop: 1, prop: 1, character: 2, texture: 2, icon: 1 };

/** `w:h` as numbers; null when it is not one. */
function aspectOf(s: string): [number, number] | null {
  const m = /^(\d+):(\d+)$/.exec(s.trim());
  return m && Number(m[1]) > 0 && Number(m[2]) > 0 ? [Number(m[1]), Number(m[2])] : null;
}

const ratio = (s: string) => {
  const [w, h] = aspectOf(s)!;
  return w / h;
};

/** Whether `aspect` is one a UI kind is drawn at (within half a percent, as og-ui's `SlotKind::allows`). */
export function uiAllows(kind: UiKind, aspect: string): boolean {
  const spec: UiKindSpec = UI_KINDS[kind];
  if (!aspectOf(aspect)) return false;
  const r = ratio(aspect);
  if (spec.aspects.length === 0 || spec.aspects.some((a) => Math.abs(ratio(a) / r - 1) < 0.005)) return true;
  if (!spec.range) return false;
  const [lo, hi] = [ratio(spec.range[0]), ratio(spec.range[1])].sort((a, b) => a - b) as [number, number];
  return r >= lo * 0.995 && r <= hi * 1.005;
}

const isUi = (kind: ArtKind): kind is UiKind => kind in UI_KINDS;
const pct = (v: number) => `${Math.round(v * 100)}%`;

/** Which sentence opens the prompt. */
function opening(kind: ArtKind, subject: string, medium: "image" | "model", metres: number): string {
  if (isUi(kind)) return `${UI_KINDS[kind].about.replace("{subject}", subject)}.`;
  switch (kind) {
    case "sprite":
      return `A game sprite of ${subject}.`;
    case "tileset":
      return `A tile set for ${subject}: ${TILESET.columns} x ${TILESET.rows} tiles on one grid, each tile meeting its neighbours edge to edge.`;
    case "portrait":
      return `A character portrait of ${subject}, head and shoulders.`;
    case "backdrop":
      return `A backdrop of ${subject}: a full-frame background layer with no characters in it.`;
    case "prop":
      return medium === "model" ? `A 3D model of ${subject}, a game prop about ${metres} m across, its base on the ground.` : `A game prop sprite of ${subject}.`;
    case "character":
      return medium === "model" ? `A 3D game character of ${subject}, full body in a neutral standing pose, about ${metres} m tall, facing the front.` : `A game character sprite of ${subject}, full body, standing.`;
    case "texture":
      return `A seamless tiling surface texture of ${subject}, seen straight on and evenly lit.`;
    case "icon":
      return `A UI icon of ${subject}: one bold shape that reads at a glance.`;
  }
}

/**
 * The request for one asset of `kind` showing `subject`, in the look's
 * style: pure, the same look giving the same brief. See the module header
 * and README §The art brief for what each field holds.
 */
export function artBrief(look: BriefLook, kind: ArtKind, subject: string, options: BriefOptions = {}): ArtBrief {
  if (!(ART_KINDS as readonly string[]).includes(kind)) throw new Error(`'${kind}' is not an asset kind: use one of ${ART_KINDS.join(", ")}`);
  const what = subject.trim();
  if (what === "") throw new Error("the brief needs a subject: what the asset shows, in a line");
  const style = look.style ?? null;
  const art = look.art;
  const palette = art.ArtPalette ?? {};
  const pixels = art.ArtPixels ?? {};
  const texels = art.ArtTexels ?? {};
  const shape = art.ArtShape ?? {};
  const line = art.ArtLine ?? {};
  const motion = art.ArtMotion ?? {};
  const camera = art.ArtCamera ?? {};
  const profile = look.profile ?? (style && style.look.profiles.length === 1 ? style.look.profiles[0]! : "3d");
  const medium: "image" | "model" = (kind === "prop" || kind === "character") && profile === "3d" ? "model" : "image";
  const metres = options.metres !== undefined && options.metres > 0 ? options.metres : (DEFAULT_METRES[kind] ?? 1);
  const uiSpec: UiKindSpec | null = isUi(kind) ? UI_KINDS[kind] : null;
  const lines: string[] = [opening(kind, what, medium, metres)];
  const avoid: string[] = [...(style?.artBible.avoid ?? [])];

  if (style) lines.push(`Style: ${style.name}. ${style.descriptor} ${style.artBible.direction}`);
  const made = style?.look.authored?.find((a) => a.kind === kind);
  if (made) lines.push(`Authored: this style's ${kind} art is made for it, not something the engine adds: ${made.what}.`);
  const mood = look.mood ?? style?.look.mood ?? null;
  if (mood) lines.push(`Mood: ${mood === "high-contrast" ? "high contrast" : mood}: ${MOOD_WORDS[mood]}. The frame's own grading adds the rest, so keep the colours true to the palette rather than tinted.`);

  // The palette.
  const colours = look.palette?.colours ?? [];
  const strict = bool(palette.strict);
  const said: string[] = [];
  if (colours.length > 0) {
    const named = colours.map((c) => `${c.name} ${c.hex}`).join(", ");
    said.push(strict ? `only these ${colours.length} colours, each pixel exactly one of them: ${named}` : `built on these ${colours.length} colours: ${named}`);
  }
  const maxColors = num(palette.maxColors);
  if (maxColors > 0) said.push(`at most ${maxColors} distinct colours in this asset`);
  const ramp = num(palette.rampSteps);
  if (ramp > 0) said.push(`at most ${ramp} shades per hue from shadow to highlight`);
  const accents = list(palette.accents)
    .map((i) => colours[i])
    .filter((c): c is PaletteColour => c !== undefined);
  if (accents.length > 0) said.push(`${accents.map((c) => `${c.name} ${c.hex}`).join(" and ")} kept for signals (the player, danger, the goal): only where this asset is one`);
  const dither = word(palette.dither, "none");
  if (dither !== "none") said.push(`tones between two colours mixed by a ${dither} dither pattern, never a smooth gradient`);
  else if (colours.length > 0) said.push("tones in flat bands of these colours, no dithering");
  if (style) said.push(`${style.palette.size}; ${lower(style.palette.rules).replace(/\.$/, "")}`);
  if (said.length > 0) lines.push(`Palette: ${said.join("; ")}.`);

  // Sizes: a pixel grid for images, texels for surfaces and models.
  const unit = num(pixels.unit);
  const logicalHeight = num(pixels.logicalHeight);
  const sizes = list(pixels.spriteSizes).filter((s) => s > 0);
  const density = num(texels.density);
  const maxSize = num(texels.maxSize);
  const filter = word(texels.filter, "any");
  const mips = word(texels.mips, "any");
  const textureSide = (m: number) => {
    let side = density > 0 ? pow2(density * m) : maxSize > 0 ? pow2Below(maxSize) : GENERATED;
    if (maxSize > 0) side = Math.min(side, pow2Below(maxSize));
    return Math.max(1, side);
  };
  let image: ImageParams | undefined;
  let model: ModelParams | undefined;
  let uiParams: UiParams | undefined;
  if (medium === "image") {
    const transparent = uiSpec ? uiSpec.transparent : kind === "sprite" || kind === "icon" || kind === "portrait" || kind === "prop" || kind === "character";
    const tiling = uiSpec ? uiSpec.tiling : kind === "tileset" || kind === "texture";
    let grid: { width: number; height: number } | null = null;
    const pixelArt = unit > 0 || sizes.length > 0;
    if (uiSpec) {
      // A UI kind: its slot's aspect, focal point, safe area and (style art) nine-slice.
      const kindName = kind as UiKind;
      const aspect = options.aspect ?? (options.cells ? `${options.cells[0]}:${options.cells[1]}` : uiSpec.aspects[0]) ?? "1:1";
      if (!aspectOf(aspect)) throw new Error(`'${aspect}' is not an aspect: write w:h, such as 16:9`);
      if (!uiAllows(kindName, aspect)) throw new Error(`${kind} art is drawn at ${[...uiSpec.aspects, ...(uiSpec.range ? [`${uiSpec.range[0]}..${uiSpec.range[1]}`] : [])].join(", ")}, not ${aspect}`);
      const cells = kind === "itemIcon" ? (options.cells ?? (aspectOf(aspect) as [number, number])) : null;
      if (cells && !ITEM_CELLS.some(([c, r]) => c === cells[0] && r === cells[1])) throw new Error(`an item is ${ITEM_CELLS.map(([c, r]) => `${c}x${r}`).join(", ")} cells, not ${cells[0]}x${cells[1]}`);
      const insets = options.insets ?? ([0.25, 0.25, 0.25, 0.25] as const);
      if (uiSpec.nineSlice && (insets.some((v) => v <= 0) || insets[0] + insets[2] >= 1 || insets[1] + insets[3] >= 1)) throw new Error("the insets are above 0 and leave a middle (left + right and top + bottom under 1)");
      const states = uiSpec.states.length > 0 ? uiSpec.states : ["normal"];
      const [aw, ah] = aspectOf(aspect)!;
      let w: number;
      let h: number;
      if (pixelArt) {
        const small = sizes.length > 0 ? Math.min(...sizes) : snapUp(unit, sizes);
        const tall = logicalHeight > 0 ? logicalHeight : 180;
        if (uiSpec.nineSlice) [w, h] = [Math.max(...(sizes.length > 0 ? sizes : [small * 3])), Math.max(...(sizes.length > 0 ? sizes : [small * 3]))];
        else if (cells) [w, h] = [small * cells[0], small * cells[1]];
        else if (uiSpec.legiblePx > 0 && aw === ah) [w, h] = [small, small];
        else [w, h] = aw >= ah ? [Math.round((tall * aw) / ah), tall] : [tall, Math.round((tall * ah) / aw)];
        const sheet = uiSpec.nineSlice ? states.length : 1;
        grid = { width: w * sheet, height: h };
        const scale = Math.max(1, Math.round(GENERATED / Math.max(grid.width, grid.height)));
        image = { transparent, width: grid.width * scale, height: grid.height * scale, grid: { ...grid, scale }, tiling };
        lines.push(`Size: pixel art on a ${grid.width} x ${grid.height} logical-pixel grid${sheet > 1 ? ` (${sheet} cells of ${w} x ${h})` : ""}, drawn at ${scale}x: every logical pixel one ${scale} x ${scale} block of one flat colour, hard edges, no anti-aliasing.`);
        avoid.push("anti-aliased or blurred edges", "soft gradients", "pixels of more than one size");
      } else {
        const exact = uiSpec.pixels?.[aspect];
        if (exact) [w, h] = [exact[0], exact[1]];
        else if (uiSpec.nineSlice) [w, h] = [Math.floor(GENERATED / states.length / 8) * 8, Math.floor(GENERATED / states.length / 8) * 8];
        else [w, h] = aw >= ah ? [GENERATED, Math.round((GENERATED * ah) / aw / 8) * 8] : [Math.round((GENERATED * aw) / ah / 8) * 8, GENERATED];
        const sheet = uiSpec.nineSlice ? states.length : 1;
        image = { transparent, width: w * sheet, height: h, grid: null, tiling };
        lines.push(`Size: ${w * sheet} x ${h} pixels${sheet > 1 ? ` (${sheet} cells of ${w} x ${h})` : ""}${exact ? ", the store's exact size" : ""}.`);
      }
      const said: string[] = [`${aspect}`];
      const others = uiSpec.aspects.filter((a) => a !== aspect);
      if (others.length > 0) said.push(`the slot also takes ${others.join(", ")}`);
      lines.push(`Aspect: ${said.join("; ")}${cells ? `; ${cells[0]} x ${cells[1]} inventory cells` : ""}.`);
      if (uiSpec.fit === "cover") lines.push(`Focal point: the subject at ${pct(uiSpec.focal[0])} across and ${pct(uiSpec.focal[1])} down; the slot crops around it, so keep it whole within the middle of the frame.`);
      if (uiSpec.safe) {
        const [x0, y0, x1, y1] = uiSpec.safe;
        lines.push(`Safe area: from ${pct(x0)} to ${pct(x1)} across and ${pct(y0)} to ${pct(y1)} down, text or controls lie over the art: keep it calm and free of the subject and of detail there.`);
      }
      if (uiSpec.legiblePx > 0) lines.push(`Legibility: it must still read at ${uiSpec.legiblePx} x ${uiSpec.legiblePx} px: one clear silhouette, no fine detail, strong value contrast.`);
      if (uiSpec.nineSlice) {
        const [l, t, r, b] = insets;
        lines.push(`Nine-slice: the corners (${pct(l)} left, ${pct(t)} top, ${pct(r)} right, ${pct(b)} bottom of each cell) keep their size, the edges between them stretch along one axis and the centre both ways: put every ornament in the corners, keep the edges even along their length and the centre plain.`);
        if (uiSpec.states.length > 0) lines.push(`States: one cell each, left to right: ${uiSpec.states.join(", ")}; the same silhouette in each, the hover lighter, the pressed darker and pushed in, the disabled flat and muted, the focused with a clear ring.`);
      }
      if (!uiSpec.generated) lines.push("Rendered: the engine draws this from a viewport of the game's own model; this brief describes the framing and lighting for it.");
      if (kind === "logo") {
        lines.push("Lettering: the game's title, the only text in the image, as one designed mark.");
      }
      const conform: string[] = [];
      const colours0 = look.palette?.colours ?? [];
      if (colours0.length > 0 && bool(palette.strict)) conform.push(`every texel mapped to the nearest of the palette's ${colours0.length} colours${word(palette.dither, "none") !== "none" ? ` with a ${word(palette.dither, "none")} dither` : ""}`);
      if (pixelArt) conform.push(`resampled onto the ${unit > 0 ? `${unit} px per metre ` : ""}pixel grid with a dominant-colour downscale, alpha cut to on or off`);
      if (num(line.width) > 0 && transparent) conform.push(`the look's ${num(line.width)}-pixel outline drawn round the shape`);
      if (conform.length > 0) lines.push(`Conform: the cook then ${conform.join("; ")} (docs/art.md §2.3): draw for that.`);
      uiParams = {
        aspect,
        aspects: [...uiSpec.aspects, ...(uiSpec.range ? [`${uiSpec.range[0]}..${uiSpec.range[1]}`] : [])],
        focal: uiSpec.focal,
        safe: uiSpec.safe,
        fit: uiSpec.fit,
        legiblePx: uiSpec.legiblePx,
        nineSlice: uiSpec.nineSlice ? { insets, states } : null,
        cells,
        conform,
        generated: uiSpec.generated,
      };
    } else if (kind === "texture" && density > 0) {
      grid = null; // a 3D surface: texels, below
    } else if (kind === "backdrop" && logicalHeight > 0) {
      grid = { width: Math.round((logicalHeight * 16) / 9), height: logicalHeight };
    } else if (pixelArt) {
      const cell = (m: number) => (unit > 0 ? snapUp(unit * m, sizes) : Math.max(...sizes));
      switch (kind) {
        case "sprite":
          grid = { width: cell(metres), height: cell(metres) };
          break;
        case "character": {
          const h = unit > 0 ? snapUp(unit * metres, sizes) : Math.max(...sizes);
          grid = { width: unit > 0 ? snapUp((unit * metres) / 2, sizes) : h, height: h };
          break;
        }
        case "prop":
          grid = unit > 0 ? { width: cell(metres), height: cell(metres) } : { width: Math.min(...sizes), height: Math.min(...sizes) };
          break;
        case "icon": {
          const s = sizes.length > 0 ? Math.min(...sizes) : snapUp(unit, sizes);
          grid = { width: s, height: s };
          break;
        }
        case "portrait": {
          const s = 2 * (sizes.length > 0 ? Math.max(...sizes) : snapUp(unit * 2, sizes));
          grid = { width: s, height: s };
          break;
        }
        case "tileset": {
          const t = unit > 0 ? snapUp(unit * metres, sizes) : Math.min(...sizes);
          grid = { width: t * TILESET.columns, height: t * TILESET.rows };
          break;
        }
        case "texture": {
          const t = unit > 0 ? Math.round(unit * metres) : Math.max(...sizes);
          grid = { width: t, height: t };
          break;
        }
        case "backdrop":
          break;
      }
    }
    if (uiSpec) {
      // Written above.
    } else if (grid) {
      const scale = Math.max(1, Math.round(GENERATED / Math.max(grid.width, grid.height)));
      image = { transparent, width: grid.width * scale, height: grid.height * scale, grid: { ...grid, scale }, tiling };
      const per = unit > 0 ? ` (${unit} pixels per metre${kind === "tileset" ? `, each tile ${grid.width / TILESET.columns} x ${grid.height / TILESET.rows}` : ""})` : kind === "tileset" ? ` (each tile ${grid.width / TILESET.columns} x ${grid.height / TILESET.rows})` : "";
      const screen = logicalHeight > 0 ? `, for a screen ${logicalHeight} logical pixels tall` : "";
      lines.push(
        `Size: pixel art on a ${grid.width} x ${grid.height} logical-pixel grid${per}${screen}, drawn at ${scale}x: every logical pixel one ${scale} x ${scale} block of one flat colour, hard edges, no anti-aliasing.`,
      );
      if (sizes.length > 0 && (kind === "sprite" || kind === "prop" || kind === "character")) lines.push(`Cells: each side of a sprite cell is one of ${sizes.join(", ")} logical pixels.`);
      avoid.push("anti-aliased or blurred edges", "soft gradients", "pixels of more than one size");
    } else if (kind === "texture") {
      const side = textureSide(metres);
      // Nearest-sampled texels are pixel art: drawn at a whole multiple, as a sprite's grid. Others are drawn at the generator's size and resampled by the cook.
      const scale = filter === "nearest" ? Math.max(1, Math.round(GENERATED / side)) : 1;
      const drawn = filter === "nearest" ? side * scale : Math.max(side, GENERATED);
      image = { transparent, width: drawn, height: drawn, grid: filter === "nearest" ? { width: side, height: side, scale } : null, tiling };
      const crisp = filter === "nearest" ? `, sampled nearest: drawn at ${scale}x, every texel one ${scale} x ${scale} block of one flat colour` : filter === "linear" ? ", sampled smoothly" : "";
      const at = drawn !== side && filter !== "nearest" ? `, drawn at ${drawn} x ${drawn} and resampled` : "";
      lines.push(`Size: ${side} x ${side} texels${density > 0 ? ` covering ${metres} m (${density} texels per metre)` : ""}${crisp}${at}${mips === "none" ? ", no mip maps: as crisp far away as near" : ""}.`);
      if (filter === "nearest") avoid.push("fine detail smaller than one texel");
    } else {
      const [width, height] = kind === "backdrop" ? [1536, 864] : kind === "portrait" ? [1024, 1024] : [GENERATED, GENERATED];
      image = { transparent, width, height, grid: null, tiling };
      const spec = style ? [style.assets.logicalResolution && `resolution ${style.assets.logicalResolution}`, style.assets.spriteScale && `scale ${style.assets.spriteScale}`].filter(Boolean).join("; ") : "";
      lines.push(`Size: ${width} x ${height} pixels${spec ? ` (${spec})` : ""}.`);
    }
    if (transparent) {
      lines.push(`Background: fully transparent, nothing outside the ${kind === "icon" ? "icon" : "subject"}.`);
      avoid.push("a background or ground shadow");
    }
    if (tiling) lines.push("Tiling: the left edge continues the right and the top the bottom, with no seam.");
  } else {
    const budget = num(kind === "character" ? shape.characterFaces : shape.propFaces);
    model = {
      faceBudget: budget > 0 ? budget : null,
      textureSize: textureSide(metres),
      normals: word(shape.normals, "any"),
      voxel: num(shape.voxel),
      bevel: num(shape.bevel),
      filter,
      mips,
      metres,
    };
    const said3: string[] = [];
    if (model.faceBudget !== null) said3.push(`at most ${model.faceBudget} triangles`);
    else if (style?.assets.faceBudget) said3.push(style.assets.faceBudget);
    if (model.normals === "flat") said3.push("flat-shaded facets, every face one colour with a hard edge to its neighbour");
    else if (model.normals === "smooth") said3.push("smooth soft forms with no hard creases");
    else if (model.normals === "angle") said3.push(`hard edges where faces meet past ${num(shape.normalAngle) || 30} degrees, curves smooth`);
    if (model.voxel > 0) said3.push(`built of ${model.voxel} m cubes on one grid, each cube one colour`);
    if (model.bevel > 0) said3.push(`every hard edge rounded by a ${model.bevel} m chamfer`);
    const silhouette = num(shape.silhouette);
    if (silhouette > 0) said3.push(`no feature thinner than ${silhouette} m`);
    if (said3.length > 0) lines.push(`Shape: ${said3.join("; ")}.`);
    const tex = `${model.textureSize} x ${model.textureSize} texels${density > 0 ? ` (${density} per metre)` : ""}${filter === "nearest" ? ", sampled nearest" : ""}${mips === "none" ? ", no mip maps" : ""}`;
    lines.push(`Texture: ${tex}.`);
    if (model.normals === "flat" || model.voxel > 0) avoid.push("smooth shading across facets");
  }

  // The line.
  const width = num(line.width);
  if (width > 0) {
    const colour = typeof line.color === "string" && rgbOf(line.color) ? line.color.toLowerCase() : Array.isArray(line.color) ? linearHex(line.color) : linearHex([0.02, 0.02, 0.03]);
    const named = `${colourName(colour)} ${colour}`;
    if (medium === "image" && image?.grid) {
      const place = word(line.place, "outside");
      const where = place === "inside" ? "on the shape's own edge pixels" : place === "both" ? "centred on the shape's edge" : "round the outside of the shape";
      const brush = word(line.brush, "diamond");
      const corners = brush === "square" ? "corners filled" : brush === "circle" ? "rounded corners" : "corners left open";
      if (kind === "backdrop" || kind === "tileset" || kind === "texture" || (uiSpec !== null && !uiSpec.transparent)) lines.push(`Line: shapes standing in front carry a ${width}-pixel ${named} line ${where}; a tile's edge that meets the next tile carries none.`);
      else lines.push(`Line: a ${width}-pixel ${named} line ${where} (${corners}).`);
    } else if (medium === "model") {
      lines.push(`Line: the engine draws a ${width}-pixel ${named} outline round the model; paint none into the texture.`);
      avoid.push("outlines painted into the texture");
    } else lines.push(`Line: a ${named} line about ${width} pixels wide at the size it is seen.`);
  } else if (style?.assets.outlinePx) {
    const spec = style.assets.outlinePx.trim();
    const none = /^0(\s|$)/.exec(spec);
    lines.push(none ? `Line: none${spec.length > 1 ? ` ${spec.slice(1).trim()}` : ", no outline"}.` : `Line: ${spec}.`);
  }

  // The shape language of a 3D style seen in an image, motion and the camera.
  if (medium === "image" && (kind === "texture" || kind === "backdrop" || kind === "icon") && num(shape.voxel) > 0) lines.push(`Shape: forms read as ${num(shape.voxel)} m cubes.`);
  if (medium === "image" && (kind === "texture" || kind === "backdrop") && word(shape.normals, "any") === "flat") lines.push("Shape: surfaces read as flat facets, each one colour.");
  const stepFps = num(motion.stepFps);
  const maxFrames = num(motion.maxFrames);
  if ((kind === "sprite" || kind === "character") && (stepFps > 0 || maxFrames > 0)) {
    lines.push(`Motion: if animated, ${[stepFps > 0 && `each pose held at ${stepFps} frames a second`, maxFrames > 0 && `at most ${maxFrames} frames a clip`].filter(Boolean).join(", ")}.`);
  }
  if (kind !== "texture" && kind !== "icon" && kind !== "portrait" && !uiSpec) {
    const seen: string[] = [];
    if (bool(camera.fixedAngle)) {
      const pitch = num(camera.pitch);
      const yaw = num(camera.yaw);
      seen.push(pitch === 0 ? "side-on, level with the subject" : pitch === 90 ? "straight down from above" : `from ${pitch} degrees above${yaw !== 0 ? `, turned ${yaw} degrees` : ""}`);
    }
    if (word(camera.projection, "any") === "orthographic" && medium === "image") seen.push("orthographic, no perspective");
    if (seen.length > 0) lines.push(`View: ${seen.join(", ")}.`);
  }

  // References and what to avoid.
  const references: BriefReference[] = [];
  const preview = look.preview ?? style?.preview;
  if (preview) references.push({ role: "style", path: preview });
  for (const r of options.references ?? []) references.push({ role: "accepted", path: r });
  if (references.length > 0) {
    lines.push(
      references.length === 1
        ? "Reference: the image shows the style; match its palette, line and rendering, not its content."
        : "References: the first image shows the style; the others are this game's accepted assets: match their palette, line, proportions and rendering, not their content.",
    );
  }
  avoid.push(kind === "logo" ? "watermarks or any text but the title" : "text, letters or watermarks");
  lines.push(`Avoid: ${avoid.join("; ")}.`);

  return {
    kind,
    subject: what,
    style: style?.slug ?? look.extends,
    medium,
    prompt: lines.join("\n"),
    avoid,
    palette: colours,
    references,
    ...(image ? { image } : {}),
    ...(model ? { model } : {}),
    ...(uiParams ? { ui: uiParams } : {}),
  };
}

// ---------------------------------------------------------------- reading a look

const HERE = dirname(fileURLToPath(import.meta.url));
/** The package root: packages/knowledge (or the engine package's knowledge/), or @origingame/create-game. */
const PKG = join(HERE, "..");

/** The first of `candidates` that exists. */
function first(candidates: readonly string[]): string | undefined {
  return candidates.find((c) => existsSync(c));
}

/** Where this copy finds the presets (templates/looks in the engine repository, presets/ in a published package). */
export function presetsDir(): string {
  const dir = first([join(PKG, "..", "..", "templates", "looks"), join(PKG, "presets")]);
  if (!dir) throw new Error(`no look presets next to ${PKG}: expected ../../templates/looks (engine repository) or presets/ (published package)`);
  return dir;
}

/** The style catalogue this copy reads. */
export function catalogueFile(): string {
  const file = first([join(PKG, "looks", "catalogue.json"), join(PKG, "..", "knowledge", "looks", "catalogue.json")]);
  if (!file) throw new Error(`no style catalogue next to ${PKG}: expected looks/catalogue.json`);
  return file;
}

/** A look file as written: its `extends` and its blocks (every block, art or not), fields as values. */
interface LookText {
  readonly extends: string | null;
  /** The look's `mood` line, or null. */
  readonly mood: Mood | null;
  readonly blocks: Map<string, Map<string, ArtValue>>;
}

const LIST_FIELDS = new Set(["spriteSizes", "accents"]);

function valueOf(field: string, tokens: string[]): ArtValue {
  const parsed = tokens.map((t) => (t.startsWith('"') ? JSON.parse(t) : t)) as string[];
  const numbers = tokens.every((t) => /^-?\d+(\.\d+)?(e-?\d+)?$/i.test(t));
  if (LIST_FIELDS.has(field) || tokens.length !== 1) {
    if (numbers) return tokens.map(Number);
    return parsed.join(" ");
  }
  const t = tokens[0]!;
  if (numbers) return Number(t);
  if (t === "true" || t === "false") return t === "true";
  return parsed[0]!;
}

/** Parse a `type look` .ogt (og-cook's grammar: top-level `extends`, blocks of `field value...`). */
export function parseLook(text: string, at = "look"): LookText {
  const lines = text.split(/\r?\n/);
  const sep = lines.findIndex((l) => l.trim() === "---");
  if (sep < 0 || !lines.slice(0, sep).some((l) => l.trim() === "type look")) throw new Error(`${at}: not an .ogt of type look (\`type look\`, \`version 1\`, \`---\`)`);
  let ext: string | null = null;
  let mood: Mood | null = null;
  const blocks = new Map<string, Map<string, ArtValue>>();
  let block: Map<string, ArtValue> | null = null;
  for (const [i, raw] of lines.slice(sep + 1).entries()) {
    const tokens: string[] = [];
    for (const m of raw.matchAll(/"(?:[^"\\]|\\.)*"|\S+/g)) {
      if (m[0].startsWith("#")) break;
      tokens.push(m[0]);
    }
    if (tokens.length === 0) continue;
    if (tokens.length === 1 && tokens[0] === "}") {
      block = null;
      continue;
    }
    if (tokens[tokens.length - 1] === "{") {
      const name = tokens[0]!;
      block = blocks.get(name) ?? new Map();
      blocks.set(name, block);
      continue;
    }
    const [key, ...rest] = tokens;
    if (block) block.set(key!, valueOf(key!, rest));
    else if (key === "extends") {
      if (rest.length !== 1 || !rest[0]!.startsWith('"')) throw new Error(`${at}: line ${sep + 2 + i}: 'extends' takes a preset's slug or a look's asset id, as a string`);
      ext = JSON.parse(rest[0]!) as string;
    } else if (key === "mood") {
      const m = rest.length === 1 && rest[0]!.startsWith('"') ? (JSON.parse(rest[0]!) as string) : "";
      if (!(MOODS as readonly string[]).includes(m)) throw new Error(`${at}: line ${sep + 2 + i}: 'mood' is one of ${MOODS.join(", ")}`);
      mood = m as Mood;
    }
  }
  return { extends: ext, mood, blocks };
}

/** An asset id as the cook keys it: `assets/` dropped. */
const assetId = (p: string) => p.replace(/^\.?\/?assets\//, "");

/**
 * The mood's layer for a preset (og-cook's `lay_mood`, docs/art.md §2.9):
 * `moods/<mood>/<slug>.ogt` beside the presets, else `moods/<mood>.ogt`. The
 * brief reads only its palette and art fields, which it lays by replacing;
 * the frame's numbers it moves are the cook's.
 */
function moodLayer(mood: Mood, slug: string, presets: string): LookText | null {
  const path = first([join(presets, "moods", mood, `${slug}.ogt`), join(presets, "moods", `${mood}.ogt`)]);
  return path ? parseLook(readFileSync(path, "utf8"), path) : null;
}

/** The flattened look: blocks merged over the chain (nearest wins, field by field), the preset the chain reaches, and the mood it is drawn in (the nearest look's that names one, else the preset's own). */
function flatten(id: string, text: string, assets: string | null, presets: string, ask: Mood | null = null): { blocks: Map<string, Map<string, ArtValue>>; preset: string | null; mood: Mood | null } {
  const seen = [id];
  const chain: LookText[] = [parseLook(text, id)];
  let preset: string | null = null;
  let asked: Mood | null = ask ?? chain[0]!.mood;
  let own: Mood | null = null;
  let from = id;
  for (let next = chain[0]!.extends; next !== null; ) {
    if (seen.length > 9) throw new Error(`${id}: extends chains at most 8 looks deep`);
    let base: { id: string; text: string; preset: boolean };
    if (next.includes("/") || next.endsWith(".ogt")) {
      const bid = assetId(next);
      const path = assets ? join(assets, bid) : "";
      if (!assets || !existsSync(path)) throw new Error(`${from}: extends "${next}": there is no assets/${bid}`);
      base = { id: bid, text: readFileSync(path, "utf8"), preset: false };
    } else {
      const own = `looks/${next}.ogt`;
      const ownPath = assets ? join(assets, own) : "";
      const presetPath = join(presets, `${next}.ogt`);
      if (assets && own !== from && existsSync(ownPath)) base = { id: own, text: readFileSync(ownPath, "utf8"), preset: false };
      else if (existsSync(presetPath)) base = { id: next, text: readFileSync(presetPath, "utf8"), preset: true };
      else throw new Error(`${from}: extends "${next}": neither assets/${own} nor a built-in preset`);
    }
    if (seen.includes(base.id)) throw new Error(`extends goes round in a circle: ${[...seen, base.id].join(" → ")}`);
    seen.push(base.id);
    const parsed = parseLook(base.text, base.id);
    if (base.preset && preset === null) {
      preset = base.id;
      own = parsed.mood;
    } else if (preset === null) asked ??= parsed.mood;
    chain.push(parsed);
    from = base.id;
    next = parsed.extends;
  }
  // The mood's layer lies over the preset, under the looks that extend it.
  if (preset !== null && asked !== null && asked !== own) {
    const layer = moodLayer(asked, preset, presets);
    if (layer) chain.splice(chain.findIndex((l, i) => i > 0 && seen[i] === preset), 0, { ...layer, blocks: stringFields(layer.blocks) });
  }
  const blocks = new Map<string, Map<string, ArtValue>>();
  for (const look of chain.reverse()) {
    for (const [name, fields] of look.blocks) {
      const merged = blocks.get(name) ?? new Map();
      for (const [k, v] of fields) merged.set(k, v);
      blocks.set(name, merged);
    }
  }
  return { blocks, preset, mood: asked ?? own };
}

/** A layer's fields the brief lays by replacing: its palettes and words, not the frame's numbers it moves. */
function stringFields(blocks: Map<string, Map<string, ArtValue>>): Map<string, Map<string, ArtValue>> {
  return new Map([...blocks].map(([name, fields]) => [name, new Map([...fields].filter(([, v]) => typeof v === "string"))]));
}

/** The palette a handle names: a built-in `@palettes/<name>` or a file under the game's assets/. */
function readPalette(handle: string, from: BriefPalette["from"], assets: string | null, presets: string): BriefPalette {
  let path: string | undefined;
  if (handle.startsWith("@palettes/")) {
    const name = handle.slice("@palettes/".length);
    const dir = join(presets, "assets", "palettes");
    path = first(/\.(hex|gpl)$/i.test(name) ? [join(dir, name)] : [join(dir, `${name}.hex`), join(dir, `${name}.gpl`)]);
    if (!path) throw new Error(`"${handle}" is not one of the engine's built-in palettes (${dir})`);
  } else {
    path = assets ? join(assets, assetId(handle)) : undefined;
    if (!path || !existsSync(path)) throw new Error(`the look's palette "${handle}" is not under assets/`);
  }
  return { id: handle, from, colours: parsePalette(path, readFileSync(path, "utf8")) };
}

function readStyle(slug: string | null, catalogue: string): { style: BriefStyle | null; preview?: string } {
  if (slug === null) return { style: null };
  const styles = (JSON.parse(readFileSync(catalogue, "utf8")) as { styles: BriefStyle[] }).styles;
  const style = styles.find((s) => s.slug === slug) ?? null;
  if (!style) return { style: null };
  const root = dirname(dirname(catalogue));
  const preview = first([join(root, style.preview), join(PKG, style.preview), join(PKG, "..", "knowledge", style.preview)]);
  return { style, ...(preview ? { preview } : {}) };
}

/** The flattened look as the brief reads it. */
function briefLook(blocks: Map<string, Map<string, ArtValue>>, preset: string | null, mood: Mood | null, assets: string | null, presets: string, catalogue: string, profile: "2d" | "3d" | undefined): BriefLook {
  const art: Record<string, Record<string, ArtValue>> = {};
  for (const r of ART_RESOURCES) {
    const fields = blocks.get(r);
    if (fields && fields.size > 0) art[r] = Object.fromEntries(fields);
  }
  const own = blocks.get("ArtPalette")?.get("colors");
  const frame = blocks.get("Stylise")?.get("palette");
  const palette =
    typeof own === "string" && own !== "" ? readPalette(own, "ArtPalette.colors", assets, presets) : typeof frame === "string" && frame !== "" ? readPalette(frame, "Stylise.palette", assets, presets) : null;
  const { style, preview } = readStyle(preset, catalogue);
  return { extends: preset, art: art as ArtBlocks, palette, style, mood: mood ?? style?.look.mood ?? null, ...(profile ? { profile } : {}), ...(preview ? { preview } : {}) };
}

export interface ReadLookOptions {
  /** The style catalogue; this copy's own when omitted. */
  readonly catalogue?: string;
  /** The presets' directory; this copy's own when omitted. */
  readonly presets?: string;
  /** For `presetLook`: the mood a game would ask of the preset; unset, its own. */
  readonly mood?: Mood;
}

/**
 * A project's look, flattened: project.ogt's `look "looks/<id>.ogt"` (the
 * look whose art contract the cook reads), its `extends` chain through the
 * game's own looks down to the preset (templates/looks in the engine
 * repository, presets/ in the published package), the palette it names, the
 * catalogue entry and preview of its preset, and project.ogt's `profile`.
 * Throws with the fix when project.ogt names no look.
 */
export function readLook(projectDir: string, options: ReadLookOptions = {}): BriefLook {
  const dir = resolve(projectDir);
  const manifest = join(dir, "project.ogt");
  if (!existsSync(manifest)) throw new Error(`${dir}: no project.ogt`);
  const project = readFileSync(manifest, "utf8");
  const look = /^look[ \t]+"([^"]+)"/m.exec(project)?.[1];
  if (!look) throw new Error(`${manifest}: no \`look "looks/<id>.ogt"\` line; name the game's look there (og-taste §12), as create writes it`);
  const profile = /^profile[ \t]+"(2d|3d)"/m.exec(project)?.[1] as "2d" | "3d" | undefined;
  const assets = join(dir, "assets");
  const id = assetId(look);
  const path = join(assets, id);
  if (!existsSync(path)) throw new Error(`${manifest}: look "${look}": there is no assets/${id}`);
  const presets = options.presets ?? presetsDir();
  const { blocks, preset, mood } = flatten(id, readFileSync(path, "utf8"), assets, presets);
  return briefLook(blocks, preset, mood, assets, presets, options.catalogue ?? catalogueFile(), profile ?? "3d");
}

/** A preset's look alone (`presetLook("16bit-pixel")`), for a game not written yet, in its own mood or the one `options.mood` asks; its profile is the style's only one, else unset. */
export function presetLook(slug: string, options: ReadLookOptions = {}): BriefLook {
  const presets = options.presets ?? presetsDir();
  const path = join(presets, `${slug}.ogt`);
  if (!existsSync(path)) throw new Error(`no look preset '${slug}' in ${presets}`);
  if (options.mood !== undefined && !(MOODS as readonly string[]).includes(options.mood)) throw new Error(`mood '${options.mood}': one of ${MOODS.join(", ")}`);
  // The preset as a game's look extending it would read it: the mood asked lies over it.
  const asked = `type look\nversion 1\n---\nextends ${JSON.stringify(slug)}\n${options.mood ? `mood ${JSON.stringify(options.mood)}\n` : ""}`;
  const { blocks, mood } = flatten("<preset>", asked, null, presets);
  const look = briefLook(blocks, slug, mood, null, presets, options.catalogue ?? catalogueFile(), undefined);
  const profiles = look.style?.look.profiles ?? [];
  return profiles.length === 1 ? { ...look, profile: profiles[0]! } : look;
}

/**
 * The art a `content` style needs made for it (the catalogue's
 * `look.authored`): each kind with what the art must be. Request each one
 * the game needs through `artBrief` with that kind, whose prompt then asks
 * for exactly that art; empty for a style whose look the engine draws whole.
 */
export function authoredArt(look: BriefLook): readonly AuthoredArt[] {
  return look.style?.look.authored ?? [];
}

// ---------------------------------------------------------------- the command line

const USAGE = `usage: bun brief.ts <kind> "<subject>" [--project <dir> | --look <slug> [--mood <mood>]] [--reference <path>]... [--metres <n>] [--aspect <w:h>] [--json]

Prints the request for one asset in the game's style (docs/art.md §2.2): the
prompt, then the references and the generator's parameters.
  kind         ${ART_KINDS.join(", ")}
  --project    the game whose project.ogt names its look (default: .)
  --look       a catalogue preset instead of a project
  --mood       with --look, the mood a game would ask of it (${MOODS.join(", ")})
  --reference  an accepted asset of the game to pass as a reference (repeat)
  --metres     the asset's size in metres (a prop 1, a character 2)
  --aspect     a UI kind's aspect, w:h (one its slot allows)
  --json       the whole brief as JSON

audio [--project <dir>] [--music --mood --genre] [--premise "<line>"] [--json]
prints audioBrief's requests: every music cue and sound effect the game needs
(the create block's music, mood and genre unless all three are given).`;

function audioMain(argv: string[]): number {
  let project = ".";
  const asked: Record<string, string> = {};
  let json = false;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    if (a === "--json") json = true;
    else if (["--project", "--music", "--mood", "--genre", "--premise"].includes(a) && argv[i + 1] !== undefined) {
      if (a === "--project") project = argv[++i]!;
      else asked[a.slice(2)] = argv[++i]!;
    } else throw new Error(`unknown option ${a}. ${USAGE}`);
  }
  const from = asked.music && asked.mood && asked.genre ? {} : readAudioChoices(project);
  const brief = audioBrief({ ...from, ...asked } as AudioBriefInput);
  if (json) console.log(JSON.stringify(brief, null, 2));
  else {
    for (const c of brief.cues) console.log(`# music ${c.id} -> assets/${c.file}\n${c.prompt}\n`);
    for (const e of brief.sfx) console.log(`# sfx ${e.id} -> assets/${e.file}${e.preset ? ` (until then assets/audio/${e.preset}.ogt)` : ""}\n${e.prompt}\n`);
  }
  return 0;
}

function main(argv: string[]): number {
  if (argv[0] === "audio") return audioMain(argv.slice(1));
  const [kind, subject, ...rest] = argv;
  if (!kind || kind === "-h" || kind === "--help" || subject === undefined) {
    console.log(USAGE);
    return kind ? 0 : 1;
  }
  let project = ".";
  let preset: string | undefined;
  let mood: Mood | undefined;
  let metres: number | undefined;
  let aspect: string | undefined;
  let json = false;
  const references: string[] = [];
  for (let i = 0; i < rest.length; i++) {
    const a = rest[i]!;
    const value = () => {
      const v = rest[++i];
      if (v === undefined) throw new Error(`${a} needs a value. ${USAGE}`);
      return v;
    };
    if (a === "--project") project = value();
    else if (a === "--look") preset = value();
    else if (a === "--mood") mood = value() as Mood;
    else if (a === "--reference") references.push(value());
    else if (a === "--metres") metres = Number(value());
    else if (a === "--aspect") aspect = value();
    else if (a === "--json") json = true;
    else throw new Error(`unknown option ${a}. ${USAGE}`);
  }
  const look = preset !== undefined ? presetLook(preset, mood !== undefined ? { mood } : {}) : readLook(project);
  const brief = artBrief(look, kind as ArtKind, subject, { references, ...(metres !== undefined ? { metres } : {}), ...(aspect !== undefined ? { aspect } : {}) });
  if (json) console.log(JSON.stringify(brief, null, 2));
  else {
    const params = brief.image ?? brief.model;
    console.log(`${brief.prompt}\n\n${brief.medium}: ${JSON.stringify(params)}${brief.ui ? `\nui: ${JSON.stringify(brief.ui)}` : ""}\nreferences:${brief.references.map((r) => `\n  ${r.role}: ${r.path}`).join("") || " none"}`);
  }
  return 0;
}


// ------------------------------------------------------- a style's art

/**
 * The art a generated UI style asks for: `art-requests.json`, written by
 * `og style new` beside its `style.ogt` (og-ui's `style_gen::art_requests_json`,
 * docs/ui.md §12.5 and §13.3).
 */
export interface StyleArtRequests {
  /** The style's slug. */
  readonly style: string;
  /** The look family it was generated for (og-taste §12: `ornate fantasy`). */
  readonly look: string;
  readonly premise: string;
  readonly choices: {
    readonly scheme: string;
    readonly seed: string;
    readonly shape: string;
    readonly ornament: string;
    readonly motion: string;
    readonly finish: string;
  };
  /** The style's palette roles the art must sit in (`#rrggbb`). */
  readonly palette: Readonly<Record<string, string>>;
  /** Each slot's spec, as og-ui's `slot_json` writes it. */
  readonly slots: readonly { readonly name: string; readonly kind: string; readonly insets?: readonly number[] | null }[];
}

/** One slot of a generated style and the brief that fills it. */
export interface StyleArtBrief {
  /** The slot's name (`gilded/panel`). */
  readonly slot: string;
  readonly brief: ArtBrief;
}

/** How each shape language reads in a prompt. */
const SHAPE_WORDS: Readonly<Record<string, string>> = {
  soft: "gently rounded corners",
  round: "chunky rounded corners",
  pill: "capsule shapes, fully rounded ends",
  squircle: "continuous squircle corners",
  sharp: "hard square corners",
  cut: "45-degree cut corners",
  notch: "square notched corners",
  pixel: "square pixel corners on a hard pixel grid",
};

/** How each finish reads in a prompt. */
const FINISH_WORDS: Readonly<Record<string, string>> = {
  flat: "flat fills and hairline outlines",
  gradient: "lit top-to-bottom gradients, a thick outline and a hard drop shadow",
  glass: "frosted translucent glass with a faint light edge",
  hologram: "glowing hairlines and a faint holographic inner glow",
  neon: "glowing neon tubes on a dark ground",
  paper: "paper sheets with ink outlines and soft shadows",
  bevel: "hard bevels, light top-left edges and dark bottom-right edges",
  brush: "ink washes and dry brush strokes",
};

/**
 * The briefs for a generated style's art slots (its frames, buttons,
 * ornaments and icon set), in the look's style: each slot's kind through
 * `artBrief`, its subject the style's shape language, finish, ornament
 * level and palette, so the art lands as the procedural dressings already
 * draw it. Pure: the same look and requests give the same briefs.
 */
export function styleArtBriefs(look: BriefLook, requests: StyleArtRequests): StyleArtBrief[] {
  const c = requests.choices;
  const roles = Object.entries(requests.palette)
    .map(([role, hex]) => `${role} ${colourName(hex)} ${hex}`)
    .join(", ");
  const shape = SHAPE_WORDS[c.shape] ?? c.shape;
  const finish = FINISH_WORDS[c.finish] ?? c.finish;
  const ornament = c.ornament === "rich" ? "rich ornament at the corners" : c.ornament === "light" ? "a little ornament" : "no ornament";
  return requests.slots.map((s) => {
    if (!(ART_KINDS as readonly string[]).includes(s.kind)) throw new Error(`${s.name}: '${s.kind}' is not an art kind`);
    const part = s.name.split("/").pop() ?? s.name;
    const subject =
      `the ${part.replace(/-/g, " ")} of the ${requests.style} UI style for ${requests.premise} (a ${requests.look} look, ${c.scheme} scheme): ` +
      `${shape}, ${finish}, ${ornament}; in its palette: ${roles}`;
    const insets = s.insets && s.insets.length === 4 ? (s.insets as unknown as readonly [number, number, number, number]) : undefined;
    return { slot: s.name, brief: artBrief(look, s.kind as ArtKind, subject, insets ? { insets } : {}) };
  });
}

// ------------------------------------------------------- the auto-rig's parts

/**
 * The auto-rig's templates (docs/art.md §2.8): a `type rig2d` with `parts`
 * (a layered PSD or a folder of PNGs) and one of these is rigged by the cook.
 */
export const RIG_TEMPLATES = ["humanoid-side", "humanoid-front", "quadruped", "creature"] as const;
export type RigTemplate = (typeof RIG_TEMPLATES)[number];

/** Where a part's joint with its parent is looked for (og-cook's `End`). */
export type RigJoint = "top" | "bottom" | "centre" | "near";

/** One part of an auto-rig template, as `rigParts` returns it. */
export interface RigPart {
  /** The layer's name (a folder's file is `<layer>.png`, or `NN-<layer>.png` to number the draw order). */
  readonly layer: string;
  /** Whether the cook refuses the art without it. */
  readonly required: boolean;
  /** `l` or `r` for a part with a side (docs/art.md §2.8 says what each side is per template), else null. */
  readonly side: "l" | "r" | null;
  /** The bone its pixels follow, named for the part; a chain's next bones are `<bone>-2`, `<bone>-3`, ... */
  readonly bone: string;
  /** How many bones it bends along: 1, or a chain's length (a cape 3, a tail 4). */
  readonly bones: number;
  /** Whether it bends (a limb or a chain, drawn as a mesh) or is drawn as it is. */
  readonly bends: boolean;
  /** The parts it hangs from, the first the art has; empty for the root part. */
  readonly hangsFrom: readonly string[];
  /** Where its joint with that part is looked for. */
  readonly joint: RigJoint;
  /** What it is, as the cook's refusal names it. */
  readonly what: string;
  /** The overlap and pivot rule for this part, one line. */
  readonly note: string;
}

// rig-parts: begin
// Each template's parts in the order to draw them, back to front (the order
// the example art draws in; the art's own layer order is what the cook
// draws). The fields are og-cook's (crates/og-cook/src/cook/autorig/
// template.rs): its unit test `rig_parts_match_the_templates` reads this
// block as JSON and fails on any part, field or `sides` that differs.
const RIG_PARTS = {
  "humanoid-side": {
    "sides": "`-l` is the far side (drawn behind the body), `-r` the near side; the character faces right",
    "parts": [
      { "name": "cape", "required": false, "parents": ["torso"], "end": "top", "shape": "chain", "bones": 3, "what": "a cape from the shoulders" },
      { "name": "wing-l", "required": false, "parents": ["torso"], "end": "near", "shape": "chain", "bones": 2, "what": "the far wing" },
      { "name": "tail", "required": false, "parents": ["pelvis"], "end": "near", "shape": "chain", "bones": 4, "what": "a tail" },
      { "name": "hair", "required": false, "parents": ["head"], "end": "top", "shape": "chain", "bones": 2, "what": "hair that swings" },
      { "name": "arm-upper-l", "required": true, "parents": ["torso"], "end": "top", "shape": "limb", "bones": 1, "what": "the far upper arm" },
      { "name": "arm-lower-l", "required": true, "parents": ["arm-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the far forearm" },
      { "name": "shield", "required": false, "parents": ["hand-l", "hand-r"], "end": "near", "shape": "rigid", "bones": 1, "what": "a shield, its grip under the hand" },
      { "name": "hand-l", "required": true, "parents": ["arm-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the far hand" },
      { "name": "leg-upper-l", "required": true, "parents": ["pelvis"], "end": "top", "shape": "limb", "bones": 1, "what": "the far thigh" },
      { "name": "leg-lower-l", "required": true, "parents": ["leg-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the far shin" },
      { "name": "foot-l", "required": true, "parents": ["leg-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the far foot" },
      { "name": "pelvis", "required": true, "parents": [], "end": "centre", "shape": "rigid", "bones": 1, "what": "the hips" },
      { "name": "torso", "required": true, "parents": ["pelvis"], "end": "bottom", "shape": "limb", "bones": 1, "what": "the chest and belly" },
      { "name": "neck", "required": false, "parents": ["torso"], "end": "bottom", "shape": "limb", "bones": 1, "what": "the neck" },
      { "name": "ear-l", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the far ear" },
      { "name": "head", "required": true, "parents": ["neck", "torso"], "end": "bottom", "shape": "rigid", "bones": 1, "what": "the head" },
      { "name": "ear-r", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the near ear" },
      { "name": "leg-upper-r", "required": true, "parents": ["pelvis"], "end": "top", "shape": "limb", "bones": 1, "what": "the near thigh" },
      { "name": "leg-lower-r", "required": true, "parents": ["leg-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the near shin" },
      { "name": "foot-r", "required": true, "parents": ["leg-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the near foot" },
      { "name": "wing-r", "required": false, "parents": ["torso"], "end": "near", "shape": "chain", "bones": 2, "what": "the near wing" },
      { "name": "arm-upper-r", "required": true, "parents": ["torso"], "end": "top", "shape": "limb", "bones": 1, "what": "the near upper arm" },
      { "name": "arm-lower-r", "required": true, "parents": ["arm-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the near forearm" },
      { "name": "weapon", "required": false, "parents": ["hand-r", "hand-l"], "end": "near", "shape": "rigid", "bones": 1, "what": "a weapon, its grip under the hand" },
      { "name": "hand-r", "required": true, "parents": ["arm-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the near hand" }
    ]
  },
  "humanoid-front": {
    "sides": "`-l` is the part on the image's left, `-r` on its right; the character faces the viewer",
    "parts": [
      { "name": "cape", "required": false, "parents": ["torso"], "end": "top", "shape": "chain", "bones": 3, "what": "a cape from the shoulders" },
      { "name": "wing-l", "required": false, "parents": ["torso"], "end": "near", "shape": "chain", "bones": 2, "what": "the image-left wing" },
      { "name": "wing-r", "required": false, "parents": ["torso"], "end": "near", "shape": "chain", "bones": 2, "what": "the image-right wing" },
      { "name": "tail", "required": false, "parents": ["pelvis"], "end": "near", "shape": "chain", "bones": 4, "what": "a tail" },
      { "name": "hair", "required": false, "parents": ["head"], "end": "top", "shape": "chain", "bones": 2, "what": "hair that swings" },
      { "name": "leg-upper-l", "required": true, "parents": ["pelvis"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-left thigh" },
      { "name": "leg-lower-l", "required": true, "parents": ["leg-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-left shin" },
      { "name": "foot-l", "required": true, "parents": ["leg-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the image-left foot" },
      { "name": "leg-upper-r", "required": true, "parents": ["pelvis"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-right thigh" },
      { "name": "leg-lower-r", "required": true, "parents": ["leg-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-right shin" },
      { "name": "foot-r", "required": true, "parents": ["leg-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the image-right foot" },
      { "name": "pelvis", "required": true, "parents": [], "end": "centre", "shape": "rigid", "bones": 1, "what": "the hips" },
      { "name": "torso", "required": true, "parents": ["pelvis"], "end": "bottom", "shape": "limb", "bones": 1, "what": "the chest and belly" },
      { "name": "neck", "required": false, "parents": ["torso"], "end": "bottom", "shape": "limb", "bones": 1, "what": "the neck" },
      { "name": "ear-l", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the image-left ear" },
      { "name": "ear-r", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the image-right ear" },
      { "name": "head", "required": true, "parents": ["neck", "torso"], "end": "bottom", "shape": "rigid", "bones": 1, "what": "the head" },
      { "name": "arm-upper-l", "required": true, "parents": ["torso"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-left upper arm" },
      { "name": "arm-lower-l", "required": true, "parents": ["arm-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-left forearm" },
      { "name": "shield", "required": false, "parents": ["hand-l", "hand-r"], "end": "near", "shape": "rigid", "bones": 1, "what": "a shield, its grip under the hand" },
      { "name": "hand-l", "required": true, "parents": ["arm-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the image-left hand" },
      { "name": "arm-upper-r", "required": true, "parents": ["torso"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-right upper arm" },
      { "name": "arm-lower-r", "required": true, "parents": ["arm-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the image-right forearm" },
      { "name": "weapon", "required": false, "parents": ["hand-r", "hand-l"], "end": "near", "shape": "rigid", "bones": 1, "what": "a weapon, its grip under the hand" },
      { "name": "hand-r", "required": true, "parents": ["arm-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the image-right hand" }
    ]
  },
  "quadruped": {
    "sides": "`-l` is the far side (drawn behind the body), `-r` the near side; the animal faces right",
    "parts": [
      { "name": "tail", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 4, "what": "a tail" },
      { "name": "wing-l", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 2, "what": "the far wing" },
      { "name": "leg-back-upper-l", "required": true, "parents": ["body"], "end": "top", "shape": "limb", "bones": 1, "what": "the far back leg's upper part" },
      { "name": "leg-back-lower-l", "required": true, "parents": ["leg-back-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the far back leg's lower part" },
      { "name": "paw-back-l", "required": true, "parents": ["leg-back-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the far back paw" },
      { "name": "leg-front-upper-l", "required": true, "parents": ["body"], "end": "top", "shape": "limb", "bones": 1, "what": "the far front leg's upper part" },
      { "name": "leg-front-lower-l", "required": true, "parents": ["leg-front-upper-l"], "end": "top", "shape": "limb", "bones": 1, "what": "the far front leg's lower part" },
      { "name": "paw-front-l", "required": true, "parents": ["leg-front-lower-l"], "end": "top", "shape": "rigid", "bones": 1, "what": "the far front paw" },
      { "name": "ear-l", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the far ear" },
      { "name": "body", "required": true, "parents": [], "end": "centre", "shape": "limb", "bones": 1, "what": "the body" },
      { "name": "neck", "required": false, "parents": ["body"], "end": "near", "shape": "limb", "bones": 1, "what": "the neck" },
      { "name": "mane", "required": false, "parents": ["neck", "head"], "end": "near", "shape": "chain", "bones": 2, "what": "a mane" },
      { "name": "head", "required": true, "parents": ["neck", "body"], "end": "near", "shape": "rigid", "bones": 1, "what": "the head" },
      { "name": "jaw", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "a lower jaw" },
      { "name": "ear-r", "required": false, "parents": ["head"], "end": "near", "shape": "rigid", "bones": 1, "what": "the near ear" },
      { "name": "leg-back-upper-r", "required": true, "parents": ["body"], "end": "top", "shape": "limb", "bones": 1, "what": "the near back leg's upper part" },
      { "name": "leg-back-lower-r", "required": true, "parents": ["leg-back-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the near back leg's lower part" },
      { "name": "paw-back-r", "required": true, "parents": ["leg-back-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the near back paw" },
      { "name": "leg-front-upper-r", "required": true, "parents": ["body"], "end": "top", "shape": "limb", "bones": 1, "what": "the near front leg's upper part" },
      { "name": "leg-front-lower-r", "required": true, "parents": ["leg-front-upper-r"], "end": "top", "shape": "limb", "bones": 1, "what": "the near front leg's lower part" },
      { "name": "paw-front-r", "required": true, "parents": ["leg-front-lower-r"], "end": "top", "shape": "rigid", "bones": 1, "what": "the near front paw" },
      { "name": "wing-r", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 2, "what": "the near wing" }
    ]
  },
  "creature": {
    "sides": "`-l` is the part on the image's left, `-r` on its right; limbs are numbered from the front (`limb-1`)",
    "parts": [
      { "name": "tail", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 4, "what": "a tail" },
      { "name": "wing-l", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 2, "what": "the left wing" },
      { "name": "wing-r", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 2, "what": "the right wing" },
      { "name": "limb-2", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-4", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-6", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-8", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "body", "required": true, "parents": [], "end": "centre", "shape": "limb", "bones": 1, "what": "the body" },
      { "name": "ear-l", "required": false, "parents": ["head", "body"], "end": "near", "shape": "rigid", "bones": 1, "what": "the left ear" },
      { "name": "ear-r", "required": false, "parents": ["head", "body"], "end": "near", "shape": "rigid", "bones": 1, "what": "the right ear" },
      { "name": "head", "required": false, "parents": ["body"], "end": "near", "shape": "rigid", "bones": 1, "what": "a head" },
      { "name": "eye-l", "required": false, "parents": ["head", "body"], "end": "near", "shape": "rigid", "bones": 1, "what": "the left eye" },
      { "name": "eye-r", "required": false, "parents": ["head", "body"], "end": "near", "shape": "rigid", "bones": 1, "what": "the right eye" },
      { "name": "limb-1", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-3", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-5", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" },
      { "name": "limb-7", "required": false, "parents": ["body"], "end": "near", "shape": "chain", "bones": 3, "what": "a limb" }
    ]
  }
} as const;
// rig-parts: end

/** Where the joint end is, in words. */
const JOINT_WORDS: Readonly<Record<RigJoint, string>> = { top: "its top", bottom: "its bottom", centre: "its middle", near: "where they meet" };

/**
 * The parts of an auto-rig template (docs/art.md §2.8), back to front: each
 * one's layer name, whether the cook needs it, its side, the bone it drives
 * and the rule that places its joint. Pure; throws naming the four
 * templates for any other name.
 */
export function rigParts(template: string): RigPart[] {
  if (!(RIG_TEMPLATES as readonly string[]).includes(template)) throw new Error(`'${template}' is not an auto-rig template: use one of ${RIG_TEMPLATES.join(", ")}`);
  return RIG_PARTS[template as RigTemplate].parts.map((p) => {
    const side = /-(l|r)$/.exec(p.name)?.[1] as "l" | "r" | undefined;
    const joint = p.end as RigJoint;
    const from = p.parents.map((n) => `\`${n}\``).join(", else ");
    const note =
      p.parents.length === 0
        ? "the root part: everything hangs from it, its bone at the centre of its solid pixels, the rig's root on the ground under it"
        : `hangs from ${from}: its joint is the centre of where it overlaps that part at ${JOINT_WORDS[joint]} (solid pixels, alpha at least half); with no overlap, ${joint === "near" ? "its pixel nearest that part" : JOINT_WORDS[joint]}, which must touch it; \`pivot ${p.name} <x> <y>\` sets it`;
    return {
      layer: p.name,
      required: p.required,
      side: side ?? null,
      bone: p.name,
      bones: p.bones,
      bends: p.shape !== "rigid",
      hangsFrom: [...p.parents],
      joint,
      what: p.what,
      note,
    };
  });
}

// ------------------------------------------------------- the audio brief

// The audio counterpart of artBrief (docs/art.md §2.9): from a game's music
// style, mood and genre (project.ogt's create block, or the platform's
// creation form) and its premise, the request for each music cue and each
// sound effect the game needs, plus the procedural presets `og cook`
// synthesises instead (`type sfx`, og-audio §2), so a game is never silent
// while its generated audio is pending. Pure and deterministic; a public
// contract beside artBrief (README §The audio brief).

/** The music styles a game is created with (create-game's `CREATION_CHOICES.music`, held equal by its tests). */
export const MUSIC_STYLES = ["chiptune", "orchestral", "electronic", "ambient", "folk", "rock"] as const;
export type MusicStyle = (typeof MUSIC_STYLES)[number];

/** og cook's procedural sound presets (og-cook's `sfx::PRESETS`, held equal by the knowledge tests). */
export const SFX_PRESETS = ["coin", "pickup", "powerup", "jump", "land", "footstep", "hit", "hurt", "explosion", "laser", "whoosh", "door", "death", "click", "select", "back", "error", "levelup"] as const;
export type SfxPreset = (typeof SFX_PRESETS)[number];

/** What a cue is for: when the game plays it. */
export type MusicRole = "title" | "explore" | "action" | "boss" | "calm" | "victory" | "defeat" | "stinger";

export interface AudioBriefInput {
  /** `CREATION_CHOICES.music`. */
  readonly music: string;
  /** `CREATION_CHOICES.mood`: bright, dark, warm, cold, soft, high-contrast. */
  readonly mood: string;
  /** `CREATION_CHOICES.genre`. */
  readonly genre: string;
  /** The game in one line; the prompts carry it as written. */
  readonly premise?: string;
}

/** The file a generator returns, as og cook reads it (og-audio §5). */
export interface AudioFormat {
  /** `ogg` (Vorbis) for music, `wav` for effects; og cook decodes either to QOA. */
  readonly container: "ogg" | "wav";
  readonly sampleRate: 48000;
  readonly channels: 1 | 2;
  /** Integrated loudness to master to (LUFS); effects are judged by peak. */
  readonly lufs: number | null;
  /** The true peak not to pass (dBFS). */
  readonly peakDb: number;
}

/** The request for one music cue. */
export interface MusicCueBrief {
  /** Stable within the brief (`title`, `explore`, `boss`). */
  readonly id: string;
  readonly role: MusicRole;
  /** The whole request as text, one statement per line. */
  readonly prompt: string;
  readonly style: MusicStyle;
  readonly bpm: number;
  /** Major or minor (a mode name for modal styles). */
  readonly mode: string;
  readonly beatsPerBar: number;
  /** Its length in whole bars at `bpm` (a stinger 1 to 2). */
  readonly bars: number;
  readonly seconds: number;
  /** Loops seamlessly (AudioSource.loop); a stinger and the endings do not. */
  readonly loop: boolean;
  /** Layers to deliver as separate, sample-aligned files (og-audio §5's stems); empty: one mix. */
  readonly stems: readonly string[];
  /** 0 calm to 1 the most intense of the game's cues. */
  readonly intensity: number;
  /** Where the game puts it: an asset id under assets/ (`music/<id>.ogg`). */
  readonly file: string;
  readonly format: AudioFormat;
}

/** The request for one sound effect, with the preset that stands in until it exists. */
export interface SfxBrief {
  readonly id: string;
  /** The game event that plays it. */
  readonly event: string;
  readonly prompt: string;
  readonly seconds: number;
  /** og.audio's bus (og-audio §4). */
  readonly bus: "sfx" | "ui";
  /** Takes to deliver, so repeats do not machine-gun (og-audio §1); 1 for UI sounds. */
  readonly variants: number;
  /** The preset (`presets[].id`) the game plays until this file exists; null: none fits. */
  readonly preset: string | null;
  /** `audio/<id>.wav`; variants add `-2`, `-3` before the extension. */
  readonly file: string;
  readonly format: AudioFormat;
}

/** A procedural sound og cook synthesises: write `ogt` to `assets/<file>`, then `og.load.sound(world, file)`. */
export interface SfxPresetBrief {
  readonly id: string;
  readonly preset: SfxPreset;
  readonly seed: number;
  readonly pitch: number;
  readonly length: number;
  readonly volume: number;
  /** `audio/<id>.ogt`. */
  readonly file: string;
  /** The `type sfx` document, as `og sfx <preset> --out` writes it. */
  readonly ogt: string;
}

export interface AudioBrief {
  readonly music: MusicStyle;
  readonly mood: string;
  readonly genre: string;
  readonly premise: string;
  /** The direction every request shares (instruments, texture, tempo), one statement per line. */
  readonly direction: string;
  /** What no cue or effect may contain. */
  readonly avoid: readonly string[];
  readonly cues: readonly MusicCueBrief[];
  readonly sfx: readonly SfxBrief[];
  readonly presets: readonly SfxPresetBrief[];
}

interface StyleSpec {
  readonly instruments: string;
  readonly texture: string;
  readonly bpm: readonly [number, number];
  readonly stems: readonly string[];
  readonly sfx: string;
  readonly avoid: readonly string[];
  /** Preset variation: pitch and length multipliers. */
  readonly pitch: number;
  readonly length: number;
}

const STYLES: Readonly<Record<MusicStyle, StyleSpec>> = {
  chiptune: { instruments: "pulse waves at 12.5, 25 and 50 % duty, a triangle bass, a noise channel for drums; four voices at most, as a 1980s sound chip", texture: "fast arpeggios for chords, short echo by a delayed quieter voice, no reverb", bpm: [120, 160], stems: ["lead", "harmony", "bass", "drums"], sfx: "synthesised in the same chip voices: square and noise, short envelopes, pitch sweeps, no recorded sound", avoid: ["sampled or acoustic instruments", "reverb tails", "more than four voices at once"], pitch: 1, length: 0.9 },
  orchestral: { instruments: "strings (violins, violas, cellos, basses), French horns, woodwinds, timpani and harp; brass and choir held back for the biggest moments", texture: "a clear melody over sustained harmony, dynamics that swell, recorded in a concert hall", bpm: [70, 130], stems: ["strings", "brass", "woodwinds", "percussion"], sfx: "acoustic and tactile, recorded foley with a short natural room, cut close", avoid: ["synthesisers", "drum machines", "electric guitars"], pitch: 0.9, length: 1.2 },
  electronic: { instruments: "analogue-style synthesisers (a saw lead, a sub bass, pads), a drum machine with four-on-the-floor kick, side-chained pads", texture: "steady pulse, filter sweeps for builds, a clean wide stereo mix", bpm: [110, 140], stems: ["drums", "bass", "pads", "lead"], sfx: "synthesised, clean transients, filter sweeps and digital tails", avoid: ["acoustic instruments as lead", "lo-fi hiss", "vocals with words"], pitch: 1.05, length: 1 },
  ambient: { instruments: "soft pads, piano with long sustain, bowed textures, distant field recordings", texture: "slowly evolving, few notes, lots of space, no hard downbeat, gentle reverb", bpm: [60, 90], stems: ["pads", "melody", "texture"], sfx: "soft and rounded, low transients, airy tails that sit under the music", avoid: ["loud drums", "busy melodies", "sudden dynamics"], pitch: 0.95, length: 1.25 },
  folk: { instruments: "acoustic guitar, fiddle, flute or tin whistle, accordion, hand drums and frame drum", texture: "a singable modal melody, dance rhythms, played by a small band in a warm room", bpm: [90, 140], stems: ["melody", "strings", "rhythm"], sfx: "wood, cloth, metal and hands: recorded foley, warm and dry", avoid: ["synthesisers", "electronic drums", "orchestral brass"], pitch: 0.95, length: 1.1 },
  rock: { instruments: "overdriven electric guitars (rhythm and lead), bass guitar, a live drum kit, occasional organ", texture: "driving eighth-note rhythm, power chords, a guitar lead in place of vocals", bpm: [110, 170], stems: ["drums", "bass", "rhythm guitar", "lead guitar"], sfx: "punchy and loud, distorted impacts, cymbal and amp accents", avoid: ["soft pads as lead", "vocals with words", "orchestral strings"], pitch: 1, length: 1 },
};

interface MoodSpec {
  readonly words: string;
  readonly mode: string;
  /** Where in the style's tempo range cues sit, 0 slowest to 1 fastest. */
  readonly tempo: number;
  readonly pitch: number;
}

const MOODS: Readonly<Record<string, MoodSpec>> = {
  bright: { words: "bright, hopeful and playful", mode: "major", tempo: 0.7, pitch: 1.08 },
  dark: { words: "dark, tense and uneasy", mode: "minor", tempo: 0.35, pitch: 0.85 },
  warm: { words: "warm, cosy and friendly", mode: "major", tempo: 0.45, pitch: 0.97 },
  cold: { words: "cold, distant and lonely", mode: "dorian", tempo: 0.4, pitch: 1.03 },
  soft: { words: "soft, gentle and calm", mode: "lydian", tempo: 0.25, pitch: 1 },
  "high-contrast": { words: "bold, punchy and dramatic, loud against quiet", mode: "phrygian", tempo: 0.8, pitch: 0.95 },
};

type Cue = readonly [id: string, role: MusicRole, bars: number, intensity: number, tempo: number];
type Effect = readonly [id: string, variants?: number];

interface GenreSpec {
  readonly cues: readonly Cue[];
  readonly sfx: readonly Effect[];
  readonly note?: string;
}

/** Every game's: the title theme, the menu's sounds and the endings. */
const BASE_CUES: readonly Cue[] = [["title", "title", 32, 0.4, 0], ["victory", "victory", 4, 0.7, 0.1], ["defeat", "defeat", 4, 0.3, -0.2]];
const UI_SFX: readonly Effect[] = [["ui-hover"], ["ui-press"], ["ui-back"], ["ui-error"]];

const GENRES: Readonly<Record<string, GenreSpec>> = {
  "action-platformer": { cues: [["level", "explore", 48, 0.6, 0.1], ["boss", "boss", 32, 1, 0.2]], sfx: [["jump", 3], ["land", 3], ["footstep", 4], ["coin", 3], ["hit", 3], ["hurt", 2], ["powerup"], ["death"], ["checkpoint"]] },
  shooter: { cues: [["combat", "action", 48, 0.85, 0.2], ["calm", "calm", 32, 0.3, -0.2], ["boss", "boss", 32, 1, 0.25]], sfx: [["shot", 4], ["reload", 2], ["hit", 3], ["hurt", 2], ["explosion", 2], ["pickup", 2], ["footstep", 4], ["death"], ["empty"]] },
  rpg: { cues: [["town", "calm", 48, 0.3, -0.2], ["field", "explore", 64, 0.5, 0], ["battle", "action", 48, 0.8, 0.2], ["boss", "boss", 48, 1, 0.2]], sfx: [["swing", 3], ["hit", 3], ["hurt", 2], ["spell", 2], ["heal"], ["levelup"], ["door"], ["chest"], ["footstep", 4], ["coin", 2]] },
  "puzzle-match": { cues: [["board", "calm", 48, 0.4, 0], ["hurry", "action", 16, 0.7, 0.3]], sfx: [["swap", 2], ["match", 3], ["cascade", 3], ["invalid"], ["special"], ["clear"]] },
  "strategy-tower-defense": { cues: [["build", "calm", 32, 0.35, -0.1], ["wave", "action", 48, 0.75, 0.1], ["boss", "boss", 32, 1, 0.2]], sfx: [["place", 2], ["shot", 4], ["hit", 3], ["explosion", 2], ["upgrade"], ["wave-start"], ["leak"], ["coin", 2]] },
  racing: { cues: [["race", "action", 64, 0.85, 0.3], ["lobby", "calm", 32, 0.4, 0]], sfx: [["engine-loop"], ["boost"], ["skid", 2], ["crash", 2], ["countdown"], ["lap"], ["pickup", 2]] },
  runner: { cues: [["run", "action", 64, 0.75, 0.3]], sfx: [["jump", 3], ["land", 3], ["slide"], ["coin", 4], ["hit"], ["powerup"], ["death"]] },
  "roguelike-card": { cues: [["map", "calm", 32, 0.35, -0.1], ["combat", "action", 48, 0.7, 0.1], ["boss", "boss", 32, 1, 0.2], ["shop", "calm", 16, 0.25, -0.1]], sfx: [["card-draw", 3], ["card-play", 3], ["shuffle"], ["hit", 3], ["block", 2], ["hurt", 2], ["coin", 2], ["relic"], ["death"]] },
  idle: { cues: [["loop", "calm", 64, 0.3, -0.1], ["milestone", "stinger", 2, 0.7, 0]], sfx: [["tap", 4], ["coin", 4], ["buy"], ["milestone"], ["prestige"]] },
  rhythm: { cues: [["song", "action", 64, 0.8, 0.2], ["calibrate", "calm", 16, 0.3, 0]], sfx: [["perfect"], ["good"], ["miss"], ["combo"], ["fail"]], note: "Hold the tempo exactly at the bpm with no rubato, start on a downbeat with one bar of count-in, and keep every beat audible: the game judges presses against og.Music's clock at this bpm." },
  arena: { cues: [["fight", "action", 48, 0.85, 0.2], ["lobby", "calm", 32, 0.4, 0]], sfx: [["swing", 3], ["hit", 4], ["hurt", 2], ["dash"], ["ko"], ["countdown"], ["round-start"]] },
  "open-world": { cues: [["day", "explore", 96, 0.4, -0.1], ["night", "calm", 64, 0.25, -0.3], ["danger", "action", 48, 0.8, 0.2]], sfx: [["footstep", 6], ["swing", 3], ["hit", 3], ["hurt", 2], ["pickup", 2], ["door"], ["discover"], ["wind-loop"]] },
};

/** Each effect's description, length (s), bus and the preset that stands in for it. */
const EFFECTS: Readonly<Record<string, readonly [what: string, seconds: number, bus: "sfx" | "ui", preset: SfxPreset | null]>> = {
  "ui-hover": ["a tiny soft tick as focus moves to a button, quiet", 0.06, "ui", "click"],
  "ui-press": ["a crisp confirm click for a pressed button", 0.1, "ui", "select"],
  "ui-back": ["a short falling cancel for closing a menu", 0.15, "ui", "back"],
  "ui-error": ["a low double buzz for a refused action", 0.25, "ui", "error"],
  jump: ["a quick light upward whoop as the hero leaves the ground", 0.2, "sfx", "jump"],
  land: ["a soft low thud of feet landing", 0.15, "sfx", "land"],
  footstep: ["one footstep on the ground, short", 0.12, "sfx", "footstep"],
  coin: ["a bright two-note pickup chime for a coin", 0.3, "sfx", "coin"],
  pickup: ["a short rising blip for picking up an item", 0.25, "sfx", "pickup"],
  powerup: ["a rising shimmering sweep as a power-up is taken", 0.6, "sfx", "powerup"],
  hit: ["a punchy impact of a blow landing", 0.2, "sfx", "hit"],
  hurt: ["a short pained hit on the player, distinct from hitting an enemy", 0.3, "sfx", "hurt"],
  death: ["a long falling defeat sound as the player dies", 1, "sfx", "death"],
  checkpoint: ["a warm two-step chime for reaching a checkpoint", 0.5, "sfx", "select"],
  shot: ["one weapon shot, sharp attack and a short tail", 0.3, "sfx", "laser"],
  reload: ["a mechanical reload: clip out, clip in", 0.8, "sfx", "door"],
  explosion: ["an explosion with a deep boom and debris", 1.2, "sfx", "explosion"],
  empty: ["a dry click of an empty weapon", 0.1, "sfx", "click"],
  swing: ["a weapon swing whooshing through the air", 0.25, "sfx", "whoosh"],
  spell: ["a magic spell cast, a bright swirl", 0.7, "sfx", "powerup"],
  heal: ["a soft rising sparkle of healing", 0.6, "sfx", "pickup"],
  levelup: ["a three-note rising fanfare for a level up", 0.9, "sfx", "levelup"],
  door: ["a door opening: a creak and a thump", 0.7, "sfx", "door"],
  chest: ["a treasure chest opening with a sparkle", 0.8, "sfx", "coin"],
  swap: ["two pieces sliding past each other on a board", 0.15, "sfx", "whoosh"],
  match: ["pieces matching and popping, satisfying", 0.3, "sfx", "pickup"],
  cascade: ["a chain reaction pop, one step higher than the last", 0.3, "sfx", "coin"],
  invalid: ["a soft bump for a move that is not allowed", 0.2, "sfx", "error"],
  special: ["a special piece forming, a bright charge", 0.5, "sfx", "powerup"],
  clear: ["the board cleared, a short celebration", 1, "sfx", "levelup"],
  place: ["a tower set down on the ground with a solid thunk", 0.3, "sfx", "land"],
  upgrade: ["an upgrade completing, a rising ratchet", 0.6, "sfx", "powerup"],
  "wave-start": ["a horn or alarm announcing the next wave", 1, "sfx", "levelup"],
  leak: ["a warning tone as an enemy gets through", 0.4, "sfx", "error"],
  "engine-loop": ["a vehicle engine at a steady mid rpm, seamless loop (pitched by speed in game)", 2, "sfx", null],
  boost: ["a boost kicking in, a rising rush", 0.8, "sfx", "whoosh"],
  skid: ["tyres skidding on asphalt", 0.8, "sfx", "whoosh"],
  crash: ["a vehicle crash, metal and impact", 1, "sfx", "explosion"],
  countdown: ["one countdown beep (the last one higher is a separate take)", 0.3, "sfx", "select"],
  lap: ["a lap completed, a short bright chime", 0.5, "sfx", "coin"],
  slide: ["a quick slide along the ground", 0.4, "sfx", "whoosh"],
  "card-draw": ["a card drawn from the deck, paper flick", 0.15, "sfx", "whoosh"],
  "card-play": ["a card slapped onto the table", 0.2, "sfx", "hit"],
  shuffle: ["a deck shuffled", 0.8, "sfx", "whoosh"],
  block: ["a blow blocked by a shield, a solid thud", 0.3, "sfx", "land"],
  relic: ["a relic obtained, a mysterious chime", 0.8, "sfx", "powerup"],
  tap: ["a light tap on the main button, satisfying and short", 0.08, "sfx", "click"],
  buy: ["a purchase made, a register ching", 0.3, "sfx", "coin"],
  milestone: ["a milestone reached, a rising fanfare", 0.9, "sfx", "levelup"],
  prestige: ["a prestige reset, a big sweeping whoosh into a chime", 1.5, "sfx", "powerup"],
  perfect: ["a bright accent for a perfect hit, in the song's key", 0.15, "sfx", "select"],
  good: ["a softer accent for a good hit", 0.12, "sfx", "click"],
  miss: ["a dull muted thud for a miss", 0.15, "sfx", "error"],
  combo: ["a combo milestone, a quick rising chime", 0.4, "sfx", "levelup"],
  fail: ["the song failing, a record-scratch fall", 0.8, "sfx", "death"],
  dash: ["a quick dash whoosh", 0.25, "sfx", "whoosh"],
  ko: ["a knockout, a heavy final impact", 0.8, "sfx", "explosion"],
  "round-start": ["a round begins, a gong or bell", 1, "sfx", "levelup"],
  discover: ["a new place discovered, a short wondrous chime", 1, "sfx", "levelup"],
  "wind-loop": ["outdoor wind ambience, seamless loop", 4, "sfx", null],
};

const MUSIC_FORMAT: AudioFormat = { container: "ogg", sampleRate: 48000, channels: 2, lufs: -16, peakDb: -1 };
const SFX_FORMAT: AudioFormat = { container: "wav", sampleRate: 48000, channels: 1, lufs: null, peakDb: -1 };

/** A seed from an id: the same id gets the same take, ids differ by seed. */
function seedOf(id: string): number {
  let h = 2166136261;
  for (const c of id) h = Math.imul(h ^ c.codePointAt(0)!, 16777619) >>> 0;
  return 1 + (h % 9999);
}

const round2 = (x: number) => Math.round(x * 100) / 100;

function known<T extends string>(what: string, value: string, keys: readonly T[]): T {
  if (!(keys as readonly string[]).includes(value)) throw new Error(`audioBrief: ${what} '${value}' is not one of ${keys.join(", ")}`);
  return value as T;
}

/**
 * The audio a game needs, as requests for a generator: one per music cue
 * and per sound effect, in the game's music style and mood, each with the
 * file it goes to and the format to deliver, and the procedural presets
 * that stand in until the files exist. Pure; throws naming the values for
 * an unknown style, mood or genre.
 */
export function audioBrief(input: AudioBriefInput): AudioBrief {
  const music = known("music", input.music, MUSIC_STYLES);
  const mood = known("mood", input.mood, Object.keys(MOODS));
  const genre = known("genre", input.genre, Object.keys(GENRES));
  const premise = (input.premise ?? "").trim();
  const s = STYLES[music];
  const m = MOODS[mood]!;
  const g = GENRES[genre]!;
  const about = premise ? `for a ${genre.replace(/-/g, " ")} game: ${premise}` : `for a ${genre.replace(/-/g, " ")} game`;
  const direction = [
    `Music style: ${music}. ${s.instruments[0]!.toUpperCase()}${s.instruments.slice(1)}.`,
    `Texture: ${s.texture}.`,
    `Mood: ${m.words}.`,
    `Sound effects: ${s.sfx}, ${m.words.split(",")[0]} in character.`,
  ].join("\n");
  const avoid = [...s.avoid, "vocals with lyrics", "copyrighted melodies or samples", "clipping or limiting artefacts", "silence at the start or end of a loop"];
  const beatsPerBar = 4;
  const cues: MusicCueBrief[] = [...BASE_CUES.slice(0, 1), ...g.cues, ...BASE_CUES.slice(1)].map(([id, role, bars, intensity, tempo]) => {
    const t = Math.min(1, Math.max(0, m.tempo + tempo));
    const bpm = Math.round(s.bpm[0] + (s.bpm[1] - s.bpm[0]) * t);
    const loop = role !== "victory" && role !== "defeat" && role !== "stinger";
    const seconds = round2((bars * beatsPerBar * 60) / bpm);
    const mode = role === "defeat" ? (m.mode === "major" ? "minor" : m.mode) : role === "victory" ? "major" : m.mode;
    const stems = loop && (role === "action" || role === "explore" || role === "boss") ? s.stems : [];
    const lines = [
      `A ${music} ${ROLE_WORDS[role]} ${about}.`,
      direction,
      `Tempo: ${bpm} bpm, ${beatsPerBar}/4 time, in a ${mode} key; ${bars} bars (${seconds} s).`,
      `Intensity: ${Math.round(intensity * 10)} of 10 among the game's cues.`,
      loop ? "Loop: it must loop seamlessly: end on the bar line leading back into the first bar, with no fade, reverb tail or silence at either end." : "Ending: it plays once, ends cleanly with a short tail, no fade-in.",
      stems.length ? `Stems: deliver ${stems.join(", ")} as separate files of exactly the same length, sample-aligned, that sum to the mix; the game brings layers in and out.` : "",
      genre === "rhythm" && role === "action" && g.note ? g.note : "",
      `Deliver: ${MUSIC_FORMAT.container.toUpperCase()} Vorbis, 48 kHz stereo, mastered to ${MUSIC_FORMAT.lufs} LUFS integrated, true peak below ${MUSIC_FORMAT.peakDb} dBFS.`,
      `Avoid: ${avoid.join("; ")}.`,
    ].filter(Boolean);
    return { id, role, prompt: lines.join("\n"), style: music, bpm, mode, beatsPerBar, bars, seconds, loop, stems, intensity, file: `music/${id}.ogg`, format: MUSIC_FORMAT };
  });
  const presets: SfxPresetBrief[] = [];
  const sfx: SfxBrief[] = [...UI_SFX, ...g.sfx].map(([id, variants = 1]) => {
    const [what, seconds, bus, preset] = EFFECTS[id]!;
    const lines = [
      `A sound effect ${about}: ${what}.`,
      `Sound effects: ${s.sfx}; ${m.words}.`,
      `Length: about ${seconds} s, starting on its transient with no silence before it${/-loop$/.test(id) ? "; a seamless loop" : ""}.`,
      variants > 1 ? `Takes: ${variants} variations of the same sound, slightly different in pitch and texture, so repeats do not sound identical.` : "",
      bus === "ui" ? "It is a UI sound: short, quiet and clean, never louder than gameplay." : "",
      `Deliver: WAV, 48 kHz mono, peak below ${SFX_FORMAT.peakDb} dBFS, no reverb tail longer than the sound.`,
      `Avoid: ${[...s.avoid, "voices", "music"].join("; ")}.`,
    ].filter(Boolean);
    let presetId: string | null = null;
    if (preset) {
      presetId = id;
      const seed = seedOf(id);
      const pitch = round2(s.pitch * m.pitch);
      const length = round2(s.length);
      const volume = bus === "ui" ? (id === "ui-hover" ? 0.3 : 0.6) : 1;
      presets.push({
        id, preset, seed, pitch, length, volume, file: `audio/${id}.ogt`,
        ogt: `type sfx\nversion 1\n---\n# ${what} (audioBrief: ${music}, ${mood}). og.load.sound(world, "audio/${id}.ogt") plays it until audio/${id}.wav exists.\npreset ${JSON.stringify(preset)}\nseed ${seed}\npitch ${pitch}\nlength ${length}\nvolume ${volume}\n`,
      });
    }
    return { id, event: what, prompt: lines.join("\n"), seconds, bus, variants, preset: presetId, file: `audio/${id}.wav`, format: SFX_FORMAT };
  });
  return { music, mood, genre, premise, direction, avoid, cues, sfx, presets };
}

const ROLE_WORDS: Readonly<Record<MusicRole, string>> = {
  title: "title theme, the game's main melody, played on the title screen",
  explore: "exploration loop for the main play",
  action: "action loop for the most active play",
  boss: "boss loop, the game's most intense cue",
  calm: "calm loop for menus, building or resting",
  victory: "victory jingle",
  defeat: "defeat jingle",
  stinger: "short stinger that plays over the loop at a highlight",
};

/** The audio choices of a project's create block (project.ogt): `music`, `mood` and `genre`. */
export function readAudioChoices(projectDir: string): AudioBriefInput {
  const text = readFileSync(join(projectDir, "project.ogt"), "utf8");
  const block = /^create[ \t]*\{([\s\S]*?)^\}/m.exec(text)?.[1];
  if (!block) throw new Error(`${join(projectDir, "project.ogt")} has no create block: pass music, mood and genre (og new --music --mood --genre)`);
  const field = (k: string) => new RegExp(`^\\s*${k}\\s+"([^"]*)"`, "m").exec(block)?.[1];
  const music = field("music"), mood = field("mood"), genre = field("genre");
  if (!music || !mood || !genre) throw new Error("the create block lacks music, mood or genre");
  return { music, mood, genre };
}

if (import.meta.main) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (e) {
    console.error(`brief: ${e instanceof Error ? e.message : String(e)}`);
    process.exit(1);
  }
}
