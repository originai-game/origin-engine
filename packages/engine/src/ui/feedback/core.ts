// What the feedback, HUD and world components (docs/ui.md §11, U6c) share:
// parts a style dresses, colours from the frame's style, fills, strokes
// and effects in the UI pass's types, text over the game, motion by the
// tick, edges kept in the UI's memory (so a snapshot carries them), and
// number formatting. Everything here is built from the public og.ui
// members; nothing keeps state outside the UI state.

import type * as Api from "@origingame/engine";
import { kit } from "../../ui-kit/bind.ts";
import { all, fit, frame, grow, pct, px, token, xy, type Tokens } from "../../ui-kit/util.ts";

export { all, fit, frame, grow, kit, pct, px, token, xy };
export type { Tokens };

export type Paint = Api.Paint;
export type Pt = { x: number; y: number };

export const TAU = Math.PI * 2;
export const clamp = (v: number, lo: number, hi: number): number => (v < lo ? lo : v > hi ? hi : v);
export const lerp = (a: number, b: number, t: number): number => a + (b - a) * t;
export const clamp01 = (v: number): number => clamp(v, 0, 1);

// ── parts ─────────────────────────────────────────────────────────────────

type Qualifier = string | false | null | undefined | readonly [string, string | number];

/** A part selector a style's `dress` block matches: `Meter.fill[segmented][tone=danger]`. */
export function sel(component: string, part: string, ...quals: Qualifier[]): string {
  let s = `${component}.${part}`;
  for (const q of quals) {
    if (!q) continue;
    if (typeof q === "string") s += `[${q}]`;
    else s += `[${q[0]}=${q[1]}]`;
  }
  return s;
}

export type ElementClass = "panel" | "dialog" | "card" | "tooltip" | "button" | "field" | "chip" | "badge" | "tab" | "slot" | "bar" | "hud" | "avatar";

export interface PartDesc extends Api.ElementDesc {
  /** The element class an undressed part takes its corner and elevation from. */
  partClass?: ElementClass;
}

/**
 * Opens a container that is a part, drawn by `draw` (the component's own
 * look, which a style dressing the part restyles), and answers its full path.
 */
export function open(id: string, desc: PartDesc, draw?: Api.DrawDesc): string {
  kit.open(id, desc);
  dressOrBare(desc, draw);
  return kit.openPath();
}

/**
 * A part's own draw, which a style dressing the part restyles. A part
 * without one (a layout box that only names itself for a style) draws
 * nothing until a style dresses it: without this, og-ui gives a named
 * part its class's undressed hairline, which reads as debug bounds round
 * a crosshair or a column of dials.
 */
function dressOrBare(desc: PartDesc, draw: Api.DrawDesc | undefined): void {
  if (draw) kit.setDraw({ ...draw, dressable: true });
  else if (desc.part) kit.setDraw({ dressable: true });
}

/** Closes what `open` opened, then gives it the element model's props. */
/**
 * Element props, plus what the element says to a screen reader: U6b's
 * og.ui.semantics, applied to the same element after the props.
 */
export type Props = Api.ElementProps & { a11y?: Api.SemanticsDesc };

/** Gives the element just built its props, then its semantics. */
export function apply(p: Props | undefined): void {
  if (!p) return;
  const { a11y: sem, ...rest } = p;
  if (Object.keys(rest).length > 0) kit.props(rest);
  if (sem) kit.semantics(sem);
}

export function close(props?: Props): Api.Response {
  const r = kit.close() as Api.Response;
  apply(props);
  return r;
}

/** A state group whose states carry no values: shown in dump_ui and forced by the probe. */
export function groups(current: Record<string, string>, all: Record<string, readonly string[]>): Pick<Api.ElementProps, "states" | "state"> {
  return {
    states: Object.entries(all).map(([name, states]) => ({ name, states: states.map((s) => ({ name: s, values: [], onEnter: "", onExit: "" })), transitions: [] })),
    state: Object.entries(current).map(([group, state]) => ({ group, state })),
  };
}

