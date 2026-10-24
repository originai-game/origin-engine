// Procedural modelling (IDL `shapes` and the og-geom members of `render`):
// og.Shape and og.Path2, three.js's Shape and Path with its methods and its
// curve formulas, and the render calls that take them. A shape never
// crosses the wire: it is flattened to points here (three.js's
// extractPoints(curveSegments)), so og-geom triangulates the same points
// three.js would.
//
// Determinism: the points may end up in world state through the geometry
// made from them, so the arcs read sincos (basic arithmetic, the camera
// systems' series), never Math.sin or Math.cos.

import type * as Api from "@origingame/engine";
import type { HostAbi } from "./generated/abi.ts";
import { boundHost } from "./runtime.ts";
import { sincos } from "./systems/cameras.ts";

type V2 = { x: number; y: number };
type W = { readonly id: number };

const TAU = 2 * 3.141592653589793;
const EPSILON = 2.220446049250313e-16;
const v2 = (x: number, y: number): V2 => ({ x, y });
const equals = (a: V2, b: V2) => a.x === b.x && a.y === b.y;

// three.js's Interpolations.
function catmullRom(t: number, p0: number, p1: number, p2: number, p3: number): number {
  const v0 = (p2 - p0) * 0.5;
  const v1 = (p3 - p1) * 0.5;
  const t2 = t * t;
  const t3 = t * t2;
  return (2 * p1 - 2 * p2 + v0 + v1) * t3 + (-3 * p1 + 3 * p2 - 2 * v0 - v1) * t2 + v0 * t + p1;
}
function quadratic(t: number, p0: number, p1: number, p2: number): number {
  const k = 1 - t;
  return k * k * p0 + 2 * (1 - t) * t * p1 + t * t * p2;
}
function cubic(t: number, p0: number, p1: number, p2: number, p3: number): number {
  const k = 1 - t;
  return k * k * k * p0 + 3 * k * k * t * p1 + 3 * (1 - t) * t * t * p2 + t * t * t * p3;
}

/** One of three.js's 2D curves, by its getPoint(t) and how finely CurvePath.getPoints divides it. */
interface Curve {
  point(t: number): V2;
  resolution(divisions: number): number;
}

const line = (a: V2, b: V2): Curve => ({
  // three.js LineCurve.getPoint: the end exactly at 1.
  point: (t) => (t === 1 ? v2(b.x, b.y) : v2((b.x - a.x) * t + a.x, (b.y - a.y) * t + a.y)),
  resolution: () => 1,
});

const quadraticCurve = (a: V2, c: V2, b: V2): Curve => ({
  point: (t) => v2(quadratic(t, a.x, c.x, b.x), quadratic(t, a.y, c.y, b.y)),
  resolution: (d) => d,
});

const cubicCurve = (a: V2, c1: V2, c2: V2, b: V2): Curve => ({
  point: (t) => v2(cubic(t, a.x, c1.x, c2.x, b.x), cubic(t, a.y, c1.y, c2.y, b.y)),
  resolution: (d) => d,
});

function splineCurve(points: readonly V2[]): Curve {
  return {
    point: (t) => {
      const p = (points.length - 1) * t;
      const i = Math.floor(p);
      const w = p - i;
      const p0 = points[i === 0 ? i : i - 1]!;
      const p1 = points[i]!;
      const p2 = points[i > points.length - 2 ? points.length - 1 : i + 1]!;
      const p3 = points[i > points.length - 3 ? points.length - 1 : i + 2]!;
      return v2(catmullRom(w, p0.x, p1.x, p2.x, p3.x), catmullRom(w, p0.y, p1.y, p2.y, p3.y));
    },
    resolution: (d) => d * points.length,
  };
}

