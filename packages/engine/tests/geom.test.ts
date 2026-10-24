// Procedural modelling (round 3, geom-1) through the TypeScript API on the
// native host: og.Shape and og.Path2 flatten three.js's curves to the
// points three.js's formulas give (worked out by hand below, not read from
// the code), and one geometry of each new kind is made through og.render,
// its counts the three.js formula's and its bounds where the inputs put
// them. Inputs are asymmetric, so a swapped axis or a wrong operation shows.
import { beforeAll, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { og } from "@origingame/engine";
import type { Geometry, GeometryInfo, Vector2, World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

let engine: Engine;
let world: World;

beforeAll(() => {
  loadNativeHost();
  const r = createEngine(og.game({}) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  engine = r.value;
  world = engine.world as unknown as World;
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const info = (g: Geometry): GeometryInfo => must(og.render.geometryInfo(world, g));
const near = (got: Vector2, x: number, y: number) => {
  expect(got.x).toBeCloseTo(x, 12);
  expect(got.y).toBeCloseTo(y, 12);
};
const v3near = (got: { x: number; y: number; z: number }, x: number, y: number, z: number, digits = 5) => {
  expect(got.x).toBeCloseTo(x, digits);
  expect(got.y).toBeCloseTo(y, digits);
  expect(got.z).toBeCloseTo(z, digits);
};

// ── og.Path2 and og.Shape ───────────────────────────────────────────────────

test("lines and closePath flatten to their corners, the first repeated", () => {
  const p = new og.Path2().moveTo(0, 0).lineTo(3, 0).lineTo(3, 2).lineTo(0, 2).closePath();
  expect(p.getPoints()).toEqual([
    { x: 0, y: 0 },
    { x: 3, y: 0 },
    { x: 3, y: 2 },
    { x: 0, y: 2 },
    { x: 0, y: 0 },
  ]);
  // A path built from points is the same path.
  expect(new og.Path2([{ x: 0, y: 0 }, { x: 3, y: 0 }]).getPoints()).toEqual([{ x: 0, y: 0 }, { x: 3, y: 0 }]);
});

test("Bézier curves are divided by `divisions`, ellipses twice as finely", () => {
  // Quadratic (0,0) (1,2) (2,0) at t = 1/2: 0.25·0 + 0.5·1 + 0.25·2, 0.5·2.
  const q = new og.Path2().moveTo(0, 0).quadraticCurveTo(1, 2, 2, 0).getPoints(2);
  expect(q.length).toBe(3);
  near(q[1]!, 1, 1);
  // Cubic (0,0) (0,1) (1,1) (1,0) at t = 1/2: x 3/8 + 1/8, y 3/8 + 3/8.
  const c = new og.Path2().moveTo(0, 0).bezierCurveTo(0, 1, 1, 1, 1, 0).getPoints(2);
  near(c[1]!, 0.5, 0.75);
  // A whole circle of radius 2 round (1, -1), 12 divisions: 24 segments, 25 points.
  const circle = new og.Path2().absarc(1, -1, 2, 0, Math.PI * 2, false).getPoints(12);
  expect(circle.length).toBe(25);
  near(circle[0]!, 3, -1);
  near(circle[6]!, 1, 1); // a quarter turn, counter-clockwise
  near(circle[12]!, -1, -1);
  // Clockwise, the quarter is below the centre.
  near(new og.Path2().absarc(1, -1, 2, 0, Math.PI * 2, true).getPoints(12)[6]!, 1, -3);
  // An ellipse turned a quarter: its x radius lies along y.
  near(new og.Path2().absellipse(0, 0, 3, 1, 0, Math.PI, false, Math.PI / 2).getPoints(4)[0]!, 0, 3);
});

test("an arc after a curve is joined to it by a line; arc and ellipse are relative", () => {
  const p = new og.Path2().moveTo(5, 0).lineTo(4, 0).absarc(0, 0, 1, 0, Math.PI, false);
  const pts = p.getPoints(1);
  // (5,0) → (4,0), the joining line to (1,0), then the arc's three points.
  expect(pts[0]).toEqual({ x: 5, y: 0 });
  expect(pts[1]).toEqual({ x: 4, y: 0 });
  near(pts[2]!, 1, 0);
  near(pts.at(-1)!, -1, 0);
  // arc's centre is from the current point: (2, 3) + (1, 0).
  near(new og.Path2().moveTo(2, 3).arc(1, 0, 1, 0, Math.PI, false).getPoints(1)[0]!, 4, 3);
});

test("splineThru passes through its points", () => {
  const pts = new og.Path2().moveTo(0, 0).splineThru([{ x: 1, y: 2 }, { x: 3, y: -1 }]).getPoints(4);
  // A SplineCurve of three points divided 4 × 3 = 12 times: knots at 0, 6, 12.
  expect(pts.length).toBe(13);
  near(pts[6]!, 1, 2);
  near(pts[12]!, 3, -1);
});

test("a shape's holes come out with its points", () => {
  const s = new og.Shape([{ x: 0, y: 0 }, { x: 4, y: 0 }, { x: 4, y: 4 }]);
  s.holes.push(new og.Path2([{ x: 1, y: 1 }, { x: 2, y: 1 }, { x: 2, y: 2 }]));
  expect(s.holes.length).toBe(1);
  const e = s.extractPoints();
  expect(e.shape.length).toBe(3);
  expect(e.holes).toEqual([[{ x: 1, y: 1 }, { x: 2, y: 1 }, { x: 2, y: 2 }]]);
});

// ── one geometry of each kind ───────────────────────────────────────────────

const square = (r: number) => new og.Path2([{ x: -r, y: -r }, { x: r, y: -r }, { x: r, y: r }, { x: -r, y: r }]).closePath();

test("shapeGeometry triangulates a shape with a hole as three.js does", () => {
  const s = new og.Shape().setFromPoints(square(2).getPoints());
  s.holes.push(square(1));
  const g = info(must(og.render.shapeGeometry(world, { shapes: s })));
  // Each ring loses its repeated end: 4 + 4 vertices; n points and h holes
  // take n + 2h - 2 triangles.
  expect([g.vertices, g.triangles]).toEqual([8, 8]);
  v3near(g.min, -2, -2, 0);
  v3near(g.max, 2, 2, 0);
  // A plain points array is a shape too.
  expect(info(must(og.render.shapeGeometry(world, { shapes: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 3 }] }))).max.y).toBeCloseTo(3);
});

test("extrudeGeometry has three.js's counts with and without a bevel", () => {
  const flat = info(must(og.render.extrudeGeometry(world, { shapes: square(0.5), depth: 2, bevelEnabled: false })));
  // Lids 2 × 2 triangles, walls 4 edges × 2 × 1 step; unindexed.
  expect([flat.triangles, flat.vertices]).toEqual([12, 36]);
  v3near(flat.min, -0.5, -0.5, 0);
  v3near(flat.max, 0.5, 0.5, 2);
  // Three.js's defaults: bevel thickness 0.2, size 0.1, three segments a side.
  const bevelled = info(must(og.render.extrudeGeometry(world, { shapes: [square(0.5)], depth: 1 })));
  expect(bevelled.triangles).toBe(2 * 2 + 2 * 4 * (1 + 2 * 3));
  v3near(bevelled.min, -0.6, -0.6, -0.2);
  v3near(bevelled.max, 0.6, 0.6, 1.2);
});

test("extrudeGeometry follows an extrudePath", () => {
  const path = must(og.path.create(world, { points: [{ x: 0, y: 0, z: 0 }, { x: 0, y: 0, z: 5 }], kind: "linear" }));
  const g = info(must(og.render.extrudeGeometry(world, { shapes: square(0.5), steps: 4, extrudePath: path })));
  expect(g.triangles).toBe(2 * 2 + 2 * 4 * 4);
  expect(g.min.z).toBeCloseTo(0, 5);
  expect(g.max.z).toBeCloseTo(5, 5);
  const dead = og.render.extrudeGeometry(world, { shapes: square(0.5), extrudePath: 999999 as never });
  expect(dead.ok).toBe(false);
});

test("latheGeometry and the polyhedra have three.js's counts", () => {
  // Three.js's default profile: 3 points × 13 columns, 12 × 2 × 2 triangles.
  const lathe = info(must(og.render.latheGeometry(world, {})));
  expect([lathe.vertices, lathe.triangles]).toEqual([39, 48]);
  const vase = info(must(og.render.latheGeometry(world, { points: [{ x: 0.2, y: 0 }, { x: 0.5, y: 1 }, { x: 0.3, y: 2 }], segments: 7 })));
  expect([vase.vertices, vase.triangles]).toEqual([3 * 8, 7 * 2 * 2]);
  expect(vase.max.y).toBeCloseTo(2, 5);
  // A tetrahedron's table at detail 1: 4 faces × 4.
  const poly = info(
    must(og.render.polyhedronGeometry(world, { vertices: [1, 1, 1, -1, -1, 1, -1, 1, -1, 1, -1, -1], indices: [2, 1, 0, 0, 3, 2, 1, 3, 0, 2, 3, 1], radius: 2, detail: 1 })),
  );
  expect([poly.triangles, poly.vertices]).toEqual([16, 48]);
  for (const [name, make, faces] of [
    ["icosahedron", og.render.icosahedronGeometry, 20],
    ["dodecahedron", og.render.dodecahedronGeometry, 36],
    ["octahedron", og.render.octahedronGeometry, 8],
    ["tetrahedron", og.render.tetrahedronGeometry, 4],
  ] as const) {
    const g = info(must(make(world, { radius: 3, detail: 2 })));
    expect([name, g.triangles, g.vertices]).toEqual([name, faces * 9, faces * 27]);
  }
  // The octahedron's corners are on the axes at the radius.
  v3near(info(must(og.render.octahedronGeometry(world, { radius: 3 }))).max, 3, 3, 3);
  expect(og.render.polyhedronGeometry(world, { vertices: [0, 0, 0], indices: [0, 1, 2] }).ok).toBe(false);
});

test("sweepGeometry tapers a section along a path and caps it", () => {
  const path = must(og.path.create(world, { points: [{ x: 0, y: 0, z: 0 }, { x: 0, y: 2, z: 0 }], kind: "linear" }));
  const g = info(
    must(og.render.sweepGeometry(world, { section: square(0.5), path, steps: 4, scale: [{ t: 0, value: 1 }, { t: 1, value: 0.2 }] })),
  );
  // 5 rings of 4 + 1 (the seam) and two caps of 4; 4 × 4 quads and 2 × 2 cap triangles.
  expect([g.vertices, g.triangles]).toEqual([5 * 5 + 2 * 4, 32 + 4]);
  expect(g.min.y).toBeCloseTo(0, 5);
  expect(g.max.y).toBeCloseTo(2, 5);
  // The base is the section's square (half-diagonal 0.5·√2 at most).
  expect(Math.max(-g.min.x, g.max.x, -g.min.z, g.max.z)).toBeCloseTo(0.5, 5);
  expect(og.render.sweepGeometry(world, { section: square(0.5) }).ok).toBe(false);
});

test("csgGeometry cuts, joins and intersects placed boxes", () => {
  const big = must(og.render.boxGeometry(world, { width: 2, height: 2, depth: 2 }));
  const small = must(og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 }));
  const b = { geometry: small, position: { x: 0.9, y: 0.8, z: 0.7 } };
  const cut = info(must(og.render.csgGeometry(world, { op: "subtract", a: { geometry: big }, b })));
  v3near(cut.min, -1, -1, -1);
  v3near(cut.max, 1, 1, 1);
  const joined = info(must(og.render.csgGeometry(world, { op: "union", a: { geometry: big }, b })));
  v3near(joined.max, 1.4, 1.3, 1.2);
  const both = info(must(og.render.csgGeometry(world, { op: "intersect", a: { geometry: big }, b })));
  v3near(both.min, 0.4, 0.3, 0.2);
  v3near(both.max, 1, 1, 1);
  expect(og.render.csgGeometry(world, { a: { geometry: big }, b: { geometry: 424242 as Geometry } }).ok).toBe(false);
});

test("mergeGeometries places, mirrors and repeats parts", () => {
  const box = must(og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 }));
  const at = { x: 2, y: 0.5, z: 0 };
  const g = info(
    must(og.render.mergeGeometries(world, { parts: [{ geometry: box, position: at }, { geometry: box, position: at, mirror: "x" }, { geometry: box, position: { x: 0, y: 0, z: -3 } }] })),
  );
  expect(g.triangles).toBe(36);
  v3near(g.min, -2.5, -0.5, -3.5);
  v3near(g.max, 2.5, 1, 0.5);
  expect(og.render.mergeGeometries(world, { parts: [] }).ok).toBe(false);
});

