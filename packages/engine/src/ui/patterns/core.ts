// What the genre patterns (docs/ui.md §11, U6d) share beyond the
// containers' base: rarity tiers told by colour and pips, art slots with a
// placeholder drawn from the style and the slot's kind, symbols drawn as
// shapes (a lock, a star, a coin, a gem, a crown: no style's font is asked
// for a glyph it may lack), prices that say whether they can be paid,
// timers, scrims that keep text legible over any art, and a drag the
// component runs itself from og.ui.framePointer (an inventory's item, a
// card), its state kept in og.ui.memory so a snapshot carries it.
//
// Every pattern is built from public og.ui calls only.

import type * as Api from "@origingame/engine";
import {
  clamp, fit, flags, frame, grow, hex, kit, mix, over, paint, parentOf, pct, px, role, rtl, surface, text, token, xy, type Rgba,
} from "../containers/base.ts";

export * from "../containers/base.ts";

/** A result being filled in: its readonly fields writable. */
export type Mut<T> = { -readonly [K in keyof T]: T[K] };

// ── rarity ──────────────────────────────────────────────────────────────

// rarityColor, rarityPips and teamColor are U6c's (feedback/core.ts), copied here
// as they are until that stream lands; then these three go and are imported.
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

/** A row of `n` small diamonds in a tier's colour (its pips), each `size` square. */
export function pips(id: string, r: Api.Rarity | undefined, size = 6, max = 0): void {
  const n = Math.max(0, rarityPips(r));
  const c = rarityColor(r);
  if (!c || n === 0) return;
  const shown = max > 0 ? Math.min(n, max) : n;
  kit.open(id, { direction: "row", gap: Math.max(1, Math.round(size * 0.35)), cross: "center", part: `Rarity.pips${flags(`rarity=${r}`)}`, partClass: "badge" });
  for (let k = 0; k < shown; k++) {
    kit.open(kit.idi("Pip", k), { width: px(size), height: px(size) });
    kit.setDraw({ shape: { kind: "polygon", sides: 4 }, fills: [{ kind: "solid", color: paint(c) }], strokes: [{ width: 1, align: "outside", paint: { kind: "solid", color: token("shadow", 0.55) } }] });
    kit.close();
  }
  kit.close();
}

// ── fills ───────────────────────────────────────────────────────────────

export const solid = (c: Rgba, alpha = 1): Api.Fill => ({ kind: "solid", color: paint(c, alpha) });

/** A linear gradient from the top (or along `angle` degrees clockwise from down). */
export function vgrad(stops: Array<[number, Api.Paint]>, angle = 0): Api.Fill {
  const a = (angle * Math.PI) / 180;
  const dx = Math.sin(a) * 0.5;
  const dy = Math.cos(a) * 0.5;
  return { kind: "gradient", type: "linear", from: { x: 0.5 - dx, y: 0.5 - dy }, to: { x: 0.5 + dx, y: 0.5 + dy }, stops: stops.map(([offset, color]) => ({ offset, color })) };
}

export function radial(stops: Array<[number, Api.Paint]>, cx = 0.5, cy = 0.5, rx = 0.5, ry = 0.5): Api.Fill {
  return { kind: "gradient", type: "radial", from: { x: cx, y: cy }, to: { x: rx, y: ry }, stops: stops.map(([offset, color]) => ({ offset, color })) };
}

/** A stroke along the outline, `align` inside by default. */
export function line(width: number, c: Api.Paint, align: Api.StrokeAlign = "inside", more: Partial<Api.Stroke> = {}): Api.Stroke {
  return { width, align, paint: { kind: "solid", color: c }, ...more };
}

export const shadow = (dy: number, blur: number, alpha = 0.45): Api.Effect => ({ kind: "dropShadow", offset: { x: 0, y: dy }, blur, spread: 0, color: token("shadow", alpha) });
export const glow = (blur: number, c: Api.Paint, spread = 0): Api.Effect => ({ kind: "outerGlow", blur, spread, color: c });

/**
 * The opaque ground text sits on inside a pattern's panel: the style's
 * panel raised, never translucent (a pale scene or bright art behind a
 * see-through panel takes the text's contrast with it).
 */
export function ground(level: "panel" | "panelRaised" | "well" = "panelRaised"): Api.Paint {
  return token(level);
}

/**
 * A scrim under text laid over art (a card's name, a banner's title): the
 * shadow colour rising from transparent to `strength` towards the edge the
 * text sits on, so the text holds 4.5:1 on any picture, pale or dark.
 */