function ellipseCurve(x: number, y: number, rx: number, ry: number, start: number, end: number, clockwise: boolean, rotation: number): Curve {
  return {
    point: (t) => {
      let delta = end - start;
      const same = Math.abs(delta) < EPSILON;
      while (delta < 0) delta += TAU;
      while (delta > TAU) delta -= TAU;
      if (delta < EPSILON) delta = same ? 0 : TAU;
      if (clockwise && !same) delta = delta === TAU ? -TAU : delta - TAU;
      const [s, c] = sincos(start + t * delta);
      let px = x + rx * c;
      let py = y + ry * s;
      if (rotation !== 0) {
        const [rs, rc] = sincos(rotation);
        const tx = px - x;
        const ty = py - y;
        px = tx * rc - ty * rs + x;
        py = tx * rs + ty * rc + y;
      }
      return v2(px, py);
    },
    resolution: (d) => d * 2,
  };
}

const point = (p: Api.Vector2): V2 => v2(p.x, p.y);

/** three.js's Path: curves from a current point. */
export class Path2 {
  #curves: Curve[] = [];
  #current: V2 = v2(0, 0);

  constructor(points?: readonly Api.Vector2[] | null) {
    if (points) this.setFromPoints(points);
  }

  setFromPoints(points: readonly Api.Vector2[]): this {
    if (points.length === 0) return this;
    this.moveTo(points[0]!.x, points[0]!.y);
    for (let i = 1; i < points.length; i++) this.lineTo(points[i]!.x, points[i]!.y);
    return this;
  }

  moveTo(x: number, y: number): this {
    this.#current = v2(x, y);
    return this;
  }

  lineTo(x: number, y: number): this {
    this.#curves.push(line(this.#current, v2(x, y)));
    this.#current = v2(x, y);
    return this;
  }

  quadraticCurveTo(cpX: number, cpY: number, x: number, y: number): this {
    this.#curves.push(quadraticCurve(this.#current, v2(cpX, cpY), v2(x, y)));
    this.#current = v2(x, y);
    return this;
  }

  bezierCurveTo(cp1X: number, cp1Y: number, cp2X: number, cp2Y: number, x: number, y: number): this {
    this.#curves.push(cubicCurve(this.#current, v2(cp1X, cp1Y), v2(cp2X, cp2Y), v2(x, y)));
    this.#current = v2(x, y);
    return this;
  }

