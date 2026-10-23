// The depth wave's geometry station (S1), far from the others along x,
// built when first shown:
//
//   geometry  runtime geometry and debug draw. A procedural terrain made
//             with og.render.bufferGeometry (vertex-coloured by height)
//             and rewritten every tick by og.render.updateGeometry, its
//             collider the same geometry; a tube along a Catmull-Rom
//             spline with its control polygon as a dashed og.Lines; the
//             new primitives in a row (cylinder, cone, capsule, a
//             wireframe torus, ring, circle); a point cloud kept as
//             og.Points, circled by a tube along a cooked path (the scene
//             scenes/geo-rail.ogt's closed Bezier `path "geo-rail"`, found
//             by og.path.find and made a tube by og.render.tubeGeometry's
//             `path`); bodies falling onto the terrain; dashed lines in
//             screen space framing the view; and a probe sweeping over the
//             terrain whose og.render.raycast hit is marked with og.debug
//             (arrow, sphere, label). The physics debug view is on.

import { og, defineComponent, defineSystem, f64, u32, BODY_STATIC, SHAPE_BOX, SHAPE_CAPSULE, SHAPE_SPHERE } from "@origingame/engine";
import type { Geometry, Material, Vector3 as Vec3, World } from "@origingame/engine";

export const GEO_STATIONS = ["geometry"] as const;
export type GEOStation = (typeof GEO_STATIONS)[number];

/** Where the station lives along x. */
export const GEO_ORIGIN: Record<GEOStation, number> = { geometry: 3200 };

/** The terrain's cells per side and its side in metres. */
const GRID = { n: 28, size: 7 };
/** SHAPE_MESH: a triangle mesh collider of `mesh`. */
const SHAPE_MESH = 5;

