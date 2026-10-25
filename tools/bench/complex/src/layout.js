// The complex scene's world, shared by both engines: the ground's height
// field, where every prop, character, light and emitter stands, and the
// camera's path. tools/make-assets.ts builds the ground model from
// `height`; src/main.ts (the engine) and tools/bench/three-complex.js
// (three.js) place the same things from the same lists. Plain JavaScript,
// so the three.js page imports it as served; pure arithmetic over a seeded
// hash (no Math.random, no clock), so every run and both pages get the
// same world.

/** The ground: a square of SIZE metres centred at the origin, GRID quads a side. */
export const SIZE = 320;
export const GRID = 256;
/** The lake: centre, radius (m); the water's surface is y = 0. */
export const LAKE = { x: -62, z: 46, r: 46 };
/** The village's plateau: centre, radius, height. */
export const VILLAGE = { x: 34, z: -18, r: 52, y: 3 };
/** The landing field (dropships) and the car park. */
export const FIELD = { x: 96, z: 74 };
export const PARK = { x: 88, z: -88 };
/** The sun's direction (towards which it shines), as the reference scene's. */
export const SUN = { x: -0.45, y: -0.75, z: -0.5 };
/** The sun's cascades reach this far from the camera (m). */
export const SHADOW_DISTANCE = 120;