export function scrimFill(edge: "bottom" | "top" | "start" | "end", strength = 0.82): Api.Fill {
  // Darkest at `edge`: a gradient at angle 90 runs from the left to the right.
  const angle = edge === "bottom" ? 0 : edge === "top" ? 180 : (edge === "start") !== rtl() ? 270 : 90;
  // Most of its strength from the middle on, so words laid over its darker half read on pale art.
  return vgrad([[0, token("shadow", 0)], [0.5, token("shadow", strength * 0.8)], [1, token("shadow", strength)]], angle);
}

// ── symbols drawn as shapes ─────────────────────────────────────────────

export type SymbolKind =
  | "lock" | "star" | "starOutline" | "coin" | "gem" | "crown" | "check" | "cross" | "plus" | "minus" | "dot" | "ring"
  | "clock" | "heart" | "shield" | "sword" | "flag" | "pin" | "eye" | "rotate" | "split" | "chat" | "speaker" | "camera" | "grid" | "bolt"
  | "swap" | "ban";

/**
 * A symbol drawn as shapes, `size` square, in `color`: what a style's icon
 * set may not have and a font may not hold (★ ✓ 🔒 as text draw tofu in a
 * face without them). An icon of the atlas is used instead when the
 * style has one of `icon`'s name.
 */
