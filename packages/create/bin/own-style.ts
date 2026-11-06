// The game's own UI style (docs/ui.md §12.7): `og new` copies the style
// pack the game's look (or its creation choices) names into
// assets/styles/<game>/ and recolours the copy from the look's accent and
// the game's mood. The copy is the game's to restyle; `og ui check` reports
// it and `og build --publish` refuses it while it is identical to a built-in
// pack or still as generated. "Still as generated" is decided by content,
// never by time: the style's fingerprint (below) is written into its
// style.ogt and compared with the fingerprint of what is there now.
//
// The fingerprint is shared with og-cli (crates/og-cli/src/own_look.rs,
// which computes it the same way and holds the test vector equal):
//
//   canonical(text)  every line with its trailing white space cut, without
//                    blank lines and `#` comment lines, joined by "\n";
//                    a style's top-level `name` line is left out too, so a
//                    renamed copy is the same style
//   fingerprint(dir) FNV-1a 64 (lowercase hex, 16 digits) of the canonical
//                    style.ogt followed, for every quoted string in it that
//                    names a file (and that file's @2x and @3x), in order of
//                    first mention, by "\n<string> <FNV-1a 64 hex of the
//                    file's bytes>". A string names a file relative to the
//                    style's directory, or, starting `styles/`, an asset id
//                    under the assets directory two levels up (a game's
//                    assets/styles/<name>/); `@` ids (the built-in packs)
//                    and files that do not exist add nothing.

import { cpSync, existsSync, readdirSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** A game's mood (project.ogt's create block, `--mood`): how the copy is recoloured. */
export type Mood = "bright" | "dark" | "warm" | "cold" | "soft" | "high-contrast";

/** What `writeOwnStyle` wrote. */
export interface OwnStyle {
  /** The style's name: its directory under assets/styles/ and its `name`. */
  readonly name: string;
  /** The style's asset id, as a look's `ui` line names it. */
  readonly id: string;
  /** What it was copied from (`the built-in pixel pack`, `the template's sparkorb style`). */
  readonly from: string;
  /** Its fingerprint as written: unchanged while the style's fingerprint is this. */
  readonly fingerprint: string;
  /** The files written, relative to the game. */
  readonly files: readonly string[];
}

// ── the fingerprint ────────────────────────────────────────────────────────

/** FNV-1a 64 of `bytes` in lowercase hex (og_core::hash::hash_bytes). */
export function fnv64(bytes: Uint8Array): string {
  // The 64-bit state as two 32-bit halves: times the prime 2^40 + 0x1b3.
  let hi = 0xcbf29ce4, lo = 0x84222325;
  for (let i = 0; i < bytes.length; i++) {
    lo = (lo ^ bytes[i]!) >>> 0;
    const low = lo * 0x1b3;
    const carry = Math.floor(low / 0x100000000);
    const nlo = low >>> 0;
    hi = (hi * 0x1b3 + carry + ((lo << 8) >>> 0)) % 0x100000000;
    lo = nlo;
  }
  return hi.toString(16).padStart(8, "0") + lo.toString(16).padStart(8, "0");
}

/** An .ogt's canonical text: no comments, blank lines or trailing white space; with `dropName`, no top-level `name` line. */
export function canonical(text: string, dropName = false): string {
  return text
    .split("\n")
    .map((l) => l.replace(/[ \t\r]+$/, ""))
    .filter((l) => l.trim() !== "" && !l.trim().startsWith("#") && !(dropName && /^name[ \t]/.test(l)))
    .join("\n");
}

/** File extensions a quoted string must end in to name a file the fingerprint covers. */
const FILE = /\.(png|jpe?g|webp|ttf|otf|ogt|wav|ogg|mp3|svg)$/i;

/** The style in `dir`'s fingerprint (see the header); null without a style.ogt. */
export function styleFingerprint(dir: string): string | null {
  const file = join(dir, "style.ogt");
  if (!existsSync(file)) return null;
  const text = canonical(readFileSync(file, "utf8"), true);
  let out = text;
  const seen = new Set<string>();
  for (const m of text.matchAll(/"((?:[^"\\]|\\.)*)"/g)) {
    const s = m[1]!;
    if (s.startsWith("@") || !FILE.test(s)) continue;
    const dot = s.lastIndexOf(".");
    for (const rel of [s, `${s.slice(0, dot)}@2x${s.slice(dot)}`, `${s.slice(0, dot)}@3x${s.slice(dot)}`]) {
      if (seen.has(rel)) continue;
      seen.add(rel);
      const path = [join(dir, rel), ...(rel.startsWith("styles/") ? [join(dir, "..", "..", rel)] : [])].find((p) => existsSync(p) && statSync(p).isFile());
      if (path) out += `\n${rel} ${fnv64(new Uint8Array(readFileSync(path)))}`;
    }
  }
  return fnv64(new TextEncoder().encode(out));
}

