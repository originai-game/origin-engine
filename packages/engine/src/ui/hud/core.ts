// What the HUD components (docs/ui.md §11 "HUD", Appendix C) share beyond
// the feedback core: where a component opens (its own anchored group in
// the safe area, or in place), the HUD chip every readout sits on, meter
// colours by kind and level, and a stable id for a list item.

import type * as Api from "@origingame/engine";
import { absolute, type Props, frame, kit, role, type Rgba, solid, token, drop, stroke, inner, vgrad, lit, mix, shapeOf, box, icon, leaf, px, radial, teamColor, innerGlow, apply, stepSize } from "../feedback/core.ts";

export * from "../feedback/core.ts";

/**
 * The width a HUD piece may take across the safe area, inside the style's
 * gutters: what a bar measures itself against before it compacts or
 * wraps rather than run off the screen.
 */
export function hudRoom(): number {
  const f = frame();
  const vp = f.viewport;
  // The HUD band (og.ui.viewport().hud): the safe area, at most UiDesc.hudMaxAspect wide on an ultrawide.
  // og-ui fills the band in every frame viewport (it is optional only on a viewport a host passes in).
  return vp.hud!.width - f.t.gutter * 2;
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
  // An offset away from the edge (a frame stepped under another corner's) counts in the stack's reach, so what follows clears it.
  const next = start + h + (middle ? 0 : Math.abs(more.offsetY ?? 0));
  let dy: number;
  if (middle) {
    // A stack on a side is centred as a whole on the side's middle (by its
    // extent last tick), so a tracker and a combo under it both stay on screen.
    // The stack's extent last tick, but never less than its first group's: a stack that has just appeared
    // (no total yet) is centred by the size it now has, not by none (which put it half a screen low for a tick).
    if (first) kit.remember(`${key}/top`, sideTop(anchor, Math.max(kit.memory(`${key}/total`, 0), h), t, gap));
    dy = kit.memory(`${key}/top`, 0) + start + h / 2;
  } else {
    dy = start;
  }
  kit.remember(`${key}/tick`, t);
  kit.remember(`${key}/reach`, next);
  kit.remember(`${key}/total`, next);
  if (anchor.startsWith("bottom")) dy = -dy;
  // og-ui lays anchored groups out inside the HUD band on an ultrawide.
  kit.anchor(path, { anchor, direction: "column", cross: crossOf(anchor), ...more, offsetY: dy + (more.offsetY ?? 0) }, () => build(true));
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
  // A dark well, not a grey wash: near-black (og-ui blends in linear light, so 80 % reads as a deep well over any scene),
  // a 2 px inner shadow along its top, a faint lit lip along its bottom and a hairline outside.
  return {
    shape: shape ?? { kind: "rect", radii: [r, r, r, r] },
    fills: [solid(lit(BLACK, 0.8)), vgrad([[0, lit(BLACK, 0.25)], [0.6, lit(BLACK, 0)], [1, lit(WHITE, 0.06)]])],
    effects: [inner(1, 2, lit(BLACK, 0.8)), drop(1, 4, lit(BLACK, 0.45))],
    strokes: [stroke(1, token("lineStrong", 0.35), "outside")],
    clip: "shape",
  };
}

export const BLACK: Rgba = { r: 0, g: 0, b: 0, a: 1 };
export const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

/**
 * Text shrunk below the type ramp's smallest step: at 200 % even the ramp's
 * least step outgrows a corner figure, so it is drawn at a step and scaled
 * to `target` px about its centre; `h` is its box's height, and the move
 * keeps its bottom where layout put it.
 */
export function shrunk(step: Api.TypeStepName, target: number, h: number): { k: number; transform?: Api.UiTransform } {
  const k = Math.min(1, target / stepSize(step).size);
  return k >= 1 ? { k } : { k, transform: { x: 0, y: ((1 - k) * h) / 2, scaleX: k, scaleY: k } };
}

/** A stable hue for a thing without art, from its icon or name, so a placeholder is still told apart by colour as well as glyph. */
export function hueOf(key: string): Rgba {
  let h = 0;
  for (const ch of key) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return teamColor(Math.abs(h));
}

/**
 * A thing's art filling a `side` square (an ability, a buff, a cast): the
 * texture when there is one, else a gradient in `hue` with the icon's
 * glyph large in white, never a line icon on a dark square (§15.5.3).
 */
export function artTile(id: string, image: Api.Texture | null | undefined, iconName: string, side: number, hue: Rgba, props: Props, part?: string): void {
  if (image) {
    kit.image(id, { texture: image, width: px(side), height: px(side), label: "" });
    apply(props);
    return;
  }
  box(`${id}Box`, { width: px(side), height: px(side), align: "center", cross: "center", ...(part ? { part } : {}) }, {
    fills: [radial([[0, lit(mix(hue, WHITE, 0.25))], [0.65, lit(mix(hue, BLACK, 0.35))], [1, lit(mix(hue, BLACK, 0.7))]], 0.5, 0.35, 0.75, 0.75)],
  }, () => {
    icon("Glyph", iconName, Math.round(side * 0.66), lit(WHITE, 0.95));
  }, props);
}

/**
 * A socket's material over the art it frames (§15.5.6): a top-lit rim
 * (light along the top, dark along the bottom), a 2 px inner shadow and a
 * soft vignette, so a picture reads set into its frame rather than pasted.
 */
export function socketRim(id: string, side: number, radius: number, corner: Api.CornerShape | undefined, rim: number, props: Props, lift = 0.42): void {
  leaf(id, { width: px(side), height: px(side) }, {
    shape: { kind: "rect", radii: [radius, radius, radius, radius], ...(corner ? { corner } : {}) },
    fills: [vgrad([[0, lit(WHITE, 0.1)], [0.3, lit(WHITE, 0)]])],
    strokes: [{ width: rim, align: "inside", paint: vgrad([[0, lit(WHITE, lift)], [0.5, lit(WHITE, 0.08)], [1, lit(BLACK, 0.7)]]) }],
    effects: [inner(1, 2, lit(BLACK, 0.7), 1), innerGlow(side * 0.12, lit(BLACK, 0.4))],
  }, props);
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