export function symbol(id: string, kind: SymbolKind, size: number, color: Rgba, opts: { label?: string | undefined; icon?: string | undefined; weight?: number | undefined; flat?: boolean | undefined } = {}): void {
  if (opts.icon && kit.hasIcon(opts.icon)) {
    kit.icon(id, opts.icon, { size, tint: paint(color), label: opts.label ?? "" });
    return;
  }
  const s = size;
  const w = opts.weight ?? Math.max(1.5, s * 0.12);
  const fill: Api.Fill[] = [{ kind: "solid", color: paint(color) }];
  const p = (x: number, y: number) => ({ x: x * s, y: y * s });
  const path = (pts: Api.Vector2[], width = w, closed = false): Api.DrawShape => ({ kind: "path", points: pts, width, cap: "round", join: "round", closed });
  kit.open(id, { width: px(s), height: px(s), direction: "column", align: "center", cross: "center" });
  switch (kind) {
    case "lock": {
      // A shackle (an arc's upper half) over a body.
      kit.open("Shackle", { width: px(s * 0.56), height: px(s * 0.56) });
      kit.setDraw({ shape: { kind: "arc", start: Math.PI, sweep: Math.PI, thickness: Math.max(1.5, s * 0.11), cap: "butt" }, fills: fill });
      kit.close();
      kit.open("Body", { width: px(s * 0.72), height: px(s * 0.46), radius: s * 0.08 });
      kit.setDraw({ shape: { kind: "rect" }, fills: fill });
      kit.close();
      kit.props({ transform: { y: -s * 0.22 } });
      break;
    }
    case "star":
    case "starOutline":
      kit.setDraw({
        shape: { kind: "polygon", sides: 5, inner: 0.48, rounding: s * 0.04 },
        fills: kind === "star" ? fill : [],
        strokes: kind === "starOutline" ? [line(Math.max(1.2, s * 0.08), paint(color))] : [],
      });
      break;
    case "coin":
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [vgrad([[0, paint(mix(color, hex("#ffffff"), 0.35))], [1, paint(mix(color, hex("#000000"), 0.2))]])], strokes: [line(Math.max(1, s * 0.09), paint(mix(color, hex("#000000"), 0.35)))] });
      kit.open("Inner", { width: pct(0.5), height: pct(0.5) });
      kit.setDraw({ shape: { kind: "ellipse" }, strokes: [line(Math.max(1, s * 0.07), paint(mix(color, hex("#000000"), 0.3)))] });
      kit.close();
      break;
    case "gem":
      // Flat: one ink (a gem set in a control's ink, which a light facet would wash out).
      if (opts.flat) kit.setDraw({ shape: { kind: "polygon", sides: 6, rotation: Math.PI / 6 }, fills: fill });
      else kit.setDraw({ shape: { kind: "polygon", sides: 6, rotation: Math.PI / 6 }, fills: [vgrad([[0, paint(mix(color, hex("#ffffff"), 0.45))], [1, paint(color)]])], strokes: [line(Math.max(1, s * 0.07), paint(mix(color, hex("#000000"), 0.35)))] });
      break;
    case "crown":
      kit.setDraw({ shape: { kind: "path", points: [p(0.1, 0.78), p(0.1, 0.3), p(0.32, 0.52), p(0.5, 0.2), p(0.68, 0.52), p(0.9, 0.3), p(0.9, 0.78)], closed: true, filled: true }, fills: fill });
      break;
    case "check":
      kit.setDraw({ shape: path([p(0.2, 0.52), p(0.42, 0.74), p(0.8, 0.28)]), fills: fill });
      break;
    case "cross":
      kit.setDraw({ shape: path([p(0.25, 0.25), p(0.75, 0.75), p(0.5, 0.5), p(0.75, 0.25), p(0.25, 0.75)]), fills: fill });
      break;
    case "plus":
      kit.setDraw({ shape: path([p(0.5, 0.18), p(0.5, 0.82), p(0.5, 0.5), p(0.18, 0.5), p(0.82, 0.5)]), fills: fill });
      break;
    case "minus":
      kit.setDraw({ shape: path([p(0.2, 0.5), p(0.8, 0.5)]), fills: fill });
      break;
    case "dot":
      kit.setDraw({ shape: { kind: "ellipse" }, fills: fill });
      break;
    case "ring":
      kit.setDraw({ shape: { kind: "arc", thickness: w }, fills: fill });
      break;
    case "clock":
      kit.setDraw({ shape: { kind: "arc", thickness: w }, fills: fill });
      kit.open("Hands", { width: px(s), height: px(s) });
      kit.setDraw({ shape: path([p(0.5, 0.24), p(0.5, 0.5), p(0.7, 0.62)], w * 0.9), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "heart":
      kit.setDraw({ shape: { kind: "path", points: [p(0.5, 0.86), p(0.1, 0.48), p(0.04, 0.24), p(0.26, 0.1), p(0.5, 0.3), p(0.74, 0.1), p(0.96, 0.24), p(0.9, 0.48)], closed: true, filled: true }, fills: fill });
      break;
    case "shield":
      kit.setDraw({ shape: { kind: "path", points: [p(0.5, 0.06), p(0.88, 0.2), p(0.82, 0.62), p(0.5, 0.94), p(0.18, 0.62), p(0.12, 0.2)], closed: true, filled: true }, fills: fill });
      break;
    case "sword":
      kit.setDraw({ shape: path([p(0.2, 0.8), p(0.82, 0.18), p(0.5, 0.5), p(0.34, 0.5), p(0.5, 0.66)]), fills: fill });
      break;
    case "flag":
      kit.setDraw({ shape: { kind: "path", points: [p(0.24, 0.12), p(0.84, 0.3), p(0.24, 0.5)], closed: true, filled: true }, fills: fill });
      kit.open("Pole", { width: px(s), height: px(s) });
      kit.setDraw({ shape: path([p(0.24, 0.1), p(0.24, 0.9)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "pin":
      kit.setDraw({ shape: { kind: "path", points: [p(0.5, 0.94), p(0.2, 0.5), p(0.2, 0.32), p(0.34, 0.1), p(0.66, 0.1), p(0.8, 0.32), p(0.8, 0.5)], closed: true, filled: true }, fills: fill });
      break;
    case "eye":
      kit.setDraw({ shape: { kind: "path", points: [p(0.06, 0.5), p(0.5, 0.18), p(0.94, 0.5), p(0.5, 0.82), p(0.06, 0.5)], width: w }, fills: fill });
      kit.open("Pupil", { width: pct(0.3), height: pct(0.3) });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: fill });
      kit.close();
      break;
    case "rotate":
      kit.setDraw({ shape: { kind: "arc", start: -Math.PI * 0.35, sweep: Math.PI * 1.55, thickness: w, cap: "round" }, fills: fill });
      kit.open("Head", { width: px(s), height: px(s) });
      kit.setDraw({ shape: path([p(0.62, 0.06), p(0.86, 0.2), p(0.66, 0.4)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "swap":
      // Two arrows passing: the top one to the end, the bottom one back.
      kit.setDraw({ shape: path([p(0.14, 0.34), p(0.84, 0.34), p(0.66, 0.16), p(0.84, 0.34), p(0.66, 0.52)]), fills: fill });
      kit.open("Back", { width: px(s), height: px(s) });
      kit.setDraw({ shape: path([p(0.86, 0.66), p(0.16, 0.66), p(0.34, 0.48), p(0.16, 0.66), p(0.34, 0.84)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "ban":
      // A ring struck through (⊘), which a font may not hold.
      kit.setDraw({ shape: { kind: "arc", thickness: w }, fills: fill });
      kit.open("Bar", { width: px(s), height: px(s) });
      kit.setDraw({ shape: path([p(0.26, 0.26), p(0.74, 0.74)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "split":
      kit.setDraw({ shape: path([p(0.5, 0.86), p(0.5, 0.5), p(0.2, 0.18), p(0.5, 0.5), p(0.8, 0.18)]), fills: fill });
      break;
    case "chat":
      kit.setDraw({ shape: { kind: "path", points: [p(0.1, 0.18), p(0.9, 0.18), p(0.9, 0.68), p(0.42, 0.68), p(0.22, 0.88), p(0.24, 0.68), p(0.1, 0.68)], closed: true, filled: true }, fills: fill });
      break;
    case "speaker":
      kit.setDraw({ shape: { kind: "path", points: [p(0.1, 0.38), p(0.3, 0.38), p(0.56, 0.14), p(0.56, 0.86), p(0.3, 0.62), p(0.1, 0.62)], closed: true, filled: true }, fills: fill });
      break;
    case "camera":
      kit.setDraw({ shape: { kind: "rect", radii: [s * 0.12, s * 0.12, s * 0.12, s * 0.12] }, fills: fill });
      kit.open("Lens", { width: pct(0.42), height: pct(0.42) });
      kit.setDraw({ shape: { kind: "arc", thickness: Math.max(1.5, s * 0.08) }, fills: [{ kind: "solid", color: token("panelRaised") }] });
      kit.close();
      break;
    case "grid":
      kit.setDraw({ shape: path([p(0.15, 0.15), p(0.85, 0.15), p(0.85, 0.85), p(0.15, 0.85), p(0.15, 0.15), p(0.5, 0.15), p(0.5, 0.85), p(0.15, 0.85), p(0.15, 0.5), p(0.85, 0.5)], w * 0.8), fills: fill });
      break;
    case "bolt":
      kit.setDraw({ shape: { kind: "path", points: [p(0.58, 0.04), p(0.18, 0.56), p(0.46, 0.56), p(0.38, 0.96), p(0.82, 0.42), p(0.54, 0.42)], closed: true, filled: true }, fills: fill });
      break;
  }
  kit.close();
  if (opts.label) kit.semantics({ role: "image", name: opts.label });
}

// ── art slots ───────────────────────────────────────────────────────────

export interface Slot {
  /** The art; 0 or absent draws the placeholder. */
  texture?: Api.Texture | null | undefined;
  /** The slot's kind (docs/ui.md Appendix F): its placeholder's silhouette. */
  kind?: string | undefined;
  /** Width over height; 0 is the element's own. */
  aspect?: number | undefined;
  /** The point cropping keeps, 0–1. */
  focal?: Api.Vector2 | undefined;
  /** An icon of the style's set drawn in the placeholder. */
  icon?: string | undefined;
  /** What it shows, for a screen reader; empty is decoration. */
  label?: string | undefined;
  /** A tier's colour washes the placeholder and rings the slot. */
  rarity?: Api.Rarity | undefined;
  /** The slot's corner radius. */
  radius?: number | [number, number, number, number] | undefined;
  /** How the art fills it. */
  fit?: "cover" | "contain" | undefined;
  /** Its size; null grows. */
  width?: Api.Size | null | undefined;
  height?: Api.Size | null | undefined;
}

/** The placeholder silhouette per art kind: a symbol that reads as the kind (never a grey box). */
const KIND_SYMBOL: Record<string, SymbolKind> = {
  item: "sword", equipment: "shield", ability: "bolt", portrait: "heart", bust: "heart", character: "heart", splash: "heart",
  card: "star", banner: "star", jacket: "speaker", song: "speaker", track: "flag", level: "flag", map: "pin", vehicle: "bolt",
  emblem: "shield", avatar: "heart", cg: "eye", screenshot: "camera", thumbnail: "camera", currency: "coin", gem: "gem",
};

/** Kinds whose placeholder is a figure (head and shoulders) rather than a symbol. */
export const FIGURE_KINDS: ReadonlySet<string> = new Set(["portrait", "bust", "character", "splash", "avatar", "banner", "keyArt"]);

/**
 * A slot's placeholder, drawn in the open slot (docs/ui.md §13.1): a
 * gradient of two hues (the tier's or the primary's, and a second turned
 * from it), a light at the focal point, and the kind's silhouette standing
 * on it: a figure for a portrait, bust, character or key art, else the
 * kind's symbol, both sized to the slot (its size as laid out last tick).
 * og.ui.artSlot draws it for the figure kinds, with its frame's stroke.
 */
export function placeholderArt(s: Slot, tier: Rgba | null, radii: [number, number, number, number], strokes?: Api.Stroke[]): void {
  const base = tier ?? role("primary");
  // The second hue: the highlight against the primary, the primary against a tier's colour.
  const other = tier ? role("primary") : role("highlight");
  const deep = (c: Rgba, k: number): Rgba => mix(role("well"), c, k);
  const figure = FIGURE_KINDS.has(s.kind ?? "");
  const f = s.focal ?? { x: 0.5, y: figure ? 0.36 : 0.45 };
  kit.setDraw({
    shape: { kind: "rect", radii },
    fills: [
      vgrad([[0, paint(deep(base, 0.5))], [0.55, paint(deep(mix(base, other, 0.45), 0.34))], [1, paint(deep(other, 0.22))]], 135),
      radial([[0, paint(mix(base, role("ink"), 0.25), 0.55)], [0.45, paint(base, 0.18)], [1, paint(base, 0)]], f.x, f.y, 0.55, 0.55),
    ],
    strokes,
    clip: "shape",
  });
  // Its size as laid out last tick (the first tick: what the slot asked for, or a small default).
  const fixed = (z: Api.Size | null | undefined): number | null => (z && z.kind === "fixed" ? z.px : null);
  // An empty first child names the slot's path (its parent's), whose rectangle last tick is the size to draw at.
  kit.open("Sizer", { width: px(0), height: px(0) });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  const was = kit.bounds(parentOf(kit.lastPath()));
  const w = was?.width || fixed(s.width) || 96;
  const h = was?.height || fixed(s.height) || (s.aspect ? w / s.aspect : w);
  const ink = mix(deep(base, 0.2), role("shadow"), 0.35);
  const rim = mix(base, role("ink"), 0.45);
  if (figure) {
    // Head and shoulders standing on the slot's foot, the head at the focal point.
    const S = Math.min(w * 0.9, h * 0.95);
    const head = Math.round(S * 0.34);
    const shW = Math.round(S * 0.86);
    const top = Math.round(f.y * h - head / 2);
    const shH = Math.max(0, Math.round(h - top - head * 0.92));
    const x = (k: number) => Math.round(f.x * w - k / 2);
    // The shoulders first: the head stands over their top edge.
    if (shH > 0) {
      const r = Math.round(shW * 0.42);
      kit.open("Shoulders", { width: px(shW), height: px(shH) });
      kit.setDraw({ shape: { kind: "rect", radii: [r, r, 0, 0] }, fills: [vgrad([[0, paint(ink, 0.92)], [1, paint(ink, 0.6)]])], strokes: [line(Math.max(1, S * 0.008), paint(rim, 0.45))] });
      kit.close();
      over({ insets: { start: x(shW), top: Math.round(top + head * 0.98) }, inert: true });
    }
    kit.open("Head", { width: px(head), height: px(Math.round(head * 1.12)) });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: paint(ink, 0.92) }], strokes: [line(Math.max(1, S * 0.008), paint(rim, 0.55))] });
    kit.close();
    over({ insets: { start: x(head), top }, inert: true });
  } else {
    const sym = KIND_SYMBOL[s.kind ?? ""] ?? "star";
    const box = Math.round(clamp(Math.min(w, h) * 0.36, 14, 160));
    kit.open("Mark", { width: px(box), height: px(box) });
    symbol("Kind", sym, box, mix(base, role("ink"), 0.35), { icon: s.icon });
    kit.close();
    over({ insets: { start: Math.round(f.x * w - box / 2), top: Math.round(f.y * h - box / 2) }, inert: true, opacity: 0.6 });
  }
}

/**
 * An art slot (docs/ui.md §13): the game's texture covering the slot
 * around its focal point, or a placeholder drawn from the style and the
 * kind (a wash in the tier's colour or the primary, a soft light, the
 * kind's silhouette), never a flat grey box. Its part is
 * `<part>[placeholder]` while it has no art.
 */
export function artSlot(id: string, part: string, s: Slot, build?: () => void): void {
  const r = s.radius ?? frame().t.small;
  const radii = typeof r === "number" ? [r, r, r, r] as [number, number, number, number] : r;
  const tier = rarityColor(s.rarity);
  const placeholder = !s.texture;
  kit.open(id, {
    direction: "column",
    width: s.width === undefined ? grow() : s.width,
    height: s.height === undefined ? null : s.height,
    aspect: s.aspect && s.aspect > 0 ? s.aspect : 0,
    align: "center",
    cross: "center",
    part: `${part}${flags(placeholder && "placeholder", s.rarity && s.rarity !== "none" && `rarity=${s.rarity}`)}`,
    partClass: "slot",
  });
  if (!placeholder) {
    kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "image", texture: s.texture!, fit: s.fit ?? "cover", focal: s.focal ?? { x: 0.5, y: 0.5 } }], clip: "shape" });
  } else {
    placeholderArt(s, tier, radii);
  }
  try {
    build?.();
  } finally {
    kit.close();
  }
  if (s.label) kit.semantics({ role: "image", name: s.label });
}

// ── prices and timers ───────────────────────────────────────────────────

/** A count with thousands grouped (a price, a stack). */
export function grouped(n: number): string {
  if (!Number.isFinite(n)) return "—";
  const sign = n < 0 ? "−" : "";
  return sign + Math.floor(Math.abs(n)).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/** Seconds as the two largest units (2d 4h, 3h 12m, 4:05). */
export function countdown(seconds: number): string {
  const s = Math.max(0, Math.ceil(seconds));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = s % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
  return `${m}:${r.toString().padStart(2, "0")}`;
}

/** The currencies' colours and symbols. */
function currencyLook(currency: string): { sym: SymbolKind; color: Rgba } {
  const c = currency.toLowerCase();
  if (/gem|crystal|diamond|primo|orb|premium/.test(c)) return { sym: "gem", color: hex("#7fd1ff") };
  if (/real|cash|money|usd|eur|\$/.test(c)) return { sym: "dot", color: role("success") };
  return { sym: "coin", color: role("gold") };
}

/**
 * A price: the currency's icon (the style's when it has one named after
 * the currency, else a coin or gem drawn), the amount in the numeric font,
 * a struck-through original price when discounted, and the amount in the
 * danger ink with `short` when it cannot be paid (said, not only coloured).
 */
export function priceTag(id: string, d: { amount: number; currency?: string; original?: number; affordable?: boolean; free?: string; text?: string; size?: "sm" | "md" | "lg"; shortLabel?: string; ink?: Rgba }): void {
  const { t } = frame();
  // Scaled with the surface (st, u): a price is read in play, 18 px at 1080p.
  const step = st(d.size === "lg" ? 20 : d.size === "sm" ? 14 : 16);
  const iconSize = u(d.size === "lg" ? 22 : d.size === "sm" ? 14 : 18);
  const can = d.affordable ?? true;
  kit.open(id, { direction: "row", gap: t.xxs + 2, cross: "center", part: `Price.tag${flags(!can && "short", !!d.original && d.original > d.amount && "discount")}`, partClass: "badge" });
  if (d.text !== undefined && d.text !== "") {
    text("Amount", d.text, { step, nowrap: true, font: "display" });
  } else if (d.amount === 0 && d.free) {
    text("Amount", d.free, { step, color: "success", nowrap: true, font: "display" });
  } else {
    const look = currencyLook(d.currency ?? "");
    // Set inside a control (`ink`): the gem in the control's ink too, so it holds 4.5:1 on any fill (a pale gem on gold did not).
    symbol("Currency", look.sym, iconSize, d.ink ?? look.color, { icon: d.ink ? undefined : d.currency, label: "", flat: !!d.ink });
    if (d.original && d.original > d.amount) {
      kit.open("Was", { direction: "row" });
      text("Value", grouped(d.original), { step: st(14), color: "inkMuted", nowrap: true, font: "mono" });
      kit.close();
      // The strike: a line across the old price, drawn (no font has a strikethrough mark everywhere).
      kit.open("Strike", { width: grow(), height: px(2), radius: 1, background: token("danger") });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, end: 0, centerY: true }, inert: true, overlaps: [kit.lastPath().replace(/\/Strike$/, "/Was")] });
    }
    // Inside a control it reads in the control's face (the thin mono face at 14 px read faint beside a display label).
    text("Amount", grouped(d.amount), { step, color: can ? d.ink ?? "ink" : "danger", nowrap: true, font: d.ink ? "display" : "mono" });
  }
  kit.close();
  const words = d.text || (d.amount === 0 && d.free ? d.free : `${grouped(d.amount)} ${d.currency ?? ""}`.trim());
  kit.semantics({ role: "text", name: can ? words : `${words}, ${d.shortLabel ?? "not enough"}` });
}

/** A timer chip: a clock drawn, the time left, warn under a minute. */
export function timerChip(id: string, seconds: number, label = ""): void {
  const { t } = frame();
  const urgent = seconds < 60;
  kit.open(id, { direction: "row", gap: u(6), cross: "center", padding: xy(u(10), u(4)), radius: 999, background: token(urgent ? "danger" : "well"), part: `Timer.chip${flags(urgent && "urgent")}`, partClass: "chip" });
  symbol("Clock", "clock", u(14), role(urgent ? "onPrimary" : "inkMuted"), { icon: "clock" });
  if (label) text("Label", label, { step: st(13), color: urgent ? hex("#ffffff") : "inkMuted", nowrap: true });
  text("Time", countdown(seconds), { step: st(13), color: urgent ? hex("#ffffff") : "ink", nowrap: true, font: "mono" });
  kit.close();
  kit.semantics({ role: "timer", name: [label, countdown(seconds)].filter(Boolean).join(" ") });
}

/** A ring timer: an arc of the time left, `size` across, warn and then danger as it runs out, the seconds in the middle. */
export function timerRing(id: string, fraction: number, size: number, label: string): void {
  const f = clamp(fraction, 0, 1);
  const c = role(f < 0.2 ? "danger" : f < 0.45 ? "warn" : "primary");
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", part: `Timer.ring${flags(f < 0.2 && "urgent")}`, partClass: "badge" });
  kit.setDraw({ shape: { kind: "arc", thickness: Math.max(3, size * 0.1) }, fills: [{ kind: "solid", color: token("well") }] });
  kit.open("Fill", { width: px(size), height: px(size) });
  kit.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.max(0.0001, f) * Math.PI * 2, thickness: Math.max(3, size * 0.1), cap: "round" }, fills: [{ kind: "solid", color: paint(c) }] });
  kit.close();
  kit.props({ position: "absolute", insets: { start: 0, top: 0 }, inert: true });
  text("Label", label, { step: size >= 56 ? "lg" : "sm", nowrap: true, font: "mono" });
  kit.close();
  kit.semantics({ role: "timer", name: label });
}

// ── a drag the component runs itself ─────────────────────────────────────

/** The frame's pointer with its edges and modifiers (og.ui.framePointer). */
export function pointer(): Api.UiPointer {
  return kit.framePointer() as Api.UiPointer;
}

/** A 31-bit hash of a key, to keep "which item" in a number memory. */
export function keyHash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 1) || 1;
}

