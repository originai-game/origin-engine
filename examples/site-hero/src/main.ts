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
// 1 a step further back, 2 close on the traveller), `brace:<seconds>`
// has the traveller brace against the wind and shield his eyes
// (og.motion.react with actions/brace.ogt), `cloth:<0|1>` takes the
// cloak's simulation off or puts it back (a diagnostic: off, the cloak is
// drawn as skinned), `clouds:<0|1>` hides the cloud sea or shows it (a
// diagnostic for what the clouds draw over).
//
//   og run examples/site-hero --size 1920x1080 --commands "6:screenshot:frame0.png;6.1:quit"

import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, TerrainData, World } from "@origingame/engine";
import { must, v3 } from "./mesh.ts";
import { buildSky } from "./sky.ts";

/** The hour and the sun's azimuth that stand it 12° up, behind the traveller's right shoulder. */
const HOUR = 17.115;
const AZIMUTH = -122;

export const View = defineResource("View", {
  shot: u32(0, { max: 2, doc: "The view: 0 the hero framing, 1 a step further back, 2 close on the traveller." }),
});

const scene = { terrain: 0 as unknown as TerrainData, traveller: 0 as unknown as Entity, paint: 0 };

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
/** The close view: the eye beside the traveller on the meadow side, in front of his left shoulder, looking at his chest. */
const CLOSE = { dx: 2.8, dz: -2.1, up: 1.4, aim: 1.0, fov: 36, paint: 1 };
/** The cloak's simulation (og.Garment on mesh 1, split from the body by tools/split-cloak.ts; its freedom is painted in its vertex alpha). */
const CLOAK = { mesh: 1, maxDistance: 0.25, bending: 0.3, iterations: 6 };
/** Where the gusts come from (og.Wind blows along +x, +z): the brace faces it. */
const WIND_FROM = { x: -0.8, y: 0, z: -0.6 };

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

  // The meadow's grass: og.Grass grows blades on the GPU out to 150 m over
  // the grass layer and the flowered edge, sage at the root to straw-gold
  // tips, some clumps dried, gusts showing as a moving sheen.
  const grass = {
    data: info.data,
    height: 0.5,
    heightVariation: 0.55,
    clumpSize: 1.3,
    tufts: 0.08,
    baseColor: { r: 0.045, g: 0.075, b: 0.03 },
    tipColor: { r: 0.3, g: 0.36, b: 0.12 },
    dryColor: { r: 0.45, g: 0.36, b: 0.16 },
    colorVariation: 0.35,
    dryness: 0.12,
    translucency: 0.6,
    sheen: 0.4,
    castShadows: true,
  };
  must(world.insert(meadow, og.Grass, { ...grass, layer: 0, seed: 1 }));
  const edge = must(world.spawn({ name: "grass-edge" }));
  must(world.insert(edge, og.Grass, { ...grass, layer: 1, density: 220, height: 0.42, seed: 2 }));
  dressCliff(world);

  // Wildflowers on the flowered edge: the authored daisy, bellflower and
  // buttercup clumps (assets/models/flowers, SOURCE.md), scattered and
  // swaying in the wind; past 12 m their octahedral impostors.
  ["daisy", "bell", "buttercup"].forEach((name, k) => {
    const model = must(og.load.gltf(world, `models/flowers/${name}.glb`));
    const gi = must(og.load.gltfInfo(world, model));
    const e = must(world.spawn({ name: `flowers-${name}` }));
    must(world.insert(e, og.TerrainScatter, { data: info.data, geometry: gi.geometries[0]!, material: gi.materials[0]!, layer: 1, density: 0.12, distance: 45, fade: 12, minScale: 0.75, maxScale: 1.2, align: 0.25, seed: 11 + k, sway: 0.7, impostor: model, impostorDistance: 12 }));
  });

  // The traveller at the edge, looking out (og.motion drives the Meshy rig through rigs/meshy.ogt).
  const model = "models/traveller.glb";
  const anim = must(og.load.animation(world, model));
  const rig = must(og.load.rig(world, "rigs/meshy.ogt"));
  const at = v3(TRAVELLER.x, ground(world, TRAVELLER.x, TRAVELLER.z), TRAVELLER.z);
  const body = must(world.instantiate(must(og.load.gltf(world, model)), { name: "traveller", position: at }));
  must(world.insert(body, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.MotionAgent, { rig, drive: true }));
  // The cloak swings in the wind.
  must(world.insert(body, og.Garment, CLOAK));
  must(og.motion.face(world, body, { heading: Math.PI + 0.25, duration: 0.1 }));
  must(og.motion.pose(world, body, { pose: "idle", duration: 600 }));
  scene.traveller = body;

  buildSky(world, v3(0, 901, 6));

  const cam = must(world.resource(og.Camera));
  cam.near = 0.15;
  cam.far = 60000;
  must(world.resource(og.RenderSettings)).sky = true;
  scene.paint = must(world.resource(og.Stylise)).paintRadius;
}

