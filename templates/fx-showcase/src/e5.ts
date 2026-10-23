// The E5 round's stations: atmosphere, weather, water, material shaders and
// the physics beyond rigid bodies, each far from the others along x.
//
//   sky         dusk over a colonnade: og.DayNight (the sun, sky, light and
//               ambient from the clock) and og.Fog (distance and height)
//   weather     rain (og.Weather) over a yard: streaks, fog, dimmed light
//   water       og.Water over an og.WaterSurface: depth colour, foam at the
//               posts, reflections, a crate dropped in that splashes
//   shaders     og.load.shader: a sphere dissolving by noise (its amount set
//               with og.render.setShaderParam), wind-swayed grass, a slab
//               drawn by a full custom shader (`mode full lit`) whose shadow
//               has the same holes as its cut, all under a CRT custom post
//               pass (og.load.post, PostEffects.custom8) after a warm
//               grade (custom5)
//   controller  og.CharacterController walking up a slope and stairs, onto
//               a moving platform that carries it, and jumping
//   joints      a motorised hinge, a chain of ball joints, a spring, and the
//               walker as an og.Ragdoll going limp from its animation
//   rope        og.Rope holding a ball, og.Cloth flags in gusting wind
//   fluid       og.Fluid2D drawn as metaballs, and og.WaterLine (drawn by
//               og.Water2D) splashed by a falling crate

import { og, defineComponent, defineSystem, f32, f64, u32, Vector3 } from "@origingame/engine";
import {
  JOINT_BALL,
  JOINT_REVOLUTE,
  JOINT_ROPE,
  JOINT_SPRING,
  MOTOR_VELOCITY,
  PIN_TOP,
  PIN_TOP_CORNERS,
  SHAPE_BOX,
  SHAPE_CAPSULE,
  SHAPE_SPHERE,
  BODY_KINEMATIC,
  WEATHER_NONE,
  WEATHER_RAIN,
} from "@origingame/engine";
import type { Asset, ClipAsset, Entity, Geometry, Material, Quaternion, SkeletonAsset, Texture, Vector3 as Vec3, World } from "@origingame/engine";

export const E5_STATIONS = ["sky", "weather", "water", "shaders", "controller", "joints", "rope", "fluid"] as const;
export type E5Station = (typeof E5_STATIONS)[number];

/** Where each station lives along x. */
export const E5_ORIGIN: Record<E5Station, number> = {
  sky: 400,
  weather: 500,
  water: 600,
  shaders: 700,
  controller: 800,
  joints: 900,
  rope: 1100,
  fluid: 1200,
};

/** The round's handles and the station shown, on the entity "e5". */
export const E5 = defineComponent("E5", {
  /** Index into E5_STATIONS + 1 (0: another station). */
  station: u32(0),
  /** Ticks since the station was shown. */
  shown: u32(0),
  dissolve: f64(0),
  /** The full-shader slab's material, the CRT post pass and the warm grade before it. */
  fullDissolve: f64(0),
  crt: f64(0),
  tint: f64(0),
  crate: f64(0),
  pondCrate: f64(0),
  hero: f64(0),
  ragdoll: f64(0),
  stone: f64(0),
});

const TAU = Math.PI * 2;