/** The key among `keys` whose hash is `h`, or null. */
export function keyOf(keys: readonly string[], h: number): string | null {
  if (!h) return null;
  for (const k of keys) if (keyHash(k) === h) return k;
  return null;
}

/** Whether a point is inside a rect. */
export function inside(r: Api.UiRect | null, x: number, y: number): boolean {
  return !!r && x >= r.x && y >= r.y && x < r.x + r.width && y < r.y + r.height;
}

/** A small caps label over a section (a pattern's eyebrow). */
export function eyebrow(id: string, s: string, color: Api.ThemeColor | Rgba = "inkMuted"): void {
  text(id, s.toLocaleUpperCase(), { step: "caption", color, nowrap: true, ellipsis: true, font: "display" });
}

/** A hairline across its parent in the line colour. */
export function hairline(id: string, vertical = false): void {
  kit.open(id, { width: vertical ? px(1) : grow(), height: vertical ? grow() : px(1), background: token("line") });
  kit.close();
}

/** A count badge on the corner of what was just opened (a stack, unread). */
export function cornerCount(id: string, s: string, corner: "bottomEnd" | "topEnd" = "bottomEnd", tone: Api.ThemeColor = "ink"): void {
  const { t } = frame();
  kit.open(id, { direction: "row", padding: xy(t.xxs + 1, 0), radius: 4, background: token("shadow", 0.78), part: "Count.badge", partClass: "badge" });
  text("Text", s, { step: "caption", color: tone, nowrap: true, font: "mono" });
  kit.close();
}

