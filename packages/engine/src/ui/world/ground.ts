// og.ui.selectionRing, og.ui.telegraph, og.ui.pathLine and
// og.ui.placementGhost: the shapes of §14 that lie on the ground. Each is
// the ground's points projected with og.Camera, drawn as one world element
// (so it is under the screen's elements, as a decal would be). With a
// GroundDesc each outline is cut into short edges and every vertex lifted
// onto the terrain or the colliders under it, so a shape on a slope or a
// hill follows it instead of floating over it or cutting into it.
// Read against StarCraft II's and Baldur's Gate 3's selection rings,
// Final Fantasy XIV's and WildStar's telegraphs (a tinted area with a
// bright edge and a fill growing to it as the cast runs), Baldur's Gate 3's
// and XCOM 2's movement paths (a line with chevrons marching to its end,
// past this turn's reach in another colour, the cost at its end) and the
// placement ghosts of Age of Empires IV and Cities: Skylines (the
// footprint in success or hatched danger, its cells, a tower's reach).

import type * as Api from "@origingame/engine";
import { a11y, fixed, box, clamp01, drape, easeOut, edge, glow, groundAt, groundCircle, groups, kit, leaf, lit, mix, msToTicks, now, onGround, paintRgba, phase, pulse, px, reactionColor, reduced, role, sel, solid, stroke, surface, text, token, ts, u, type Ground, type Rgba, type Surface, apply } from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

type P = Api.Vector2;

/**
 * A shape of screen points as one part, its element fitted round them (so
 * the probe's bounds are the shape's, and nothing beyond it is drawn).
 */
function shape(id: string, pts: P[], pad: number, part: string | null, draw: (local: P[]) => Api.DrawDesc): void {
  let x0 = Infinity;
  let y0 = Infinity;
  let x1 = -Infinity;
  let y1 = -Infinity;
  for (const p of pts) {
    x0 = Math.min(x0, p.x);
    y0 = Math.min(y0, p.y);
    x1 = Math.max(x1, p.x);
    y1 = Math.max(y1, p.y);
  }
  const left = x0 - pad;
  const top = y0 - pad;
  leaf(id, { width: px(Math.ceil(x1 - x0 + pad * 2)), height: px(Math.ceil(y1 - y0 + pad * 2)), ...(part ? { part } : {}) }, draw(pts.map((p) => ({ x: p.x - left, y: p.y - top }))), fixed({ start: left, top }));
}

/** A polyline part. */
function line(id: string, pts: P[], width: number, paint: Api.Paint, part: string | null, more: { closed?: boolean; effects?: Api.Effect[]; outline?: boolean } = {}): void {
  if (pts.length < 2) return;
  shape(id, pts, width / 2 + 2, part, (local) => ({
    shape: { kind: "path", points: local, width, closed: !!more.closed, cap: "round", join: "round" },
    fills: [solid(paint)],
    strokes: more.outline ? [stroke(1, token("shadow", 0.6), "outside")] : [],
    effects: more.effects ?? [],
  }));
}

/** An area of the ground filled as one shape: the inside of its outline (even-odd, so a loop inside it is a hole). */
function area(id: string, poly: P[], paint: Api.Paint, part: string | null): void {
  if (poly.length < 3) return;
  shape(id, poly, 1, part, (local) => ({
    shape: { kind: "path", points: local, width: 1, closed: true, filled: true },
    fills: [solid(paint)],
    strokes: [],
    effects: [],
  }));
}

/** Projected points, or null when any is behind the camera (a shape is drawn whole or not at all). */
function all(g: Ground, pts: Api.Vector3[]): P[] | null {
  const out: P[] = [];
  for (const p of pts) {
    const s = g.local(p);
    if (!s) return null;
    out.push(s);
  }
  return out;
}