/** Where the meadow ends falls along (outward, level): the cliff's face turns this way. */
const OUTWARD = { x: -0.6, z: -0.8 };

/**
 * The cliff's face: the authored rock (models/cliff.glb, Meshy, SOURCE.md),
 * a column of strata about 1.9 units tall and 1.56 deep, its sandstone face
 * towards +z. The height field's wall is opaque rock, but seen edge-on from
 * the close shot it reads as a pale stretched sheet with a sawtooth lip; the
 * authored rock stands in front of it along the edge, each piece turned and
 * sized differently. The edge is found from the terrain itself: walking out
 * from the meadow until the ground falls away.
 */
function dressCliff(world: World): void {
  const yaw = Math.atan2(OUTWARD.x, OUTWARD.z);
  [-34, -16, 2, 18, 36, 56].forEach((x0, k) => {
    // A point well inside the meadow (the edge runs near z = -1.5 - 0.75 x),
    // then outward until the ground drops 4 m.
    let x = x0 - OUTWARD.x * 12;
    let z = -1.5 - 0.75 * x0 - OUTWARD.z * 12;
    const lip = ground(world, x, z);
    for (let i = 0; i < 160 && ground(world, x, z) > lip - 4; i++) {
      x += OUTWARD.x * 0.25;
      z += OUTWARD.z * 0.25;
    }
    const top = ground(world, x - OUTWARD.x * 3, z - OUTWARD.z * 3);
    const size = 27 + ((k * 7) % 5) * 2.5;
    // The height field's face falls about 6 m per metre out from the edge: the
    // rock's front stands 9 m out (proud of the face for its top 50 m), its
    // grassy top 3 m under the lip, a ledge stepping down from the meadow.
    const back = 0.78 * size - 9;
    const at = v3(x - OUTWARD.x * back, top - 3 - 0.95 * size, z - OUTWARD.z * back);
    const turn = yaw + (((k * 5) % 7) / 6 - 0.5) * 0.9;
    const e = must(world.instantiate(must(og.load.gltf(world, "models/cliff.glb")), { name: `cliff-${k}`, position: at }));
    must(world.set(e, og.Transform, { position: at, scale: { x: size, y: size, z: size }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) }));
  });
}

const Frame = defineSystem("Frame", { reads: [View, og.Transform], writes: [og.Camera, og.Stylise] }, (_q, _time, world) => {
  const n = must(world.resource(View)).shot;
  const cam = must(world.resource(og.Camera));
  // Close up, the painterly filter is lighter: at the look's radius it smears the hands and face.
  must(world.resource(og.Stylise)).paintRadius = n === 2 ? CLOSE.paint : scene.paint;
  if (n === 2) {
    const t = must(world.get(scene.traveller, og.Transform)).position;
    const x = t.x + CLOSE.dx;
    const z = t.z + CLOSE.dz;
    cam.position = { x, y: Math.max(ground(world, x, z), t.y) + CLOSE.up, z };
    cam.quaternion = og.transform.lookAt(cam.position, { x: t.x, y: t.y + CLOSE.aim, z: t.z });
    cam.fov = CLOSE.fov;
    return;
  }
  const s = SHOTS[n] ?? SHOTS[0]!;
  cam.position = { x: s.x, y: ground(world, s.x, s.z) + s.up, z: s.z };
  const yaw = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, -s.yaw * DEG);
  const pitch = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, s.pitch * DEG);
  cam.quaternion = mul(yaw, pitch);
  cam.fov = s.fov;
});

