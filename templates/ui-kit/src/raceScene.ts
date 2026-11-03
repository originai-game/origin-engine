// The racing showcase's scene (the owner's call of 2026-11-03: an
// engine-rendered track behind the racing HUD, not a flat plate): the
// Meridian Loop's circuit (assets/models/race, src/raceTrack.ts) with the
// player's car and two rivals driven along it, seen from a chase camera,
// lit by the pack's family: night for the realistic packs, golden hour for
// broadcast, a bright noon for the arcade ones (cartoon, kawaii).
//
// Built once at the start under one root parked far under the world; the
// racing HUD asks for it each tick it is built (raceScene), and RaceView,
// after the kit's system, raises it, drives the cars, places the camera and
// sets the light. On any other screen RaceView parks it again and gives the
// kit back its orthographic camera and flat render settings.

import { og, defineSystem } from "@origingame/engine";
import type { Color, Entity, World } from "@origingame/engine";
import { at, STEP, WIDTH } from "./raceTrack.ts";

type V = { x: number; y: number; z: number };
type Q = { x: number; y: number; z: number; w: number };

/** The scene's light: night (the realistic packs), dusk (broadcast), noon (the arcade packs). */
export type RaceLight = "night" | "dusk" | "noon";

/** What the racing HUD asks this tick: the light, how far the player has driven and how fast. */
interface Ask {
  light: RaceLight;
  /** Metres past the start line (negative: on the grid behind it). */
  distance: number;
  kmh: number;
  /** A phone in portrait: the camera aims lower so the car rides above the HUD's instrument. */
  portrait: boolean;
}
let ask: Ask | null = null;
let active = false;
/** The kit's own camera and render settings, taken when the scene first shows and given back when it hides. */
let saved: Record<string, unknown> | null = null;

/** The racing HUD wants the scene this tick. */
export function raceScene(a: Ask): void {
  ask = a;
}

const PARK = -10_000;
const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** The rivals ahead: metres in front of the player and across the road (positive to the right). */
export const RIVALS: ReadonlyArray<[number, number, Color]> = [[24, -3.2, { r: 0.12, g: 0.42, b: 0.86 }], [58, 2.6, { r: 0.86, g: 0.16, b: 0.18 }]];

/** Where rival i is when the player has driven d metres: its lead, breathing a little. */
export function rivalDistance(d: number, i: number): number {
  return d + RIVALS[i]![0] + Math.sin(d / 90 + i) * 4;
}
/** Lamps along the circuit: sample, colour (as the Meridian Loop's). */
const LAMPS: ReadonlyArray<[number, number, number, number]> = [
  [0, 1, 0.3, 0.85], [70, 1, 0.2, 0.7], [150, 0.3, 0.9, 1], [235, 1, 0.65, 0.3], [300, 1, 0.65, 0.3], [370, 1, 0.15, 0.6], [430, 0.4, 0.8, 1],
];
const WHEELS: ReadonlyArray<[number, number]> = [[-0.85, 1.3], [0.85, 1.3], [-0.85, -1.3], [0.85, -1.3]];