export function selectionRing(world: Api.World, id: string, desc: Api.SelectionRingDesc = {}): void {
  const ground = surface(world, desc.ground);
  const c = ground.at(desc.center ?? { x: 0, y: 0, z: 0 });
  const r0 = desc.radius ?? 0.6;
  const col = paintRgba(desc.color, reactionColor(desc.reaction ?? "self"));
  const age = now() - (desc.selectedAt ?? -1e9);
  const grow = reduced() ? 1 : 1 + 0.35 * (1 - easeOut(clamp01(age / msToTicks(240))));
  const r = r0 * grow;
  const hover = !!desc.hover;
  const ring3 = drape(ground, groundCircle(c, r, 56), true);
  onGround(world, id, c, ring3, (g) => {
    const ring = all(g, ring3);
    if (!ring) return;
    if (!hover) area("Glow", ring, lit(col, 0.1), sel("SelectionRing", "glow"));
    const segs = desc.segments ?? 0;
    const ink = lit(col, hover ? 0.6 : 1);
    const part = sel("SelectionRing", "ring", desc.reaction ?? "self", hover ? "hover" : null);
    const look = { effects: hover ? [] : [glow(3, lit(col, 0.45))], outline: true };
    if (segs > 1) {
      const turn = phase(0.15) * Math.PI * 2;
      const span = (Math.PI * 2) / segs;
      for (let k = 0; k < segs; k++) {
        const pts = all(g, drape(ground, groundCircle(c, r, 12, turn + k * span, span * 0.62), false));
        if (pts) line(kit.idi("Arc", k), pts, hover ? 1.5 : 2.5, ink, part, look);
      }
    } else {
      line("Ring", ring, hover ? 1.5 : 2.5, ink, part, { ...look, closed: true });
    }
  }, { occlusion: "none" });
}

/**
 * A telegraph at `scale` of its reach: its area as one outline (a ring's
 * hole joined to it, filled even-odd) and the loops of its edge, each
 * draped on the surface.
 */
function telegraphShape(d: Api.TelegraphDesc, c: Api.Vector3, ground: Surface, scale: number): { area: Api.Vector3[]; outline: Api.Vector3[][] } {
  const shape: Api.TelegraphShape = d.shape ?? "circle";
  const R = (d.radius ?? 4) * scale;
  const dir = d.direction ?? 0;
  const loop = (pts: Api.Vector3[]) => drape(ground, pts, true);
  if (shape === "circle") {
    const p = loop(groundCircle(c, R, 56));
    return { area: p, outline: [p] };
  }
  if (shape === "ring") {
    const inner = d.inner ?? 0;
    const outer = loop(groundCircle(c, inner + ((d.radius ?? 4) - inner) * scale, 56));
    if (inner <= 0) return { area: outer, outline: [outer] };
    const hole = loop(groundCircle(c, inner, 40));
    return { area: [...outer, outer[0]!, ...hole, hole[0]!], outline: [outer, hole] };
  }
  if (shape === "cone") {
    const a = Math.min(Math.PI * 1.99, d.angle ?? Math.PI / 3);
    const p = loop([c, ...groundCircle(c, R, 32, dir - a / 2, a)]);
    return { area: p, outline: [p] };
  }
  const w = (d.width ?? 2) / 2;
  const rect = loop([groundAt(c, dir, 0, -w), groundAt(c, dir, R, -w), groundAt(c, dir, R, w), groundAt(c, dir, 0, w)]);
  return { area: rect, outline: [rect] };
}

export function telegraph(world: Api.World, id: string, desc: Api.TelegraphDesc = {}): void {
  const ground = surface(world, desc.ground);
  const c = ground.at(desc.center ?? { x: 0, y: 0, z: 0 });
  const hostile = desc.hostile !== false;
  const col = paintRgba(desc.color, hostile ? role("danger") : role("info"));
  const progress = clamp01(desc.progress ?? 0);
  const impact = progress >= 1;
  const full = telegraphShape(desc, c, ground, 1);
  const shape: Api.TelegraphShape = desc.shape ?? "circle";
  const drawn = onGround(world, id, c, full.outline.flat(), (g) => {
    const e = edge("impact", impact ? 1 : 0);
    const flash = impact && e.since < msToTicks(260) ? 1 - e.since / msToTicks(260) : 0;
    const body = all(g, full.area);
    if (body) area("Area", body, lit(col, 0.13 + 0.45 * flash), sel("Telegraph", "area", shape, hostile ? "hostile" : "friendly"));
    if (progress > 0 && !impact) {
      const grown = all(g, telegraphShape(desc, c, ground, progress).area);
      if (grown) area("Fill", grown, lit(mix(col, WHITE, 0.1), 0.2), sel("Telegraph", "fill"));
    }
    full.outline.forEach((o, k) => {
      const pts = all(g, o);
      if (pts) line(kit.idi("Edge", k), pts, 2, lit(mix(col, WHITE, 0.2), 0.85), k === 0 ? sel("Telegraph", "edge") : null, { closed: true, effects: [glow(3, lit(col, 0.4))] });
    });
  });
  if (drawn) apply(groups({ windup: impact ? "impact" : "charging" }, { windup: ["charging", "impact"] }));
}