function hash(x, y, s) {
  let h = Math.imul(x | 0, 374761393) ^ Math.imul(y | 0, 668265263) ^ Math.imul(s | 0, 2147483647);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
const smooth = (t) => t * t * (3 - 2 * t);
function noise(x, y, s) {
  const xi = Math.floor(x), yi = Math.floor(y);
  const u = smooth(x - xi), v = smooth(y - yi);
  const a = hash(xi, yi, s), b = hash(xi + 1, yi, s), c = hash(xi, yi + 1, s), d = hash(xi + 1, yi + 1, s);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
function fbm(x, y, s) {
  let sum = 0, amp = 0.5, f = 1;
  for (let o = 0; o < 5; o++) {
    sum += amp * noise(x * f, y * f, s + o);
    f *= 2.03;
    amp *= 0.5;
  }
  return sum;
}
const clamp01 = (v) => Math.max(0, Math.min(1, v));

/** The ground's height at (x, z): rolling hills, higher towards the edges, a lake basin and the village's flat plateau. */
export function height(x, z) {
  const edge = Math.max(Math.abs(x), Math.abs(z)) / (SIZE / 2);
  let h = (fbm(x / 70 + 10, z / 70 + 10, 7) - 0.45) * 26 + Math.max(0, edge - 0.55) * 60;
  const dl = Math.hypot(x - LAKE.x, z - LAKE.z) / LAKE.r;
  h = h * smooth(clamp01((dl - 0.55) / 0.9)) + (1 - smooth(clamp01(dl))) * -4 + smooth(clamp01(dl)) * 1.5 * (1 - smooth(clamp01((dl - 1) / 0.8)));
  const dv = Math.hypot(x - VILLAGE.x, z - VILLAGE.z) / VILLAGE.r;
  const k = smooth(clamp01((dv - 0.8) / 0.6));
  h = h * k + VILLAGE.y * (1 - k);
  for (const f of [FIELD, PARK]) {
    const d = Math.hypot(x - f.x, z - f.z) / 24;
    const m = smooth(clamp01((d - 0.7) / 0.6));
    h = h * m + 2 * (1 - m);
  }
  return h;
}

/** The ground's slope at (x, z): 0 flat, 1 at 45 degrees. */
export function slope(x, z) {
  const e = 1;
  return Math.hypot(height(x + e, z) - height(x - e, z), height(x, z + e) - height(x, z - e)) / (2 * e);
}

/** A seeded stream of numbers in [0, 1). */
export function rng(seed) {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Every model the scene draws: file (under assets/models), how many copies
 * and where. A copy is { x, y, z, yaw, scale }. Models with `moving` are
 * moved every tick (ghostPose); the rest stand still.
 */
export const MODELS = [
  { file: "pine.glb", count: 1100 },
  { file: "oak.glb", count: 700 },
  { file: "rock-a.glb", count: 300 },
  { file: "rock-b.glb", count: 300 },
  { file: "rock-c.glb", count: 300 },
  { file: "bush.glb", count: 900 },
  { file: "berry.glb", count: 300 },
  { file: "palm.glb", count: 180 },
  { file: "islet.glb", count: 24 },
  { file: "crate.glb", count: 260 },
  { file: "barrel.glb", count: 240 },
  { file: "fence.glb", count: 420 },
  { file: "lamppost.glb", count: 96 },
  { file: "house.glb", count: 36 },
  { file: "stall.glb", count: 32 },
  { file: "car.glb", count: 40 },
  { file: "stair.glb", count: 6 },
  { file: "dropship.glb", count: 3 },
  { file: "ghost.glb", count: 24, moving: true },
];

const inLake = (x, z, pad = 0) => Math.hypot(x - LAKE.x, z - LAKE.z) < LAKE.r + pad;
const inVillage = (x, z, pad = 0) => Math.hypot(x - VILLAGE.x, z - VILLAGE.z) < VILLAGE.r + pad;
const nearPad = (x, z, pad) => Math.hypot(x - FIELD.x, z - FIELD.z) < pad || Math.hypot(x - PARK.x, z - PARK.z) < pad;
const LIMIT = SIZE / 2 - 6;

/** The houses' ring and the market's rows, used by several placements. */
function houses() {
  const out = [];
  for (let i = 0; i < 36; i++) {
    const ring = i < 16 ? 0 : 1;
    const n = ring === 0 ? 16 : 20;
    const j = ring === 0 ? i : i - 16;
    const a = (j / n) * Math.PI * 2 + ring * 0.13;
    const r = ring === 0 ? 26 : 40;
    out.push({ x: VILLAGE.x + Math.cos(a) * r, z: VILLAGE.z + Math.sin(a) * r, yaw: -a - Math.PI / 2 });
  }
  return out;
}

/** Where each model's copies stand, by model file. */
export function placements() {
  const r = rng(20261025);
  const out = {};
  const put = (file, x, z, yaw, scale, lift = 0) => (out[file] ??= []).push({ x, y: height(x, z) + lift, z, yaw, scale });
  const scatter = (file, n, ok, scale, tries = 60) => {
    for (let i = 0; i < n; i++) {
      for (let t = 0; t < tries; t++) {
        const x = (r() * 2 - 1) * LIMIT, z = (r() * 2 - 1) * LIMIT;
        if (!ok(x, z) && t < tries - 1) continue;
        put(file, x, z, r() * Math.PI * 2, scale[0] + r() * (scale[1] - scale[0]));
        break;
      }
    }
  };
  const wild = (x, z) => !inLake(x, z, 4) && !inVillage(x, z, 6) && !nearPad(x, z, 26);
  scatter("pine.glb", 1100, (x, z) => wild(x, z) && height(x, z) > 3 && slope(x, z) < 0.8, [0.8, 1.4]);
  scatter("oak.glb", 700, (x, z) => wild(x, z) && height(x, z) < 9 && slope(x, z) < 0.5, [0.8, 1.3]);
  for (const f of ["rock-a.glb", "rock-b.glb", "rock-c.glb"]) scatter(f, 300, (x, z) => !inVillage(x, z, 2) && !nearPad(x, z, 24) && !inLake(x, z, -6), [0.5, 2.2]);
  scatter("bush.glb", 900, (x, z) => wild(x, z) || (inVillage(x, z, 4) && !inVillage(x, z, -4)), [0.7, 1.4]);
  scatter("berry.glb", 300, (x, z) => !inLake(x, z, 2) && !nearPad(x, z, 24) && Math.hypot(x - VILLAGE.x, z - VILLAGE.z) > 44, [1.4, 2.2]);
  // Palms round the shore, islets in the lake.
  for (let i = 0; i < 180; i++) {
    const a = (i / 180) * Math.PI * 2 + r() * 0.02, d = LAKE.r + 3 + r() * 10;
    put("palm.glb", LAKE.x + Math.cos(a) * d, LAKE.z + Math.sin(a) * d, r() * Math.PI * 2, 1.6 + r() * 0.8);
  }
  for (let i = 0; i < 24; i++) {
    const a = r() * Math.PI * 2, d = 8 + r() * (LAKE.r - 16);
    const x = LAKE.x + Math.cos(a) * d, z = LAKE.z + Math.sin(a) * d;
    out["islet.glb"] ??= [];
    out["islet.glb"].push({ x, y: -0.6, z, yaw: r() * Math.PI * 2, scale: 1.4 + r() * 1.2 });
  }
  // The village: houses in two rings round the plaza, stalls in the plaza, crates and barrels by the doors and stalls.
  const hs = houses();
  for (const h of hs) put("house.glb", h.x, h.z, h.yaw, 1);
  for (let i = 0; i < 32; i++) {
    const row = i % 4, col = Math.floor(i / 4);
    put("stall.glb", VILLAGE.x - 14 + col * 4, VILLAGE.z - 9 + row * 6, row % 2 ? Math.PI : 0, 1);
  }
  for (let i = 0; i < 260; i++) {
    const h = hs[i % hs.length], a = r() * Math.PI * 2, d = 4.5 + r() * 2.5;
    put("crate.glb", h.x + Math.cos(a) * d, h.z + Math.sin(a) * d, r() * Math.PI, 0.8 + r() * 0.4);
  }
  for (let i = 0; i < 240; i++) {
    const near = i % 3 === 0 ? { x: VILLAGE.x - 14 + (i % 8) * 4, z: VILLAGE.z - 12 + (i % 5) * 6 } : hs[i % hs.length];
    const a = r() * Math.PI * 2, d = 2 + r() * 3.5;
    put("barrel.glb", near.x + Math.cos(a) * d, near.z + Math.sin(a) * d, r() * Math.PI * 2, 0.9 + r() * 0.2);
  }
  // Fences: 14 paddocks of 3 m panels (8 by 7 a paddock) round the village, clear of the lake and the pads.
  for (let p = 0, placed = 0; placed < 14; p++) {
    const a = p * 2.399 + 0.3, d = VILLAGE.r + 22 + (p % 3) * 14;
    const cx = VILLAGE.x + Math.cos(a) * d, cz = VILLAGE.z + Math.sin(a) * d;
    if (inLake(cx, cz, 20) || nearPad(cx, cz, 40) || Math.max(Math.abs(cx), Math.abs(cz)) > LIMIT - 20) continue;
    placed++;
    const turn = a, c = Math.cos(turn), s = Math.sin(turn);
    const side = (u, v, yaw) => put("fence.glb", cx + u * c - v * s, cz + u * s + v * c, yaw - turn, 1);
    for (let k = 0; k < 8; k++) {
      side(-12 + 1.5 + k * 3, -10.5, 0);
      side(-12 + 1.5 + k * 3, 10.5, 0);
    }
    for (let k = 0; k < 7; k++) {
      side(-12, -10.5 + 1.5 + k * 3, Math.PI / 2);
      side(12, -10.5 + 1.5 + k * 3, Math.PI / 2);
    }
  }
  // Lamp posts along the plaza's ring road and the two roads out.
  for (let i = 0; i < 96; i++) {
    if (i < 48) {
      const a = (i / 48) * Math.PI * 2;
      put("lamppost.glb", VILLAGE.x + Math.cos(a) * 33, VILLAGE.z + Math.sin(a) * 33, -a, 1);
    } else {
      const k = i - 48, road = k % 2, t = Math.floor(k / 2) / 24;
      const to = road ? FIELD : PARK;
      const x = VILLAGE.x + (to.x - VILLAGE.x) * (0.55 + t * 0.4) + (k % 4 < 2 ? 3 : -3), z = VILLAGE.z + (to.z - VILLAGE.z) * (0.55 + t * 0.4);
      put("lamppost.glb", x, z, 0, 1);
    }
  }
  // Cars in rows on the car park, dropships on the field, the old stairs on the hills.
  for (let i = 0; i < 40; i++) put("car.glb", PARK.x - 18 + (i % 10) * 4, PARK.z - 9 + Math.floor(i / 10) * 6, Math.floor(i / 10) % 2 ? Math.PI : 0, 1, 0.35);
  for (let i = 0; i < 3; i++) put("dropship.glb", FIELD.x - 14 + i * 14, FIELD.z + (i % 2) * 6, 0.4 * i, 3, 3);
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2 + 0.4, d = 122;
    const x = Math.cos(a) * d * 0.9, z = Math.sin(a) * d;
    put("stair.glb", x, z, a, 0.15, -0.3);
  }
  out["ghost.glb"] = Array.from({ length: 24 }, (_, i) => ghostPose(i, 0));
  for (const m of MODELS) if ((out[m.file] ?? []).length !== m.count) throw new Error(`layout: ${m.file} has ${(out[m.file] ?? []).length} copies, not ${m.count}`);
  return out;
}

/** The ghost carts: 24 translucent carts round the village's ring road, a full lap every 40 s. */
export function ghostPose(i, t) {
  const a = (i / 24) * Math.PI * 2 + (t * Math.PI * 2) / 40;
  const x = VILLAGE.x + Math.cos(a) * 30, z = VILLAGE.z + Math.sin(a) * 30;
  return { x, y: VILLAGE.y + 0.5, z, yaw: -a, scale: 1 };
}

/**
 * The characters: 48 skinned characters on two models, gathered on the
 * plaza, each on its own clip, speed and phase.
 */
export const WALKER_CLIPS = ["idle", "walk", "run", "wave", "turn"];
export const VILLAGER_CLIPS = ["idle", "walk", "run", "attack", "hurt", "jump", "bite", "slam", "die"];
export function characters() {
  const out = [];
  for (let i = 0; i < 48; i++) {
    const villager = i % 2 === 1;
    const k = Math.floor(i / 2);
    const a = (k / 24) * Math.PI * 2 + (villager ? 0.13 : 0);
    const d = 10 + (i % 3) * 3.5;
    const x = VILLAGE.x + 6 + Math.cos(a) * d, z = VILLAGE.z + Math.sin(a) * d * 0.7 + 4;
    const clips = villager ? VILLAGER_CLIPS : WALKER_CLIPS;
    out.push({
      model: villager ? "villager.glb" : "walker.glb",
      clip: clips[k % clips.length],
      x, y: height(x, z), z,
      yaw: Math.atan2(VILLAGE.x - x, VILLAGE.z - z),
      speed: 0.8 + (i % 5) * 0.1,
      time: (i * 0.137) % 1,
    });
  }
  return out;
}

/** The shadowed local lights: two spots over the market, two points at the campfires. */
export const CAMPFIRES = [
  { x: VILLAGE.x - 24, z: VILLAGE.z + 14 },
  { x: VILLAGE.x + 20, z: VILLAGE.z + 18 },
  { x: LAKE.x + 50, z: LAKE.z - 8 },
  { x: VILLAGE.x + 12, z: VILLAGE.z - 30 },
];
export function lights() {
  const y = (p) => height(p.x, p.z);
  return [
    { kind: "spot", x: VILLAGE.x - 6, y: VILLAGE.y + 11, z: VILLAGE.z - 4, tx: VILLAGE.x - 6, ty: VILLAGE.y, tz: VILLAGE.z, color: [1, 0.85, 0.6], intensity: 160, distance: 34, angle: 0.62, penumbra: 0.35 },
    { kind: "spot", x: VILLAGE.x + 10, y: VILLAGE.y + 10, z: VILLAGE.z + 8, tx: VILLAGE.x + 6, ty: VILLAGE.y, tz: VILLAGE.z + 6, color: [0.7, 0.8, 1], intensity: 140, distance: 30, angle: 0.55, penumbra: 0.4 },
    { kind: "point", x: CAMPFIRES[0].x, y: y(CAMPFIRES[0]) + 1.6, z: CAMPFIRES[0].z, color: [1, 0.55, 0.2], intensity: 60, distance: 18 },
    { kind: "point", x: CAMPFIRES[1].x, y: y(CAMPFIRES[1]) + 1.6, z: CAMPFIRES[1].z, color: [1, 0.5, 0.18], intensity: 60, distance: 18 },
  ];
}

/** The particle emitters: a fire and its smoke at each campfire, a smoke at 12 chimneys. */
export function emitters() {
  const out = [];
  CAMPFIRES.forEach((c, i) => {
    const y = height(c.x, c.z);
    out.push({ fx: "fire", x: c.x, y: y + 0.1, z: c.z, seed: 100 + i });
    out.push({ fx: "smoke", x: c.x, y: y + 1.2, z: c.z, seed: 200 + i });
  });
  houses().slice(0, 12).forEach((h, i) => {
    // The chimney's top: house.glb puts it 2.2 m right and 1.4 m back of the centre, 7.4 m up.
    const c = Math.cos(h.yaw), s = Math.sin(h.yaw);
    const lx = 2.2, lz = -1.4;
    const x = h.x + lx * c + lz * s, z = h.z - lx * s + lz * c;
    out.push({ fx: "smoke", x, y: height(h.x, h.z) + 7.6, z, seed: 300 + i });
  });
  return out;
}

/**
 * The particle emitters' parameters, as assets/fx/<name>.ogt says them
 * (tools/make-assets.ts writes those files from this table, so both pages
 * simulate the same emitters).
 */
export const FX = {
  fire: { max: 64, rate: 60, life: [0.6, 1.0], speed: [0.8, 1.4], cone: 14, radius: 0.35, size: [0.5, 0.8], sizeOverLife: [1, 0.2], color: [[0, 2.4, 1, 0.25, 0], [0.15, 2.4, 0.85, 0.2, 0.85], [1, 0.8, 0.1, 0.02, 0]], gravity: 0.6, blend: "additive", texture: "textures/flame.png" },
  smoke: { max: 64, rate: 14, life: [2.5, 3.5], speed: [0.4, 0.8], cone: 20, radius: 0.25, size: [0.6, 1.0], sizeOverLife: [1, 2.4], color: [[0, 0.22, 0.22, 0.24, 0], [0.2, 0.3, 0.3, 0.32, 0.5], [1, 0.45, 0.45, 0.47, 0]], gravity: 0.35, blend: "alpha", texture: "textures/smoke.png" },
};

/** The camera at time t (s): a slow orbit round the village, high enough to see the lake and the hills. */
export function cameraAt(t) {
  const a = 0.9 + t * 0.05;
  const eye = { x: VILLAGE.x + Math.cos(a) * 78, y: VILLAGE.y + 34, z: VILLAGE.z + Math.sin(a) * 78 };
  const at = { x: VILLAGE.x - 8, y: VILLAGE.y, z: VILLAGE.z + 6 };
  return { eye, at, fov: 55, near: 0.3, far: 600 };
}
