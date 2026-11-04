// The saga map (docs/ui.md §11 genre patterns, Appendix B: level and saga
// maps): a run's levels along a painted path that scrolls, the avatar's pin
// on the level the player stands on. Which level the map last opened on is
// the component's own state (og.ui.memory, so a snapshot carries it); the
// game holds the levels, their stars and where the player stands.

import type * as Api from "@origingame/engine";
import {
  accent, artSlot, clamp, fill, frame, glow, grow, hex, keyHash, kit, line, mix, over, paint, px, radial, role, rtl, shadow, solid, st, still, surface,
  symbol, text, token, u, vgrad, vstate, type Mut, type Rgba,
} from "./core.ts";
import { STAR_GOLD } from "./stars.ts";

/** A point on the path. */
interface Pt { x: number; y: number }

/** A smooth line through `pts` as a path of cubic Béziers (Catmull-Rom): the points first, then each segment's two controls and its end. */
function smooth(pts: Pt[]): Api.Vector2[] {
  if (pts.length < 2) return pts.map((p) => ({ x: p.x, y: p.y }));
  const out: Api.Vector2[] = [{ x: pts[0]!.x, y: pts[0]!.y }];
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)]!;
    const p1 = pts[i]!;
    const p2 = pts[i + 1]!;
    const p3 = pts[Math.min(pts.length - 1, i + 2)]!;
    out.push({ x: p1.x + (p2.x - p0.x) / 6, y: p1.y + (p2.y - p0.y) / 6 }, { x: p2.x - (p3.x - p1.x) / 6, y: p2.y - (p3.y - p1.y) / 6 }, { x: p2.x, y: p2.y });
  }
  return out;
}

/** A line of the path, an absolute layer over the whole content (inert): its points, width, ink and glow. */
/** The smoothed path as a dense polyline (each curve sampled), so it can be cut anywhere along it. */
function flatten(pts: Pt[]): Pt[] {
  const c = smooth(pts);
  const out: Pt[] = [c[0]!];
  for (let i = 1; i + 2 < c.length; i += 3) {
    const [a, b, d, e] = [c[i - 1]!, c[i]!, c[i + 1]!, c[i + 2]!];
    for (let k = 1; k <= 16; k++) {
      const t = k / 16;
      const m = 1 - t;
      out.push({ x: m * m * m * a.x + 3 * m * m * t * b.x + 3 * m * t * t * d.x + t * t * t * e.x, y: m * m * m * a.y + 3 * m * m * t * b.y + 3 * m * t * t * d.y + t * t * t * e.y });
    }
  }
  return out;
}

/** The part of a polyline at or above `cut` (content y), cut exactly there. */
function above(line: Pt[], cut: number): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i < line.length; i++) {
    const p = line[i]!;
    const q = line[i - 1];
    if (q && (q.y <= cut) !== (p.y <= cut)) {
      const t = (cut - q.y) / (p.y - q.y);
      out.push({ x: q.x + (p.x - q.x) * t, y: cut });
    }
    if (p.y <= cut) out.push(p);
  }
  return out;
}

/**
 * A line of the path, an absolute layer over the whole content (inert): its points, width, ink and glow. `endY`
 * (content y, a vertical map's view edge of last tick) ends it before the view's clip: it stops a halo's reach above
 * the edge and fades to nothing over its last 80 px there, so the clip never cuts a line or its glow.
 */
