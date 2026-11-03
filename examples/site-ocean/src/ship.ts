// The schooner: a two-masted gaff schooner (24 m on deck) built from code
// as four meshes (hull, spars and rigging, sails, lamps) and floated on the
// sea by buoyancy: a dynamic og.RigidBody whose hull is sampled at ten
// points against og.physics.waterHeight every tick (the same Gerstner sea
// the renderer draws), each point pushed up by how deep it lies and damped
// by how fast it moves, so it heaves, pitches and rolls on the swell. Its
// course is held by a steering force towards where the loop wants it.
//
// Ship space: +x the bow, +y up, +z the side the camera sees (leeward: the
// wind blows from -z, so the sails belly and the booms swing towards +z).

import { og, defineComponent, f32, SHAPE_BOX } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { add, len, lerp3, MeshBuilder, must, norm, scale, sub, v3 } from "./mesh.ts";
import type { Rgb, V3 } from "./mesh.ts";

const L = 24; // hull length, stern to stem
const BEAM = 2.9; // half beam
const MASS = 48000; // kg

/** The ship's steering and buoyancy state: what the probe and the playtest read. */
export const Ship = defineComponent("Ship", {
  targetX: f32(0, { unit: "m", doc: "Where the loop wants the ship along x this tick." }),
  submerged: f32(0, { doc: "How many of the ten hull points are under the sea." }),
  heave: f32(0, { unit: "m", doc: "The hull's height above the sea's rest level." }),
  roll: f32(0, { unit: "deg", doc: "Its heel, leeward positive." }),
  pitch: f32(0, { unit: "deg", doc: "Its pitch, bow up positive." }),
});

// ── the hull's lines ──────────────────────────────────────────────────────

const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

/** Half beam at u (0 the transom, 1 the stem). */
function halfBeam(u: number): number {
  if (u >= 0.45) {
    const k = (u - 0.45) / 0.55;
    return BEAM * Math.pow(Math.max(0, 1 - k * k), 0.6);
  }
  const k = (0.45 - u) / 0.45;
  return BEAM * (1 - 0.42 * k * k);
}

/** The sheer (the deck edge's height) at u: rising to the bow and a little to the stern. */
function sheer(u: number): number {
  if (u >= 0.45) {
    const k = (u - 0.45) / 0.55;
    return 1.35 + 1.0 * k * k;
  }
  const k = (0.45 - u) / 0.45;
  return 1.35 + 0.45 * k * k;
}

/** The keel's height at u: a long keel, the forefoot cut away, the counter stern lifted. */
function keel(u: number): number {
  return -0.25 + (-2.0 + 0.25) * smooth(0, 0.16, u) + (2.0 + 0.7) * smooth(0.7, 1, u);
}

/** How full the section is (lower: boxier bilges; higher: a V). */
function fullness(u: number): number {
  return 0.38 + 0.55 * smooth(0.55, 1, u) + 0.25 * (1 - smooth(0, 0.3, u));
}

const ANTIFOUL: Rgb = [0.2, 0.045, 0.03];
const BOOTTOP: Rgb = [0.72, 0.68, 0.58];
const TOPSIDE: Rgb = [0.02, 0.032, 0.035];
const STRIPE: Rgb = [0.52, 0.36, 0.1];
const DECK: Rgb = [0.33, 0.24, 0.15];
const CABIN: Rgb = [0.42, 0.29, 0.17];
const WOOD: Rgb = [0.4, 0.27, 0.15];
const IRON: Rgb = [0.03, 0.028, 0.026];
const CANVAS: Rgb = [0.83, 0.79, 0.69];

function hullColor(y: number, top: number): Rgb {
  if (y < -0.04) return ANTIFOUL;
  if (y < 0.16) return BOOTTOP;
  if (y > top - 0.34 && y < top - 0.22) return STRIPE;
  return TOPSIDE;
}

const STATIONS = 48;
const SECTION = 32;