/** Whether a breakpoint is a phone's (either orientation). */
export function isPhone(): boolean {
  return frame().phone;
}

/** The surface's width less its safe area. */
export function safeWidth(): number {
  const v = frame().viewport;
  return v.width - v.safeArea.left - v.safeArea.right;
}

/** The surface's height less its safe area. */
export function safeHeight(): number {
  const v = frame().viewport;
  return v.height - v.safeArea.top - v.safeArea.bottom;
}

// ── scale, type and buttons ─────────────────────────────────────────────

/**
 * The patterns' reference scale (as a canvas scaler does): 1 at 1280×720
 * and on every touch breakpoint (their CSS px are already a finger's),
 * growing with a desktop surface to 1.5 at 1920×1080 and 2 at 2560×1440
 * (an ultrawide by its height), so a pattern keeps its proportions and the
 * text read in play stays over the 18 px docs/ui.md §15.2 asks at 1080p.
 */
export function scale(): number {
  const v = frame().viewport;
  if (v.coarsePointer || v.breakpoint !== "desktop") return 1;
  return clamp(Math.min(v.width, (v.height * 16) / 9) / 1280, 1, 2);
}

/** `n` CSS px at the reference scale, rounded. */
export function u(n: number): number {
  return Math.round(n * scale());
}