/** A kinematic platform moving between two points over `period` seconds and back. */
export const PlatformPath = defineComponent("PlatformPath", {
  from: Vector3(),
  to: Vector3(),
  period: f32(3),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const axisAngle = (axis: Vec3, angle: number): Quaternion => og.transform.fromAxisAngle(axis, angle);

// ── systems ────────────────────────────────────────────────────────────────

/**
 * Per tick on the shown station: the dissolve's amount, the character's
 * walk, the crates dropped into the water, the ragdoll going limp, and the
 * fluid station's basin sprites.
 */
export const E5Show = defineSystem(
  "E5Show",
  { writes: [E5, og.CharacterController, og.Ragdoll, og.Transform, og.Velocity] },
  (_q, time, world) => {
    const e = world.find("e5");
    if (e === null) return;
    const s = must(world.get(e, E5));
    world.set(e, E5, { shown: s.shown + 1 });
    if (s.station === 0) return;
    const station = E5_STATIONS[s.station - 1]!;
    const shown = s.shown + 1;
    const t = shown / 60;
    const drop = (body: number, at: Vec3) => {
      world.set(body as Entity, og.Transform, { position: at, quaternion: axisAngle({ x: 1, y: 0.3, z: 0.2 }, 0.4) });
      world.set(body as Entity, og.Velocity, { linear: { x: 0, y: -2, z: 0 }, angular: { x: 0, y: 0, z: 0 } });
    };
    switch (station) {
      case "shaders": {
        // The dissolve sweeps 0 → 0.8 over 3 s and back.
        const phase = (time.tick % 360) / 360;
        must(og.render.setShaderParam(world, s.dissolve as unknown as Material, "amount", [0.8 * (phase < 0.5 ? phase * 2 : 2 - phase * 2)]));
        // The full shader's cut opens from 0.25 to 0.55 over the same period.
        must(og.render.setShaderParam(world, s.fullDissolve as unknown as Material, "amount", [0.25 + 0.3 * (phase < 0.5 ? phase * 2 : 2 - phase * 2)]));
        break;
      }
      case "controller": {
        // Up the stairs and the slope, a jump on the landing, then off its
        // end onto the lift, which carries it.
        const at = must(world.get(s.hero as Entity, og.Transform)).position.x;
        const walk = at < E5_ORIGIN.controller + 7 ? 2.4 : 0;
        world.set(s.hero as Entity, og.CharacterController, { move: { x: walk, y: 0, z: 0 }, jump: t > 2.4 && t < 2.6 });
        break;
      }
      case "water":
        if (shown === 20) drop(s.crate, { x: E5_ORIGIN.water + 0.6, y: 3.5, z: -0.5 });
        break;
      case "fluid":
        if (shown === 20) drop(s.pondCrate, { x: E5_ORIGIN.fluid + 4.4, y: 1.5, z: 0 });
        basin(world, s.stone as unknown as Texture);
        break;
      case "joints":
        world.set(s.ragdoll as Entity, og.Ragdoll, { weight: shown > 40 ? 1 : 0 });
        break;
    }
  },
);

/** Moves kinematic platforms between their two points, easing at the ends. */
export const PlatformMove = defineSystem(
  "PlatformMove",
  { reads: [PlatformPath], writes: [og.Transform, og.Velocity] },
  (q, time) => {
    const dt = 1 / 60;
    for (const a of q.archetypes()) {
      const path = a.column(PlatformPath);
      const p = a.column(og.Transform).position;
      const v = a.column(og.Velocity).linear;
      for (let i = 0; i < a.count; i++) {
        const w = (time.tick * dt * TAU) / path.period[i]!;
        const k = 0.5 - 0.5 * Math.cos(w);
        const dk = 0.5 * Math.sin(w) * (TAU / path.period[i]!);
        for (let c = 0; c < 3; c++) {
          const from = path.from[i * 3 + c]!;
          const to = path.to[i * 3 + c]!;
          p[i * 3 + c] = from + (to - from) * k;
          v[i * 3 + c] = (to - from) * dk;
        }
      }
    }
  },
);

/** The fluid station's walls, drawn as sprites over the colliders. */
function basin(world: World, stone: Texture): void {
  const x0 = E5_ORIGIN.fluid;
  for (const [x, y, w, h] of WALLS) {
    og.render.sprite(world, { texture: stone, position: { x: x0 + x, y, z: 0 }, width: w, height: h, layer: 4, color: { r: 0.55, g: 0.52, b: 0.5 } });
  }
}

/** The fluid station's walls: centre x (from the station), centre y, width, height. */
const WALLS: ReadonlyArray<[number, number, number, number]> = [
  [-1.2, -3.6, 6.4, 0.4],
  [-4.2, -1.6, 0.4, 4.4],
  [1.8, -1.6, 0.4, 4.4],
];

// ── stations ───────────────────────────────────────────────────────────────

/** Turns the round's look off (every station starts from it). */
export function resetE5(world: World): void {
  must(world.resource(og.Fog)).enabled = false;
  must(world.resource(og.Sky)).enabled = false;
  must(world.resource(og.DayNight)).enabled = false;
  must(world.resource(og.Weather)).kind = WEATHER_NONE;
  const e = world.find("e5");
  if (e !== null) world.set(e, E5, { station: 0, shown: 0 });
}

/** Shows an E5 station: its camera, light and atmosphere. */
export function showE5(world: World, station: E5Station): void {
  if (world.find(`e5-${station}`) === null) {
    builders[station]?.(world);
    must(world.spawn({ name: `e5-${station}` }));
  }
  const e = world.find("e5");
  if (e !== null) world.set(e, E5, { station: E5_STATIONS.indexOf(station) + 1, shown: 0 });
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = E5_ORIGIN[station];
  const look = (eye: Vec3, target: Vec3) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, target);
  };
  const sunAt = (target: Vec3, intensity: number) => {
    if (sun === null) return;
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, target) });
    world.set(sun, og.DirectionalLight, { intensity });
  };
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.4;
  settings.environmentIntensity = 0.55;
  switch (station) {
    case "sky": {
      look({ x: x + 6, y: 2.4, z: 0 }, { x: x - 40, y: 4, z: 0 });
      const d = must(world.resource(og.DayNight));
      Object.assign(d, { enabled: true, start: 17.35, speed: 0.01, tilt: 20, azimuth: 0, lightIntensity: 1, tintFog: true, step: 5 });
      const fog = must(world.resource(og.Fog));
      Object.assign(fog, { enabled: true, density: 0.006, start: 10, heightDensity: 0.04, height: 0, falloff: 0.8, sky: 0.2, color: { r: 0.62, g: 0.6, b: 0.62 } });
      break;
    }
    case "weather": {
      look({ x: x, y: 2, z: 7 }, { x: x, y: 1.4, z: -4 });
      sunAt({ x: -0.3, y: -0.8, z: -0.4 }, 3);
      const w = must(world.resource(og.Weather));
      Object.assign(w, { kind: WEATHER_RAIN, intensity: 1, wind: { x: 2, y: 0, z: 0.6 }, radius: 16, dim: 1 });
      const sky = must(world.resource(og.Sky));
      Object.assign(sky, {
        enabled: true,
        zenith: { r: 0.32, g: 0.35, b: 0.4 },
        horizon: { r: 0.55, g: 0.57, b: 0.6 },
        ground: { r: 0.12, g: 0.12, b: 0.13 },
        sunDirection: { x: 0.3, y: 0.8, z: 0.4 },
        sunSize: 0,
        glow: 0.1,
      });
      break;
    }
    case "water":
      look({ x: x, y: 3.2, z: 9 }, { x: x, y: 0, z: -2 });
      sunAt({ x: 0.5, y: -0.45, z: 0.75 }, 3);
      break;
    case "shaders":
      look({ x: x, y: 2.2, z: 6.5 }, { x: x, y: 0.9, z: -1 });
      sunAt({ x: -0.45, y: -0.8, z: -0.35 }, 3);
      // A chain over the whole station (main.ts' show clears it
      // elsewhere): a warm grade in slot 5, then the CRT in the last slot,
      // 8, the slots between them left 0.
      if (e !== null) {
        const h = must(world.get(e, E5));
        Object.assign(must(world.resource(og.PostEffects)), { custom5: h.tint as never, custom8: h.crt as never });
      }
      break;
    case "controller":
      look({ x: x + 2.5, y: 3.5, z: 11 }, { x: x + 2.5, y: 1.2, z: 0 });
      sunAt({ x: -0.4, y: -0.8, z: -0.45 }, 3);
      break;
    case "joints":
      look({ x: x, y: 2.6, z: 8 }, { x: x, y: 1.4, z: 0 });
      sunAt({ x: -0.4, y: -0.8, z: -0.45 }, 3);
      break;
    case "rope":
      look({ x: x, y: 2.6, z: 7.5 }, { x: x, y: 2, z: 0 });
      sunAt({ x: -0.5, y: -0.7, z: -0.5 }, 3);
      break;
    case "fluid":
      cam.orthographic = true;
      cam.height = 9;
      cam.position = { x: x - 0.6, y: -1.4, z: 10 };
      cam.quaternion = { x: 0, y: 0, z: 0, w: 1 };
      cam.background = { r: 0.07, g: 0.07, b: 0.09 };
      settings.sky = false;
      settings.toneMapped = false;
      settings.bloom = false;
      break;
  }
}