/** The station's state, on the entity "geo". */
export const GEO = defineComponent("GEO", {
  /** Whether the station is shown (its per-tick work runs only then). */
  active: u32(0),
  /** Ticks since it was shown. */
  shown: u32(0),
  /** The dynamic terrain geometry. */
  terrain: f64(0),
  /** Written per tick: the probe's raycast hits on the terrain (count) and the distance of the nearest. */
  hits: u32(0),
  distance: f64(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The terrain's height at (x, z) in its own space, at time t seconds. */
function height(x: number, z: number, t: number): number {
  const ridge = 0.9 * Math.exp(-((x + 0.8) ** 2) / 2.5 - ((z - 0.4) ** 2) / 1.5);
  const swell = 0.18 * Math.sin(x * 1.4 + t * 1.6) * Math.cos(z * 1.1 - t * 0.9);
  return ridge + swell + 0.25;
}

/** The terrain's attributes at time t: positions, colours by height, indices. */
function terrainAttributes(t: number, withIndices: boolean) {
  const { n, size } = GRID;
  const count = (n + 1) * (n + 1);
  const positions = new Float32Array(count * 3);
  const colors = new Float32Array(count * 3);
  for (let j = 0; j <= n; j++) {
    for (let i = 0; i <= n; i++) {
      const k = j * (n + 1) + i;
      const x = (i / n - 0.5) * size;
      const z = (j / n - 0.5) * size;
      const y = height(x, z, t);
      positions.set([x, y, z], k * 3);
      // Low: teal; high: warm sand.
      const s = Math.min(Math.max((y - 0.1) / 1.1, 0), 1);
      colors.set([0.08 + 0.75 * s, 0.35 + 0.4 * s, 0.4 - 0.15 * s], k * 3);
    }
  }
  if (!withIndices) return { positions, colors };
  const indices = new Uint32Array(n * n * 6);
  let o = 0;
  for (let j = 0; j < n; j++) {
    for (let i = 0; i < n; i++) {
      const a = j * (n + 1) + i;
      const b = a + 1;
      const c = a + n + 2;
      const d = a + n + 1;
      indices.set([a, d, c, a, c, b], o);
      o += 6;
    }
  }
  return { positions, colors, indices };
}

let indices: Uint32Array | null = null;

/** Per tick while shown: the terrain rewritten, the screen lines and the probe's raycast. */
export const GEOShow = defineSystem("GEOShow", { writes: [GEO] }, (_q, time, world) => {
  const e = world.find("geo");
  if (e === null) return;
  const s = must(world.get(e, GEO));
  if (s.active === 0 || s.terrain === 0) return;
  const t = time.tick / 60;
  const terrain = s.terrain as unknown as Geometry;
  const a = terrainAttributes(t, false);
  must(og.render.updateGeometry(world, terrain, { positions: a.positions, colors: a.colors, indices: indices! }));

  // Dashed lines in screen space: a frame inset from the view, and a
  // zigzag under the title (CSS pixels, y down).
  const vw = 1280;
  const vh = 720;
  const m = 24;
  must(og.render.lines(world, {
    screen: true, mode: "loop", dashed: true, dashSize: 14, gapSize: 8, width: 2,
    color: { r: 1, g: 0.85, b: 0.3 },
    points: [{ x: m, y: m, z: 0 }, { x: vw - m, y: m, z: 0 }, { x: vw - m, y: vh - m, z: 0 }, { x: m, y: vh - m, z: 0 }],
  }));
  const zig = Array.from({ length: 13 }, (_, i) => ({ x: vw - 330 + i * 24, y: 60 + (i % 2) * 18, z: 0 }));
  must(og.render.lines(world, {
    screen: true, width: 3, dashed: true, dashSize: 10, gapSize: 5,
    points: zig,
    colors: zig.map((_, i) => ({ r: 0.3 + 0.7 * (i / 12), g: 0.9 - 0.6 * (i / 12), b: 1 })),
  }));

  // A probe sweeping over the terrain: its ray straight down, the hit
  // marked (the render raycast, against what is drawn).
  const x0 = GEO_ORIGIN.geometry;
  const p = { x: x0 + 2.2 * Math.cos(t * 0.8), y: 4, z: 0.6 + 1.6 * Math.sin(t * 0.8) };
  const hits = must(og.render.raycast(world, { origin: p, direction: { x: 0, y: -1, z: 0 } }, { first: true }));
  const hit = hits[0];
  if (hit !== undefined) {
    og.debug.arrow(world, p, hit.point, { color: { r: 1, g: 0.3, b: 0.9 } });
    og.debug.sphere(world, hit.point, 0.18, { color: { r: 1, g: 0.3, b: 0.9 } });
    og.debug.axes(world, hit.point, 0.5, { width: 3 });
    og.debug.text(world, p, `${hit.distance.toFixed(2)} m`, { color: { r: 1, g: 0.75, b: 1 } });
  }
  world.set(e, GEO, { shown: s.shown + 1, hits: hits.length, distance: hit?.distance ?? 0 });
});

export interface GEOKit {
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
}

let kit: GEOKit;

/** Prepares the station; it is built when first shown (showGEO). */
export function buildGEO(world: World, k: GEOKit): void {
  kit = k;
  world.insert(must(world.spawn({ name: "geo" })), GEO);
}

function build(world: World): void {
  const k = kit;
  const x0 = GEO_ORIGIN.geometry;
  k.ground(x0, { x: 0.3, y: 0.31, z: 0.34 });
  const mesh = (name: string, geometry: Geometry, material: Material, at: Vec3, turn?: { axis: Vec3; angle: number }) => {
    const e = must(world.spawn({ name, position: at, ...(turn ? { quaternion: og.transform.fromAxisAngle(turn.axis, turn.angle) } : {}) }));
    world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
    return e;
  };

  // The procedural terrain: dynamic, its own collider.
  const a = terrainAttributes(0, true);
  indices = a.indices!;
  const terrain = must(og.render.bufferGeometry(world, { ...a, dynamic: true }));
  const ground = mesh("geo-terrain", terrain, k.mat({ color: { r: 1, g: 1, b: 1 }, roughness: 0.8 }), { x: x0 + 0.5, y: 0, z: 0.4 });
  world.insert(ground, og.RigidBody, { kind: BODY_STATIC });
  world.insert(ground, og.Collider, { shape: SHAPE_MESH, mesh: terrain });

  // A tube along a spline behind it, and the spline's control polygon.
  const spline: Vec3[] = [
    { x: -3.4, y: 0.4, z: -2.6 }, { x: -2, y: 1.8, z: -3.2 }, { x: -0.4, y: 0.9, z: -2.4 },
    { x: 1, y: 2.4, z: -3 }, { x: 2.4, y: 1.2, z: -2.4 }, { x: 3.6, y: 2.2, z: -3.1 },
  ];
  const tube = must(og.render.tubeGeometry(world, { points: spline, radius: 0.16, tubularSegments: 160, radialSegments: 14 }));
  mesh("geo-tube", tube, k.mat({ color: { r: 0.95, g: 0.62, b: 0.12 }, metalness: 1, roughness: 0.3 }), { x: x0, y: 0, z: 0 });
  const polygon = must(og.render.bufferGeometry(world, { positions: new Float32Array(spline.flatMap((p) => [p.x, p.y + 0.35, p.z])), physics: false }));
  const control = must(world.spawn({ name: "geo-control", position: { x: x0, y: 0, z: 0 } }));
  world.insert(control, og.Lines, { geometry: polygon, color: { r: 0.4, g: 0.85, b: 1 }, width: 2, dashSize: 0.18, gapSize: 0.1 });

  // The new primitives in a row on the left.
  const row = (i: number): Vec3 => ({ x: x0 - 5.2, y: 0, z: -2.2 + i * 1.05 });
  const lift = (p: Vec3, y: number): Vec3 => ({ ...p, y });
  const coral = k.mat({ color: { r: 0.9, g: 0.35, b: 0.3 }, roughness: 0.5 });
  const sky = k.mat({ color: { r: 0.3, g: 0.55, b: 0.95 }, roughness: 0.4 });
  const both = k.mat({ color: { r: 0.85, g: 0.85, b: 0.3 }, roughness: 0.5, side: "double" });
  mesh("geo-cylinder", must(og.render.cylinderGeometry(world, { radiusTop: 0.3, radiusBottom: 0.4, height: 0.8 })), coral, lift(row(0), 0.4));
  mesh("geo-cone", must(og.render.coneGeometry(world, { radius: 0.4, height: 0.9 })), sky, lift(row(1), 0.45));
  const capsule = mesh("geo-capsule", must(og.render.capsuleGeometry(world, { radius: 0.25, height: 0.5, capSegments: 6, radialSegments: 16 })), coral, lift(row(2), 0.5));
  world.insert(capsule, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.25, height: 0.5 });
  mesh("geo-torus", must(og.render.torusGeometry(world, { radius: 0.32, tube: 0.12, radialSegments: 12, tubularSegments: 36 })),
    k.mat({ color: { r: 0.2, g: 1, b: 0.6 }, emissive: { r: 0.1, g: 0.6, b: 0.3 }, wireframe: true }), lift(row(3), 0.5));
  mesh("geo-ring", must(og.render.ringGeometry(world, { innerRadius: 0.18, outerRadius: 0.4 })), both, lift(row(4), 0.5), { axis: { x: 0, y: 1, z: 0 }, angle: 0.6 });
  mesh("geo-circle", must(og.render.circleGeometry(world, { radius: 0.4, segments: 6 })), both, lift(row(5), 0.02), { axis: { x: 1, y: 0, z: 0 }, angle: -Math.PI / 2 });

  // A point cloud kept as data: a shell coloured by height, on the right.
  const cloud: number[] = [];
  const colors: number[] = [];
  for (let i = 0; i < 600; i++) {
    const y = 1 - (2 * (i + 0.5)) / 600;
    const r = Math.sqrt(1 - y * y);
    const phi = i * Math.PI * (3 - Math.sqrt(5));
    cloud.push(Math.cos(phi) * r * 0.8, y * 0.8, Math.sin(phi) * r * 0.8);
    colors.push(0.5 + 0.5 * y, 0.3, 1 - 0.5 * (y + 1) * 0.5);
  }
  const points = must(og.render.bufferGeometry(world, { positions: new Float32Array(cloud), colors: new Float32Array(colors), physics: false }));
  const shell = must(world.spawn({ name: "geo-cloud", position: { x: x0 + 5, y: 1.3, z: -1 } }));
  world.insert(shell, og.Points, { geometry: points, size: 0.05 });

  // A tube along a cooked path: the scene's closed Bezier loop around the
  // cloud, in world coordinates, followed exactly (tubeGeometry's `path`).
  must(world.instantiate(must(og.load.scene(world, "scenes/geo-rail.ogt"))));
  const rail = must(og.path.find(world, "geo-rail"));
  const loop = must(og.render.tubeGeometry(world, { path: rail, radius: 0.07, tubularSegments: 200, radialSegments: 10 }));
  mesh("geo-rail", loop, k.mat({ color: { r: 0.2, g: 0.95, b: 0.75 }, emissive: { r: 0.05, g: 0.4, b: 0.3 }, roughness: 0.35 }), { x: 0, y: 0, z: 0 });

  // Bodies dropped onto the terrain (the physics view shows their shapes).
  const box = must(og.render.boxGeometry(world, { width: 0.4, height: 0.4, depth: 0.4 }));
  const ball = must(og.render.sphereGeometry(world, { radius: 0.22 }));
  const crate = k.mat({ color: { r: 0.75, g: 0.55, b: 0.35 }, roughness: 0.7 });
  for (let i = 0; i < 5; i++) {
    const at = { x: x0 - 1.4 + i * 0.85, y: 2.4 + 0.5 * i, z: 0.2 + 0.4 * Math.sin(i * 1.7) };
    const round = i % 2 === 1;
    const b = mesh(`geo-body-${i}`, round ? ball : box, crate, at);
    world.insert(b, og.RigidBody, { mass: 1 });
    world.insert(b, og.Collider, round ? { shape: SHAPE_SPHERE, radius: 0.22 } : { shape: SHAPE_BOX, size: { x: 0.4, y: 0.4, z: 0.4 } });
  }
  world.set(world.find("geo")!, GEO, { terrain });
}

/** Leaves the station: its per-tick work stops and the debug view goes. */
export function resetGEO(world: World): void {
  const e = world.find("geo");
  if (e !== null) world.set(e, GEO, { active: 0 });
  og.debug.setView(world, "none");
}

/** Shows the station: builds it the first time, then its camera, light and the physics view. */
export function showGEO(world: World, station: GEOStation): void {
  if (world.find(`geo-${station}`) === null) {
    build(world);
    must(world.spawn({ name: `geo-${station}` }));
  }
  world.set(world.find("geo")!, GEO, { active: 1, shown: 0 });
  og.debug.setView(world, "physics");
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = GEO_ORIGIN[station];
  const eye = { x: x - 0.3, y: 5.2, z: 8.2 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: x - 0.1, y: 0.4, z: -0.6 });
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.8, z: -0.4 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
  settings.sky = true;
  settings.environmentIntensity = 0.5;
  settings.bloom = false;
  cam.exposure = 0.75;
}