const STEPS: readonly Api.TypeStepName[] = ["xs", "caption", "body", "base", "lg", "xl", "xl2", "xl3", "displaySm", "xl4", "displayMd", "xl5", "displayLg", "xl6", "displayXl", "xl7", "displayXl2", "xl8"];

/**
 * The smallest type step at least `px` CSS px at the reference scale (the
 * style's ramp, before the player's text size, which then grows it): the
 * patterns name their text by size, so it scales with the surface.
 */
export function st(px0: number): Api.TypeStepName {
  const th = frame().theme;
  const k = Math.max(1, th.textScale ?? 1);
  const want = px0 * scale() - 0.01;
  let best: Api.TypeStepName = "xl8";
  let bestSize = Infinity;
  for (const s of STEPS) {
    const size = (th.typeRamp[s]?.size ?? 0) / k;
    if (size >= want && size < bestSize) {
      best = s;
      bestSize = size;
    }
  }
  return best;
}

/**
 * `px0`'s step (st), or the largest smaller one no taller than `box` CSS px:
 * a number inside a badge that grows less than the text does (a cost gem at
 * 200 % text keeps its digits inside it).
 */
export function stIn(px0: number, box: number): Api.TypeStepName {
  const ramp = frame().theme.typeRamp;
  const want = st(px0);
  if ((ramp[want]?.size ?? 0) <= box) return want;
  let best: Api.TypeStepName = "xs";
  let bestSize = 0;
  for (const s of STEPS) {
    const size = ramp[s]?.size ?? 0;
    if (size <= box && size > bestSize) {
      best = s;
      bestSize = size;
    }
  }
  return best;
}

