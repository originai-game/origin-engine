// What the HUD components (docs/ui.md §11 "HUD", Appendix C) share beyond
// the feedback core: where a component opens (its own anchored group in
// the safe area, or in place), the HUD chip every readout sits on, meter
// colours by kind and level, and a stable id for a list item.

import type * as Api from "@origingame/engine";
import { absolute, type Props, frame, hudBandInset, kit, role, type Rgba, solid, token, drop, stroke, inner, vgrad, lit, mix, shapeOf } from "../feedback/core.ts";

export * from "../feedback/core.ts";

/**
 * The width a HUD piece may take across the safe area, inside the style's
 * gutters: what a bar measures itself against before it compacts or
 * wraps rather than run off the screen.
 */
export function hudRoom(): number {
  const f = frame();
  const vp = f.viewport;
  const band = (vp as { hud?: Api.UiRect }).hud;
  if (band) return band.width - f.t.gutter * 2;
  return vp.width - vp.safeArea.left - vp.safeArea.right - f.t.gutter * 2 - hudBandInset() * 2;
}

/**
 * How far a piece must fold to fit `room`: 0 as designed, then one step
 * more each tick its laid-out width (last tick's) overflows, up to `most`.
 * Kept per viewport width, so a resize starts over from 0.
 */
export function fitLevel(path: string, room: number, most: number): number {
  const key = `${path}~fit/${Math.round(room)}`;
  let level = kit.memory(key, 0);
  const w = kit.bounds(path)?.width ?? 0;
  if (w > room + 0.5 && level < most) level += 1;
  kit.remember(key, level);
  return level;
}

/** A desc's anchor: its own when it names one (null too), else the component's default. */
export function anchorOf(desc: { anchor?: Api.Anchor | null }, fallback: Api.Anchor | null): Api.Anchor | null {
  return "anchor" in desc && desc.anchor !== undefined ? desc.anchor : fallback;
}

/**
 * Builds `build` in an anchored group (in the safe area, at the style's
 * gutter) when `anchor` is set, else in place.
 */
export function hudPlace(id: string, anchor: Api.Anchor | null, build: (inAnchor: boolean) => void, more: Partial<Api.AnchorDesc> = {}): void {
  if (!anchor) {
    build(false);
    return;
  }
  // Components anchored at one point stack in the order they are built
  // (a compass over the round timer, a killfeed under the minimap), each
  // moved past the ones before it by their sizes last tick, kept in the
  // UI's memory: the defaults never draw one HUD piece over another.
  const t = kit.tick();
  const key = `~hudStack/${anchor}`;
  const first = kit.memory(`${key}/tick`, -1) !== t;
  // How far the groups before it reach from the anchor's line: down from
  // a top edge, up from a bottom one, down from the middle of a side.
  const reach = first ? 0 : kit.memory(`${key}/reach`, 0);
  const path = `${id}Hud`;
  const h = kit.bounds(path)?.height ?? 0;
  const gap = frame().t.sm;
  const middle = !anchor.startsWith("top") && !anchor.startsWith("bottom");
  // Where this one starts from the anchor's line, and where the next will.
  const start = first ? 0 : reach + gap;
  const next = start + h;
  let dy: number;
  if (middle) {
    // A stack on a side is centred as a whole on the side's middle (by its
    // extent last tick), so a tracker and a combo under it both stay on screen.
    if (first) kit.remember(`${key}/top`, sideTop(anchor, kit.memory(`${key}/total`, 0), t, gap));
    dy = kit.memory(`${key}/top`, 0) + start + h / 2;
  } else {
    dy = start;
  }
  kit.remember(`${key}/tick`, t);
  kit.remember(`${key}/reach`, next);
  kit.remember(`${key}/total`, next);
  if (anchor.startsWith("bottom")) dy = -dy;
  const band = hudBandInset();
  const dx = crossOf(anchor) === "start" ? band : crossOf(anchor) === "end" ? -band : 0;
  kit.anchor(path, { anchor, direction: "column", cross: crossOf(anchor), ...more, offsetX: dx + (more.offsetX ?? 0), offsetY: dy + (more.offsetY ?? 0) }, () => build(true));
}

/**
 * The span a stack on a side has between the stacks at the top and the
 * bottom edges of that side (their extents last tick), from the safe
 * area's middle: where it may start (lo) and how tall it may be (room).
 */
export function sideSpan(anchor: Api.Anchor, t = kit.tick(), gap = frame().t.sm): { half: number; lo: number; room: number } {
  const f = frame();
  const vp = f.viewport;
  const half = (vp.height - vp.safeArea.top - vp.safeArea.bottom) / 2 - f.t.gutter;
  const side = anchor === "left" ? "Left" : anchor === "right" ? "Right" : null;
  const reach = (a: string): number => {
    const k = `~hudStack/${a}`;
    return kit.memory(`${k}/tick`, -10) >= t - 1 ? kit.memory(`${k}/total`, 0) : 0;
  };
  const above = side ? Math.max(reach("top"), reach(`top${side}`)) : 0;
  const below = side ? Math.max(reach("bottom"), reach(`bottom${side}`)) : 0;
  const lo = -half + (above > 0 ? above + gap : 0);
  const room = half - (below > 0 ? below + gap : 0) - lo;
  return { half, lo, room };
}