export function pathLine(world: Api.World, id: string, desc: Api.PathLineDesc = {}): void {
  const ground = surface(world, desc.ground);
  const pts3 = (desc.points ?? []).map((p) => ground.at(p));
  if (pts3.length < 2) return;
  const base = desc.invalid ? role("danger") : paintRgba(desc.color, role("highlight"));
  const beyondInk = role("danger");
  const width = desc.width || 4;
  // Where along the path the reach ends, in world units.
  const lengths = [0];
  for (let i = 1; i < pts3.length; i++) {
    const a = pts3[i - 1]!;
    const b = pts3[i]!;
    lengths.push(lengths[i - 1]! + Math.hypot(b.x - a.x, b.y - a.y, b.z - a.z));
  }
  const total = lengths[lengths.length - 1]!;
  const reach = desc.reach && desc.reach > 0 ? Math.min(desc.reach, total) : total;
  const within: Api.Vector3[] = [];
  const beyond: Api.Vector3[] = [];
  for (let i = 0; i < pts3.length; i++) {
    if (lengths[i]! <= reach) within.push(pts3[i]!);
    if (i > 0 && lengths[i - 1]! < reach && lengths[i]! > reach) {
      const a = pts3[i - 1]!;
      const b = pts3[i]!;
      const t = (reach - lengths[i - 1]!) / (lengths[i]! - lengths[i - 1]!);
      const m = { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t, z: a.z + (b.z - a.z) * t };
      within.push(m);
      beyond.push(m);
    }
    if (lengths[i]! > reach) beyond.push(pts3[i]!);
  }
  const within3 = drape(ground, within, false);
  const beyond3 = beyond.length >= 2 ? drape(ground, beyond, false) : beyond;
  onGround(world, id, pts3[0]!, [...within3, ...beyond3], (g) => {
    const a = within3.map((p) => g.local(p)).filter((p): p is P => p !== null);
    const b = beyond3.map((p) => g.local(p)).filter((p): p is P => p !== null);
    line("Line", a, width, lit(base, 0.9), sel("PathLine", "line", "reach", desc.invalid ? "invalid" : null), { outline: true, effects: [glow(3, lit(base, 0.35))] });
    if (b.length >= 2) line("Beyond", b, width * 0.8, lit(beyondInk, 0.85), sel("PathLine", "line", "beyond"), { outline: true });
    // Chevrons marching to the end, every 28 px along the screen line.
    const all2 = [...a, ...b.slice(1)];
    const gapPx = u(28);
    let walked = -phase(0.9) * gapPx;
    let k = 0;
    for (let i = 1; i < all2.length && k < 60; i++) {
      const p = all2[i - 1]!;
      const q = all2[i]!;
      const seg = Math.hypot(q.x - p.x, q.y - p.y);
      if (seg < 0.01) continue;
      const ux = (q.x - p.x) / seg;
      const uy = (q.y - p.y) / seg;
      let s = gapPx - (walked % gapPx);
      if (walked < 0) s = -walked;
      while (s < seg) {
        const cx = p.x + ux * s;
        const cy = p.y + uy * s;
        const sz = width * 1.3;
        line(kit.idi("Chevron", k++), [
          { x: cx - ux * sz - uy * sz, y: cy - uy * sz + ux * sz },
          { x: cx, y: cy },
          { x: cx - ux * sz + uy * sz, y: cy - uy * sz - ux * sz },
        ], Math.max(1.5, width * 0.45), token("shadow", 0.55), sel("PathLine", "chevron"));
        s += gapPx;
      }
      walked += seg;
    }
    const end = all2[all2.length - 1];
    const prev = all2[all2.length - 2];
    if (end && prev) {
      const endInk = b.length >= 2 || desc.invalid ? beyondInk : base;
      if (desc.invalid) {
        const s = u(8);
        line("End", [{ x: end.x - s, y: end.y - s }, { x: end.x + s, y: end.y + s }], 3, lit(endInk), sel("PathLine", "end", "invalid"), { outline: true });
        line("End2", [{ x: end.x - s, y: end.y + s }, { x: end.x + s, y: end.y - s }], 3, lit(endInk), null, { outline: true });
      } else if (desc.arrow !== false) {
        const d = Math.hypot(end.x - prev.x, end.y - prev.y) || 1;
        const ux = (end.x - prev.x) / d;
        const uy = (end.y - prev.y) / d;
        const s = width * 3;
        line("End", [
          { x: end.x - ux * s - uy * s * 0.8, y: end.y - uy * s + ux * s * 0.8 },
          { x: end.x, y: end.y },
          { x: end.x - ux * s + uy * s * 0.8, y: end.y - uy * s - ux * s * 0.8 },
        ], width, lit(endInk), sel("PathLine", "end"), { outline: true, effects: [glow(4, lit(endInk, 0.6))] });
      }
      if (desc.label) {
        box("Label", { padding: { left: u(6), right: u(6), top: u(2), bottom: u(2) }, part: sel("PathLine", "label"), partClass: "chip" }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] },
          fills: [solid(token("hud", 0.9))],
          strokes: [stroke(1, lit(endInk, 0.8))],
        }, () => text("Text", desc.label ?? "", { step: ts(12), font: "mono", color: token("ink"), nowrap: true }),
          fixed({ start: end.x + u(10), top: end.y - u(26) }, { a11y: a11y("text", desc.label ?? "") }));
      }
    }
  });
}