/** A part with children: open, build, close (even when build throws), props. */
export function box(id: string, desc: PartDesc, draw: Api.DrawDesc | undefined, build: (path: string) => void, props?: Props): Api.Response {
  const path = open(id, desc, draw);
  let r: Api.Response;
  try {
    build(path);
  } finally {
    r = close(props);
  }
  return r;
}

/** A part without children (a fill, a ring, a pip). */
export function leaf(id: string, desc: PartDesc, draw?: Api.DrawDesc, props?: Props): void {
  kit.open(id, desc);
  dressOrBare(desc, draw);
  kit.close();
  apply(props);
}

/** An element placed inside its parent's box by insets, out of the flow. */
export function absolute(insets: Api.Edges, more: Props = {}): Props {
  return { position: "absolute", insets, ...more };
}

/** What an element says to a screen reader (og.ui.semantics); empty texts keep what it shows. */
export function a11y(role: string, name: string, more: Partial<Api.SemanticsDesc> = {}): Api.SemanticsDesc {
  const d: Record<string, unknown> = { role, ...more };
  if (name) d.name = name;
  for (const k of ["name", "description", "value"] as const) if (d[k] === "") delete d[k];
  return d as Api.SemanticsDesc;
}

/** Runs `build` in an anchored group at `anchor`, or in place when there is none. */
export function placed(id: string, anchor: Api.Anchor | null | undefined, desc: Api.AnchorDesc, build: () => void): void {
  if (anchor) kit.anchor(id, { ...desc, anchor }, build);
  else build();
}

// ── colours ───────────────────────────────────────────────────────────────

export interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

/** `#rrggbb` or `#rrggbbaa` as sRGB components 0–1. */
export function hex(s: string | undefined, fallback: Rgba = { r: 1, g: 1, b: 1, a: 1 }): Rgba {
  if (!s || s[0] !== "#" || (s.length !== 7 && s.length !== 9)) return fallback;
  const n = (i: number) => Number.parseInt(s.slice(i, i + 2), 16) / 255;
  return { r: n(1), g: n(3), b: n(5), a: s.length === 9 ? n(7) : 1 };
}

/** A palette role of the frame's style as sRGB. */
export function role(name: Api.ThemeColor): Rgba {
  return hex(frame().theme.palette[name]);
}

/** A literal colour as a Paint. */
export function lit(c: Rgba, alpha = 1): Paint {
  return { color: { r: c.r, g: c.g, b: c.b }, alpha: clamp01(c.a * alpha) };
}

/** Mixes two colours in sRGB (enough for flashes between two style colours). */
export function mix(a: Rgba, b: Rgba, t: number): Rgba {
  return { r: lerp(a.r, b.r, t), g: lerp(a.g, b.g, t), b: lerp(a.b, b.b, t), a: lerp(a.a, b.a, t) };
}

/** Relative luminance (WCAG) of an sRGB colour. */
export function luminance(c: Rgba): number {
  const ch = (v: number) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b);
}

/** Black or white ink, whichever reads better on `c`. */
export function inkOn(c: Rgba): Rgba {
  return luminance(c) > 0.36 ? { r: 0.06, g: 0.06, b: 0.08, a: 1 } : { r: 1, g: 1, b: 1, a: 1 };
}

/** The palette role a tone draws in. */
export function toneRole(tone: Api.Tone | undefined, neutral: Api.ThemeColor = "primary"): Api.ThemeColor {
  switch (tone) {
    case "success": return "success";
    case "warn": return "warn";
    case "danger": return "danger";
    case "info": return "info";
    case "primary": return "primary";
    case "inverse": return "ink";
    default: return neutral;
  }
}

const RARITY_FALLBACK: Record<string, string> = {
  common: "#b8bec7", uncommon: "#4cc35a", rare: "#3d8bff", epic: "#a64dff", legendary: "#ff9a1f", mythic: "#ff3d6e",
};

