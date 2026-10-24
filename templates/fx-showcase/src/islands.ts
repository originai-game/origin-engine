// Round 3's generated terrain station, far from the others along x, built
// when first shown:
//
//   islands  terrain/islands.ogt: an island the cook generates instead of
//            reading a heightmap (a `generate` block: warped noise faded
//            to the sea floor, erosion by seeded drops, a sea with its
//            shore flattened into beaches, rivers carved where the flow
//            gathers, biome rules writing the control map), loaded with
//            og.terrain.load and placed with og.Terrain; the sea is an
//            og.Water at the TerrainInfo's seaLevel, as the og-game skill
//            §11 places it.

import { og, defineComponent, f32 } from "@origingame/engine";
import { WEATHER_NONE } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const GEN_STATIONS = ["islands"] as const;
export type GenStation = (typeof GEN_STATIONS)[number];

/** Where the island lives along x. */
export const GEN_ORIGIN = 5000;

/** What the playtest reads, on the entity "islands". */
export const Gen = defineComponent("Gen", {
  /** og.terrain.load's seaLevel (-1000 when it answered none). */
  seaLevel: f32(0),
  /** The terrain's height (og.terrain.sample) at its centre and at a corner. */
  centre: f32(0),
  corner: f32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The island and its sea. */
function build(world: World): void {
  const x = GEN_ORIGIN;
  const info = must(og.terrain.load(world, "terrain/islands.ogt"));
  const ground = must(world.spawn({ name: "islands-ground", position: { x, y: 0, z: 0 } }));
  world.insert(ground, og.Terrain, { data: info.data });
  if (info.seaLevel !== null) {
    const sea = must(world.spawn({ name: "islands-sea", position: { x, y: info.seaLevel, z: 0 } }));
    // Six times the terrain's size, so the open sea reaches the horizon.
    world.insert(sea, og.Water, { size: { x: info.size.x * 6, y: info.size.y * 6 }, segments: 64, waveStrength: 0.12, foamWidth: 0.8 });
  }
  const at = (dx: number, dz: number) => must(og.terrain.sample(world, { data: info.data, origin: { x, y: 0, z: 0 }, x: x + dx, z: dz })).height;
  const e = must(world.spawn({ name: "islands" }));
  world.insert(e, Gen, { seaLevel: info.seaLevel ?? -1000, centre: at(0, 0), corner: at(-250, -250) });
}

/** Turns the station off (every station starts from it). */
export function resetGen(_world: World): void {}

/** Shows the station: its world (built the first time), camera, light and atmosphere. */
export function showGen(world: World, _station: GenStation): void {
  if (world.find("islands") === null) build(world);
  const x = GEN_ORIGIN;
  const cam = must(world.resource(og.Camera));
  const eye = { x: x + 250, y: 120, z: 330 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: x - 10, y: 0, z: -20 });
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.3;
  settings.environmentIntensity = 0.6;
  const sun = world.find("sun");
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.55, y: -0.45, z: -0.4 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3.2 });
  }
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, {
    enabled: true,
    zenith: { r: 0.18, g: 0.34, b: 0.62 },
    horizon: { r: 0.62, g: 0.7, b: 0.78 },
    ground: { r: 0.2, g: 0.2, b: 0.18 },
    sunDirection: { x: 0.55, y: 0.45, z: 0.4 },
    sunSize: 1,
    glow: 0.3,
  });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.0006, start: 200, heightDensity: 0, sky: 0, color: { r: 0.62, g: 0.7, b: 0.78 } });
  must(world.resource(og.Weather)).kind = WEATHER_NONE;
}