export function placementGhost(world: Api.World, id: string, desc: Api.PlacementGhostDesc = {}): void {
  const ground = surface(world, desc.ground);
  const c = ground.at(desc.center ?? { x: 0, y: 0, z: 0 });
  const w = (desc.width ?? 2) / 2;
  const d = (desc.depth ?? 2) / 2;
  const rot = desc.rotation ?? 0;
  const valid = desc.valid !== false;
  const col = valid ? role("success") : role("danger");
  // A point of the footprint, u across its width and v along its depth.
  const at = (u0: number, v0: number) => groundAt(c, rot, v0, u0);
  const corners = drape(ground, [at(-w, -d), at(-w, d), at(w, d), at(w, -d)], true);
  const range = desc.range ?? null;
  const ring3 = range ? drape(ground, groundCircle(c, range, 64), true) : null;
  const extent = ring3 ?? corners;
  // A straight line of the footprint (a cell, a hatch), draped.
  const across = (a: Api.Vector3, b: Api.Vector3) => drape(ground, [a, b], false, 64);
  const breath = reduced() ? 1 : 0.8 + 0.2 * pulse(0.8);
  const drawn = onGround(world, id, c, [...extent, ...corners], (g) => {
    if (ring3) {
      const ring = all(g, ring3);
      if (ring) {
        area("RangeFill", ring, lit(role("info"), 0.04), null);
        line("Range", ring, 1.5, lit(role("info"), 0.5), sel("PlacementGhost", "range"), { closed: true });
      }
    }
    const foot = all(g, corners);
    if (!foot) return;
    area("Footprint", foot, lit(col, 0.18 * breath), sel("PlacementGhost", "footprint", valid ? "valid" : "invalid"));
    const cell = desc.cell ?? 0;
    if (cell > 0) {
      let k = 0;
      for (let x = -w + cell; x < w - 1e-4 && k < 64; x += cell) {
        const p = all(g, across(at(x, -d), at(x, d)));
        if (p) line(kit.idi("CellU", k++), p, 1, lit(col, 0.3), k === 1 ? sel("PlacementGhost", "cells") : null);
      }
      for (let y = -d + cell; y < d - 1e-4 && k < 128; y += cell) {
        const p = all(g, across(at(-w, y), at(w, y)));
        if (p) line(kit.idi("CellV", k++), p, 1, lit(col, 0.3), null);
      }
    }
    if (!valid) {
      // Hatching, so the invalid footprint does not rest on colour alone: lines u − v = k clipped to the rectangle.
      const s = Math.max(0.25, Math.min(w, d) / 3);
      let k = 0;
      for (let off = -(w + d) + s; off < w + d && k < 40; off += s) {
        const v0 = Math.max(-d, -w - off);
        const v1 = Math.min(d, w - off);
        if (v1 <= v0) continue;
        const p = all(g, across(at(off + v0, v0), at(off + v1, v1)));
        if (p) line(kit.idi("Hatch", k++), p, 2, lit(col, 0.35), null);
      }
    }
    line("Edge", foot, 2, lit(mix(col, WHITE, 0.15), 0.9), null, { closed: true, effects: [glow(3, lit(col, 0.4))] });
    if (desc.label) {
      const mid = g.local(c);
      if (mid) {
        box("Label", { padding: { left: u(7), right: u(7), top: u(2), bottom: u(2) }, part: sel("PlacementGhost", "label"), partClass: "chip" }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] },
          fills: [solid(token("hud", 0.92))],
          strokes: [stroke(1, lit(col))],
        }, () => text("Text", desc.label ?? "", { step: ts(12), font: "display", color: valid ? token("ink") : lit(role("danger")), nowrap: true }),
          fixed({ start: mid.x - u(40), top: mid.y - u(40) }, { a11y: a11y("status", desc.label ?? "") }));
      }
    }
  });
  if (drawn) apply(groups({ place: valid ? "valid" : "invalid" }, { place: ["valid", "invalid"] }));
}