function shot(world: World, args: readonly string[]): void {
  const n = Number(args[0]);
  if (!Number.isInteger(n) || n < 0 || n > SHOTS.length) throw new Error(`write shot:<0..${SHOTS.length}>`);
  must(world.resource(View)).shot = n;
}

function cloth(world: World, args: readonly string[]): void {
  if (args[0] === "0") must(world.remove(scene.traveller, og.Garment));
  else if (args[0] === "1") must(world.insert(scene.traveller, og.Garment, CLOAK));
  else throw new Error("write cloth:<0|1>");
}

function clouds(world: World, args: readonly string[]): void {
  if (args[0] !== "0" && args[0] !== "1") throw new Error("write clouds:<0|1>");
  must(world.resource(og.Clouds)).enabled = args[0] === "1";
}

// Diagnostic (branch site-hero-sheet-diag only): hide the cloak (the
// traveller drawn from a copy of its model without mesh 1), the whole
// traveller, or the ground (the terrain and the cliff rocks).
function hide(world: World, args: readonly string[]): void {
  const t = scene.traveller;
  if (args[0] === "cloak") {
    must(world.remove(t, og.Garment));
    const m = must(world.get(t, og.Model));
    must(world.set(t, og.Model, { ...m, asset: must(og.load.gltf(world, "models/traveller-nocloak.glb")) }));
  } else if (args[0] === "traveller") {
    const m = must(world.get(t, og.Model));
    must(world.set(t, og.Model, { ...m, visible: false }));
  } else if (args[0] === "ground") {
    const g = world.find("meadow")!;
    const tr = must(world.get(g, og.Terrain));
    must(world.set(g, og.Terrain, { ...tr, visible: false }));
    for (let k = 0; k < 6; k++) {
      const c = world.find(`cliff-${k}`)!;
      const m = must(world.get(c, og.Model));
      must(world.set(c, og.Model, { ...m, visible: false }));
    }
  } else throw new Error("write hide:<cloak|traveller|ground>");
}

function brace(world: World, args: readonly string[]): void {
  const seconds = args.length ? Number(args[0]) : 1.5;
  if (!(seconds >= 0)) throw new Error("write brace:<seconds held>");
  must(og.motion.react(world, scene.traveller, { action: "actions/brace.ogt", direction: WIND_FROM, strength: 1, duration: seconds, blend: 0.25 }));
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/skyreach.ogt",
  resources: [View],
  systems: [Frame],
  start,
  commands: { shot, brace, cloth, clouds, hide },
  commandHelp: {
    shot: { usage: "<n>", doc: "Pick the view (0 the hero framing, 1 a step further back, 2 close on the traveller)." },
    brace: { usage: "<seconds>", doc: "Brace against the wind and shield the eyes, held this long (1.5 s if omitted)." },
    cloth: { usage: "<0|1>", doc: "Take the cloak's simulation off (0, drawn as skinned) or put it back (1)." },
    clouds: { usage: "<0|1>", doc: "Hide the cloud sea (0) or show it (1)." },
    hide: { usage: "<cloak|traveller|ground>", doc: "Diagnostic: hide the cloak, the traveller, or the terrain and cliff rocks." },
  },
  facts: (world) => {
    const t = world.get(scene.traveller, og.Transform);
    return t.ok ? { traveller: t.value.position } : null;
  },
});