  splineThru(points: readonly Api.Vector2[]): this {
    if (points.length === 0) return this;
    this.#curves.push(splineCurve([this.#current, ...points.map(point)]));
    this.#current = point(points[points.length - 1]!);
    return this;
  }

  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number, clockwise = false): this {
    return this.absarc(x + this.#current.x, y + this.#current.y, radius, startAngle, endAngle, clockwise);
  }

  absarc(x: number, y: number, radius: number, startAngle: number, endAngle: number, clockwise = false): this {
    return this.absellipse(x, y, radius, radius, startAngle, endAngle, clockwise);
  }

  ellipse(x: number, y: number, xRadius: number, yRadius: number, startAngle: number, endAngle: number, clockwise = false, rotation = 0): this {
    return this.absellipse(x + this.#current.x, y + this.#current.y, xRadius, yRadius, startAngle, endAngle, clockwise, rotation);
  }

  absellipse(x: number, y: number, xRadius: number, yRadius: number, startAngle: number, endAngle: number, clockwise = false, rotation = 0): this {
    const curve = ellipseCurve(x, y, xRadius, yRadius, startAngle, endAngle, clockwise, rotation);
    if (this.#curves.length > 0) {
      // A previous curve: join it with a line when the arc starts elsewhere.
      const first = curve.point(0);
      if (!equals(first, this.#current)) this.lineTo(first.x, first.y);
    }
    this.#curves.push(curve);
    this.#current = curve.point(1);
    return this;
  }

  closePath(): this {
    if (this.#curves.length === 0) return this;
    const start = this.#curves[0]!.point(0);
    const end = this.#curves[this.#curves.length - 1]!.point(1);
    if (!equals(start, end)) this.#curves.push(line(end, start));
    return this;
  }

  getPoints(divisions = 12): Api.Vector2[] {
    const out: V2[] = [];
    let last: V2 | undefined;
    for (const curve of this.#curves) {
      const resolution = curve.resolution(divisions);
      for (let d = 0; d <= resolution; d++) {
        const p = curve.point(d / resolution);
        if (last && equals(last, p)) continue;
        out.push(p);
        last = p;
      }
    }
    return out;
  }
}

/** three.js's Shape: a path with holes. */
export class Shape extends Path2 {
  readonly holes: Path2[] = [];

  extractPoints(divisions = 12): Api.ShapePoints {
    return { shape: this.getPoints(divisions), holes: this.holes.map((h) => h.getPoints(divisions)) };
  }
}

// ── the render calls ────────────────────────────────────────────────────────

function abi(): HostAbi<Uint32Array, unknown> {
  const h = boundHost();
  if (!h) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return h.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

type Desc = Record<string, unknown>;

/** A shape as it crosses: its points and its holes' points. */
function flat(s: Path2, divisions: number): Api.ShapePoints {
  return s instanceof Shape ? s.extractPoints(divisions) : { shape: s.getPoints(divisions), holes: [] };
}

/** `shapes` (a Shape, a list of them, or the outline's points) as points. */
function shapes(desc: unknown): string | undefined {
  if (desc === undefined || desc === null || typeof desc !== "object") return json(desc);
  const d: Desc = { ...(desc as Desc) };
  const divisions = typeof d.curveSegments === "number" ? d.curveSegments : 12;
  const v = d.shapes;
  if (v instanceof Path2) d.shapes = [flat(v, divisions)];
  else if (Array.isArray(v) && v.length > 0 && v[0] instanceof Path2) d.shapes = (v as Path2[]).map((s) => flat(s, divisions));
  return JSON.stringify(d);
}

/** A sweep's `section`: a Shape or Path2 flattened, its holes dropped. */
function section(desc: unknown): string | undefined {
  if (desc === undefined || desc === null || typeof desc !== "object") return json(desc);
  const d: Desc = { ...(desc as Desc) };
  const divisions = typeof d.curveSegments === "number" ? d.curveSegments : 12;
  if (d.section instanceof Path2) d.section = d.section.getPoints(divisions);
  return JSON.stringify(d);
}

export const geomRender = Object.freeze({
  shapeGeometry: (world: W, desc?: unknown) => parse(abi().og_render_shape_geometry(world.id, shapes(desc))),
  extrudeGeometry: (world: W, desc?: unknown) => parse(abi().og_render_extrude_geometry(world.id, shapes(desc))),
  latheGeometry: (world: W, desc?: unknown) => parse(abi().og_render_lathe_geometry(world.id, json(desc))),
  polyhedronGeometry: (world: W, desc?: unknown) => parse(abi().og_render_polyhedron_geometry(world.id, json(desc))),
  icosahedronGeometry: (world: W, desc?: unknown) => parse(abi().og_render_icosahedron_geometry(world.id, json(desc))),
  dodecahedronGeometry: (world: W, desc?: unknown) => parse(abi().og_render_dodecahedron_geometry(world.id, json(desc))),
  octahedronGeometry: (world: W, desc?: unknown) => parse(abi().og_render_octahedron_geometry(world.id, json(desc))),
  tetrahedronGeometry: (world: W, desc?: unknown) => parse(abi().og_render_tetrahedron_geometry(world.id, json(desc))),
  sweepGeometry: (world: W, desc?: unknown) => parse(abi().og_render_sweep_geometry(world.id, section(desc))),
  csgGeometry: (world: W, desc?: unknown) => parse(abi().og_render_csg_geometry(world.id, json(desc))),
  mergeGeometries: (world: W, desc?: unknown) => parse(abi().og_render_merge_geometries(world.id, json(desc))),
});
