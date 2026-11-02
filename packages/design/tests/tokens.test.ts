// The design language's pairs hold in both themes (design-language.md
// §Contrast): text at 4.5:1 and focus at 3:1 on what they are drawn on, a
// translucent ground composited as a browser does; text over glass at 4.5:1
// over the brightest and darkest scenes; and the `linear` fills draw in
// linear light what their CSS values draw in a browser.

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