function hull(m: MeshBuilder): void {
  // The planking: a grid of stations (stern to stem) by section points
  // (port sheer, down round the keel, to starboard sheer).
  const first = m.positions.length / 3;
  for (let i = 0; i < STATIONS; i++) {
    const u = i / (STATIONS - 1);
    const x = -L / 2 + u * L;
    const b = halfBeam(u);
    const s = sheer(u);
    const k = keel(u);
    const e = fullness(u);
    for (let j = 0; j < SECTION; j++) {
      const phi = (j / (SECTION - 1)) * Math.PI;
      const f = Math.pow(Math.sin(phi), e);
      const y = s - (s - k) * f;
      m.vertex(v3(x, y, b * Math.cos(phi)), hullColor(y, s));
    }
  }
  m.grid(first, STATIONS, SECTION);
  // The transom: the stern's section closed by a fan.
  const c = m.vertex(v3(-L / 2, (sheer(0) + keel(0)) * 0.5, 0), TOPSIDE);
  for (let j = 0; j + 1 < SECTION; j++) m.tri(c, first + j + 1, first + j);
  // The deck, 0.65 m under the sheer (the bulwarks above it are the
  // planking's own top, drawn from both sides).
  const deck = m.positions.length / 3;
  for (let i = 0; i < STATIONS; i++) {
    const u = i / (STATIONS - 1);
    const x = -L / 2 + u * L;
    const b = halfBeam(u) * 0.96;
    const y = sheer(u) - 0.65 + 0.06;
    m.vertex(v3(x, y, b), DECK);
    m.vertex(v3(x, y + 0.06, 0), DECK);
    m.vertex(v3(x, y, -b), DECK);
  }
  m.grid(deck, STATIONS, 3, true);
  // Rails, a cabin trunk, a deckhouse and hatches.
  const deckY = (u: number) => sheer(u) - 0.59;
  const at = (x: number) => (x + L / 2) / L;
  m.box(v3(-4.2, deckY(at(-4.2)) + 0.45, 0), v3(5.2, 0.9, 2.6), CABIN);
  m.box(v3(-4.2, deckY(at(-4.2)) + 0.94, 0), v3(5.4, 0.08, 2.8), WOOD);
  m.box(v3(1.2, deckY(at(1.2)) + 0.3, 0), v3(2.2, 0.6, 1.8), CABIN);
  m.box(v3(6.4, deckY(at(6.4)) + 0.25, 0), v3(1.4, 0.5, 1.4), CABIN);
  m.box(v3(-9.6, deckY(at(-9.6)) + 0.55, 0), v3(0.2, 1.1, 0.2), WOOD); // the wheel's post
  for (const side of [1, -1]) {
    // The cap rail along the bulwark's top.
    let prev: V3 | null = null;
    for (let i = 0; i < STATIONS; i += 3) {
      const u = i / (STATIONS - 1);
      const p = v3(-L / 2 + u * L, sheer(u) + 0.03, side * halfBeam(u));
      if (prev !== null) m.rod(prev, p, 0.07, 0.07, WOOD, 5);
      prev = p;
    }
  }
}

// ── spars, rigging and sails ──────────────────────────────────────────────

interface Rig {
  fore: { foot: V3; head: V3; hounds: V3; top: V3 };
  main: { foot: V3; head: V3; hounds: V3; top: V3 };
  sprit: V3;
}

/** Swings a point aft of a mast round it, `deg` towards +z (the boom's sheet let out). */
function swing(mast: V3, p: V3, deg: number): V3 {
  const a = (deg * Math.PI) / 180;
  const d = sub(p, mast);
  return v3(mast.x + d.x * Math.cos(a) - d.z * Math.sin(a), p.y, mast.z - d.x * Math.sin(a) + d.z * Math.cos(a));
}

