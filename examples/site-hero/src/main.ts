// The introduction site's live hero, "Skyreach Meadow": a traveller in a
// red cloak at the edge of a cliff-top meadow, over a sea of cloud with
// floating islands, at a fixed late-afternoon sun (og.PhysicalSky with
// og.DayNight held still, og.Clouds below the eye, a layered og.Terrain
// with grass and flowers swaying in og.Wind's gusts, og.motion driving
// the traveller).
//
// The shot is built in its own frame: the camera near (0, 901, 6) looks
// down -z, and DayNight's azimuth turns the sun to stand low behind the
// traveller on the right.
//
// Commands (dev builds): `shot:<n>` picks the view (0 the hero framing,
// 1 a step further back).
//
//   og run examples/site-hero --size 1920x1080 --commands "6:screenshot:frame0.png;6.1:quit"

import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, TerrainData, World } from "@origingame/engine";
import { must, v3 } from "./mesh.ts";
import { buildSky, flower, grassClump } from "./sky.ts";

/** The hour and the sun's azimuth that stand it 12° up, behind the traveller's right shoulder. */
const HOUR = 17.115;
const AZIMUTH = -122;

export const View = defineResource("View", {
  shot: u32(0, { max: 1, doc: "The view: 0 the hero framing, 1 a step further back." }),
});

const scene = { terrain: 0 as unknown as TerrainData, traveller: 0 as unknown as Entity };

const DEG = Math.PI / 180;
type Quat = { x: number; y: number; z: number; w: number };
function mul(a: Quat, b: Quat): Quat {
  return {
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
  };
}

function ground(world: World, x: number, z: number): number {
  const s = og.terrain.sample(world, { data: scene.terrain, x, z });
  return s.ok ? s.value.height : 900;
}

/** The views: where the eye stands (x, z; y is the ground's plus `up`), yaw right of -z, pitch up, fov. */
const SHOTS = [
  { x: -1, z: 6.5, up: 1.45, yaw: 15, pitch: -6, fov: 40 },
  { x: -3, z: 14, up: 2.6, yaw: 12, pitch: -7, fov: 44 },
];
const TRAVELLER = { x: 4, z: -3.2 };

function start(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true, shadowDistance: 120, shadowSoftness: 0.03 }));
  Object.assign(must(world.resource(og.DayNight)), { enabled: true, start: HOUR, speed: 0, tilt: 25, azimuth: AZIMUTH, lightIntensity: 1.1, tintFog: false, step: 6 });
  Object.assign(must(world.resource(og.PhysicalSky)), { enabled: true, haze: 0.8 });
  // The cloud sea: its tops some 300 m under the meadow, broken enough for gaps.
  Object.assign(must(world.resource(og.Clouds)), {
    enabled: true,
    coverage: 0.6,
    density: 0.05,
    base: 200,
    thickness: 420,
    scale: 1800,
    detail: 0.75,
    steps: 64,
    layer: 0.25,
    layerAltitude: 8000,
    wind: { x: 3, y: 0.2, z: 1 },
  });
  // Aerial perspective: a lilac haze that lifts the far islands and the tower.
  Object.assign(must(world.resource(og.Fog)), { enabled: true, color: { r: 0.56, g: 0.56, b: 0.78 }, start: 80, density: 0.00022, sky: 0.04, heightDensity: 0, maxOpacity: 0.8 });
  // The wind: a steady lean towards the camera's right, gusts sweeping across the meadow.
  Object.assign(must(world.resource(og.Wind)), { strength: 0.22, direction: { x: 0.8, y: 0, z: 0.6 }, speed: 4, gust: 0.75 });

  const info = must(og.terrain.load(world, "terrain/meadow.ogt"));
  scene.terrain = info.data;
  const meadow = must(world.spawn({ name: "meadow" }));
  must(world.insert(meadow, og.Terrain, { data: info.data }));

  // Grass over the meadow layers, flowers on the flower layer.
  const grassMat = must(og.render.standardMaterial(world, { roughness: 0.9, side: "double" }));
  for (const layer of [0, 1]) {
    const e = must(world.spawn({ name: `grass-${layer}` }));
    must(world.insert(e, og.TerrainScatter, { data: info.data, geometry: grassClump(world), material: grassMat, layer, density: 14, distance: 70, fade: 20, minScale: 0.4, maxScale: 0.85, align: 0.4, seed: 3 + layer, sway: 1 }));
  }
  const petals: Array<[number, number, number]> = [[0.95, 0.93, 0.86], [0.72, 0.66, 0.92], [0.98, 0.86, 0.45]];
  const flowerMat = must(og.render.standardMaterial(world, { roughness: 0.8, side: "double" }));
  petals.forEach((p, k) => {
    const e = must(world.spawn({ name: `flowers-${k}` }));
    must(world.insert(e, og.TerrainScatter, { data: info.data, geometry: flower(world, p), material: flowerMat, layer: 1, density: 3, distance: 40, fade: 12, minScale: 0.8, maxScale: 1.3, align: 0.2, seed: 11 + k, sway: 0.8 }));
  });

  // The traveller at the edge, looking out (og.motion drives the Meshy rig through rigs/meshy.ogt).
  const model = "models/traveller.glb";
  const anim = must(og.load.animation(world, model));
  const rig = must(og.load.rig(world, "rigs/meshy.ogt"));
  const at = v3(TRAVELLER.x, ground(world, TRAVELLER.x, TRAVELLER.z), TRAVELLER.z);
  const body = must(world.instantiate(must(og.load.gltf(world, model)), { name: "traveller", position: at }));
  must(world.insert(body, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.MotionAgent, { rig, drive: true }));
  must(og.motion.face(world, body, { heading: Math.PI + 0.25, duration: 0.1 }));
  must(og.motion.pose(world, body, { pose: "idle", duration: 600 }));
  scene.traveller = body;

  buildSky(world, v3(0, 901, 6));

  const cam = must(world.resource(og.Camera));
  cam.near = 0.15;
  cam.far = 60000;
  must(world.resource(og.RenderSettings)).sky = true;
}

const Frame = defineSystem("Frame", { reads: [View], writes: [og.Camera] }, (_q, _time, world) => {
  const s = SHOTS[must(world.resource(View)).shot] ?? SHOTS[0]!;
  const cam = must(world.resource(og.Camera));
  cam.position = { x: s.x, y: ground(world, s.x, s.z) + s.up, z: s.z };
  const yaw = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, -s.yaw * DEG);
  const pitch = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, s.pitch * DEG);
  cam.quaternion = mul(yaw, pitch);
  cam.fov = s.fov;
});

function shot(world: World, args: readonly string[]): void {
  const n = Number(args[0]);
  if (!Number.isInteger(n) || n < 0 || n >= SHOTS.length) throw new Error(`write shot:<0..${SHOTS.length - 1}>`);
  must(world.resource(View)).shot = n;
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/skyreach.ogt",
  resources: [View],
  systems: [Frame],
  start,
  commands: { shot },
  commandHelp: { shot: { usage: "<n>", doc: "Pick the view (0 the hero framing, 1 a step further back)." } },
  facts: (world) => {
    const t = world.get(scene.traveller, og.Transform);
    return t.ok ? { traveller: t.value.position } : null;
  },
});