/** A rarity tier's colour from the style's tokens. */
export function rarityColor(r: Api.Rarity | undefined): Rgba | null {
  if (!r || r === "none") return null;
  const tokens = frame().theme.tokens as { rarity?: Record<string, string> } | null;
  return hex(tokens?.rarity?.[r] ?? RARITY_FALLBACK[r]);
}

/** How many pips a rarity carries (so no tier is told by colour alone). */
export function rarityPips(r: Api.Rarity | undefined): number {
  return ["none", "common", "uncommon", "rare", "epic", "legendary", "mythic"].indexOf(r ?? "none");
}

/** The style's team colour k (its eight), else a spread of hues. */
export function teamColor(k: number): Rgba {
  const tokens = frame().theme.tokens as { team?: string[] } | null;
  const list = tokens?.team ?? [];
  if (list.length > 0) return hex(list[((k % list.length) + list.length) % list.length]);
  const hues = ["#3d8bff", "#ff4d4d", "#3ccf6e", "#ffc23d", "#b45cff", "#2fd3d3", "#ff8a3d", "#e85cbc"];
  return hex(hues[((k % 8) + 8) % 8]);
}

// ── fills, strokes, effects ───────────────────────────────────────────────

export const solid = (color: Paint): Api.Fill => ({ kind: "solid", color });

/** A linear gradient top to bottom (or along `angle` degrees clockwise from down). */
export function vgrad(stops: Array<[number, Paint]>, angle = 0): Api.Fill {
  const a = (angle * Math.PI) / 180;
  const dx = Math.sin(a) * 0.5;
  const dy = Math.cos(a) * 0.5;
  return {
    kind: "gradient",
    type: "linear",
    from: { x: 0.5 - dx, y: 0.5 - dy },
    to: { x: 0.5 + dx, y: 0.5 + dy },
    stops: stops.map(([offset, color]) => ({ offset, color })),
  };
}

export function radial(stops: Array<[number, Paint]>, cx = 0.5, cy = 0.5, rx = 0.5, ry = 0.5): Api.Fill {
  return { kind: "gradient", type: "radial", from: { x: cx, y: cy }, to: { x: rx, y: ry }, stops: stops.map(([offset, color]) => ({ offset, color })) };
}

export function conic(stops: Array<[number, Paint]>, start = -Math.PI / 2): Api.Fill {
  return { kind: "gradient", type: "conic", from: { x: 0.5, y: 0.5 }, to: { x: start, y: 0 }, stops: stops.map(([offset, color]) => ({ offset, color })) };
}

export function stroke(width: number, paint: Paint, align: Api.StrokeAlign = "inside", more: Partial<Api.Stroke> = {}): Api.Stroke {
  return { width, align, paint: solid(paint), ...more };
}

export const drop = (dy: number, blur: number, color: Paint, spread = 0): Api.Effect => ({ kind: "dropShadow", offset: { x: 0, y: dy }, blur, spread, color });
export const inner = (dy: number, blur: number, color: Paint, spread = 0): Api.Effect => ({ kind: "innerShadow", offset: { x: 0, y: dy }, blur, spread, color });
export const glow = (blur: number, color: Paint, spread = 0): Api.Effect => ({ kind: "outerGlow", blur, spread, color });
export const innerGlow = (blur: number, color: Paint, spread = 0): Api.Effect => ({ kind: "innerGlow", blur, spread, color });

/** A rect cornered `r` with the style's corner for `cls` unless one is given. */
export function rect(r: number | [number, number, number, number], corner?: Api.CornerShape): Api.DrawShape {
  return { kind: "rect", radii: typeof r === "number" ? [r, r, r, r] : r, ...(corner ? { corner } : {}) };
}

/** The corner shape and radius the style gives an element class. */
export function shapeOf(cls: ElementClass): { corner: Api.CornerShape; radius: number } {
  const tokens = frame().theme.tokens as { shape?: Record<string, { corner: string; radius: number }> } | null;
  const s = tokens?.shape?.[cls];
  const corner = (s?.corner === "square" ? "round" : (s?.corner ?? "round")) as Api.CornerShape;
  return { corner, radius: s?.corner === "square" ? 0 : (s?.radius ?? frame().t.control) };
}

