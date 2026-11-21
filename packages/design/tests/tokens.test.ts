// The design language's pairs hold in both themes (design-language.md
// §Contrast): text at 4.5:1 and focus at 3:1 on what they are drawn on, a
// translucent ground composited as a browser does; text over glass at 4.5:1
// over the brightest and darkest scenes; and the `linear` fills draw in
// linear light what their CSS values draw in a browser.

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";
import tokens from "../tokens.json";

type Rgba = [number, number, number, number];
type Scheme = "light" | "dark";
const SCHEMES: Scheme[] = ["light", "dark"];

function parse(v: string): Rgba {
  const hex = /^#([0-9a-f]{6})$/i.exec(v);
  if (hex) {
    const n = parseInt(hex[1]!, 16);
    return [n >> 16, (n >> 8) & 255, n & 255, 1];
  }
  const rgb = /^rgb\((\d+) (\d+) (\d+)(?: \/ ([\d.]+))?\)$/.exec(v);
  if (!rgb) throw new Error(`not a colour: ${v}`);
  return [+rgb[1]!, +rgb[2]!, +rgb[3]!, rgb[4] === undefined ? 1 : +rgb[4]];
}

const colors = tokens.color as Record<string, string>;
const dark = tokens.dark.color as Record<string, string>;
function color(name: string, s: Scheme): Rgba {
  if (name.startsWith("#")) return parse(name);
  const v = (s === "dark" ? dark[name] : undefined) ?? colors[name];
  if (!v) throw new Error(`no colour ${name}`);
  return parse(v);
}
function linearColor(name: string, s: Scheme): Rgba {
  const l = tokens.linear;
  const v = (s === "dark" ? (l.dark.color as Record<string, string>)[name] : undefined) ?? (l.color as Record<string, string>)[name];
  if (!v) throw new Error(`no linear colour ${name}`);
  return parse(v);
}

const toLinear = (c: number) => {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
};
const toByte = (v: number) => 255 * (v <= 0.0031308 ? v * 12.92 : 1.055 * v ** (1 / 2.4) - 0.055);
const luminance = (c: Rgba) => 0.2126 * toLinear(c[0]) + 0.7152 * toLinear(c[1]) + 0.0722 * toLinear(c[2]);
const ratio = (a: Rgba, b: Rgba) => {
  const [x, y] = [luminance(a), luminance(b)];
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
};
/** `top` over an opaque `below` as a browser mixes it: in encoded sRGB. */
const css = (top: Rgba, below: Rgba): Rgba => [0, 1, 2].map((k) => top[k]! * top[3] + below[k]! * (1 - top[3])).concat(1) as Rgba;
/** `top` over an opaque `below` as the engine's UI pass blends a flat fill: in linear light. */
const linear = (top: Rgba, below: Rgba): Rgba =>
  [0, 1, 2].map((k) => toByte(toLinear(top[k]!) * top[3] + toLinear(below[k]!) * (1 - top[3]))).concat(1) as Rgba;

/** A ground as drawn: a translucent one over `under` (the ground `bg` when unnamed), as a browser does. */
function ground(name: string, under: string | undefined, s: Scheme): Rgba {
  const g = color(name, s);
  return g[3] < 1 ? css(g, ground(under ?? "bg", undefined, s)) : g;
}

test("every text pair reads 4.5:1 and every focus pair 3:1, in both themes", () => {
  const low: string[] = [];
  for (const s of SCHEMES) {
    for (const [ink, on, under] of tokens.contrast as [string, string, string?][]) {
      if (s === "dark" && (ink.startsWith("stage") || on.startsWith("stage"))) continue;
      const r = ratio(color(ink, s), ground(on, under, s));
      if (r < 4.5) low.push(`${s}: ${ink} on ${on}${under ? ` over ${under}` : ""} ${r.toFixed(2)}`);
    }
    for (const [ink, on] of tokens.nonText as [string, string][]) {
      const r = ratio(color(ink, s), ground(on, undefined, s));
      if (r < 3) low.push(`${s}: ${ink} on ${on} ${r.toFixed(2)} (non-text)`);
    }
  }
  expect(low).toEqual([]);
});

test("text over glass reads 4.5:1 over the brightest and darkest scenes, a selected row's fill included", () => {
  const low: string[] = [];
  for (const s of SCHEMES) {
    for (const [ink, material, under, fill] of tokens.glassContrast.pairs as [string, string, string, string?][]) {
      let g = css(color(material, s), parse(under));
      if (fill) g = linear(linearColor(fill, s), g);
      const r = ratio(color(ink, s), g);
      if (r < 4.5) low.push(`${s}: ${ink} on ${material} over ${under}${fill ? ` with ${fill}` : ""} ${r.toFixed(2)}`);
    }
  }
  expect(low).toEqual([]);
});

test("each linear fill draws in linear light what its CSS value draws in a browser over its ground", () => {
  const off: string[] = [];
  for (const s of SCHEMES) {
    for (const [name, over] of Object.entries(tokens.linear.over)) {
      const g = ground(over, undefined, s);
      const want = css(color(name, s), g);
      const got = linear(linearColor(name, s), g);
      const d = Math.abs(luminance(got) - luminance(want)) / Math.max(luminance(want), 1e-3);
      if (d > 0.02) off.push(`${s}: ${name} over ${over}: luminance ${luminance(got).toFixed(4)}, CSS ${luminance(want).toFixed(4)}`);
    }
  }
  expect(off).toEqual([]);
});

test("the dark scheme names only keys the light one has", () => {
  expect(Object.keys(dark).filter((k) => !(k in colors))).toEqual([]);
  expect(Object.keys(tokens.linear.dark.color).filter((k) => !(k in tokens.linear.color))).toEqual([]);
});