/** The provenance line `writeOwnStyle` puts after a style's `---` (og-cli reads its fingerprint). */
export function provenance(from: string, fingerprint: string): string {
  return `# og:generated from ${from}, fingerprint ${fingerprint}: the game's own style, a starting point to restyle. og ui check reports it and og build --publish refuses it until it changes (palette first, then the frames and buttons in art/, fonts and icons; docs/ui.md §12.7).`;
}

// ── colour ─────────────────────────────────────────────────────────────────

type Lch = { l: number; c: number; h: number };
const toLinear = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
const toSrgb = (v: number) => (v <= 0.0031308 ? v * 12.92 : 1.055 * v ** (1 / 2.4) - 0.055);

function hexToLch(hex: string): Lch {
  const n = (k: number) => toLinear(parseInt(hex.slice(1 + 2 * k, 3 + 2 * k), 16) / 255);
  return linearToLch(n(0), n(1), n(2));
}

/** Linear RGB to OKLCH (Björn Ottosson's OKLab). */
function linearToLch(r: number, g: number, b: number): Lch {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  return { l: L, c: Math.hypot(A, B), h: ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360 };
}

function lchToRgb({ l: L, c, h }: Lch): [number, number, number] {
  const A = c * Math.cos((h * Math.PI) / 180), B = c * Math.sin((h * Math.PI) / 180);
  const l = (L + 0.3963377774 * A + 0.2158037573 * B) ** 3;
  const m = (L - 0.1055613458 * A - 0.0638541728 * B) ** 3;
  const s = (L - 0.0894841775 * A - 1.291485548 * B) ** 3;
  return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
}

/** The colour inside sRGB (its chroma cut until it is), linear RGB 0 to 1. */
function lchToLinear(x: Lch): [number, number, number] {
  const inside = (rgb: number[]) => rgb.every((v) => v >= -1e-4 && v <= 1 + 1e-4);
  let c = x.c;
  if (!inside(lchToRgb({ ...x, c }))) {
    let lo = 0;
    for (let i = 0; i < 24; i++) {
      const mid = (lo + c) / 2;
      if (inside(lchToRgb({ ...x, c: mid }))) lo = mid;
      else c = mid;
    }
    c = lo;
  }
  return lchToRgb({ ...x, l: Math.min(1, Math.max(0, x.l)), c }).map((v) => Math.min(1, Math.max(0, v))) as [number, number, number];
}

/** The colour back in hex, its chroma cut until it is inside sRGB. */
function lchToHex(x: Lch): string {
  return `#${lchToLinear(x).map((v) => Math.round(toSrgb(v) * 255).toString(16).padStart(2, "0")).join("")}`;
}

/** `h` turned toward `to` by `t` of the shorter way round. */
function toward(h: number, to: number, t: number): number {
  const d = ((to - h + 540) % 360) - 180;
  return (h + d * t + 360) % 360;
}

/** The palette roles a recolour moves: the accent family and the chrome's neutrals. Semantic and game roles keep their hues (§12.1). */
const ACCENT = new Set(["primary", "primaryHover", "primaryActive", "primarySoft", "highlight", "focus"]);
const SURFACE = new Set(["backdrop", "well", "panel", "panelHover", "panelRaised", "control", "controlStrong", "knob", "hud"]);
const INK = new Set(["ink", "inkMuted", "inkFaint"]);
const NEUTRAL = new Set([...SURFACE, ...INK, "line", "lineStrong", "inkInverse"]);

/**
 * A style's text with its palette recoloured: the accent family turned to
 * the look's accent hue (its lightness kept, so the text on it keeps its
 * contrast), the neutrals half way toward it, then the mood: bright lifts
 * the accent's chroma, dark deepens the grounds, warm and cold turn the
 * neutrals toward orange or blue, soft lowers the chroma, high-contrast
 * spreads the grounds and the ink apart. Other lines stay: a dressing's
 * `tone` is what og cook measured of its painted art (an authored slice
 * keeps its own colours), so painted frames and buttons change only when
 * the game repaints them (`og style art`).
 */