function pathLayer(id: string, pts: Pt[], width: number, ink: Api.Paint, effects: Api.Effect[] = [], endY = 0, H = 0): void {
  if (pts.length < 2) return;
  const reach = width / 2 + effects.reduce((r, e) => Math.max(r, e.kind === "outerGlow" ? (e as { blur: number }).blur + ((e as { spread?: number }).spread ?? 0) : 0), 0);
  const cut = endY > 0 && H > 0 ? endY - reach : 0;
  const line = cut > 0 ? above(flatten(pts), cut) : null;
  if (line && line.length < 2) return;
  const fade = u(80);
  // A path's gradient runs over the line's own box (its points' extent and 0.75 of its width round it, og-render's
  // polyline reach for a round join), not the element's: the fade's ends are placed in that box.
  let fills: Api.Fill[] = [{ kind: "solid", color: ink }];
  if (line) {
    const ys = line.map((p) => p.y);
    const top = Math.min(...ys) - 0.75 * width;
    const h = Math.max(1, Math.max(...ys) - Math.min(...ys) + 1.5 * width);
    fills = [{ kind: "gradient", type: "linear", from: { x: 0.5, y: (cut - top) / h }, to: { x: 0.5, y: (cut - fade - top) / h }, stops: [{ offset: 0, color: { ...ink, alpha: 0 } as Api.Paint }, { offset: 1, color: ink }] }];
  }
  kit.open(id, { width: grow(), height: grow(), part: `Saga.path${id === "Lit" ? "[lit]" : ""}`, partClass: "badge" });
  kit.setDraw({ shape: line ? { kind: "path", points: line, curves: false, width, cap: "round", join: "round" } : { kind: "path", points: smooth(pts), curves: true, width, cap: "round", join: "round" }, fills, effects } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
}

/**
 * A saga map (Candy Crush Saga, Royal Match): see `fn sagaMap` in
 * crates/og-api/api/ui.ogt.
 */
export function sagaMap(id: string, desc: Api.SagaMapDesc): Api.SagaMapResult {
  const { viewport: vp } = frame();
  const nodes = desc.nodes ?? [];
  const n = nodes.length;
  const out: Mut<Api.SagaMapResult> = { picked: null, refused: null, hovered: null };
  const vertical = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const moving = !still();
  const mirror = rtl() && !vertical;
  const curIdx = Math.max(0, nodes.findIndex((nd) => nd.key === desc.current));
  const hl = accent();

  // Its size of last tick (the view's cross size the path winds in); before that, the screen's.
  const crossKey = `~s.cross.${id}`;
  const crossSize = kit.memory(crossKey, 0) || Math.round((vertical ? vp.width : vp.height) * 0.7);
  const spacing = desc.spacing && desc.spacing > 0 ? u(desc.spacing) : u(vertical ? 128 : 150);
  const pad = u(vertical ? 120 : 130);
  const length = Math.max(vertical ? vp.height : vp.width, pad * 2 + spacing * Math.max(0, n - 1));
  const W = vertical ? crossSize : length;
  const H = vertical ? length : crossSize;
  // Where each level sits: along the path from the start (the foot in portrait), across it winding as the map's art
  // or a gentle wave decides, inside the room a badge, its stars and the pin need.
  const amp = Math.max(0, crossSize / 2 - u(vertical ? 70 : 96));
  const mid = crossSize / 2 + (vertical ? 0 : u(18));
  const at: Pt[] = nodes.map((nd, i) => {
    const along = pad + i * spacing;
    const lane = nd.lane !== undefined && nd.lane !== 0 ? clamp(nd.lane, -1, 1) : Math.sin(i * 1.15 + 0.4) * 0.62;
    const across = mid + lane * amp;
    return vertical ? { x: across, y: length - along } : { x: mirror ? length - along : along, y: across };
  });

  // An absolute child's start inset for a point at x (from the content's left): layout mirrors start insets right to
  // left, and the path's points are already drawn mirrored, so a mirrored map measures from the right.
  const startOf = (x: number, w: number): number => Math.round(rtl() ? W - x - w / 2 : x - w / 2);
  kit.open(id, { direction: "column", width: grow(), height: grow(), part: "Saga.map", partClass: "panel" });
  // Where the view's foot met the content last tick (a vertical map with chrome docked under it); remembered under
  // this element after the view closes, so read here, not inside the view.
  const endY = vertical && (desc.footInset ?? 0) > 0 ? kit.memory(`~s.end.${id}`, 0) : 0;
  // The backdrop: the map's art behind the view, still while the path scrolls over it (a parallax sky), a shade over
  // it so the badges read; without art, a dusk gradient.
  kit.open("Backdrop", { width: grow(), height: grow(), part: "Saga.backdrop", partClass: "badge" });
  kit.setDraw({
    shape: { kind: "rect", radii: [0, 0, 0, 0] },
    fills: [
      ...(desc.backdrop ? [{ kind: "image", texture: desc.backdrop, fit: "cover", focal: { x: 0.5, y: 0.5 } } as Api.Fill] : [vgrad([[0, paint(mix(role("backdrop"), hex("#20305a"), 0.6))], [1, paint(mix(role("backdrop"), hex("#5a2e48"), 0.5))]])]),
      vgrad([[0, token("shadow", 0.35)], [0.5, token("shadow", 0.1)], [1, token("shadow", 0.45)]]),
    ],
  } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });

  kit.openScroll("View", { width: grow(), height: grow(), padding: { left: 0, top: 0, right: 0, bottom: 0 }, gap: 0, axes: vertical ? "vertical" : "horizontal" });
  kit.open("Path", { width: px(W), height: px(H), part: "Saga.route", partClass: "badge" });
  // The path: a wide soft trail under every level, the way ahead a quiet line on it, and the way travelled lit in the
  // accent with a glow, from the first level to the one the player stands on.
  pathLayer("Trail", at, u(26), token("shadow", 0.42), [], endY, H);
  pathLayer("Ahead", at.slice(Math.max(0, curIdx)), u(5), paint(mix(role("ink"), role("inkFaint"), 0.5), 0.55), [], endY, H);
  pathLayer("Lit", at.slice(0, curIdx + 1), u(6), paint(mix(hl, hex("#ffffff"), 0.25)), [glow(u(12), paint(hl, 0.85), 1)], endY, H);

  const rt = kit.memory("~s.rt", 0);
  const rk = kit.memory("~s.rk", 0);
  let curPath = "";
  nodes.forEach((nd, i) => {
    const key = nd.key || `n${i}`;
    const label = nd.label || `${i + 1}`;
    const cur = i === curIdx && !!desc.current;
    const done = !nd.locked && i < curIdx;
    const progress = nd.locked ? "locked" : cur ? "current" : done ? "done" : "open";
    const maxStars = nd.boss ? 0 : nd.maxStars ?? 3;
    const b = u(nd.boss ? 96 : cur ? 84 : 66);
    const starH = maxStars > 0 ? u(18) : 0;
    const ink: Rgba = nd.boss ? role("danger") : nd.locked ? mix(role("inkFaint"), role("panel"), 0.3) : hl;
    const shaking = moving && rt > 0 && tick - rt < 18 && rk === keyHash(key);
    const shake = shaking ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
    const name = fill(nd.boss ? desc.bossLabel ?? "Boss, level {label}" : desc.levelLabel ?? "Level {label}", { label });
    const words = [
      name,
      maxStars > 0 && !nd.locked ? fill(desc.starsLabel ?? "{n} of {max} stars", { n: Math.min(nd.stars ?? 0, maxStars), max: maxStars }) : "",
      nd.locked ? desc.lockedLabel ?? "Locked" : "",
      cur ? desc.currentLabel ?? "You are here" : "",
    ].filter(Boolean).join(", ");
    const p = at[i]!;
    const r = surface(key, {
      part: "Saga.node",
      // Class badge: og-ui strokes no outline round an undressed badge (a slot takes a hairline in the line ink), so
      // neither a style's button outline nor a slot's box frames the badge and its stars.
      cls: "badge",
      flags: [progress, nd.boss ? "boss" : ""],
      fill: null,
      direction: "column",
      width: px(b),
      height: px(b + Math.round(starH * 0.6)),
      cross: "center",
      align: "start",
      radius: Math.round(b / 2),
      lift: nd.locked ? 0 : -u(4),
      pressScale: 0.94,
      semantics: { role: "button", name: words },
      layer: {
        insets: { start: startOf(p.x, b), top: Math.round(p.y - b / 2) },
        z: cur ? 3 : nd.boss ? 2 : 1,
        transform: { x: shake },
        states: [{ name: "progress", states: [vstate("done"), vstate("current"), vstate("open"), vstate("locked")], transitions: [] }],
        state: [{ group: "progress", state: progress }],
      },
    }, () => {
      // The current level breathes: a soft ring of the accent behind its badge.
      if (cur) {
        const k = moving ? 1 + 0.07 * Math.sin(tick * 0.09) : 1.04;
        kit.open("Ring", { width: px(b), height: px(b), part: "Saga.ring", partClass: "badge" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [radial([[0, paint(hl, 0)], [0.62, paint(hl, 0.0)], [0.78, paint(hl, 0.5)], [1, paint(hl, 0)]])] } as Api.DrawDesc);
        kit.close();
        over({ insets: { start: 0, top: 0 }, transform: { scaleX: k * 1.32, scaleY: k * 1.32 }, inert: true });
      }
      // The badge: a hexagon of dark glass ringed in the level's ink, the number in the display face (a lock under it
      // when locked), a boss's portrait clipped inside.
      kit.open("Badge", { direction: "column", width: px(b), height: px(b), align: "center", cross: "center", gap: 0, part: "Saga.badge", partClass: "badge" });
      const hexShape: Api.DrawShape = { kind: "polygon", sides: 6, rotation: Math.PI / 6, rounding: u(6) };
      kit.setDraw({
        shape: hexShape,
        fills: [vgrad([[0, paint(mix(role("panelRaised"), ink, nd.locked ? 0.05 : 0.22), 0.94)], [1, paint(mix(role("panel"), role("shadow"), 0.4), 0.94)]])],
        strokes: [line(cur || nd.boss ? 3 : 2, paint(nd.locked ? ink : mix(ink, hex("#ffffff"), 0.2))), line(1, token("shadow", 0.7), "outside")],
        effects: [shadow(u(4), u(10), 0.6), ...(nd.locked ? [] : [glow(u(cur || nd.boss ? 16 : 8), paint(ink, cur || nd.boss ? 0.7 : 0.4), 0)])],
      } as Api.DrawDesc);
      if (nd.boss) {
        const inner = Math.round(b * 0.8);
        kit.open("Portrait", { width: px(inner), height: px(inner), part: "Saga.portrait", partClass: "badge" });
        if (nd.image) kit.setDraw({ shape: hexShape, fills: [{ kind: "image", texture: nd.image, fit: "cover", focal: { x: 0.5, y: 0.35 } }], clip: "shape" } as Api.DrawDesc);
        else kit.setDraw({ shape: hexShape, fills: [radial([[0, paint(mix(ink, role("panel"), 0.4))], [1, paint(mix(ink, role("shadow"), 0.6))]])] } as Api.DrawDesc);
        if (!nd.image) symbol("Crown", "crown", Math.round(inner * 0.5), mix(ink, hex("#ffffff"), 0.4));
        kit.close();
        over({ insets: { start: Math.round((b - inner) / 2), top: Math.round((b - inner) / 2) }, inert: true });
        // The boss's number on a plate over the portrait's foot.
        kit.open("Plate", { direction: "row", padding: { left: u(8), top: 1, right: u(8), bottom: 1 }, radius: 999, background: paint(mix(ink, role("shadow"), 0.35), 0.95), part: "Saga.label", partClass: "badge" });
        text("Label", label, { step: st(15), font: "display", nowrap: true, color: hex("#ffffff") });
        kit.close();
        over({ insets: { start: Math.round(b / 2 - u(22)), bottom: u(2) }, inert: true, z: 2 });
      } else {
        text("Label", label, { step: st(cur ? 30 : 22), font: "display", nowrap: true, color: nd.locked ? mix(role("inkFaint"), role("ink"), 0.25) : role("ink") });
        if (nd.locked) symbol("Lock", "lock", u(16), mix(role("inkFaint"), role("ink"), 0.3), { icon: "lock" });
      }
      kit.close();
      // The stars on a dark pill over the badge's foot: earned lit in gold, the rest as outlines.
      if (maxStars > 0 && !nd.locked) {
        const sz = u(cur ? 17 : 14);
        kit.open("Stars", { direction: "row", gap: 1, cross: "center", padding: { left: u(5), top: 2, right: u(5), bottom: 2 }, radius: 999, background: token("shadow", 0.78), part: "Saga.stars", partClass: "badge" });
        for (let k = 0; k < maxStars; k++) {
          const on = k < (nd.stars ?? 0);
          symbol(kit.idi("Star", k), on ? "star" : "starOutline", sz, on ? STAR_GOLD : mix(role("inkFaint"), role("ink"), 0.25));
        }
        kit.close();
        over({ insets: { start: Math.round(b / 2 - (maxStars * (sz + 1) + u(10)) / 2), top: Math.round(b - sz * 0.75) }, inert: true, z: 2 });
      }
    });
    if (cur) curPath = kit.lastPath();
    if (r.hovered || r.focused) out.hovered = key;
    if (r.clicked) {
      if (nd.locked) {
        out.refused = key;
        kit.remember("~s.rt", tick);
        kit.remember("~s.rk", keyHash(key));
        kit.feedback("error", -1);
      } else {
        out.picked = key;
        kit.feedback("confirm", -1);
      }
    }
  });

  // The avatar's pin over the level the player stands on: a round portrait ringed in the accent on a point, bobbing.
  if (desc.current && n > 0) {
    const p = at[curIdx]!;
    const s = u(58);
    const bob = moving ? Math.round(Math.sin(tick * 0.07) * u(4)) : 0;
    kit.open("Pin", { direction: "column", width: px(s), cross: "center", gap: 0, part: "Saga.pin", partClass: "badge" });
    kit.open("Disc", { width: px(s), height: px(s), part: "Saga.avatar", partClass: "badge" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [solid(role("panel"))], strokes: [line(3, paint(mix(hl, hex("#ffffff"), 0.3)))], effects: [glow(u(10), paint(hl, 0.7), 0), shadow(u(4), u(10), 0.6)] } as Api.DrawDesc);
    artSlot("Art", "Saga.avatarArt", { texture: desc.avatar ?? null, kind: "avatar", width: px(s - 8), height: px(s - 8), radius: Math.round(s / 2), focal: { x: 0.5, y: 0.3 } });
    over({ insets: { start: 4, top: 4 }, inert: true });
    kit.close();
    kit.open("Point", { width: px(u(16)), height: px(u(11)) });
    kit.setDraw({ shape: { kind: "path", points: [{ x: 0, y: 0 }, { x: u(16), y: 0 }, { x: u(8), y: u(11) }], closed: true, filled: true }, fills: [solid(mix(hl, hex("#ffffff"), 0.3))] } as Api.DrawDesc);
    kit.close();
    kit.close();
    const top = p.y - u(nodes[curIdx]!.boss ? 96 : 84) / 2 - s - u(14);
    over({ insets: { start: startOf(p.x, s), top: Math.round(top) }, transform: { y: bob }, inert: true, z: 4 });
  }
  kit.close();
  kit.close();
  {
    // The view's foot in the content's coordinates, for the path's end next tick. Bounds are clipped to the view, so
    // the scroll is read off the current level, whose top is at its point less half its badge in the content.
    const view = kit.lastPath();
    const vb = kit.bounds(view);
    const cb = curPath ? kit.bounds(curPath) : null;
    const cp = at[curIdx];
    if (vb && cb && cp && cb.height >= u(nodes[curIdx]!.boss ? 96 : 84) - 1) {
      const top = cp.y - u(nodes[curIdx]!.boss ? 96 : 84) / 2 - (cb.y - vb.y);
      kit.remember(`~s.end.${id}`, Math.round(top + vb.height));
    }
  }
  // Open on the current level, centred, and follow it when it changes. og-ui scrolls by where the badge was last
  // frame (clipped or not), so it is asked until the badge shows in the view, then left to the player's scrolling.
  const want = keyHash(desc.current ?? "");
  if (curPath && kit.memory("~s.shown", 0) !== want) {
    kit.scrollIntoView(curPath, true);
    if (kit.bounds(curPath)) kit.remember("~s.shown", want);
  }
  // The game's docked chrome at the foot: the view stops above it (the backdrop runs on behind it), and a little more
  // where that leaves a level cut by the view's edge. The view opens centred on the current level and the levels are
  // evenly spaced along it, so a level's distance from either edge is the half view less a whole number of spacings:
  // the spacer grows until both edges fall between two levels (a whole level shows above the button, or none).
  if ((desc.footInset ?? 0) > 0) {
    const foot = Math.round(desc.footInset!);
    let extra = 0;
    if (vertical) {
      const mainH = kit.memory(`~s.main.${id}`, 0) || vp.height;
      const half = (mainH - foot) / 2;
      const r = ((half % spacing) + spacing) % spacing;
      // A level's footprint about its centre: its badge, and its stars under it.
      const lo = u(66) / 2 + u(24);
      const hi = spacing - u(66) / 2 - u(10);
      const d = r > hi ? r - hi : r < lo ? r - hi + spacing : 0;
      extra = Math.round(2 * d);
    }
    kit.spacer("Foot", px(foot + extra));
    // The path runs on past the view's edge: its last 80 px fade into a shade that holds on behind the button, so the
    // edge reads as the way going on, not a cut (undressed: a style's dress would replace the gradient).
    const fadeH = foot + extra + u(80);
    kit.open("FootFade", { width: px(vertical ? crossSize : vp.width), height: px(fadeH) });
    kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("shadow", 0)], [(u(80) * 0.5) / fadeH, token("shadow", 0.3)], [u(80) / fadeH, token("shadow", 0.62)], [1, token("shadow", 0.72)]])] });
    kit.close();
    over({ insets: { start: 0, bottom: 0 }, inert: true, z: 3 });
  }
  kit.close();
  const me = kit.lastPath();
  const box = kit.bounds(me);
  if (box) {
    kit.remember(crossKey, Math.round(vertical ? box.width : box.height));
    kit.remember(`~s.main.${id}`, Math.round(vertical ? box.height : box.width));
  }
  kit.semantics({ role: "list", name: desc.label ?? "Levels" });
  return out;
}
