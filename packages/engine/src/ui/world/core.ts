// What the world components (docs/ui.md §1, U6c) share beyond the HUD
// core: a reaction's colour, mass templates declared once per engine, the
// ground plane projected with og.Camera, and a world element that draws
// shapes in screen pixels round a ground point.

import type * as Api from "@origingame/engine";
import { BUILTINS, physics, type Token } from "../../runtime.ts";
import { terrain } from "../../p2.ts";
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

/** Lifts world points to the surface a ground shape lies on (GroundDesc). */
export interface Surface {
  /** The point on the surface under p (lifted by the bias); p itself when flat. */
  at(p: Api.Vector3): Api.Vector3;
  /** The longest edge between sampled vertices; Infinity when flat. */
  spacing: number;
}

const FLAT: Surface = { at: (p) => p, spacing: Infinity };

type Res<T> = { ok: true; value: T } | { ok: false; reason: string };

/**
 * The surface `ground` names: a terrain's height (og.terrain.sample) and
 * a downward cast against colliders (og.physics.castRay), the higher of
 * the two, each vertex sampled once a call.
 */
export function surface(world: Api.World, ground: Api.GroundDesc | null | undefined): Surface {
  if (!ground || (!ground.terrain && !ground.colliders)) return FLAT;
  const w = world as unknown as { readonly id: number };
  const reach = ground.reach || 50;
  const bias = ground.bias ?? 0.04;
  const seen = new Map<string, number>();
  const height = (p: Api.Vector3): number => {
    const key = `${p.x.toFixed(3)},${p.z.toFixed(3)},${p.y.toFixed(1)}`;
    const known = seen.get(key);
    if (known !== undefined) return known;
    let y = -Infinity;
    if (ground.terrain) {
      const r = terrain.sample(w, { data: ground.terrain, origin: ground.origin, quaternion: ground.quaternion, scale: ground.scale, x: p.x, z: p.z }) as Res<{ height: number; inside: boolean }>;
      if (r.ok && r.value.inside) y = r.value.height;
    }
    if (ground.colliders) {
      const r = physics.castRay(w as never, {
        origin: { x: p.x, y: p.y + reach, z: p.z },
        direction: { x: 0, y: -1, z: 0 },
        maxDistance: reach * 2,
        filter: { mask: ground.mask ?? 0, exclude: ground.exclude ?? null },
      }) as Res<{ point: Api.Vector3 } | null>;
      if (r.ok && r.value) y = Math.max(y, r.value.point.y);
    }
    const out = Number.isFinite(y) ? y + bias : p.y;
    seen.set(key, out);
    return out;
  };
  return { at: (p) => ({ x: p.x, y: height(p), z: p.z }), spacing: Math.max(0.05, ground.spacing || 0.5) };
}

/**
 * A ground outline lifted onto the surface: each edge cut into pieces no
 * longer than its spacing (wider when the whole would pass `cap`
 * vertices), each vertex on the surface under it.
 */
export function drape(s: Surface, pts: readonly Api.Vector3[], closed: boolean, cap = 512): Api.Vector3[] {
  if (s.spacing === Infinity || pts.length === 0) return pts.slice();
  const edges = closed ? pts.length : pts.length - 1;
  let total = 0;
  for (let i = 0; i < edges; i++) {
    const a = pts[i]!;
    const b = pts[(i + 1) % pts.length]!;
    total += Math.hypot(b.x - a.x, b.z - a.z);
  }
  const spacing = Math.max(s.spacing, total / Math.max(1, cap - pts.length));
  const out: Api.Vector3[] = [];
  for (let i = 0; i < edges; i++) {
    const a = pts[i]!;
    const b = pts[(i + 1) % pts.length]!;
    const n = Math.max(1, Math.ceil(Math.hypot(b.x - a.x, b.z - a.z) / spacing));
    for (let k = 0; k < n; k++) {
      const t = k / n;
      out.push(s.at({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t, z: a.z + (b.z - a.z) * t }));
    }
  }
  if (!closed) out.push(s.at(pts[pts.length - 1]!));
  return out;
}