function rig(spars: MeshBuilder, lines: MeshBuilder): Rig {
  const deck = (x: number) => sheer((x + L / 2) / L) - 0.59;
  const mast = (x: number, height: number, rake: number) => {
    const foot = v3(x, deck(x), 0);
    const head = v3(x - height * Math.sin((rake * Math.PI) / 180), deck(x) + height, 0);
    return { foot, head, hounds: lerp3(foot, head, 0.78), top: add(head, v3(-0.3, 4.2, 0)) };
  };
  const fore = mast(4.4, 16.5, 3);
  const main = mast(-3.4, 19, 4);
  for (const m of [fore, main]) {
    spars.rod(add(m.foot, v3(0, -0.6, 0)), m.head, 0.2, 0.15, WOOD, 10);
    spars.rod(m.head, m.top, 0.1, 0.06, WOOD, 8);
    spars.rod(sub(m.hounds, v3(0, 0.15, 0)), add(m.hounds, v3(0, 0.15, 0)), 0.26, 0.26, IRON, 10); // the mast band
  }
  const stem = v3(L / 2 - 0.2, sheer(1) - 0.1, 0);
  const sprit = add(stem, v3(7.5, 1.1, 0));
  spars.rod(sub(stem, v3(2.2, 0.35, 0)), sprit, 0.17, 0.09, WOOD, 8);
  // Standing rigging: shrouds to the rails, stays fore and aft.
  const rail = (x: number, side: number) => {
    const u = (x + L / 2) / L;
    return v3(x, sheer(u) + 0.05, side * halfBeam(u) * 1.02);
  };
  const r = 0.028;
  for (const m of [fore, main]) {
    for (const side of [1, -1]) {
      for (const dx of [-0.6, 0.5]) lines.rod(m.hounds, rail(m.foot.x + dx, side), r, r, IRON, 4);
      lines.rod(sub(m.top, v3(0, 0.4, 0)), rail(m.foot.x - 2.2, side), r * 0.8, r * 0.8, IRON, 4);
    }
  }
  lines.rod(fore.hounds, stem, r, r, IRON, 4); // the forestay
  lines.rod(sub(fore.top, v3(0, 0.3, 0)), sprit, r, r, IRON, 4); // the jib stay
  lines.rod(sub(main.top, v3(0, 0.3, 0)), sub(fore.top, v3(0, 0.6, 0)), r, r, IRON, 4); // the spring stay
  lines.rod(main.hounds, fore.hounds, r, r, IRON, 4);
  lines.rod(sprit, v3(stem.x - 0.4, 0.4, 0), r, r, IRON, 4); // the bobstay
  return { fore, main, sprit };
}

/** A sail: a patch over its four corners (tack, clew, throat, peak), bellied by `depth` towards +z. */
function sail(m: MeshBuilder, tack: V3, clew: V3, throat: V3, peak: V3, depth: number, nu = 12, nv = 16): void {
  const first = m.positions.length / 3;
  for (let j = 0; j < nv; j++) {
    const v = j / (nv - 1);
    const a = lerp3(tack, throat, v);
    const b = lerp3(clew, peak, v);
    const chord = len(sub(b, a));
    for (let i = 0; i < nu; i++) {
      const u = i / (nu - 1);
      const p = lerp3(a, b, u);
      // The draft is deepest a third aft of the luff, flattening aloft.
      const draft = depth * chord * Math.sin(Math.PI * Math.pow(u, 0.75)) * (1 - 0.45 * v) * Math.sin(Math.PI * (0.06 + 0.94 * v));
      const shade = 0.82 + 0.18 * v;
      m.vertex(add(p, v3(0, 0, draft)), [CANVAS[0] * shade, CANVAS[1] * shade, CANVAS[2] * shade]);
    }
  }
  m.grid(first, nv, nu);
}

