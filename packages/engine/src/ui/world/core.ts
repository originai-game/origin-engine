// What the world components (docs/ui.md §14, U6c) share beyond the HUD
// core: a reaction's colour, mass templates declared once per engine, the
// ground plane projected with og.Camera, and a world element that draws
// shapes in screen pixels round a ground point.

import type * as Api from "@origingame/engine";
import { BUILTINS, type Token } from "../../runtime.ts";
import { kit, rarityColor, role, teamColor, type Rgba } from "../hud/core.ts";

export * from "../hud/core.ts";

/** A reaction's colour. */
export function reactionColor(r: Api.Reaction | undefined): Rgba {
  switch (r ?? "hostile") {
    case "hostile": return role("danger");
    case "neutral": return role("warn");
    case "friendly": return role("success");
    case "player": return role("info");
    default: return role("primary");
  }
}

/** A waypoint kind's colour. */
export function waypointColor(kind: Api.WaypointKind, rarity?: Api.Rarity): Rgba {
  switch (kind) {
    case "objective": return role("gold");
    case "marker": return role("info");
    case "ping": return role("warn");
    case "enemy": return role("danger");
    case "ally": return teamColor(0);
    default: return rarityColor(rarity) ?? role("ink");
  }
}

/**
 * A mass template declared once per engine under `name` (its handle kept
 * in the UI's memory, so a snapshot carries it); declared again when its
 * desc's key changes.
 */
export function massTemplate(name: string, key: string, desc: () => Api.MassTemplateDesc): number {
  const keyHash = hash(key);
  const h = kit.memory(`${name}~mass`, -1);
  const k = kit.memory(`${name}~massKey`, -1);
  if (h >= 0 && k === keyHash) return h;
  const handle = kit.defineMass(name, desc());
  kit.remember(`${name}~mass`, handle);
  kit.remember(`${name}~massKey`, keyHash);
  return handle;
}

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619) >>> 0;
  return h % 1e9;
}

const Camera = (BUILTINS as unknown as Record<string, Token>).Camera!;

type CameraValue = { worldToScreen(p: Api.Vector3): Api.Vector2 | null };

/** Projects world points with this tick's og.Camera; null without one. */
export function projector(world: Api.World): ((p: Api.Vector3) => Api.Vector2 | null) | null {
  const r = (world as unknown as { resource(t: Token): { ok: boolean; value?: unknown } }).resource(Camera);
  if (!r.ok || !r.value) return null;
  const cam = r.value as CameraValue;
  return (p) => cam.worldToScreen(p);
}

/** A circle of the ground (xz) round c, n points. */
export function groundCircle(c: Api.Vector3, radius: number, n = 48, from = 0, sweep = Math.PI * 2): Api.Vector3[] {
  const out: Api.Vector3[] = [];
  const closed = sweep >= Math.PI * 2 - 1e-6;
  const count = closed ? n : n + 1;
  for (let i = 0; i < count; i++) {
    const a = from + (sweep * i) / n;
    // Clockwise from +z seen from above: +z, then +x.
    out.push({ x: c.x + Math.sin(a) * radius, y: c.y, z: c.z + Math.cos(a) * radius });
  }
  return out;
}

/** A point of the ground `dist` from c toward `dir` (clockwise from +z), `side` to its right. */
export function groundAt(c: Api.Vector3, dir: number, dist: number, side = 0): Api.Vector3 {
  return {
    x: c.x + Math.sin(dir) * dist + Math.cos(dir) * side,
    y: c.y,
    z: c.z + Math.cos(dir) * dist - Math.sin(dir) * side,
  };
}

export interface Ground {
  /** Screen points relative to the element's top left, null where behind the camera. */
  local(p: Api.Vector3): Api.Vector2 | null;
  /** The element's size. */
  size: { w: number; h: number };
}

/**
 * Builds `build` in a world element centred on `center`'s projection and
 * large enough to hold `extent` (the shape's projected points), so the
 * shapes `build` draws in its pixels sit on the ground. Drawn under the
 * screen's elements, as world anchors are. Nothing when there is no
 * camera, or the centre is behind it.
 */
export function onGround(world: Api.World, id: string, center: Api.Vector3, extent: readonly Api.Vector3[], build: (g: Ground) => void, more: Partial<Api.WorldAnchorDesc> = {}): boolean {
  const project = projector(world);
  if (!project) return false;
  const c = project(center);
  if (!c) return false;
  let hx = 8;
  let hy = 8;
  for (const p of extent) {
    const s = project(p);
    if (!s) continue;
    hx = Math.max(hx, Math.abs(s.x - c.x) + 12);
    hy = Math.max(hy, Math.abs(s.y - c.y) + 12);
  }
  const w = Math.ceil(hx * 2);
  const h = Math.ceil(hy * 2);
  kit.openWorldAnchor(id, {
    target: { entity: null, point: center },
    pivot: "center",
    scale: "constant",
    declutter: "none",
    occlusion: "none",
    ...more,
  });
  try {
    // The element's box, so the shapes have their place even before any child sizes it.
    kit.open("Box", { width: { kind: "fixed", px: w }, height: { kind: "fixed", px: h } });
    try {
      build({
        local: (p) => {
          const s = project(p);
          return s ? { x: s.x - c.x + w / 2, y: s.y - c.y + h / 2 } : null;
        },
        size: { w, h },
      });
    } finally {
      kit.close();
    }
  } finally {
    kit.close();
  }
  return true;
}

/**
 * The points of a convex polygon's inside as one zig-zag line of `step`
 * rows: a path drawn `step` wide over them covers the polygon with no
 * seams (a path is one shape), and its outline hides the half-row it
 * reaches past the edges.
 */
export function convexFill(poly: Api.Vector2[], step: number): Api.Vector2[] {
  if (poly.length < 3) return [];
  let y0 = Infinity;
  let y1 = -Infinity;
  for (const p of poly) {
    y0 = Math.min(y0, p.y);
    y1 = Math.max(y1, p.y);
  }
  const rows = Math.max(1, Math.ceil((y1 - y0) / step));
  const s = (y1 - y0) / rows;
  const out: Api.Vector2[] = [];
  for (let i = 0; i <= rows; i++) {
    const y = Math.min(y1 - 0.01, Math.max(y0 + 0.01, y0 + i * s));
    let lo = Infinity;
    let hi = -Infinity;
    for (let k = 0; k < poly.length; k++) {
      const a = poly[k]!;
      const b = poly[(k + 1) % poly.length]!;
      if ((a.y <= y && b.y >= y) || (b.y <= y && a.y >= y)) {
        const t = b.y === a.y ? 0 : (y - a.y) / (b.y - a.y);
        const x = a.x + (b.x - a.x) * t;
        lo = Math.min(lo, x);
        hi = Math.max(hi, x);
      }
    }
    if (lo > hi) continue;
    if (i % 2 === 0) out.push({ x: lo, y }, { x: hi, y });
    else out.push({ x: hi, y }, { x: lo, y });
  }
  return out;
}
