// The phase 2 3D fluid station (owed row 29), far from the others along x,
// built when first shown:
//
//   fluids3d  og.Fluid3D in a tiled basin: a blue pool, and a heavy orange
//             fluid of the same group (density 3, mixing 0) poured into one
//             end by an emitter, sinking under the blue. Drawn as one
//             screen-space surface: the tiles show through the pool tinted
//             and bent, the sun and an orange lamp light it, the sky
//             reflects off it, the low front wall cuts it.

import { og, defineComponent, defineSystem, f32, f64, u32, SHAPE_BOX } from "@origingame/engine";
import type { Entity, Geometry, Material, Vector3 as Vec3, World } from "@origingame/engine";

export const SPH_STATIONS = ["fluids3d"] as const;
export type SPHStation = (typeof SPH_STATIONS)[number];

/** Where the station lives along x. */
export const SPH_ORIGIN: Record<SPHStation, number> = { fluids3d: 2400 };

/** The basin's inside (the fluids' bounds), its floor's top and its walls' heights. */
export const BASIN = { width: 2.4, depth: 1.6, floor: 0.2, back: 0.9, front: 0.35 };

/** The station's state, on the entity "sph". */
export const SPH = defineComponent("SPH", {
  /** The two fluids. */
  pool: f64(0),
  honey: f64(0),
  /** Written per tick: the fluids' mean heights, their particle counts, and the particles outside the basin. */
  poolY: f32(0),
  honeyY: f32(0),
  poolCount: u32(0),
  honeyCount: u32(0),
  outside: u32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Per tick once built: the fluids' heights and counts, and how many particles left the basin. */
export const SPHShow = defineSystem("SPHShow", { reads: [og.Fluid3D], writes: [SPH] }, (_q, _time, world) => {
  const e = world.find("sph");
  if (e === null) return;
  const s = must(world.get(e, SPH));
  if (s.pool === 0) return;
  const x0 = SPH_ORIGIN.fluids3d;
  let outside = 0;
  const read = (f: number): [number, number] => {
    const p = og.physics.fluid3dParticles(world, f as unknown as Entity);
    if (!p.ok || p.value.length === 0) return [0, 0];
    let y = 0;
    for (const q of p.value) {
      y += q.y;
      const inside = Math.abs(q.x - x0) <= BASIN.width / 2 + 0.01 && Math.abs(q.z) <= BASIN.depth / 2 + 0.01 && q.y > BASIN.floor - 0.05;
      if (!inside) outside++;
    }
    return [y / p.value.length, p.value.length];
  };
  const [poolY, poolCount] = read(s.pool);
  const [honeyY, honeyCount] = read(s.honey);
  world.set(e, SPH, { poolY, poolCount, honeyY, honeyCount, outside });
});

export interface SPHKit {
  box: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
}

let kit: SPHKit;

/** Prepares the station; it is built when first shown (showSPH). */
export function buildSPH(world: World, k: SPHKit): void {
  kit = k;
  world.insert(must(world.spawn({ name: "sph" })), SPH);
}

function buildBasin(world: World): void {
  const k = kit;
  const x0 = SPH_ORIGIN.fluids3d;
  k.ground(x0, { x: 0.34, y: 0.33, z: 0.31 });
  const { width: w, depth: d, floor: top } = BASIN;
  // The floor: coloured tiles, so what the pool bends and tints shows.
  const tiles = [
    k.mat({ color: { r: 0.8, g: 0.78, b: 0.7 }, roughness: 0.5 }),
    k.mat({ color: { r: 0.7, g: 0.08, b: 0.06 }, roughness: 0.5 }),
    k.mat({ color: { r: 0.85, g: 0.62, b: 0.05 }, roughness: 0.5 }),
    k.mat({ color: { r: 0.06, g: 0.35, b: 0.16 }, roughness: 0.5 }),
  ];
  const n = { x: 6, z: 4 };
  for (let i = 0; i < n.x; i++) {
    for (let j = 0; j < n.z; j++) {
      const e = must(world.spawn({ name: `sph-tile-${i}-${j}`, position: { x: x0 - w / 2 + (i + 0.5) * (w / n.x), y: top / 2, z: -d / 2 + (j + 0.5) * (d / n.z) } }));
      world.set(e, og.Transform, { scale: { x: w / n.x - 0.02, y: top, z: d / n.z - 0.02 } });
      world.insert(e, og.Mesh, { geometry: k.box, material: tiles[(i + 2 * j) % tiles.length]!, receiveShadow: true });
    }
  }
  // The floor the fluids stand on (the tiles are only drawn).
  const floor = must(world.spawn({ name: "sph-floor", position: { x: x0, y: top - 0.5, z: 0 } }));
  world.insert(floor, og.Collider, { shape: SHAPE_BOX, size: { x: w + 1, y: 1, z: d + 1 } });
  // The walls (drawn; the fluids' bounds hold them): high at the back and
  // sides, low at the front so the camera sees the pool over it.
  const stone = k.mat({ color: { r: 0.2, g: 0.19, b: 0.17 }, roughness: 0.85 });
  const wall = (name: string, at: Vec3, size: Vec3) => {
    const e = must(world.spawn({ name, position: at }));
    world.set(e, og.Transform, { scale: size });
    world.insert(e, og.Mesh, { geometry: k.box, material: stone, castShadow: true, receiveShadow: true });
  };
  const t = 0.16;
  wall("sph-wall-back", { x: x0, y: BASIN.back / 2, z: -d / 2 - t / 2 }, { x: w + 2 * t, y: BASIN.back, z: t });
  wall("sph-wall-left", { x: x0 - w / 2 - t / 2, y: BASIN.back / 2, z: 0 }, { x: t, y: BASIN.back, z: d });
  wall("sph-wall-right", { x: x0 + w / 2 + t / 2, y: BASIN.back / 2, z: 0 }, { x: t, y: BASIN.back, z: d });
  wall("sph-wall-front", { x: x0, y: BASIN.front / 2, z: d / 2 + t / 2 }, { x: w + 2 * t, y: BASIN.front, z: t });
  // The two fluids share the basin (the same entity position and bounds)
  // and a group: the orange one, three times as heavy, is poured in at the
  // left and sinks under the blue (mixing 0 keeps them in layers).
  const at = { x: x0, y: top + 0.3, z: 0 };
  const bounds = { x: w, y: 0, z: d };
  const pool = must(world.spawn({ name: "sph-pool", position: at }));
  world.insert(pool, og.Fluid3D, {
    count: 1600, size: { x: w - 0.1, y: 0.5, z: d - 0.1 }, bounds, group: 1, density: 1, seed: 1,
    color: { x: 0.1, y: 0.42, z: 0.85 }, opacity: 0.75, reflectivity: 0.7,
  });
  const honey = must(world.spawn({ name: "sph-honey", position: at }));
  world.insert(honey, og.Fluid3D, {
    count: 0, maxCount: 500, emitRate: 400, emitOffset: { x: -w / 2 + 0.4, y: 1.2, z: 0 }, emitVelocity: { x: 0.6, y: -0.5, z: 0 }, emitRadius: 0.1,
    bounds, group: 1, density: 3, viscosity: 1.2, seed: 2,
    color: { x: 0.95, y: 0.4, z: 0.08 }, opacity: 0.9,
  });
  world.set(world.find("sph")!, SPH, { pool, honey });
  const lamp = must(world.spawn({ name: "sph-lamp", position: { x: x0 + 0.7, y: 1.1, z: 0.7 } }));
  world.insert(lamp, og.PointLight, { color: { r: 1, g: 0.6, b: 0.25 }, intensity: 6, distance: 4 });
}

/** Shows the station: builds it the first time, then its camera and light. */
export function showSPH(world: World, station: SPHStation): void {
  if (world.find(`sph-${station}`) === null) {
    buildBasin(world);
    must(world.spawn({ name: `sph-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = SPH_ORIGIN[station];
  const eye = { x: x + 0.2, y: 2.2, z: 2.9 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x, y: 0.35, z: -0.15 });
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5, y: -0.7, z: -0.45 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
  settings.sky = true;
  settings.environmentIntensity = 0.45;
  settings.bloom = false;
  cam.exposure = 0.65;
}