/**
 * The type step for one line of `s` in `width` CSS px: `px0`'s step (st),
 * or the largest smaller one whose line fits (a card's name in a narrow fan
 * at large text, where a word cannot wrap). Never below the reference
 * size `min`. Measured by an estimate of the face's advance (0.6 em a
 * character), so it errs on the small side.
 */
export function fitStep(s: string, width: number, px0: number, min = 11): Api.TypeStepName {
  const th = frame().theme;
  const want = st(px0);
  const longest = Math.max(1, ...s.split(/\s+/).map((w) => [...w].length));
  const fits = (size: number) => longest * size * 0.6 <= width;
  if (fits(th.typeRamp[want]?.size ?? 0)) return want;
  const floor = min * scale() - 0.01;
  let best: Api.TypeStepName = st(min);
  let bestSize = 0;
  for (const k of STEPS) {
    const size = th.typeRamp[k]?.size ?? 0;
    if (size >= floor && fits(size) && size > bestSize && size <= (th.typeRamp[want]?.size ?? 0)) {
      best = k;
      bestSize = size;
    }
  }
  return best;
}

/** A colour at full opacity. */
const opaque = (c: Rgba): Rgba => ({ ...c, a: 1 });

export type BtnVariant = "primary" | "secondary" | "quiet" | "danger" | "premium";