function sails(spars: MeshBuilder, cloth: MeshBuilder, r: Rig): void {
  const gaffRig = (mast: Rig["fore"], boomLen: number, throatH: number, gaffLen: number, swingDeg: number) => {
    const tack = lerp3(mast.foot, mast.head, 0.1);
    const throat = lerp3(mast.foot, mast.head, throatH);
    const clew = swing(tack, add(tack, v3(-boomLen, 0.3, 0)), swingDeg);
    const peakFlat = add(throat, scale(norm(v3(-1, 0.72, 0)), gaffLen));
    const peak = swing(throat, peakFlat, swingDeg + 6);
    spars.rod(sub(tack, v3(-0.2, 0.05, 0)), clew, 0.11, 0.09, WOOD, 8); // the boom
    spars.rod(throat, peak, 0.09, 0.07, WOOD, 8); // the gaff
    sail(cloth, add(tack, v3(-0.25, 0.15, 0)), clew, add(throat, v3(-0.25, -0.1, 0)), peak, 0.09);
    // The gaff topsail: from the topmast's head to the gaff's peak and the throat.
    sail(cloth, add(throat, v3(-0.3, 0.3, 0)), add(peak, v3(0.2, 0.2, 0)), sub(mast.top, v3(0, 0.3, 0)), sub(mast.top, v3(0, 0.3, 0)), 0.05, 8, 10);
    return { tack, clew, peak };
  };
  gaffRig(r.fore, 6.8, 0.74, 6.2, 22);
  gaffRig(r.main, 10.5, 0.8, 7.6, 18);
  // Headsails: the staysail on the forestay, the jib on the jib stay.
  const stem = v3(L / 2 - 0.2, sheer(1) - 0.1, 0);
  const foreStayTop = r.fore.hounds;
  sail(cloth, add(stem, v3(-0.5, 0.6, 0)), swing(stem, add(stem, v3(-6.2, 0.9, 0)), 20), lerp3(stem, foreStayTop, 0.88), lerp3(stem, foreStayTop, 0.88), 0.08, 10, 12);
  const jibTop = lerp3(r.sprit, sub(r.fore.top, v3(0, 0.3, 0)), 0.86);
  sail(cloth, lerp3(r.sprit, jibTop, 0.06), swing(r.sprit, add(r.sprit, v3(-8.4, 1.4, 0)), 26), jibTop, jibTop, 0.08, 10, 12);
}

// ── the ship ──────────────────────────────────────────────────────────────

/** The hull points the sea is sampled at, in ship space (a metre under the waterline). */
const FLOATS: V3[] = [];
for (const x of [-9, -4.5, 0, 4.5, 9]) for (const z of [-2, 2]) FLOATS.push(v3(x, -1, z));

export interface ShipParts {
  ship: Entity;
  lamps: Material;
  lights: Entity[];
}

export function buildShip(world: World, at: V3): ShipParts {
  const hullM = new MeshBuilder();
  const spars = new MeshBuilder();
  const lines = new MeshBuilder();
  const cloth = new MeshBuilder();
  hull(hullM);
  const r = rig(spars, lines);
  sails(spars, cloth, r);
  // Lines join the spars' mesh: one draw for the dark rigging and the wood.
  const base = spars.positions.length / 3;
  spars.positions.push(...lines.positions);
  spars.colors.push(...lines.colors);
  spars.indices.push(...lines.indices.map((i) => i + base));

  const ship = must(world.spawn({ name: "ship", position: at }));
  must(world.insert(ship, og.RigidBody, { mass: MASS, linearDamping: 0.05, angularDamping: 0.3, lock: 16, centerOfMass: { x: 0, y: -1.1, z: 0 }, canSleep: false }));
  must(world.insert(ship, og.Collider, { shape: SHAPE_BOX, size: { x: L * 0.9, y: 3.4, z: BEAM * 2 }, offset: { x: 0, y: -0.2, z: 0 } }));
  must(world.insert(ship, og.Velocity, { linear: { x: 3.6, y: 0, z: 0 } }));
  must(world.insert(ship, Ship, { targetX: at.x }));

  const part = (name: string, g: MeshBuilder, material: Material) => {
    const e = must(world.spawn({ name, parent: ship }));
    must(world.insert(e, og.Mesh, { geometry: g.geometry(world), material, castShadow: true, receiveShadow: true }));
    return e;
  };
  part("ship-hull", hullM, must(og.render.standardMaterial(world, { roughness: 0.55, side: "double", clearcoat: 0.4, clearcoatRoughness: 0.35 })));
  part("ship-spars", spars, must(og.render.standardMaterial(world, { roughness: 0.6 })));
  part("ship-sails", cloth, must(og.render.standardMaterial(world, { roughness: 0.85, side: "double", sheen: 0.4, sheenColor: { r: 0.9, g: 0.85, b: 0.75 }, sheenRoughness: 0.6 })));

  // Two lamps, a stern lantern and one at the main's hounds, lit after sunset.
  const lamps = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.75, b: 0.45 }, emissive: { r: 1, g: 0.62, b: 0.3 }, emissiveIntensity: 0 }));
  const ball = must(og.render.sphereGeometry(world, { radius: 0.18 }));
  const lights: Entity[] = [];
  for (const [name, p] of [
    ["ship-lamp-stern", v3(-L / 2 + 0.6, sheer(0) + 0.9, 0)],
    ["ship-lamp-mast", add(r.main.hounds, v3(0.4, -0.6, 0.4))],
  ] as const) {
    const e = must(world.spawn({ name, parent: ship, position: p }));
    must(world.insert(e, og.Mesh, { geometry: ball, material: lamps }));
    must(world.insert(e, og.PointLight, { intensity: 0, distance: 18, color: { r: 1, g: 0.62, b: 0.3 } }));
    lights.push(e);
  }
  return { ship, lamps, lights };
}