export function recolour(style: string, accent: string | null, mood: Mood | null): string {
  const dark = !/^scheme[ \t]+light/m.test(style);
  const a = accent ? hexToLch(accent) : null;
  const move = (key: string, from: Lch): Lch => {
    let x = { ...from };
    if (a && ACCENT.has(key)) x = { l: x.l, c: (x.c + a.c) / 2, h: a.h };
    else if (a && x.c > 0.004) x = { ...x, h: toward(x.h, a.h, 0.5) };
    const surface = SURFACE.has(key), ink = INK.has(key), acc = ACCENT.has(key);
    switch (mood) {
      case "bright":
        if (acc) x.c *= 1.15;
        if (surface) x.l = dark ? x.l + 0.02 : Math.min(0.99, x.l + 0.01);
        break;
      case "dark":
        if (surface) x.l *= dark ? 0.85 : 0.97;
        x.c *= acc ? 0.85 : 0.8;
        break;
      case "warm":
      case "cold": {
        const [n, ac] = mood === "warm" ? [70, 55] : [250, 235];
        if (acc) x.h = toward(x.h, ac, 0.25);
        else {
          x.h = x.c > 0.004 ? toward(x.h, n, 0.35) : n;
          if (surface) x.c = Math.max(x.c, 0.012);
        }
        break;
      }
      case "soft":
        x.c *= 0.7;
        break;
      case "high-contrast":
        if (surface) x.l = dark ? x.l * 0.8 : Math.min(0.99, x.l + 0.02);
        if (ink) x.l = dark ? Math.min(0.98, x.l + 0.04) : x.l * 0.9;
        if (acc) x.c *= 1.1;
        break;
      default:
        break;
    }
    return x;
  };
  let inPalette = false;
  return style
    .split("\n")
    .map((line) => {
      if (/^palette[ \t]*\{/.test(line)) inPalette = true;
      else if (inPalette && line.trim() === "}") inPalette = false;
      const m = inPalette ? /^([ \t]+)([A-Za-z]+)([ \t]+)"#([0-9a-fA-F]{6})([0-9a-fA-F]{2})?"(.*)$/.exec(line) : null;
      if (!m || (!ACCENT.has(m[2]!) && !NEUTRAL.has(m[2]!))) return line;
      return `${m[1]}${m[2]}${m[3]}"${lchToHex(move(m[2]!, hexToLch(`#${m[4]}`)))}${m[5] ?? ""}"${m[6]}`;
    })
    .join("\n");
}

// ── writing the copy ───────────────────────────────────────────────────────

/** The style name for a game id: the id, or `game-<id>` for one that does not start with a letter (an extension's name must). */
export function styleName(id: string): string {
  return /^[a-z]/.test(id) ? id : `game-${id}`;
}

function files(root: string, at = root): string[] {
  return readdirSync(at, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? files(root, join(at, e.name)) : [join(at, e.name).slice(root.length + 1)]));
}

/**
 * Copies the style in `source` into `<game>/assets/styles/<name>/` as the
 * game's own: renamed, recoloured (`recolour`), its extension manifest
 * renamed with it, and the provenance line with its fingerprint after its
 * `---`. An existing directory of that name is replaced. `renamed`, the
 * name of a style of the game's own the copy is made from: its asset ids
 * (`styles/<renamed>/atlas.png`) name the copy's files instead.
 */
export function writeOwnStyle(game: string, source: string, name: string, from: string, accent: string | null, mood: Mood | null, title: string, renamed: string | null = null): OwnStyle {
  if (!existsSync(join(source, "style.ogt"))) throw new Error(`no style.ogt in ${source}`);
  const target = join(game, "assets", "styles", name);
  const staging = `${target}.og-new-${process.pid}`;
  rmSync(staging, { recursive: true, force: true });
  cpSync(source, staging, { recursive: true });
  const styleFile = join(staging, "style.ogt");
  // A copy of a copy keeps one provenance line: its own.
  let text = readFileSync(styleFile, "utf8").replace(/^# og:generated from .*\n/m, "");
  if (renamed !== null) text = text.replaceAll(`"styles/${renamed}/`, `"styles/${name}/`);
  text = /^name[ \t]+"[^"]*"/m.test(text) ? text.replace(/^name[ \t]+"[^"]*"/m, `name "${name}"`) : text.replace(/^---\n/m, `---\nname "${name}"\n`);
  writeFileSync(styleFile, recolour(text, accent, mood));
  const manifest = join(staging, "extension.json");
  if (existsSync(manifest)) {
    const m = JSON.parse(readFileSync(manifest, "utf8")) as Record<string, unknown>;
    writeFileSync(manifest, `${JSON.stringify({ ...m, name, version: "0.1.0", description: `${title}'s own style, from ${from}` }, null, 2)}\n`);
  }
  rmSync(target, { recursive: true, force: true });
  renameSync(staging, target);
  // Taken in place: its asset ids name files under assets/styles/<name>/.
  const fingerprint = styleFingerprint(target)!;
  const placed = join(target, "style.ogt");
  writeFileSync(placed, readFileSync(placed, "utf8").replace(/^---\n/m, `---\n${provenance(from, fingerprint)}\n`));
  return { name, id: `styles/${name}/style.ogt`, from, fingerprint, files: files(target).sort().map((f) => `assets/styles/${name}/${f}`) };
}