/**
 * Where a stack on a side starts, from the safe area's middle: centred on
 * it, but moved clear of the stacks at the top and bottom edges of that
 * side (their extents last tick) when it fits between them.
 */
function sideTop(anchor: Api.Anchor, total: number, t: number, gap: number): number {
  const { lo, room } = sideSpan(anchor, t, gap);
  const hi = lo + room - total;
  const centred = -total / 2;
  return lo <= hi ? Math.min(hi, Math.max(lo, centred)) : centred;
}

/** How an anchored group lines its children up by where it sits. */
export function crossOf(anchor: Api.Anchor): Api.Cross {
  if (anchor === "topLeft" || anchor === "left" || anchor === "bottomLeft") return "start";
  if (anchor === "topRight" || anchor === "right" || anchor === "bottomRight") return "end";
  return "center";
}

/** The HUD chip: the style's hud fill, a hairline sheen at the top, a soft drop so it lifts off the game. */
export function chip(radius?: number, strong = false): Api.DrawDesc {
  const r = radius ?? shapeOf("hud").radius;
  return {
    shape: { kind: "rect", radii: [r, r, r, r], corner: shapeOf("hud").corner },
    fills: [solid(token("hud", strong ? 0.94 : 0.82)), vgrad([[0, token("sheen", 0.1)], [0.4, token("sheen", 0)]])],
    strokes: [stroke(1, token("lineStrong", 0.55))],
    effects: [drop(2, 8, token("shadow", 0.45))],
  };
}

/** A track a fill sits in: recessed, dark. */
export function trackDraw(r: number, shape: Api.DrawShape | null = null): Api.DrawDesc {
  return {
    shape: shape ?? { kind: "rect", radii: [r, r, r, r] },
    fills: [solid(token("shadow", 0.55)), solid(token("controlStrong", 0.35))],
    effects: [inner(1, 3, token("shadow", 0.7))],
    strokes: [stroke(1, token("lineStrong", 0.5), "outside")],
    clip: "shape",
  };
}

export type Level = "normal" | "low" | "critical" | "empty";

/** A meter's level by its fraction. */
export function levelOf(frac: number, low: number, critical: number): Level {
  if (frac <= 0) return "empty";
  if (frac < critical) return "critical";
  if (frac < low) return "low";
  return "normal";
}

/** A meter kind's colour; health turns warn and danger by level. */
export function kindColor(kind: Api.MeterKind | undefined, level: Level, custom?: Api.Paint | null): Rgba {
  if (custom) return paintRgba(custom, role("health"));
  const k = kind ?? "health";
  if (k === "health") {
    if (level === "critical" || level === "empty") return role("danger");
    if (level === "low") return mix(role("health"), role("warn"), 0.7);
    return role("health");
  }
  return role(k);
}

/** The paint's colour as sRGB (a token resolved from the frame's style). */
export function paintRgba(p: Api.Paint | null | undefined, fallback: Rgba): Rgba {
  if (!p) return fallback;
  if (p.token) {
    const c = role(p.token);
    return { ...c, a: c.a * (p.alpha ?? 1) };
  }
  if (p.color) return { r: p.color.r, g: p.color.g, b: p.color.b, a: p.alpha ?? 1 };
  return fallback;
}

/** A filled bar's look: its colour as a top-lit gradient with a sheen line. */
export function fillDraw(c: Rgba, r: number, alpha = 1): Api.DrawDesc {
  const top = mix(c, { r: 1, g: 1, b: 1, a: c.a }, 0.22);
  const bottom = mix(c, { r: 0, g: 0, b: 0, a: c.a }, 0.22);
  return {
    shape: { kind: "rect", radii: [r, r, r, r] },
    fills: [vgrad([[0, lit(top, alpha)], [0.5, lit(c, alpha)], [1, lit(bottom, alpha)]])],
    strokes: [stroke(1, token("sheen", 0.25 * alpha))],
  };
}

/** The ellipse in the element's box (a circle when square). */
export const ellipseShape = (): Api.DrawShape => ({ kind: "ellipse" });

/** The frame's text direction: true lays out right to left. */
export const rtl = (): boolean => kit.frameDirection() === "rtl";

/**
 * An absolute place that stays left to right under a right-to-left
 * layout: og-ui mirrors every element's place and transform after the
 * build, but not the paths drawn inside one, so an instrument (a dial, a
 * map, a compass, a flight ladder) that pairs placed elements with drawn
 * ones pre-mirrors its places to keep east east and its numbers on their
 * ticks.
 */
export function fixed(insets: Api.Edges, more: Props = {}): Props {
  if (!rtl()) return absolute(insets, more);
  const { start, end, ...rest } = insets;
  const flipped: Api.Edges = { ...rest, ...(end !== undefined ? { start: end } : {}), ...(start !== undefined ? { end: start } : {}) };
  const t = more.transform;
  const tm = t ? { ...t, ...(t.x !== undefined ? { x: -t.x } : {}), ...(t.rotate !== undefined ? { rotate: -t.rotate } : {}), ...(t.pivotX !== undefined ? { pivotX: 1 - t.pivotX } : {}) } : undefined;
  return absolute(flipped, tm ? { ...more, transform: tm } : more);
}