/** A ring arc: `fraction` of a turn clockwise from the top. */
export function arc(fraction: number, thickness: number, cap: Api.LineCap = "round", start = -Math.PI / 2): Api.DrawShape {
  return { kind: "arc", start, sweep: Math.max(0.0001, fraction) * TAU, thickness, cap };
}

/** The HUD's text treatment: an outline and a soft shadow in the style's shadow, so text reads over any game. */
export function hudText(strength = 1): Api.TextEffectsDesc {
  return {
    outline: { width: 2 * strength, color: token("shadow", 0.85), align: "outside" },
    shadows: [{ offset: { x: 0, y: 1.5 }, blur: 3, color: token("shadow", 0.6) }],
  };
}

export interface TextDesc extends Api.LabelDesc {
  /** Draw it with the HUD's outline and shadow. */
  outline?: boolean | number;
  /** A gradient over its glyphs. */
  gradient?: Api.Gradient;
}

/** A label, optionally outlined for the HUD. */
export function text(id: string, s: string, desc: TextDesc = {}, props?: Props): void {
  const { outline, gradient, ...label } = desc;
  kit.label(id, s, label);
  if (outline || gradient) {
    const fx = outline ? hudText(typeof outline === "number" ? outline : 1) : {};
    kit.setDraw({ text: gradient ? { ...fx, fill: gradient } : fx });
  }
  apply(props);
}

// ── sizes by the screen ───────────────────────────────────────────────────

/**
 * The HUD's scale: 1 on a 1280×720 surface (and on touch, which lays out
 * in its own CSS pixels), growing with a larger one up to its 16:9 part,
 * so a HUD keeps its proportion at 1080p and 4K and an ultrawide's extra
 * width stays margin.
 */
export function scale(): number {
  const v = frame().viewport;
  if (v.coarsePointer) return 1;
  return Math.max(1, Math.min(v.width, v.height * (16 / 9)) / 1280);
}

/** The player's text size in effect (UiDesc.textScale), 1 to 2. */
export function textScale(): number {
  return Math.min(2, Math.max(1, frame().theme.textScale || 1));
}

/** A length that holds text (a row's height, a readout's box): the design's, grown with the player's text size. */
export const ut = (n: number): number => Math.round(n * scale() * textScale());

/** A length of a 1280×720 design at this surface. */
export const u = (n: number): number => Math.round(n * scale());

/** The smallest text the checks allow here (docs/ui.md §15.2: 18 px at 1080p on desktop). */
export function minText(): number {
  const v = frame().viewport;
  if (v.coarsePointer) return 12;
  return Math.max(12, Math.round(((18 * Math.min(v.width, v.height * (16 / 9))) / 1920) * 10) / 10);
}

const TEXT_STEPS: readonly Api.TypeStepName[] = ["xs", "caption", "sm", "reading", "base", "lg", "xl", "xl2", "xl3", "xl4", "xl5", "xl6", "xl7", "xl8"];
const DISPLAY_STEPS: readonly Api.TypeStepName[] = ["displaySm", "displayMd", "displayLg", "displayXl", "displayXl2", "xl7", "xl8"];

/**
 * The type step for text of `n` px in a 1280×720 design: scaled with the
 * surface, never under the minimum, the smallest of the style's steps
 * that reaches it.
 */
export function ts(n: number, display = false): Api.TypeStepName {
  const ramp = frame().theme.typeRamp;
  // The ramp already carries the player's text size, so the size wanted
  // does too: at 200 % a label takes a step twice as large, not the same one.
  const want = Math.max(n * scale(), minText()) * textScale();
  const list = display && n >= 28 ? DISPLAY_STEPS : TEXT_STEPS;
  let best: Api.TypeStepName = list[list.length - 1]!;
  let bestSize = Infinity;
  for (const st of list) {
    const size = ramp[st]?.size ?? 0;
    if (size >= want - 0.01 && size < bestSize) {
      best = st;
      bestSize = size;
    }
  }
  return best;
}

