// What the HUD components (docs/ui.md §11 "HUD", Appendix C) share beyond
// the feedback core: where a component opens (its own anchored group in
// the safe area, or in place), the HUD chip every readout sits on, meter
// colours by kind and level, and a stable id for a list item.

import type * as Api from "@origingame/engine";
import { absolute, type Props, frame, kit, role, type Rgba, solid, token, drop, stroke, inner, vgrad, lit, mix, shapeOf } from "../feedback/core.ts";

export * from "../feedback/core.ts";

/** A desc's anchor: its own when it names one (null too), else the component's default. */
export function anchorOf(desc: { anchor?: Api.Anchor | null }, fallback: Api.Anchor | null): Api.Anchor | null {
  return "anchor" in desc && desc.anchor !== undefined ? desc.anchor : fallback;
}

/**
 * Builds `build` in an anchored group (in the safe area, at the style's
 * gutter) when `anchor` is set, else in place.
 */
export function hudPlace(id: string, anchor: Api.Anchor | null, build: (inAnchor: boolean) => void, more: Partial<Api.AnchorDesc> = {}): void {
  if (anchor) kit.anchor(`${id}Hud`, { anchor, direction: "column", cross: crossOf(anchor), ...more }, () => build(true));
  else build(false);
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