// ── buoyancy and steering ─────────────────────────────────────────────────

type Q = { x: number; y: number; z: number; w: number };
function rotate(q: Q, p: V3): V3 {
  // p + 2w (q × p) + 2 q × (q × p)
  const tx = 2 * (q.y * p.z - q.z * p.y);
  const ty = 2 * (q.z * p.x - q.x * p.z);
  const tz = 2 * (q.x * p.y - q.y * p.x);
  return v3(p.x + q.w * tx + (q.y * tz - q.z * ty), p.y + q.w * ty + (q.z * tx - q.x * tz), p.z + q.w * tz + (q.x * ty - q.y * tx));
}

const G = 9.81;
const K = (MASS * G) / FLOATS.length; // N per metre of depth: afloat with the points a metre under
const C = (MASS * Math.sqrt(G)) / FLOATS.length; // half of critical damping in heave
const HEEL = 0.07 * K * FLOATS.reduce((s, p) => s + p.z * p.z, 0); // the wind's steady heel, about 4°

/**
 * One tick of the ship: buoyancy at each hull point, the wind's heel, and
 * a steering force that holds it on its course at `targetX` (moving at
 * `speed`), the sea's resistance across it.
 */
export function floatShip(world: World, ship: Entity, sea: Entity, targetX: number, speed: number, targetZ: number): void {
  const t = must(world.get(ship, og.Transform));
  const vel = must(world.get(ship, og.Velocity));
  const p = t.position;
  const q = t.quaternion;
  const v = vel.linear;
  const w = vel.angular;
  let under = 0;
  for (const local of FLOATS) {
    const r = rotate(q, local);
    const at = add(p, r);
    const h = must(og.physics.waterHeight(world, sea, at.x, at.z));
    const depth = Math.min(2.5, h - at.y);
    if (depth <= 0) continue;
    under++;
    // The point's vertical speed: v + ω × r.
    const vy = v.y + (w.z * r.x - w.x * r.z);
    must(og.physics.applyForce(world, ship, { x: 0, y: K * depth - C * vy, z: 0 }, at));
  }
  // The wind heels it to leeward (+z): a torque about +x.
  must(og.physics.applyTorque(world, ship, { x: HEEL * Math.min(1, under / 6), y: 0, z: 0 }));
  // Course: a spring towards the loop's place along x, the keel's grip across.
  const ax = 0.25 * (targetX - p.x) + 1.1 * (speed - v.x);
  const az = 0.15 * (targetZ - p.z) - 1.5 * v.z;
  must(og.physics.applyForce(world, ship, { x: MASS * ax, y: 0, z: MASS * az }));
  const s = must(world.get(ship, Ship));
  const up = rotate(q, v3(0, 1, 0));
  const fwd = rotate(q, v3(1, 0, 0));
  must(
    world.set(ship, Ship, {
      targetX,
      submerged: under,
      heave: p.y,
      roll: (Math.asin(Math.max(-1, Math.min(1, up.z))) * 180) / Math.PI,
      pitch: (Math.asin(Math.max(-1, Math.min(1, fwd.y))) * 180) / Math.PI,
    }),
  );
  void s;
}