/**
 * The type step for text that sits inside a fixed mark (a slot's
 * cooldown, a dial's readout, an icon's fallback letter): ts(n), grown
 * with the player's text size, but never over `maxPx`, so it stays inside
 * the art it labels rather than spilling past it.
 */
export function tsFit(n: number, maxPx: number, display = false): Api.TypeStepName {
  const best = ts(n, display);
  const ramp = frame().theme.typeRamp;
  if ((ramp[best]?.size ?? 0) <= maxPx) return best;
  const list = display && n >= 28 ? DISPLAY_STEPS : TEXT_STEPS;
  let fit: Api.TypeStepName = list[0]!;
  let fitSize = -Infinity;
  for (const st of [...TEXT_STEPS, ...(display ? DISPLAY_STEPS : [])]) {
    const size = ramp[st]?.size ?? 0;
    if (size <= maxPx && size > fitSize) {
      fit = st;
      fitSize = size;
    }
  }
  return fit;
}

/** A type step's size and line height in the frame's style. */
export function stepSize(step: Api.TypeStepName): { size: number; line: number } {
  const s = frame().theme.typeRamp[step];
  return { size: s?.size ?? 16, line: s?.lineHeight ?? 22 };
}

/**
 * An icon of the atlas, else a mark drawn from shapes in its place
 * (never a grey box): a polygon with the name's first letter.
 */
export function icon(id: string, name: string, size: number, tint: Paint, label = ""): void {
  if (name && kit.hasIcon(name)) {
    kit.icon(id, name, { size, tint, label });
    return;
  }
  box(id, { width: px(size), height: px(size), align: "center", cross: "center" }, {
    shape: { kind: "polygon", sides: 6, rounding: size * 0.12, rotation: Math.PI / 6 },
    fills: [solid({ ...tint, alpha: (tint.alpha ?? 1) * 0.22 })],
    strokes: [stroke(Math.max(1.5, size / 14), tint)],
  }, () => {
    if (name) text("Mark", name.slice(0, 1).toUpperCase(), { step: tsFit(size >= 30 ? 16 : 13, size * 0.6), color: tint, nowrap: true, font: "display" });
  }, label ? { a11y: a11y("image", label) } : undefined);
}

// ── time and motion, by the tick ──────────────────────────────────────────

/** The UI's clock: frames built. */
export const now = (): number => kit.tick();

/** Frames a second (the tick rate). */
export function tps(): number {
  const t = kit.tick();
  const s = kit.seconds();
  return t > 0 && s > 0 ? Math.round(t / s) : 60;
}

export const msToTicks = (ms: number): number => Math.max(1, Math.round((ms / 1000) * tps()));
export const ticksToMs = (ticks: number): number => (ticks / tps()) * 1000;

/** Reduced motion: movement jumps, decorative loops stop. */
export const reduced = (): boolean => frame().theme.reducedMotion;

export interface Edge {
  /** The value changed this tick. */
  changed: boolean;
  /** Ticks since it last changed (a large number before it ever did). */
  since: number;
  /** The value before the last change. */
  prev: number;
}

/**
 * Watches a number under the open element (the UI's memory, in the
 * snapshot): whether it changed this tick, how long ago it last did, and
 * what it was before.
 */
export function edge(id: string, value: number): Edge {
  const t = now();
  const last = kit.memory(`${id}~v`, value);
  const at = kit.memory(`${id}~t`, -1e6);
  const prev = kit.memory(`${id}~p`, value);
  if (last !== value) {
    kit.remember(`${id}~v`, value);
    kit.remember(`${id}~t`, t);
    kit.remember(`${id}~p`, last);
    return { changed: true, since: 0, prev: last };
  }
  return { changed: false, since: t - at, prev };
}