test("the chrome has no hue: the primary, its states, the highlight and focus are neutral; meaning keeps its colours", () => {
  // The owner, 2026-11-02: every game brings its own colours, so the default chrome is neutral and set apart by
  // fill and weight; the semantic and game roles (danger, success, warn, info, health, mana, xp, gold) keep theirs.
  const chroma = (c: Rgba) => Math.max(c[0], c[1], c[2]) - Math.min(c[0], c[1], c[2]);
  const hued: string[] = [];
  for (const s of SCHEMES) {
    for (const k of ["accent", "accent-hover", "accent-active", "accent-soft", "accent-fg", "accent-ink", "focus"]) {
      if (chroma(color(k, s)) > 8) hued.push(`${s}: ${k}`);
    }
    for (const k of ["danger", "success", "warn", "info", "health", "mana", "xp", "gold"]) {
      if (chroma(color(k, s)) < 60) hued.push(`${s}: ${k} lost its hue`);
    }
  }
  expect(hued).toEqual([]);
});

// High contrast (color.css `[data-contrast=more]`) as the engine resolves it:
// og-ui-next's default pack, each scheme's theme with its `contrast` layer
// over it. Text reads 7:1 on the theme's grounds and a control's edge, a track
// and a tile 3:1 on the surface and page they sit on (WCAG 1.4.6 and 1.4.11).
import pack from "../../../crates/og-ui-next/default/pack.json";

const MORE_TEXT = ["fg", "fg-muted", "fg-subtle"].flatMap((ink) =>
  ["bg", "bg-subtle", "surface", "surface-raised"].map((on) => [ink, on]),
);
const MORE_NON_TEXT = ["border", "track", "tile"].flatMap((edge) => ["bg", "surface"].map((on) => [edge, on]));
// A tile is also a chip's ground. Its 3:1 edge and 7:1 text on it would
// need 21:1 from page to ink (black to white), so text on a tile holds AA.
const MORE_ON_TILE = ["fg", "fg-muted"].map((ink) => [ink, "tile"]);

test("high contrast reads 7:1 for text and 3:1 for edges, tracks and tiles, in both themes", () => {
  type Entry = string | { light: string; dark: string };
  const theme = pack.theme.colors as Record<string, Entry>;
  const more = pack.contrast.colors as Record<string, Entry>;
  const pick = (e: Entry | undefined, s: Scheme) => (e === undefined ? undefined : typeof e === "string" ? e : e[s]);
  const hex = (v: string): Rgba => {
    const m = /^#([0-9a-f]{6})([0-9a-f]{2})?$/i.exec(v);
    if (!m) throw new Error(`not a colour: ${v}`);
    const n = parseInt(m[1]!, 16);
    return [n >> 16, (n >> 8) & 255, n & 255, m[2] ? parseInt(m[2], 16) / 255 : 1];
  };
  const get = (n: string, s: Scheme) => {
    const v = pick(more[n], s) ?? pick(theme[n], s);
    if (!v) throw new Error(`no colour ${n}`);
    const c = hex(v);
    return c[3] < 1 ? css(c, get("bg", s)) : c;
  };
  const low: string[] = [];
  for (const s of SCHEMES) {
    for (const [ink, on] of MORE_TEXT) {
      const r = ratio(get(ink!, s), get(on!, s));
      if (r < 7) low.push(`${s}: ${ink} on ${on} ${r.toFixed(2)}`);
    }
    for (const [edge, on] of MORE_NON_TEXT) {
      const r = ratio(get(edge!, s), get(on!, s));
      if (r < 3) low.push(`${s}: ${edge} on ${on} ${r.toFixed(2)} (non-text)`);
    }
    for (const [ink, on] of MORE_ON_TILE) {
      const r = ratio(get(ink!, s), get(on!, s));
      if (r < 4.5) low.push(`${s}: ${ink} on ${on} ${r.toFixed(2)}`);
    }
  }
  expect(low).toEqual([]);
});

test("docs/ui.md §3.1's token families exist in the tokens", () => {
  // The section's tables are pointers, not copies: they name each family and
  // what it controls, and the CSS owns every value and member list. What they
  // name must exist, so a family renamed or removed there fails here instead
  // of leaving the doc pointing at nothing. The doc-facing names are the theme
  // generator's aliases, so the set the doc can point at is the CSS's declared
  // and referenced names plus tokens.json's keys (the emitted hooks among
  // them); names no part of packages/design declares are written out in full,
  // never as a family.
  const names = new Set<string>();
  for (const file of readdirSync(join(import.meta.dir, "../tokens"))) {
    const text = readFileSync(join(import.meta.dir, "../tokens", file), "utf8");
    for (const m of text.matchAll(/(--[a-z0-9-]+)\s*:/g)) names.add(m[1]!.slice(2));
    for (const m of text.matchAll(/var\(\s*(--[a-z0-9-]+)/g)) names.add(m[1]!.slice(2));
  }
  const walk = (o: unknown) => {
    if (o === null || typeof o !== "object") return;
    for (const [k, v] of Object.entries(o)) {
      if (v !== null && typeof v === "object") walk(v);
      else if (!k.startsWith("--")) names.add(k);
    }
  };
  walk(tokens);
  const doc = readFileSync(join(import.meta.dir, "../../../docs/ui.md"), "utf8");
  const section = doc.slice(doc.indexOf("### 3.1"), doc.indexOf("### 3.2"));
  expect(section).toContain("Token family");
  const orphan = new Set<string>();
  for (const m of section.matchAll(/--[a-z0-9]+(?:-[a-z0-9]+)*-\*/g)) {
    const family = m[0].slice(2, -2);
    if (![...names].some(n => n === family || n.startsWith(`${family}-`))) orphan.add(m[0]);
  }
  expect([...orphan].sort()).toEqual([]);
});