/** Loads the circuit and the cars under a parked root (the game's start). */
export function buildRaceScene(world: World): void {
  const root = must(world.spawn({ name: "race-root", position: { x: 0, y: PARK, z: 0 } }));
  for (const [m, shadow] of [["track", false], ["scenery", true], ["ground", false]] as const) {
    const asset = og.load.gltf(world, `models/race/${m}.glb`, { castShadow: shadow, receiveShadow: true });
    if (asset.ok) must(world.instantiate(asset.value, { name: `race-${m}`, parent: root }));
  }
  // The arcade packs' noon: a grass field in place of the Meridian Loop's neon grid (race-ground, parked under the world then).
  const grass = must(world.spawn({ name: "race-grass", parent: root, position: { x: 100, y: -0.56, z: 0 }, quaternion: { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 } }));
  const plane = og.render.planeGeometry(world, { width: 1800, height: 1800 });
  const turf = og.render.standardMaterial(world, { color: { r: 0.36, g: 0.66, b: 0.3 }, roughness: 0.95 });
  if (plane.ok && turf.ok) must(world.insert(grass, og.Mesh, { geometry: plane.value, material: turf.value, receiveShadow: true, visible: false }));
  const car = og.load.gltf(world, "models/race/car.glb", { castShadow: true });
  const wheel = og.load.gltf(world, "models/race/wheel.glb", { castShadow: true });
  ["race-car", ...RIVALS.map((_, i) => `race-rival-${i}`)].forEach((name, i) => {
    const e = must(world.spawn({ name, parent: root }));
    if (car.ok) must(world.insert(e, og.Model, { asset: car.value, ...(i > 0 ? { tint: RIVALS[i - 1]![2] } : {}) }));
    WHEELS.forEach(([x, z], w) => {
      const we = must(world.spawn({ name: `${name}-wheel-${w}`, parent: e, position: { x, y: -0.1, z } }));
      if (wheel.ok) must(world.insert(we, og.Model, { asset: wheel.value }));
    });
  });
  const sun = must(world.spawn({ name: "race-sun", parent: root, quaternion: { x: -0.42, y: 0.28, z: 0.14, w: 0.85 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 0, castShadow: true, shadowDistance: 90, shadowSoftness: 0.02 }));
  LAMPS.forEach(([k, r, g, b], i) => {
    const s = at(k);
    const e = must(world.spawn({ name: `race-lamp-${i}`, parent: root, position: { x: s.x - s.rx * (WIDTH / 2 + 3), y: s.y + 7, z: s.z - s.rz * (WIDTH / 2 + 3) } }));
    must(world.insert(e, og.PointLight, { color: { r, g, b }, intensity: 0, distance: 38 }));
  });
}

/** The fields of a resource, copied (a resource is a live view: spreading it copies nothing). */
function pick(o: object, keys: readonly string[]): Record<string, unknown> {
  const r = o as Record<string, unknown>;
  return Object.fromEntries(keys.map((k) => [k, typeof r[k] === "object" && r[k] !== null ? { ...(r[k] as object) } : r[k]]));
}

function yaw(a: number): Q {
  return { x: 0, y: Math.sin(a / 2), z: 0, w: Math.cos(a / 2) };
}

function mul(a: Q, b: Q): Q {
  return {
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
  };
}

/** The rotation that looks from eye at target, +y up (the camera looks down its -z): examples/meridian-loop/src/view.ts. */
function lookAt(eye: V, target: V): Q {
  let zx = eye.x - target.x, zy = eye.y - target.y, zz = eye.z - target.z;
  const zl = Math.hypot(zx, zy, zz) || 1;
  zx /= zl; zy /= zl; zz /= zl;
  let xx = zz, xz = -zx;
  const xl = Math.hypot(xx, xz) || 1;
  xx /= xl; xz /= xl;
  const xy = 0;
  const yx = zy * xz - zz * xy, yy = zz * xx - zx * xz, yz = zx * xy - zy * xx;
  const tr = xx + yy + zz;
  if (tr > 0) {
    const s = Math.sqrt(tr + 1) * 2;
    return { w: 0.25 * s, x: (yz - zy) / s, y: (zx - xz) / s, z: (xy - yx) / s };
  }
  if (xx > yy && xx > zz) {
    const s = Math.sqrt(1 + xx - yy - zz) * 2;
    return { w: (yz - zy) / s, x: 0.25 * s, y: (yx + xy) / s, z: (zx + xz) / s };
  }
  if (yy > zz) {
    const s = Math.sqrt(1 + yy - xx - zz) * 2;
    return { w: (zx - xz) / s, x: (yx + xy) / s, y: 0.25 * s, z: (zy + yz) / s };
  }
  const s = Math.sqrt(1 + zz - xx - yy) * 2;
  return { w: (xy - yx) / s, x: (zx + xz) / s, y: (zy + yz) / s, z: 0.25 * s };
}

/** A pose on the road `d` metres past the line, `side` metres right of the centre, the car's chassis 0.45 m over it. */
export function pose(d: number, side: number): { position: V; heading: number } {
  const k = d / STEP;
  const a = at(Math.floor(k));
  const b = at(Math.floor(k) + 1);
  const u = k - Math.floor(k);
  const x = a.x + (b.x - a.x) * u + a.rx * side;
  const z = a.z + (b.z - a.z) * u + a.rz * side;
  const y = a.y + (b.y - a.y) * u + a.ry * side + 0.45;
  return { position: { x, y, z }, heading: Math.atan2(a.tx + (b.tx - a.tx) * u, a.tz + (b.tz - a.tz) * u) };
}

/** A directional light's rotation: pitched down by `down` radians, then turned by `turn` about y (it shines down its -z). */
function sunAt(down: number, turn: number): Q {
  return mul(yaw(turn), { x: -Math.sin(down / 2), y: 0, z: 0, w: Math.cos(down / 2) });
}

// Each light keeps the fog off its sky (fog.sky, the share of the fog laid over the sky: at its default the sky was a flat
// haze in every pack) and gives the sun a direction, its shadow soft and close under the car.
const LIGHTS: Record<RaceLight, { sun: Color; sunI: number; sunQ: Q; lamps: number; bg: Color; sky: Record<string, unknown>; fog: Record<string, unknown>; env: number; exposure: number; bloom: boolean }> = {
  // Real night: a near-black sky with stars, so the neon edges, the windows and the tail lights carry the picture; a cool moon high behind.
  night: {
    sun: { r: 0.55, g: 0.62, b: 1 }, sunI: 0.9, sunQ: sunAt(1.15, 2.6), lamps: 60, bg: { r: 0.008, g: 0.006, b: 0.025 }, env: 0.25, exposure: 1.05, bloom: true,
    sky: { enabled: true, zenith: { r: 0.006, g: 0.006, b: 0.02 }, horizon: { r: 0.09, g: 0.04, b: 0.16 }, ground: { r: 0.01, g: 0.008, b: 0.02 }, sunDirection: { x: 0.1, y: 0.06, z: -1 }, sunColor: { r: 0.9, g: 0.4, b: 0.7 }, sunSize: 0, sunIntensity: 0, glow: 0.4, stars: 0.8 },
    fog: { enabled: true, color: { r: 0.05, g: 0.025, b: 0.09 }, density: 0.0016, sky: 0.05 },
  },
  // Golden hour: a warm low sun ahead and to the side, a blue zenith fading to amber, a light warm haze on the distance only.
  dusk: {
    sun: { r: 1, g: 0.74, b: 0.48 }, sunI: 3.4, sunQ: sunAt(0.45, 0.5), lamps: 10, bg: { r: 0.3, g: 0.36, b: 0.55 }, env: 0.6, exposure: 1, bloom: true,
    sky: { enabled: true, zenith: { r: 0.12, g: 0.24, b: 0.55 }, horizon: { r: 1, g: 0.62, b: 0.32 }, ground: { r: 0.22, g: 0.18, b: 0.16 }, sunDirection: { x: 0.3, y: 0.1, z: -1 }, sunColor: { r: 1, g: 0.7, b: 0.38 }, sunSize: 7, sunIntensity: 8, glow: 1 },
    fog: { enabled: true, color: { r: 0.95, g: 0.68, b: 0.48 }, density: 0.0009, sky: 0.1 },
  },
  // A clear cartoon noon: a saturated blue sky lightening only near the horizon, a strong sun, almost no haze.
  noon: {
    sun: { r: 1, g: 0.96, b: 0.86 }, sunI: 4.4, sunQ: sunAt(1.35, 0.3), lamps: 0, bg: { r: 0.3, g: 0.6, b: 1 }, env: 1, exposure: 1.1, bloom: false,
    sky: { enabled: true, zenith: { r: 0.08, g: 0.36, b: 0.95 }, horizon: { r: 0.5, g: 0.78, b: 1 }, ground: { r: 0.36, g: 0.62, b: 0.3 }, sunDirection: { x: 0.4, y: 0.8, z: -0.6 }, sunColor: { r: 1, g: 0.97, b: 0.88 }, sunSize: 4, sunIntensity: 5, glow: 0.4 },
    fog: { enabled: true, color: { r: 0.62, g: 0.82, b: 1 }, density: 0.0004, sky: 0 },
  },
};

function lights(world: World, on: RaceLight | null): void {
  const l = on ? LIGHTS[on] : null;
  // Noon swaps the neon grid for grass.
  const noon = on === "noon";
  const ground = world.find("race-ground");
  if (ground !== null) must(world.set(ground, og.Transform, { position: { x: 0, y: noon ? -500 : 0, z: 0 } }));
  const grass = world.find("race-grass");
  if (grass !== null) must(world.set(grass, og.Mesh, { visible: noon }));
  const sun = world.find("race-sun");
  if (sun !== null) {
    must(world.set(sun, og.DirectionalLight, { intensity: l ? l.sunI : 0, shadowSoftness: 0.25, ...(l ? { color: l.sun } : {}) }));
    if (l) must(world.set(sun, og.Transform, { quaternion: l.sunQ }));
  }
  LAMPS.forEach((_, i) => {
    const e = world.find(`race-lamp-${i}`);
    if (e !== null) must(world.set(e, og.PointLight, { intensity: l ? l.lamps : 0 }));
  });
}

export const RaceView = defineSystem("RaceView", { reads: [], writes: [og.Camera, og.RenderSettings, og.Sky, og.Fog, og.Transform, og.DirectionalLight, og.PointLight, og.Mesh], after: ["Kit"] }, (_q, _time, world) => {
  const a = ask;
  ask = null;
  const root = world.find("race-root");
  if (root === null) return;
  const cam = must(world.resource(og.Camera));
  const rs = must(world.resource(og.RenderSettings));
  const sky = must(world.resource(og.Sky));
  const fog = must(world.resource(og.Fog));
  if (!a) {
    if (!active) return;
    // Back to the kit: the root parked, the lights out, the camera and settings as they were.
    active = false;
    must(world.set(root, og.Transform, { position: { x: 0, y: PARK, z: 0 } }));
    lights(world, null);
    if (saved) {
      Object.assign(cam, saved["camera"]);
      Object.assign(rs, saved["render"]);
      Object.assign(sky, saved["sky"]);
      Object.assign(fog, saved["fog"]);
    }
    og.render.snapCamera(world);
    return;
  }
  const l = LIGHTS[a.light];
  if (!active) {
    saved ??= {
      camera: { orthographic: cam.orthographic, height: cam.height, position: { ...cam.position }, quaternion: { ...cam.quaternion }, background: { ...cam.background }, exposure: cam.exposure, fov: cam.fov, near: cam.near, far: cam.far },
      render: { toneMapped: rs.toneMapped, environmentIntensity: rs.environmentIntensity, sky: rs.sky, bloom: rs.bloom, exposure: rs.exposure, bloomIntensity: rs.bloomIntensity, bloomThreshold: rs.bloomThreshold },
      sky: pick(sky, ["enabled", "zenith", "horizon", "ground", "sunDirection", "sunColor", "sunSize", "sunIntensity", "glow", "stars"]),
      fog: pick(fog, ["enabled", "color", "density", "sky"]),
    };
    active = true;
    must(world.set(root, og.Transform, { position: { x: 0, y: 0, z: 0 } }));
    og.render.snapCamera(world);
  }
  lights(world, a.light);
  rs.toneMapped = true;
  rs.environmentIntensity = l.env;
  rs.sky = true;
  rs.bloom = l.bloom;
  // Half the Meridian Loop's bloom, over a higher threshold: the tail lights glow without outshining the HUD.
  rs.bloomIntensity = 0.45;
  rs.bloomThreshold = 0.9;
  rs.exposure = l.exposure;
  Object.assign(sky, l.sky);
  Object.assign(fog, l.fog);
  // The cars: the player's at its distance, the rivals ahead in their lanes, the wheels turning with the road.
  const me = pose(a.distance, 0.6);
  const place = (name: string, p: { position: V; heading: number }, d: number) => {
    const e = world.find(name);
    if (e === null) return;
    must(world.set(e, og.Transform, { position: p.position, quaternion: yaw(p.heading) }));
    const spin = d / 0.35;
    WHEELS.forEach(([x], w) => {
      const we = world.find(`${name}-wheel-${w}`);
      if (we === null) return;
      const turn = { x: Math.sin(spin / 2), y: 0, z: 0, w: Math.cos(spin / 2) };
      // The right wheels face out: turned half round about y, then rolled.
      must(world.set(we, og.Transform, { quaternion: x > 0 ? mul(yaw(Math.PI), { ...turn, x: -turn.x }) : turn }));
    });
  };
  place("race-car", me, a.distance);
  RIVALS.forEach(([ahead, side], i) => {
    const d = rivalDistance(a.distance, i);
    void ahead;
    place(`race-rival-${i}`, pose(d, side), d);
  });
  // The chase view: behind and over the car on its heading, looking a little ahead of it; wider with speed.
  const back = 6.8;
  const h = me.heading;
  const p = me.position;
  const eye = { x: p.x - Math.sin(h) * back, y: p.y + 2.3, z: p.z - Math.cos(h) * back };
  // On a phone in portrait the aim drops in front of the car, which rides higher in the frame, clear of the instrument.
  const target = a.portrait ? { x: p.x + Math.sin(h) * 3, y: p.y - 1.4, z: p.z + Math.cos(h) * 3 } : { x: p.x + Math.sin(h) * 6, y: p.y + 0.9, z: p.z + Math.cos(h) * 6 };
  cam.orthographic = false;
  cam.position = eye;
  cam.quaternion = lookAt(eye, target);
  cam.fov = 56 + Math.min(14, a.kmh / 20);
  cam.near = 0.1;
  cam.far = 1600;
  cam.background = l.bg;
  cam.exposure = 1;
});