/** A pop: 1 rising to about 1 + amount, a small dip under 1 and settling, over `ms`. */
export function pop(sinceTicks: number, amount = 0.18, ms = 320): number {
  if (reduced()) return 1;
  const x = ticksToMs(sinceTicks) / ms;
  if (x >= 1 || x < 0) return 1;
  return 1 + amount * 2.2 * Math.exp(-4 * x) * Math.sin(TAU * 1.25 * x);
}

/** Ease out cubic. */
export const easeOut = (x: number): number => 1 - (1 - clamp01(x)) ** 3;
/** Ease in-out cubic. */
export const easeInOut = (x: number): number => {
  const t = clamp01(x);
  return t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
};
/** Ease out back: arrives past its end and settles. */
export const easeBack = (x: number, s = 1.7): number => {
  const t = clamp01(x) - 1;
  return 1 + (s + 1) * t * t * t + s * t * t;
};

/** A loop's phase 0–1 at `hz` on the UI clock (frozen under reduced motion). */
export function phase(hz: number): number {
  if (reduced()) return 0;
  const s = kit.seconds() * hz;
  return s - Math.floor(s);
}

/** A soft pulse 0–1 at `hz`. */
export function pulse(hz: number): number {
  return 0.5 - 0.5 * Math.cos(phase(hz) * TAU);
}

/** A number on the style's spring under the open element (og.ui.animate). */
export function spring(id: string, target: number, token: Api.MotionToken = "ui"): number {
  return kit.animate(id, target, { kind: "token", token });
}

/** A number on a spring of its own. */
export function springTo(id: string, target: number, durationMs: number, bounce = 0): number {
  return kit.animate(id, target, { kind: "spring", durationMs, bounce });
}

/** Plays a feedback event once on the tick `cond` holds. */
export function sound(event: string, cond: boolean, rise = 0): void {
  if (cond) kit.feedback(event, rise);
}

// ── numbers ───────────────────────────────────────────────────────────────

const SHORT = ["", "K", "M", "B", "T"];

/** Idle letters after T: aa, ab, … az, ba, … zz, aaa … */
function letters(n: number): string {
  let k = n;
  let s = "";
  let len = 2;
  let span = 26 ** len;
  while (k >= span) {
    k -= span;
    len += 1;
    span = 26 ** len;
  }
  for (let i = 0; i < len; i++) {
    s = String.fromCharCode(97 + (k % 26)) + s;
    k = Math.floor(k / 26);
  }
  return s;
}

function group(intText: string, sep: string): string {
  return sep ? intText.replace(/\B(?=(\d{3})+(?!\d))/g, sep) : intText;
}

function fixed(v: number, decimals: number, point: string): string {
  const s = v.toFixed(decimals);
  return point === "." ? s : s.replace(".", point);
}

/** Three significant digits: 1.23, 12.3, 123. */
function sig3(m: number, decimals: number | undefined): number {
  if (decimals !== undefined) return decimals;
  return m < 10 ? 2 : m < 100 ? 1 : 0;
}

/**
 * og.ui.formatNumber: a number in a notation. Exact for the integers a
 * double holds; past 1e308 through `exponent`. Pure arithmetic and
 * toFixed, so every host writes the same text.
 */