test("a made geometry is dynamic only when asked", () => {
  const g = must(og.render.icosahedronGeometry(world, { radius: 1, detail: 1, dynamic: true }));
  expect(info(g).dynamic).toBe(true);
  expect(info(must(og.render.icosahedronGeometry(world, {}))).dynamic).toBe(false);
});

// bevelSize, as three.js derives it when not given: bevelThickness - 0.1.
// The IDL declares no fixed default for it (it said 0.1, true only at the
// default thickness); unset and null both take the derived value.
test("extrude's bevelSize is bevelThickness - 0.1 when not given", () => {
  const schema = JSON.parse(readFileSync(new URL("../schema/api.schema.json", import.meta.url), "utf8"));
  const decl = schema.$defs.ExtrudeGeometryDesc.properties.bevelSize;
  expect(decl.default).toBeNull();
  // The default square (half-width 0.5) grows by the bevel's size.
  const width = (desc: Record<string, unknown>) => info(must(og.render.extrudeGeometry(world, desc as never))).max.x;
  expect(width({ bevelThickness: 0.3 })).toBeCloseTo(0.7, 5);
  expect(width({ bevelThickness: 0.3, bevelSize: null })).toBeCloseTo(0.7, 5);
  expect(width({ bevelThickness: 0.3, bevelSize: 0.05 })).toBeCloseTo(0.55, 5);
  expect(width({})).toBeCloseTo(0.6, 5);
});
