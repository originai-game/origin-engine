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
// down to the preset) and `presetLook(slug)` a preset's alone, for a game not
// written yet.
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

import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** The kinds of asset a brief is written for. */
export const ART_KINDS = ["sprite", "tileset", "portrait", "backdrop", "prop", "character", "texture", "icon"] as const;
export type ArtKind = (typeof ART_KINDS)[number];

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
  readonly look: { readonly profiles: readonly ("2d" | "3d")[] };
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
}

export interface BriefOptions {
  /** The game's accepted assets of this style (paths or URLs), passed on as references after the style's preview. */
  readonly references?: readonly string[];
  /** The asset's size in metres: a prop 1, a character 2, a texture's tile 2 by default. It sets the cell of a 2D sprite (ArtPixels.unit per metre) and the texture of a model (ArtTexels.density per metre). */
  readonly metres?: number;
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

const DEFAULT_METRES: Readonly<Record<ArtKind, number>> = { sprite: 2, tileset: 1, portrait: 1, backdrop: 1, prop: 1, character: 2, texture: 2, icon: 1 };

/** Which sentence opens the prompt. */
function opening(kind: ArtKind, subject: string, medium: "image" | "model", metres: number): string {
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
  const metres = options.metres !== undefined && options.metres > 0 ? options.metres : DEFAULT_METRES[kind];
  const lines: string[] = [opening(kind, what, medium, metres)];
  const avoid: string[] = [...(style?.artBible.avoid ?? [])];

  if (style) lines.push(`Style: ${style.name}. ${style.descriptor} ${style.artBible.direction}`);

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
  if (medium === "image") {
    const transparent = kind === "sprite" || kind === "icon" || kind === "portrait" || kind === "prop" || kind === "character";
    const tiling = kind === "tileset" || kind === "texture";
    let grid: { width: number; height: number } | null = null;
    const pixelArt = unit > 0 || sizes.length > 0;
    if (kind === "texture" && density > 0) {
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
    if (grid) {
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
      if (kind === "backdrop" || kind === "tileset" || kind === "texture") lines.push(`Line: shapes standing in front carry a ${width}-pixel ${named} line ${where}; a tile's edge that meets the next tile carries none.`);
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
  if (kind !== "texture" && kind !== "icon" && kind !== "portrait") {
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
  avoid.push("text, letters or watermarks");
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
    }
  }
  return { extends: ext, blocks };
}

/** An asset id as the cook keys it: `assets/` dropped. */
const assetId = (p: string) => p.replace(/^\.?\/?assets\//, "");

/** The flattened look: blocks merged over the chain (nearest wins, field by field), and the preset the chain reaches. */
function flatten(id: string, text: string, assets: string | null, presets: string): { blocks: Map<string, Map<string, ArtValue>>; preset: string | null } {
  const seen = [id];
  const chain: LookText[] = [parseLook(text, id)];
  let preset: string | null = null;
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
    if (base.preset && preset === null) preset = base.id;
    const parsed = parseLook(base.text, base.id);
    chain.push(parsed);
    from = base.id;
    next = parsed.extends;
  }
  const blocks = new Map<string, Map<string, ArtValue>>();
  for (const look of chain.reverse()) {
    for (const [name, fields] of look.blocks) {
      const merged = blocks.get(name) ?? new Map();
      for (const [k, v] of fields) merged.set(k, v);
      blocks.set(name, merged);
    }
  }
  return { blocks, preset };
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
function briefLook(blocks: Map<string, Map<string, ArtValue>>, preset: string | null, assets: string | null, presets: string, catalogue: string, profile: "2d" | "3d" | undefined): BriefLook {
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
  return { extends: preset, art: art as ArtBlocks, palette, style, ...(profile ? { profile } : {}), ...(preview ? { preview } : {}) };
}

export interface ReadLookOptions {
  /** The style catalogue; this copy's own when omitted. */
  readonly catalogue?: string;
  /** The presets' directory; this copy's own when omitted. */
  readonly presets?: string;
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
  const { blocks, preset } = flatten(id, readFileSync(path, "utf8"), assets, presets);
  return briefLook(blocks, preset, assets, presets, options.catalogue ?? catalogueFile(), profile ?? "3d");
}

/** A preset's look alone (`presetLook("16bit-pixel")`), for a game not written yet; its profile is the style's only one, else unset. */
export function presetLook(slug: string, options: ReadLookOptions = {}): BriefLook {
  const presets = options.presets ?? presetsDir();
  const path = join(presets, `${slug}.ogt`);
  if (!existsSync(path)) throw new Error(`no look preset '${slug}' in ${presets}`);
  const { blocks } = flatten(slug, readFileSync(path, "utf8"), null, presets);
  const look = briefLook(blocks, slug, null, presets, options.catalogue ?? catalogueFile(), undefined);
  const profiles = look.style?.look.profiles ?? [];
  return profiles.length === 1 ? { ...look, profile: profiles[0]! } : look;
}

// ---------------------------------------------------------------- the command line

const USAGE = `usage: bun brief.ts <kind> "<subject>" [--project <dir> | --look <slug>] [--reference <path>]... [--metres <n>] [--json]

Prints the request for one asset in the game's style (docs/art.md §2.2): the
prompt, then the references and the generator's parameters.
  kind         ${ART_KINDS.join(", ")}
  --project    the game whose project.ogt names its look (default: .)
  --look       a catalogue preset instead of a project
  --reference  an accepted asset of the game to pass as a reference (repeat)
  --metres     the asset's size in metres (a prop 1, a character 2)
  --json       the whole brief as JSON`;

function main(argv: string[]): number {
  const [kind, subject, ...rest] = argv;
  if (!kind || kind === "-h" || kind === "--help" || subject === undefined) {
    console.log(USAGE);
    return kind ? 0 : 1;
  }
  let project = ".";
  let preset: string | undefined;
  let metres: number | undefined;
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
    else if (a === "--reference") references.push(value());
    else if (a === "--metres") metres = Number(value());
    else if (a === "--json") json = true;
    else throw new Error(`unknown option ${a}. ${USAGE}`);
  }
  const look = preset !== undefined ? presetLook(preset) : readLook(project);
  const brief = artBrief(look, kind as ArtKind, subject, { references, ...(metres !== undefined ? { metres } : {}) });
  if (json) console.log(JSON.stringify(brief, null, 2));
  else {
    const params = brief.image ?? brief.model;
    console.log(`${brief.prompt}\n\n${brief.medium}: ${JSON.stringify(params)}\nreferences:${brief.references.map((r) => `\n  ${r.role}: ${r.path}`).join("") || " none"}`);
  }
  return 0;
}

if (import.meta.main) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (e) {
    console.error(`brief: ${e instanceof Error ? e.message : String(e)}`);
    process.exit(1);
  }
}