export function formatNumber(value: number, format: Api.NumberFormatDesc = {}): string {
  const notation = format.notation ?? "grouped";
  const sep = format.group ?? ",";
  const point = format.point ?? ".";
  const pre = format.prefix ?? "";
  const post = format.suffix ?? "";
  if (!Number.isFinite(value)) return `${pre}—${post}`;
  const dec = format.decimals ?? undefined;
  const shift = format.exponent ?? 0;
  // The value as a mantissa in [1, 10) and a base-10 exponent.
  let mant = Math.abs(value);
  let exp10 = 0;
  if (mant > 0) {
    const k = Math.floor(Math.log10(mant));
    mant /= 10 ** k;
    exp10 = k + shift;
    if (mant >= 10) {
      mant /= 10;
      exp10 += 1;
    } else if (mant < 1) {
      mant *= 10;
      exp10 -= 1;
    }
  }
  const sci = (m: number, e: number, d: number) => `${fixed(m, d, point)}e${e}`;
  let body: string;
  if (notation === "grouped" || exp10 < 4) {
    if (exp10 >= 21) body = sci(mant, exp10, dec ?? 2);
    else {
      const plain = shift === 0 ? Math.abs(value) : mant * 10 ** exp10;
      const [i, f] = fixed(plain, dec ?? 0, ".").split(".");
      body = `${group(i ?? "0", sep)}${f !== undefined ? point + f : ""}`;
    }
  } else if (notation === "scientific") body = sci(mant, exp10, dec ?? 2);
  else if (notation === "engineering") {
    const k = Math.floor(exp10 / 3) * 3;
    const em = mant * 10 ** (exp10 - k);
    body = sci(em, k, sig3(em, dec));
  } else {
    // compact and letters: groups of thousands.
    let tier = Math.floor(exp10 / 3);
    let tm = mant * 10 ** (exp10 - tier * 3);
    let d = sig3(tm, dec);
    // 999.95 rounds up into the next group.
    if (Number(tm.toFixed(d)) >= 1000) {
      tier += 1;
      tm /= 1000;
      d = sig3(tm, dec);
    }
    if (tier < SHORT.length) body = fixed(tm, d, point) + SHORT[tier]!;
    else if (notation === "letters") body = fixed(tm, d, point) + letters(tier - SHORT.length);
    else body = sci(mant, exp10, dec ?? 2);
  }
  const zero = !/[1-9]/.test(body.replace(/e.*$/, ""));
  const sign = value < 0 && !zero ? "-" : format.sign && value > 0 && !zero ? "+" : "";
  return `${pre}${sign}${body}${post}`;
}

/** A template's `{name}` holes filled. */
export function fill(template: string, args: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (m, k: string) => (k in args ? String(args[k]) : m));
}

/** A key as an element id: what a path can carry (no `/` or `#`). */
export const keyId = (key: string): string => key.replace(/[/#\s]/g, "-") || "_";

/** Seconds as a countdown: 1:05, 1:02:05 past an hour, 9.4 under ten seconds with tenths. */
export function countdown(seconds: number, tenths = true): string {
  const s = Math.max(0, seconds);
  if (tenths && s < 10) return s.toFixed(1);
  const total = Math.ceil(s);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const r = total % 60;
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}` : `${m}:${String(r).padStart(2, "0")}`;
}

/** Seconds as a short label for a cooldown or a buff: 1h, 12m, 45, 4.2 (tenths under ten when asked). */
export function shortTime(seconds: number, tenths = true): string {
  const s = Math.max(0, seconds);
  if (s >= 3600) return `${Math.ceil(s / 3600)}h`;
  if (s >= 60) return `${Math.ceil(s / 60)}m`;
  if (tenths && s < 10) return s.toFixed(1);
  return String(Math.ceil(s));
}

/**
 * How far in from each side of the safe area the HUD's band starts: on a
 * display wider than 21:9 (an ultrawide) the HUD keeps to a centred 21:9
 * band, so the corners stay in the eye's reach. G1-7's
 * UiDesc.hudMaxAspect narrows the safe area itself; once it does, the safe
 * area is never wider than the band and this answers 0.
 */
export function hudBandInset(): number {
  const vp = frame().viewport;
  const w = vp.width - vp.safeArea.left - vp.safeArea.right;
  const h = vp.height - vp.safeArea.top - vp.safeArea.bottom;
  return Math.max(0, Math.round((w - h * (21 / 9)) / 2));
}

/** The viewport's breakpoint flags. */
export function bp(): { phone: boolean; portrait: boolean; tablet: boolean; desktop: boolean; touch: boolean; wide: boolean } {
  const f = frame();
  const b = f.viewport.breakpoint;
  return {
    phone: f.phone,
    portrait: b === "phonePortrait",
    tablet: b === "tablet",
    desktop: b === "desktop",
    touch: f.touch,
    wide: f.viewport.width / Math.max(1, f.viewport.height) > 2.1,
  };
}