// ── the world ──────────────────────────────────────────────────────────────

/** Each station's world, built when it is first shown (buildE5 fills it). */
let builders: Partial<Record<E5Station, (world: World) => void>> = {};

interface Kit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
}

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: Vec3, extra?: { quaternion?: Quaternion; scale?: Vec3; parent?: Entity }): Entity {
  const e = must(world.spawn({ name, position, ...(extra?.quaternion ? { quaternion: extra.quaternion } : {}), ...(extra?.parent !== undefined ? { parent: extra.parent } : {}) }));
  if (extra?.scale) world.set(e, og.Transform, { scale: extra.scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

/** A static box collider with a mesh of the same size. */
function block(world: World, k: Kit, name: string, material: Material, at: Vec3, size: Vec3, q?: Quaternion): Entity {
  const e = mesh(world, name, k.box, material, at, { scale: size, ...(q ? { quaternion: q } : {}) });
  world.insert(e, og.Collider, { shape: SHAPE_BOX });
  return e;
}

/** A dynamic box with a mesh, a quarter as dense as the water it floats on. */
function crate(world: World, k: Kit, name: string, material: Material, at: Vec3, size: number): Entity {
  const e = mesh(world, name, k.box, material, at, { scale: { x: size, y: size, z: size } });
  world.insert(e, og.RigidBody, { mass: 0.25 * size * size * size });
  world.insert(e, og.Collider, { shape: SHAPE_BOX });
  return e;
}

/**
 * Prepares the E5 stations: the shared materials now, each station's world
 * when it is first shown (showE5), so a run that never shows a station never
 * simulates it. A marker entity `e5-<station>` says a station is built (a
 * restored snapshot keeps it, so a station is never built twice).
 */
export function buildE5(world: World, k: Kit, walkerModel: Asset, walkerRig: { skeleton: SkeletonAsset; clips: ReadonlyArray<{ clip: ClipAsset; name: string }> }, stoneTexture: Texture): void {
  const stone = k.mat({ color: { r: 0.7, g: 0.66, b: 0.6 }, roughness: 0.85 });
  const wood = k.mat({ color: { r: 0.42, g: 0.28, b: 0.16 }, roughness: 0.7 });
  const red = k.mat({ color: { r: 0.8, g: 0.16, b: 0.1 }, roughness: 0.45 });
  const grass = k.mat({ color: { r: 0.3, g: 0.42, b: 0.2 }, roughness: 0.95 });
  const sand = k.mat({ color: { r: 0.76, g: 0.68, b: 0.5 }, roughness: 0.9 });

  builders = {
  // sky: a colonnade towards the west.
    sky: (world: World) => {
    const x = E5_ORIGIN.sky;
    k.ground(x, { x: 0.3, y: 0.32, z: 0.26 });
    for (let i = 0; i < 12; i++) {
      for (const z of [-5, 5]) mesh(world, `col-${i}-${z}`, k.box, stone, { x: x - i * 8, y: 4, z }, { scale: { x: 1, y: 8, z: 1 } });
    }
    },
  // weather: a yard of crates and a wall.
    weather: (world: World) => {
    const x = E5_ORIGIN.weather;
    k.ground(x, { x: 0.22, y: 0.24, z: 0.22 });
    mesh(world, "yard-wall", k.box, stone, { x, y: 2, z: -6 }, { scale: { x: 14, y: 4, z: 0.6 } });
    for (let i = 0; i < 6; i++) mesh(world, `yard-crate-${i}`, k.box, wood, { x: x - 5 + i * 2, y: 0.5, z: -3 + (i % 2) * 1.5 }, {});
    },
  // water: a sloping bed, posts and a red ball breaking the surface, and a crate to drop.
    water: (world: World) => {
    const x = E5_ORIGIN.water;
    block(world, k, "bed", sand, { x, y: -2.4, z: -6 }, { x: 40, y: 0.4, z: 40 }, axisAngle({ x: 1, y: 0, z: 0 }, -0.1));
    for (const s of [-1, 1]) block(world, k, `bank-${s}`, grass, { x: x + s * 18, y: -1.2, z: -6 }, { x: 20, y: 2, z: 40 }, axisAngle({ x: 0, y: 0, z: 1 }, s * 0.35));
    for (const [px, pz] of [[-3, -2], [-1, -5], [2.5, -3], [4, -8]]) mesh(world, `post-${px}`, k.box, wood, { x: x + px!, y: 0.6, z: pz! }, { scale: { x: 0.35, y: 4, z: 0.35 } });
    mesh(world, "buoy", k.sphere, red, { x: x - 1.6, y: 0.1, z: -0.6 }, { scale: { x: 1.2, y: 1.2, z: 1.2 } });
    const pool = must(world.spawn({ name: "pool", position: { x, y: 0, z: -2 } }));
    world.insert(pool, og.WaterSurface, { resolution: 64, size: 16, waveSpeed: 3, damping: 0.5, splash: 1.2 });
    world.insert(pool, og.Water, { size: { x: 40, y: 50 }, segments: 128 });
    // Parked on a shelf out of sight until the station drops it.
    block(world, k, "water-shelf", wood, { x: x + 20, y: -21, z: -0.5 }, { x: 2, y: 0.2, z: 2 });
    const c = crate(world, k, "water-crate", wood, { x: x + 20, y: -20.4, z: -0.5 }, 0.8);
    world.set(world.find("e5")!, E5, { crate: c });
    },
  // shaders: a dissolving sphere and a patch of grass.
    shaders: (world: World) => {
    const x = E5_ORIGIN.shaders;
    k.ground(x, { x: 0.3, y: 0.27, z: 0.2 });
    const dissolveShader = must(og.load.shader(world, "shaders/dissolve.ogt"));
    const grassShader = must(og.load.shader(world, "shaders/grass.ogt"));
    const dissolve = must(og.render.shaderMaterial(world, dissolveShader, { color: { r: 0.9, g: 0.62, b: 0.2 }, roughness: 0.4, alphaTest: 0.5, side: "double" }));
    mesh(world, "dissolving", k.sphere, dissolve, { x: x - 1.6, y: 1.2, z: -1 }, { scale: { x: 2, y: 2, z: 2 } });
    // A full custom shader: its own vertex and fragment entries; the
    // shadow maps and the prepass run them too, so the slab's shadow on the
    // ground behind it has the holes of its cut.
    const fullShader = must(og.load.shader(world, "shaders/dissolve-full.ogt"));
    const fullDissolve = must(og.render.shaderMaterial(world, fullShader, { color: { r: 0.15, g: 0.55, b: 0.6 }, roughness: 0.4, alphaTest: 0.5, side: "double" }));
    // One double-sided plane (a box's back face would cut its own holes
    // elsewhere and fill the front's in the shadow).
    const panel = must(og.render.planeGeometry(world, { width: 2.2, height: 1.9 }));
    mesh(world, "full-dissolve", panel, fullDissolve, { x: x + 1.9, y: 2.1, z: -1.4 }, { quaternion: axisAngle({ x: 0, y: 1, z: 0 }, 0.9) });
    const crt = must(og.load.post(world, "post/crt.ogt"));
    const tint = must(og.load.post(world, "post/tint.ogt"));
    const blade = must(og.render.boxGeometry(world, { width: 0.06, height: 0.7, depth: 0.01 }));
    const bladeMat = must(og.render.shaderMaterial(world, grassShader, { color: { r: 0.3, g: 0.55, b: 0.16 }, roughness: 0.8, side: "double" }));
    must(og.render.setShaderParam(world, bladeMat, "base", [-0.35]));
    let seed = 7;
    const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) >>> 8) / 16777216;
    for (let i = 0; i < 600; i++) {
      const bx = x - 4.5 + (i % 30) * 0.3 + (rnd() - 0.5) * 0.25;
      const bz = 0.6 + Math.floor(i / 30) * 0.12 + (rnd() - 0.5) * 0.1;
      const s = 0.7 + rnd() * 0.6;
      mesh(world, `blade-${i}`, blade, bladeMat, { x: bx, y: 0.35 * s, z: bz }, { quaternion: axisAngle({ x: 0, y: 1, z: 0 }, (rnd() - 0.5) * 1.6), scale: { x: s, y: s, z: s } });
    }
    world.set(world.find("e5")!, E5, { dissolve, fullDissolve, crt, tint });
    },
  // controller: a slope, stairs, a moving platform, and the character.
    controller: (world: World) => {
    const x = E5_ORIGIN.controller;
    block(world, k, "ctl-floor", k.mat({ color: { r: 0.36, g: 0.38, b: 0.4 }, roughness: 0.9 }), { x: x + 4, y: -0.5, z: 0 }, { x: 30, y: 1, z: 12 });
    // Three steps (each under stepHeight), a slope, a landing, and a lift
    // at the landing's end going between the floor and the landing.
    for (let i = 0; i < 3; i++) block(world, k, `step-${i}`, stone, { x: x - 2.2 + i * 0.5, y: 0.1 + i * 0.1, z: 0 }, { x: 0.5 + (2 - i) * 0.5, y: 0.2 + i * 0.2, z: 3 });
    block(world, k, "ramp", stone, { x: x + 0.6, y: 0.95, z: 0 }, { x: 4.6, y: 0.3, z: 3 }, axisAngle({ x: 0, y: 0, z: 1 }, 0.3));
    block(world, k, "landing", stone, { x: x + 4.4, y: 1.6, z: 0 }, { x: 3, y: 0.3, z: 3 });
    const platform = block(world, k, "platform", wood, { x: x + 7.1, y: 0.4, z: 0 }, { x: 2.4, y: 0.25, z: 2.4 });
    world.insert(platform, og.RigidBody, { kind: BODY_KINEMATIC });
    world.insert(platform, og.Velocity);
    world.insert(platform, PlatformPath, { from: { x: x + 7.1, y: 0.4, z: 0 }, to: { x: x + 7.1, y: 1.6, z: 0 }, period: 4 });
    const hero = must(world.spawn({ name: "hero", position: { x: x - 3.6, y: 0.8, z: 0 } }));
    world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 0.9 });
    world.insert(hero, og.CharacterController, { jumpSpeed: 5 });
    mesh(world, "hero-body", k.sphere, red, { x: 0, y: 0, z: 0 }, { parent: hero, scale: { x: 0.6, y: 1.5, z: 0.6 } });
    world.set(world.find("e5")!, E5, { hero });
    },
  // joints: a motorised hinge, a chain, a spring, and a ragdoll.
    joints: (world: World) => {
    const x = E5_ORIGIN.joints;
    block(world, k, "joint-floor", k.mat({ color: { r: 0.34, g: 0.34, b: 0.38 }, roughness: 0.9 }), { x, y: -0.5, z: 0 }, { x: 30, y: 1, z: 12 });
    const door = mesh(world, "spinner", k.box, wood, { x: x - 3.5, y: 1.4, z: -1 }, { scale: { x: 1.6, y: 0.2, z: 0.6 } });
    world.insert(door, og.RigidBody, { mass: 5 });
    world.insert(door, og.Collider, { shape: SHAPE_BOX });
    world.insert(door, og.Joint, { kind: JOINT_REVOLUTE, otherAnchor: { x: x - 3.5, y: 1.4, z: -1 }, axis: { x: 0, y: 0, z: 1 }, motor: MOTOR_VELOCITY, motorTarget: 2.5, motorStiffness: 200 });
    let prev: Entity | null = null;
    for (let i = 0; i < 6; i++) {
      const b = mesh(world, `chain-${i}`, k.sphere, stone, { x: x - 0.5 + i * 0.45, y: 3.2, z: -1 }, { scale: { x: 0.35, y: 0.35, z: 0.35 } });
      world.insert(b, og.RigidBody, { mass: 1, linearDamping: 0.1 });
      world.insert(b, og.Collider, { shape: SHAPE_SPHERE });
      if (prev === null) world.insert(b, og.Joint, { kind: JOINT_BALL, otherAnchor: { x: x - 0.5, y: 3.2, z: -1 } });
      else world.insert(b, og.Joint, { kind: JOINT_BALL, other: prev, anchor: { x: -0.225, y: 0, z: 0 }, otherAnchor: { x: 0.225, y: 0, z: 0 } });
      prev = b;
    }
    const bob = mesh(world, "bob", k.box, red, { x: x + 3.2, y: 1.6, z: -1 }, { scale: { x: 0.5, y: 0.5, z: 0.5 } });
    world.insert(bob, og.RigidBody, { mass: 2 });
    world.insert(bob, og.Collider, { shape: SHAPE_BOX });
    world.insert(bob, og.Joint, { kind: JOINT_SPRING, otherAnchor: { x: x + 3.2, y: 3.4, z: -1 }, restLength: 1, stiffness: 60, damping: 1 });
    const walk = walkerRig.clips.find((c) => c.name === "walk") ?? walkerRig.clips[0]!;
    const doll = must(world.instantiate(walkerModel, { name: "ragdoll", position: { x: x + 0.4, y: 0, z: 1.6 }, quaternion: axisAngle({ x: 0, y: 1, z: 0 }, 0.3) }));
    world.insert(doll, og.Skeleton, { asset: walkerRig.skeleton, rootMotion: "off" });
    world.insert(doll, og.AnimationPlayer, { clip: walk.clip, speed: 1, loop: true });
    world.insert(doll, og.Ragdoll, { weight: 0, blendTime: 0.3 });
    must(og.physics.setRagdollRig(world, doll, must(og.physics.ragdollRigFromSkeleton(world, walkerRig.skeleton))));
    world.set(world.find("e5")!, E5, { ragdoll: doll });
    },
  // rope: ropes holding a ball under a beam, and two flags.
    rope: (world: World) => {
    const x = E5_ORIGIN.rope;
    k.ground(x, { x: 0.3, y: 0.32, z: 0.36 });
    mesh(world, "beam", k.box, wood, { x, y: 4.2, z: -1 }, { scale: { x: 6, y: 0.3, z: 0.3 } });
    const ball = mesh(world, "rope-ball", k.sphere, red, { x: x - 1.4, y: 1.8, z: -1 }, { scale: { x: 0.7, y: 0.7, z: 0.7 } });
    world.insert(ball, og.RigidBody, { mass: 2 });
    world.insert(ball, og.Collider, { shape: SHAPE_SPHERE });
    world.set(ball, og.Velocity, { linear: { x: 2.5, y: 0, z: 0 } });
    world.insert(ball, og.Joint, { kind: JOINT_ROPE, otherAnchor: { x: x - 2, y: 4.05, z: -1 }, length: 2.3 });
    const anchor = must(world.spawn({ name: "rope-anchor", position: { x: x - 2, y: 4.05, z: -1 } }));
    world.insert(anchor, og.Rope, { segments: 16, length: 2.3, end: ball, endOffset: { x: 0, y: 0.35, z: 0 }, width: 0.06, color: { x: 0.85, y: 0.75, z: 0.5 } });
    const slack = must(world.spawn({ name: "rope-slack", position: { x: x + 0.2, y: 4.05, z: -1 } }));
    world.insert(slack, og.Rope, { segments: 20, length: 3.2, width: 0.05, wind: { x: 0.6, y: 0, z: 0.4 }, color: { x: 0.8, y: 0.7, z: 0.45 } });
    for (const [i, fx] of [[0, x + 1.4], [1, x + 2.6]] as const) {
      mesh(world, `pole-${i}`, k.box, stone, { x: fx, y: 1.8, z: -1.6 }, { scale: { x: 0.12, y: 3.6, z: 0.12 } });
      const flag = must(world.spawn({ name: `flag-${i}`, position: { x: fx + 0.06, y: 3.5, z: -1.6 } }));
      world.insert(flag, og.Cloth, { columns: 14, rows: 10, spacing: 0.1, pins: i === 0 ? PIN_TOP : PIN_TOP_CORNERS, wind: { x: 3, y: 0, z: 1.2 }, gust: 0.7, material: k.mat({ color: i === 0 ? { r: 0.8, g: 0.12, b: 0.1 } : { r: 0.12, g: 0.3, b: 0.8 }, roughness: 0.85, side: "double" }) });
    }
    },
  // fluid: a basin of water particles, a dam, and a pond with a crate to drop.
    fluid: (world: World) => {
    const x = E5_ORIGIN.fluid;
    for (const [i, [wx, wy, w, h]] of WALLS.entries()) {
      const e = must(world.spawn({ name: `basin-${i}`, position: { x: x + wx, y: wy, z: 0 } }));
      world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: h, z: 4 } });
    }
    const fluid = must(world.spawn({ name: "fluid", position: { x: x - 2.7, y: -0.6, z: 0 } }));
    world.insert(fluid, og.Fluid2D, { count: 700, size: { x: 2.4, y: 3 }, radius: 0.16, color: { x: 0.15, y: 0.5, z: 0.9 }, layer: 3, seed: 3 });
    const lava = must(world.spawn({ name: "lava", position: { x: x + 0.6, y: 0.2, z: 0 } }));
    world.insert(lava, og.Fluid2D, { count: 160, size: { x: 1, y: 1 }, radius: 0.16, viscosity: 0.6, color: { x: 1.4, y: 0.45, z: 0.08 }, layer: 3, seed: 5 });
    const pondAt = { x: x + 4.6, y: -1.6, z: 0 };
    const line = must(world.spawn({ name: "water-line", position: pondAt }));
    world.insert(line, og.WaterLine, { width: 4, depth: 2, columns: 64, splash: 0.06 });
    world.insert(line, og.Water2D, { layer: 2 });
    const pondFloor = must(world.spawn({ name: "pond-floor", position: { x: pondAt.x, y: pondAt.y - 2.2, z: 0 } }));
    world.insert(pondFloor, og.Collider, { shape: SHAPE_BOX, size: { x: 5, y: 0.4, z: 4 } });
    const shelf = must(world.spawn({ name: "pond-shelf", position: { x: pondAt.x + 20, y: -31, z: 0 } }));
    world.insert(shelf, og.Collider, { shape: SHAPE_BOX, size: { x: 2, y: 0.2, z: 2 } });
    const c = must(world.spawn({ name: "pond-crate", position: { x: pondAt.x + 20, y: -30.5, z: 0 } }));
    // Light enough to float: the line's buoyancy lifts by submerged area.
    world.insert(c, og.RigidBody, { mass: 0.45 });
    world.insert(c, og.Collider, { shape: SHAPE_BOX, size: { x: 0.7, y: 0.7, z: 0.7 } });
    world.insert(c, og.Sprite, { texture: stoneTexture, width: 0.7, height: 0.7, layer: 1 });
    world.set(world.find("e5")!, E5, { pondCrate: c, stone: stoneTexture });
    },
  };
}