/**
 * A button the patterns draw at the reference scale: a surface in its
 * variant's fill with its label (and an optional symbol or price before
 * it), the style's part `<part>[<variant>]` with its interaction states, at
 * least 44 CSS px on touch. Answers its response (zero while disabled).
 */
export function btn(id: string, label: string, d: { variant?: BtnVariant; part?: string; disabled?: boolean; size?: "sm" | "md" | "lg"; width?: Api.Size | null; sym?: SymbolKind; before?: () => void; name?: string; selected?: boolean } = {}): Api.Response {
  const { t, touch } = frame();
  const v = d.variant ?? "secondary";
  const size = d.size ?? "md";
  const h = Math.max(touch ? 44 : 0, u(size === "lg" ? 48 : size === "sm" ? 32 : 40));
  const fillRole: Api.ThemeColor | null = v === "primary" ? "primary" : v === "danger" ? "danger" : v === "premium" ? "gold" : v === "secondary" ? "control" : null;
  const inkRole: Api.ThemeColor = v === "primary" || v === "danger" ? "onPrimary" : "ink";
  const ink = v === "premium" ? hex("#1d1406") : role(inkRole);
  return surface(id, {
    part: d.part ?? "Pattern.button",
    cls: "button",
    flags: [v, size],
    // Opaque: a style's control fill may be translucent, and a button over a busy scene must not vanish into it.
    // A secondary button is the raised panel lifted toward the ink (a style's control colour is meant to be laid thin).
    fill: fillRole ? opaque(fillRole === "control" ? mix(role("panelRaised"), role("ink"), 0.12) : role(fillRole)) : null,
    wash: v === "primary" || v === "danger" || v === "premium" ? hex("#ffffff") : undefined,
    washBy: 0.14,
    disabled: d.disabled ?? false,
    width: d.width ?? null,
    height: fit(h),
    padding: xy(u(size === "sm" ? 12 : 18), 0),
    gap: u(8),
    align: "center",
    radius: t.control,
    // Selected, a coloured variant keeps its colour (Banish armed is danger); a secondary one takes the primary's soft fill.
    ...(d.selected === undefined ? {} : { selected: d.selected, selectedFill: opaque(fillRole && fillRole !== "control" ? role(fillRole) : mix(role("panelRaised"), role("primary"), 0.3)) }),
    // A toggle (Banish armed) is said selected as it is drawn: the tree has no pressed state.
    semantics: { role: "button", name: d.name ?? label, ...(d.selected === undefined ? {} : { selected: d.selected }) },
  }, () => {
    if (d.sym) symbol("Sym", d.sym, u(size === "sm" ? 14 : 18), ink);
    d.before?.();
    if (label) text("Label", label, { step: st(size === "lg" ? 16 : size === "sm" ? 13 : 14), color: d.disabled ? "inkFaint" : ink, nowrap: true, font: "display" });
  });
}
